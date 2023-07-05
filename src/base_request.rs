use jsonpath_lib::select;
use std::collections::HashMap;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestResult {
    pub stage_name: String,
    pub assert_results: Vec<bool>,
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

pub async fn base_request(
    stage: &TestPlan,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    println!("================================================================================================================");
    log::info!("Executing Test: {}", stage.name);
    println!("================================================================================================================");

    let client = reqwest::Client::builder()
        .connection_verbose(true)
        .build()?;
    // let client = ClientBuilder::connection_verbose(true).build()?;
    let mut results = Vec::new();

    for stage in &stage.stages {
        log::info!("Executing stage: {}", stage.name);
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

        let assert_results = check_assertions(&stage.asserts, response).await?;
        // if let Some(outputs) = &stage.outputs {
        //     update_outputs(outputs, &response_json);
        // }

        results.push(RequestResult {
            stage_name: stage.name.clone(),
            assert_results: assert_results,
        });
    }
    // println!("{:?}", results);
    println!("================================================================================================================");
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
fn evaluate_expressions<'a, T: Clone + 'static>(expr: &String, object: &'a Value) -> T {
    let paths = find_all_jsonpaths(&expr);
    let mut expr = expr.clone();

    for path in paths {
        if let Some(selected_value) = select(&object, &path).unwrap().first() {
            expr = expr.replace(path, &selected_value.to_string());
        }
    }
    log::debug!(target: "api_workflows", "normalized pre-evaluation assert expression: {:?}", &expr);
    parse_expression::<T>(&expr).unwrap()
}

async fn check_assertions(
    asserts: &[Assert],
    response: Response,
) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
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
    let mut assert_results: Vec<bool> = Vec::new();

    for assertion in asserts {
        let eval_result = match assertion {
            Assert::IsTrue(expr) => evaluate_expressions::<bool>(expr, &json_body) == true,
            Assert::IsFalse(expr) => evaluate_expressions::<bool>(expr, &json_body) == false,
            Assert::IsArray(_expr) => todo!(),
            Assert::IsEmpty(_expr) => todo!(),
            Assert::IsString(_expr) => todo!(),
        };
        assert_results.push(eval_result);
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
                ],
                outputs: None,
            }],
        };
        let resp = base_request(&stage).await;
        m.assert();
        log::debug!("{:?}", resp);
        assert_ok!(resp);
    }
}
