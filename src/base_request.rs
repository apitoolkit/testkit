use jsonpath_lib::select;
use log;
use miette::{Diagnostic, GraphicalReportHandler, GraphicalTheme, NamedSource, Report, SourceSpan};
use regex::Regex;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use rhai::Engine;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::{serde_as, EnumMap};
use std::collections::HashMap;
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestPlan {
    pub name: Option<String>,
    pub stages: Vec<TestStage>,
}

#[serde_as]
#[derive(Debug, Serialize, Deserialize)]
pub struct TestStage {
    name: Option<String>,
    dump: Option<bool>,
    request: RequestConfig,
    #[serde_as(as = "EnumMap")]
    asserts: Vec<Assert>,
    outputs: Option<HashMap<String, String>>,
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
    #[serde(rename = "is_number")]
    IsNumber(String),
    #[serde(rename = "is_boolean")]
    IsBoolean(String),
    #[serde(rename = "is_null")]
    IsNull(String),
    // Add other assertion types as needed
}

#[derive(Clone, Debug, Serialize, Deserialize)]
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

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "UPPERCASE")]
pub enum HttpMethod {
    GET(String),
    POST(String),
    DELETE(String),
    PUT(String), // Add other HTTP methods as needed
}

#[derive(Debug)]
pub struct RequestResult {
    pub stage_name: Option<String>,
    pub stage_index: u32,
    pub assert_results: Vec<Result<bool, AssertionError>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseAssertion {
    req: RequestConfig,
    resp: ResponseObject,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseObject {
    status: u16,
    headers: Value,
    json: Value,
    raw: String,
}

#[derive(Error, Debug, Diagnostic)]
#[error("request assertion failed!")]
#[diagnostic(
    // code(asertion),
    severity(error)
    // help("try doing it better next time?")
)]
pub struct AssertionError {
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
        .render_report(&mut out, diag.as_ref())
        .unwrap();
    out
}

#[derive(Default, Clone)]
pub struct TestContext {
    pub plan: Option<String>,
    pub stage: Option<String>,
    pub stage_index: u32,
    pub path: String,
    pub file: String,
    pub file_source: String,
}

pub async fn run(ctx: TestContext, exec_string: String) -> Result<(), anyhow::Error> {
    let test_plans: Vec<TestPlan> = serde_yaml::from_str(&exec_string)?;
    log::debug!("test_plans: {:#?}", test_plans);

    for test in test_plans {
        let result = base_request(ctx.clone(), &test).await;
        match result {
            Ok(res) => {
                log::debug!("Test passed: {:?}", res);
            }
            Err(err) => {
                log::error!("{}", err)
            }
        }
    }

    Ok(())
}

