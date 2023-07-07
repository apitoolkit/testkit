use jsonpath_lib::select;
use std::collections::HashMap;
use miette::{Diagnostic, NamedSource, SourceSpan, GraphicalReportHandler, GraphicalTheme,Report };
use thiserror::Error;

use log;
use reqwest::{Client, ClientBuilder, Response};
use rhai::{Engine, Scope};
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestPlan {
    pub name: String,
    pub stages: Vec<TestStage>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct TestStage {
    name: String,
    request: RequestConfig,
    asserts: Vec<Assert>,
    outputs: Option<Outputs>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Assert {
    #[serde(rename = "is_true")]
    IsTrue(String),
    #[serde(rename = "is_false")]
    IsFalse(String),
    #[serde(rename = "is_array")]
    IsArray(String),
    #[serde(rename = "is_empty")]
    IsEmpty(String),
    #[serde(rename = "is_string")]
    IsString(String),
    // Add other assertion types as needed
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    #[serde(flatten)]
    pub http_method: HttpMethod,
    pub headers: Option<HashMap<String, String>>,
    pub json: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    name: String,
    value: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET(String),
    POST(String),
    DELETE(String),
    PUT(String), // Add other HTTP methods as needed
}

#[derive(Debug)]
pub struct RequestResult {
    pub stage_name: String,
    pub assert_results: Vec<Result<bool, AssertionError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Outputs {
    #[serde(rename = "todoItem")]
    pub todo_item: Option<String>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseAssertion {
    resp: ResponseObject,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseObject {
    status: u16,
    headers: Value,
    body: Value,
}

#[derive(Error, Debug, Diagnostic)]
#[error("request assertion failed!")]
#[diagnostic(
    // code(asertion),
    severity(error)
    // help("try doing it better next time?")
)]
pub struct AssertionError {
    // The Source that we're gonna be printing snippets out of.
    // This can be a String if you don't have or care about file names.
    // #[source_code]
    // src: NamedSource,
    // Snippets and highlights can be included in the diagnostic!
    // #[label("This bit here")]
    // bad_bit: SourceSpan,
    #[help]
    advice: Option<String>,
    #[source_code]
    src: NamedSource,
    #[label("This jsonpath here")]
    bad_bit: SourceSpan,
}

fn report_error(diag: Report) -> String {
    let mut out = String::new();
        GraphicalReportHandler::new_themed(GraphicalTheme::unicode())
            .with_width(80)
            // .with_footer("this is a footer".into())
            .render_report(&mut out, diag.as_ref())
            .unwrap();
    out
}

#[derive(Default, Clone)]
pub struct TestContext { 
    pub plan: String,
    pub stage: String,
    pub path: String,
    pub file: String,
    pub file_source: String,
}

// base_request would process a test plan, logging status updates as they happen.
// Logging in place allows tracking of the results earlier
pub async fn base_request(
    ctx: TestContext,
    plan: &TestPlan,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    // log::info!("{}", plan.name);

    let client = reqwest::Client::builder()
        .connection_verbose(true)
        .build()?;
    // let client = ClientBuilder::connection_verbose(true).build()?;
    let mut results = Vec::new();

    for stage in &plan.stages {
        let mut ctx = ctx.clone();
        ctx.plan = plan.name.clone();
        ctx.stage = stage.name.clone();
        // log::info!("{}/{}", plan.name, stage.name);
        let mut request_builder = match &stage.request.http_method {
            HttpMethod::GET(url) => client.get(url),
            HttpMethod::POST(url) => client.post(url),
            HttpMethod::PUT(url) => client.put(url),
            HttpMethod::DELETE(url) => client.delete(url),
        };
        if let Some(headers) = &stage.request.headers {
            for (name, value) in headers {
                request_builder = request_builder.header(name, value);
            }
        }

        if let Some(json) = &stage.request.json {
            request_builder = request_builder.json(json);
        }

        let response = request_builder.send().await?;
        let assert_results = check_assertions(ctx, &stage.asserts, response).await?;
        // if let Some(outputs) = &stage.outputs {
        //     update_outputs(outputs, &response_json);
        // }

        results.push(RequestResult {
            stage_name: stage.name.clone(),
            assert_results: assert_results,
        });
    }
    Ok(results)
}

// Naive implementation that might not work for all jsonpaths and might need to be changed.
// Should add tests to check which jsonpaths would not be supported
fn find_all_jsonpaths<'a>(input: &'a String) -> Vec<&'a str> {
    input
        .split_whitespace()
        .filter(|x| x.starts_with("$"))
        .collect()
}

// 1. First we extract a list of jsonpaths
// 2. Build a json with all the fields which can be referenced via jsonpath
// 3. Apply the jsonpaths over this json and save their values to a map
// 4. replace the jsonpaths with that value in the original expr string
// 5. Evaluate the expression with the expressions library.
// TODO: decide on both error handling and the reporting approach
fn evaluate_expressions<'a, T: Clone + 'static>(
    ctx: TestContext,
    original_expr: &String,
    object: &'a Value,
) -> Result<(T, String), AssertionError> {
    let paths = find_all_jsonpaths(&original_expr);
    let mut expr = original_expr.clone();

    for path in paths {
        match select(&object, &path) {
            Ok(selected_value) => {
                if let Some(selected_value) = selected_value.first() {
                    expr = expr.replace(path, &selected_value.to_string());
                } else {
                    let i = original_expr.find(path).unwrap_or(0);

                    return Err(AssertionError {
                        advice: Some(
                            "The given json path could not be located in the context. Add the 'dump: true' to the test stage, to print out the requests and responses which can be refered to via jsonpath. ".to_string(),
                        ),
                        src: NamedSource::new(ctx.file, original_expr.clone()),
                        bad_bit: (i, i+path.len()).into(),
                    });
                }
            }
            Err(err) => {
                // The given jsonpath could not be evaluated to a value
                return Err(AssertionError {
                    advice: Some("could not resolve jsonpaths to any real variables".to_string()),
                        src: NamedSource::new(ctx.file, expr),
                        bad_bit: (0, 4).into(),
                })
            }
        }
    }
    log::debug!(target: "api_workflows", "normalized pre-evaluation assert expression: {:?}", &expr);
    let evaluated =  parse_expression::<T>(&expr.clone()).map_err(|e| AssertionError {
        advice: Some("check that you're using correct jsonpaths".to_string()),
        src: NamedSource::new(ctx.file, expr.clone()),
        bad_bit: (0, 4).into(),
    })?;
    Ok((evaluated, expr.clone()))
}

async fn check_assertions(
    ctx: TestContext,
    asserts: &[Assert],
    response: Response,
) -> Result<Vec<Result<bool, AssertionError>>, Box<dyn std::error::Error>> {
    log::info!("{}/{}", ctx.plan, ctx.stage);

    let status_code = response.status().as_u16();
    let body = response.json().await?;
    // let headers_json = format!("{:?}", response.headers()).into();
    let assert_object = ResponseAssertion {
        resp: ResponseObject {
            status: status_code,
            headers: Value::Null,
            body,
        },
    };

    let json_body: Value = serde_json::json!(&assert_object);
    let mut assert_results: Vec<Result<bool, AssertionError>> = Vec::new();

    for assertion in asserts {
        let eval_result = match assertion {
            Assert::IsTrue(expr) => {
                evaluate_expressions::<bool>(ctx.clone(), expr, &json_body).map(|(e, eval_expr)| ("IS TRUE ", e == true, expr, eval_expr))
            }
            Assert::IsFalse(expr) => {
                evaluate_expressions::<bool>(ctx.clone(), expr, &json_body).map(|(e, eval_expr)| ("IS FALSE ", e == false, expr, eval_expr))
            }
            Assert::IsArray(_expr) => todo!(),
            Assert::IsEmpty(_expr) => todo!(),
            Assert::IsString(_expr) => todo!(),
        };

        match eval_result {
            Err(err) => log::error!("{}", report_error((err).into())),
            Ok((prefix, result, expr, eval_expr)) => if result{
                    log::info!("✅ {: <10} ==>   {} ", prefix, expr)
                }else{
                    log::error!("❌ {: <10} ==>   {} ", prefix, expr);
                    log::error!("{} ", report_error((AssertionError {
                        advice: Some("check that you're using correct jsonpaths".to_string()),
                        src: NamedSource::new("bad_file.rs", "blablabla"),
                        bad_bit: (0, 4).into(),
                    }).into()))
                },
        }

    }
    Ok(assert_results)
}

// parse_expression would take a normalized math-like expression and evaluate it to a premitive or simpler
// value. Eg `5 + 5` becomes `10`
fn parse_expression<T: Clone + 'static>(expr: &str) -> Result<T, Box<dyn std::error::Error>> {
    let engine = Engine::new();
    let result = engine.eval_expression::<T>(expr)?;
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    use claim::*;
    use httpmock::prelude::*;
    use serde_json::json;

    #[tokio::test]
    async fn test_kitchen_sink() {
        env_logger::init();
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/todos")
                // .header("content-type", "application/json")
                // .body("{\"number\":5}")
                // .body_contains("number")
                // .body_matches(Regex::new(r#"(\d+)"#).unwrap())
                .json_body(json!({ "number": 5 }));
            then.status(201).json_body(json!({ "number": 5 }));
        });

        log::debug!("{}", server.url("/todos"));

        let stage = TestPlan {
            name: String::from("stage1"),
            stages: vec![TestStage {
                name: String::from("test_stage"),
                request: RequestConfig {
                    http_method: HttpMethod::POST(server.url("/todos")),
                    headers: Some(HashMap::from([(
                        String::from("Content-Type"),
                        String::from("application/json"),
                    )])),
                    json: Some(json!({ "number": 5 })),
                },
                asserts: vec![
                    Assert::IsTrue(String::from("$.resp.body.number == 5")),
                    Assert::IsTrue(String::from("$.resp.status == 201")),
                    Assert::IsFalse(String::from("$.resp.body.number != 5")),
                    Assert::IsTrue(String::from("$.respx.nonexisting == 5")),
                ],
                outputs: None,
            }],
        };
        let ctx = TestContext{
            plan: "plan".into(),
            file_source: "file source".into(),
            file: "file.tp.yml".into(),
            path: ".".into(),
            stage: "stage_name".into(),
        };
        let resp = base_request(ctx, &stage).await;
        m.assert();
        log::debug!("{:?}", resp);
        assert_ok!(resp);
    }
}