// base_request would process a test plan, logging status updates as they happen.
// Logging in place allows tracking of the results earlier
pub async fn base_request(
    ctx: TestContext,
    plan: &TestPlan,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    let client = reqwest::Client::builder()
        .connection_verbose(true)
        .build()?;
    let mut results = Vec::new();
    let mut outputs_map: HashMap<String, Value> = HashMap::new();

    for (i, stage) in plan.stages.iter().enumerate() {
        let mut ctx = ctx.clone();
        ctx.plan = plan.name.clone();
        ctx.stage = stage.name.clone();
        ctx.stage_index = i as u32;
        log::info!(
            "{:?} ⬅ {}/{}",
            stage.request.http_method,
            ctx.plan.clone().unwrap_or("_plan".into()),
            ctx.stage.clone().unwrap_or(ctx.stage_index.to_string())
        );
        let mut request_builder = match &stage.request.http_method {
            HttpMethod::GET(url) => {
                let url = format_url(&ctx, url, &outputs_map)?;
                client.get(url)
            }
            HttpMethod::POST(url) => {
                let url = format_url(&ctx, url, &outputs_map)?;
                client.post(url)
            }
            HttpMethod::PUT(url) => {
                let url = format_url(&ctx, url, &outputs_map)?;
                client.put(url)
            }
            HttpMethod::DELETE(url) => {
                let url = format_url(&ctx, url, &outputs_map)?;
                client.delete(url)
            }
        };
        if let Some(headers) = &stage.request.headers {
            for (name, value) in headers {
                request_builder = request_builder.header(name, value);
            }
        }

        if let Some(json) = &stage.request.json {
            let mut j_string = json.to_string();
            for (k, v) in &outputs_map {
                let normalized_jsonp_key = format!("\"$.outputs.{}\"", k);
                j_string = j_string.replace(&normalized_jsonp_key, &v.to_string());
                // Remove twice. Workaround to support text and number types
                let normalized_jsonp_key = format!("$.outputs.{}", k);
                j_string = j_string.replace(&normalized_jsonp_key, &v.to_string());
            }
            let clean_json: Value = serde_json::from_str(&j_string)?;
            request_builder = request_builder.json(&clean_json);
        }

        let response = request_builder.send().await?;
        let status_code = response.status().as_u16();
        let header_hashmap = header_map_to_hashmap(response.headers());

        let raw_body = response.text().await?;
        let json_body: Value = serde_json::from_str(&raw_body)?;

        let assert_object = ResponseAssertion {
            req: stage.request.clone(),
            resp: ResponseObject {
                status: status_code,
                headers: serde_json::json!(header_hashmap),
                json: json_body.clone(),
                raw: raw_body,
            },
        };

        let assert_context: Value = serde_json::json!(&assert_object);
        if stage.dump.unwrap_or(false) {
            log::info!(
                "💡 DUMP jsonpath request response context:\n {}",
                colored_json::to_colored_json_auto(&assert_context)?
            )
        }
        let assert_results =
            check_assertions(ctx, &stage.asserts, assert_context, &outputs_map).await?;
        // if let Some(outputs) = &stage.outputs {
        //     update_outputs(outputs, &response_json);
        // }
        if let Some(outputs) = &stage.outputs {
            for (key, value) in outputs.into_iter() {
                if let Some(evaled) = select(&serde_json::json!(assert_object), &value)?.first() {
                    outputs_map
                        .insert(format!("{}_{}", i, key.to_string()), evaled.clone().clone());
                }
            }
        }

        results.push(RequestResult {
            stage_name: stage.name.clone(),
            stage_index: i as u32,
            assert_results,
        });
    }
    Ok(results)
}

fn header_map_to_hashmap(headers: &HeaderMap<HeaderValue>) -> HashMap<String, Vec<String>> {
    let mut header_hashmap = HashMap::new();
    for (k, v) in headers {
        let k = k.as_str().to_owned();
        let v = String::from_utf8_lossy(v.as_bytes()).into_owned();
        header_hashmap.entry(k).or_insert_with(Vec::new).push(v)
    }
    header_hashmap
}

// Naive implementation that might not work for all jsonpaths and might need to be changed.
// Should add tests to check which jsonpaths would not be supported
fn find_all_jsonpaths(input: &String) -> Vec<&str> {
    input
        .split_whitespace()
        .filter(|x| x.starts_with("$"))
        .collect()
}

fn get_var_stage(input: &str, current_stage: u32) -> Option<u32> {
    let start_pos = input.find('[')?;
    let end_pos = input[start_pos + 1..].find(']')? + start_pos + 1;
    let n_str = &input[start_pos + 1..end_pos];

    if let Ok(n) = n_str.parse::<i32>() {
        if n < 0 {
            let target_stage = (current_stage as i32) + n;
            if target_stage >= 0 {
                return Some(target_stage.try_into().unwrap());
            }
            None
        } else {
            Some(n.try_into().unwrap())
        }
    } else {
        None
    }
}

// Replacce output variables with actual values in request url
fn format_url(
    ctx: &TestContext,
    original_url: &String,
    outputs: &HashMap<String, Value>,
) -> Result<String, AssertionError> {
    let re = Regex::new(r"\{\{(.*?)\}\}").unwrap_or_else(|err| panic!("{}", err));
    let mut url = original_url.clone();
    for caps in re.captures_iter(original_url) {
        if let Some(matched_string) = caps.get(1) {
            let st = matched_string.as_str().to_string();
            let target_stage = get_var_stage(&st.clone(), ctx.stage_index).unwrap_or_default();
            let elements: Vec<&str> = st.split('.').collect();
            let target_key = elements.last().unwrap_or(&"");
            let output_key = format!("{}_{}", target_stage, target_key);
            if let Some(value) = outputs.get(&output_key) {
                let repl = format!("{{{{{}}}}}", st);
                url = url.replace(&repl, &value.to_string());
            } else {
                return Err(AssertionError {
                    advice: Some(format!(
                        "{}: could not resolve output variable path to any real value",
                        st
                    )),
                    src: NamedSource::new(&ctx.file, st.clone()),
                    bad_bit: (0, st.len()).into(),
                });
            }
        }
    }
    Ok(url)
}

fn find_all_output_vars(
    input: &str,
    outputs: &HashMap<String, Value>,
    stage_index: u32,
) -> HashMap<String, Option<Value>> {
    let mut val_map = HashMap::new();

    let vars: Vec<String> = input
        .split_whitespace()
        .filter(|x| x.starts_with("{{") && x.ends_with("}}"))
        .map(|x| x.to_string())
        .collect();

    for var in vars {
        let mut s = var.clone();
        s.truncate(s.len() - 2);
        s = s.split_off(2);
        let target_stage = get_var_stage(&s.clone(), stage_index).unwrap_or_default();
        let elements: Vec<&str> = s.split('.').collect();
        let target_key = elements.last().unwrap_or(&"");
        let output_key = format!("{}_{}", target_stage, target_key);
        val_map.insert(var, outputs.get(&output_key).cloned());
    }
    val_map
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
    outputs: &HashMap<String, Value>,
) -> Result<(T, String), AssertionError> {
    let paths = find_all_jsonpaths(&original_expr);
    let output_vars = find_all_output_vars(&original_expr, outputs, ctx.stage_index);
    let mut expr = original_expr.clone();

    for (var_path, var_value) in output_vars.iter() {
        if let Some(value) = var_value {
            expr = expr.replace(var_path, value.to_string().as_str());
        } else {
            return Err(AssertionError {
                advice: Some(format!(
                    "{}: could not resolve output variable path to any real value",
                    var_path
                )),
                src: NamedSource::new(ctx.file, var_path.clone()),
                bad_bit: (0, var_path.len()).into(),
            });
        }
    }
    for path in paths {
        match select(&object, &path) {
            Ok(selected_value) => {
                if let Some(selected_value) = selected_value.first() {
                    expr = expr.replace(path, &selected_value.to_string());
                } else {
                    let i = original_expr.find(path).unwrap_or(0);
                    // TODO: reproduce and improve this error
                    return Err(AssertionError {
                        advice: Some(
                            "The given json path could not be located in the context. Add the 'dump: true' to the test stage, to print out the requests and responses which can be refered to via jsonpath. ".to_string(),
                        ),
                        src: NamedSource::new(ctx.file, original_expr.clone()),
                        bad_bit: (i, i+path.len()).into(),
                    });
                }
            }
            Err(_err) => {
                // TODO: reproduce and improve this error. Use the _err argument
                // The given jsonpath could not be evaluated to a value
                return Err(AssertionError {
                    advice: Some("could not resolve jsonpaths to any real variables".to_string()),
                    src: NamedSource::new(ctx.file, expr),
                    bad_bit: (0, 4).into(),
                });
            }
        }
    }
    log::debug!("normalized pre-evaluation assert expression: {:?}", &expr);
    // TODO: reproduce and improve this error
    let evaluated = parse_expression::<T>(&expr.clone()).map_err(|_e| AssertionError {
        advice: Some("check that you're using correct jsonpaths".to_string()),
        src: NamedSource::new(ctx.file, expr.clone()),
        bad_bit: (0, 4).into(),
    })?;
    Ok((evaluated, expr.clone()))
}

fn evaluate_value<'a, T: Clone + 'static>(
    ctx: TestContext,
    expr: &'a String,
    object: &'a Value,
    value_type: &str,
) -> Result<(bool, String), AssertionError> {
    match select(&object, expr) {
        Ok(selected_value) => {
            if let Some(selected_value) = selected_value.first() {
                match selected_value {
                    Value::Array(v) => {
                        if value_type == "empty" {
                            return Ok((v.is_empty(), expr.clone()));
                        }
                        Ok((value_type == "array", expr.clone()))
                    }
                    Value::String(v) => {
                        if value_type == "empty" {
                            return Ok((v.is_empty(), expr.clone()));
                        }
                        Ok((value_type == "str", expr.clone()))
                    }
                    Value::Number(_v) => Ok((value_type == "num", expr.clone())),
                    Value::Bool(_v) => Ok((value_type == "bool", expr.clone())),
                    Value::Null => Ok((value_type == "null", expr.clone())),
                    _ => todo!(),
                }
            } else {
                // TODO: reproduce and improve this error
                return Err(AssertionError {
                        advice: Some(
                            "The given json path could not be located in the context. Add the 'dump: true' to the test stage, to print out the requests and responses which can be refered to via jsonpath. ".to_string(),
                        ),
                        src: NamedSource::new(ctx.file, expr.clone()),
                        bad_bit: (0,  expr.len()).into(),
                    });
            }
        }
        Err(_err) => {
            // TODO: reproduce and improve this error. Use the _err argument
            // The given jsonpath could not be evaluated to a value
            return Err(AssertionError {
                advice: Some("could not resolve jsonpaths to any real variables".to_string()),
                src: NamedSource::new(ctx.file, expr.clone()),
                bad_bit: (0, 4).into(),
            });
        }
    }
}

async fn check_assertions(
    ctx: TestContext,
    asserts: &[Assert],
    json_body: Value,
    outputs: &HashMap<String, Value>,
) -> Result<Vec<Result<bool, AssertionError>>, Box<dyn std::error::Error>> {
    let assert_results: Vec<Result<bool, AssertionError>> = Vec::new();

    for assertion in asserts {
        let eval_result = match assertion {
            Assert::IsTrue(expr) => {
                evaluate_expressions::<bool>(ctx.clone(), expr, &json_body, outputs)
                    .map(|(e, eval_expr)| ("IS TRUE ", e == true, expr, eval_expr))
            }
            Assert::IsFalse(expr) => {
                evaluate_expressions::<bool>(ctx.clone(), expr, &json_body, outputs)
                    .map(|(e, eval_expr)| ("IS FALSE ", e == false, expr, eval_expr))
            }
            Assert::IsArray(expr) => evaluate_value::<bool>(ctx.clone(), expr, &json_body, "array")
                .map(|(e, eval_expr)| ("IS ARRAY ", e == true, expr, eval_expr)),
            Assert::IsEmpty(expr) => evaluate_value::<bool>(ctx.clone(), expr, &json_body, "empty")
                .map(|(e, eval_expr)| ("IS EMPTY ", e == true, expr, eval_expr)),
            Assert::IsString(expr) => evaluate_value::<bool>(ctx.clone(), expr, &json_body, "str")
                .map(|(e, eval_expr)| ("IS STRING ", e == true, expr, eval_expr)),
            Assert::IsNumber(expr) => evaluate_value::<bool>(ctx.clone(), expr, &json_body, "num")
                .map(|(e, eval_expr)| ("IS NUMBER ", e == true, expr, eval_expr)),
            Assert::IsBoolean(expr) => {
                evaluate_value::<bool>(ctx.clone(), expr, &json_body, "bool")
                    .map(|(e, eval_expr)| ("IS BOOLEAN ", e == true, expr, eval_expr))
            }
            Assert::IsNull(expr) => evaluate_value::<bool>(ctx.clone(), expr, &json_body, "null")
                .map(|(e, eval_expr)| ("IS NULL ", e == true, expr, eval_expr)),
        };

        match eval_result {
            Err(err) => log::error!("{}", report_error((err).into())),
            Ok((prefix, result, expr, _eval_expr)) => {
                if result {
                    log::info!("✅ {: <10}  ⮕   {} ", prefix, expr)
                } else {
                    log::error!("❌ {: <10}  ⮕   {} ", prefix, expr);
                    log::error!(
                        "{} ",
                        report_error(
                            (AssertionError {
                                advice: Some(
                                    "check that you're using correct jsonpaths".to_string()
                                ),
                                src: NamedSource::new("bad_file.rs", expr.to_string()),
                                bad_bit: (0, 4).into(),
                            })
                            .into()
                        )
                    )
                }
            }
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
    async fn test_yaml_kitchen_sink() {
        env_logger::init();
        // testing_logger::setup();
        let server = MockServer::start();
        let m = server.mock(|when, then| {
            when.method(POST)
                .path("/todos")
                .header("content-type", "application/json")
                .json_body(json!({ "req_number": 5 }));
            then.status(201)
                .json_body(json!({ "resp_string": "test", "resp_number": 4, "resp_bool": true, "resp_null": null}));
        });
        let m2 = server.mock(|when, then| {
            when.method(GET).path("/todo_get");
            // .json_body(json!({ "req_string": "test"  }));
            then.status(200)
                .json_body(json!({ "tasks": ["task one", 4, "task two", "task three"], "empty_str": "", "empty_arr": []}));
        });

        let yaml_str = format!(
            r#"
---
- name: stage1
  stages:
    - request:
        POST: {}
        headers:
          Content-Type: application/json
        json:
          req_number: 5
      asserts:
        is_true: $.resp.json.resp_string == "test"
        is_true: $.resp.status == 201
        is_number: $.resp.json.resp_number
        is_string: $.resp.json.resp_string
        is_boolean: $.resp.json.resp_bool
        is_null: $.resp.json.resp_null
        # is_false: $.resp.json.resp_string != 5
        # is_true: $.respx.nonexisting == 5
      outputs:
        todoResp: $.resp.json.resp_string
    - request: 
        GET: {}
        json:
            req_string: $.outputs.todoResp
      asserts:
        is_true: $.resp.status == 200
        is_array: $.resp.json.tasks
        is_true: $.resp.json.tasks[0] == "task one"
        is_number: $.resp.json.tasks[1]
        is_empty: $.resp.json.empty_str
        is_empty: $.resp.json.empty_arr
"#,
            server.url("/todos"),
            server.url("/todo_get")
        );

        let ctx = TestContext {
            plan: Some("plan".into()),
            file_source: "file source".into(),
            file: "file.tp.yml".into(),
            path: ".".into(),
            stage: Some("stage_name".into()),
            stage_index: 0,
        };
        let resp = run(ctx, yaml_str.into()).await;
        log::debug!("{:?}", resp);
        assert_ok!(resp);
        m2.assert_hits(1);
        m.assert_hits(1);

        // // We test the log output, because the logs are an important part of the user facing API of a cli tool like this
        // // TODO: figure out returning the correct exit code to show error or failure.
        // testing_logger::validate(|captured_logs| {
        //     println!("PPP");
        //     for c in captured_logs {
        //         println!("xx {:?}", c.body);
        //     }

        //     assert_eq!(captured_logs.len(), 1);
        //     // assert_eq!(captured_logs[0].body, "Something went wrong with 10");
        //     // assert_eq!(captured_logs[0].level, Level::Warn);
        // });
    }
}
