use jsonpath_lib::select;
use std::collections::HashMap;

use log;
use reqwest::{Client, Response};
use rhai::Engine;
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
pub struct Assert {
    #[serde(rename = "is_true")]
    pub is_true: Option<String>,
    #[serde(rename = "is_false")]
    pub is_false: Option<String>,
    #[serde(rename = "is_array")]
    pub is_array: Option<String>,
    #[serde(rename = "is_empty")]
    pub is_empty: Option<String>,
    #[serde(rename = "is_string")]
    pub is_string: Option<String>,
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
    status: u16,
    body: Option<Value>,
}

pub async fn base_request(
    stage: &TestPlan,
) -> Result<Vec<RequestResult>, Box<dyn std::error::Error>> {
    println!("================================================================================================================");
    log::info!("Executing Test: {}", stage.name);
    println!("================================================================================================================");

    let client = Client::new();
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

async fn check_assertions(
    asserts: &[Assert],
    response: Response,
) -> Result<Vec<bool>, Box<dyn std::error::Error>> {
    let status_code = response.status().as_u16();
    let body = response.json().await?;
    let jjj = ResponseAssertion {
        status: status_code,
        body,
    };

    let json_body: Value = serde_json::json!(&jjj);
    let mut assert_results = Vec::new();

    for assertion in asserts {
        if let Some(expr) = &assertion.is_true {
            if let Some((operator, index)) = find_operator(&expr) {
                // Extract the value before the operator
                let value = &expr[..index].trim();

                let selected_values = select(&json_body, &value).unwrap();
                let values: Vec<String> = selected_values.iter().map(|v| v.to_string()).collect();
                let res = expr.replace(value, &values[0]);
                let result = parse_expression(&res).unwrap();
                assert_results.push(result);
                println!("is_True: {:?}", result);
            } else {
                let result = parse_expression(&expr).unwrap();
                assert_results.push(result);
            }
        }

        if let Some(expr) = &assertion.is_false {
            if let Some((operator, index)) = find_operator(&expr) {
                // Extract the value before the operator
                let value = &expr[..index].trim();

                let selected_values = select(&json_body, &value).unwrap();
                let values: Vec<String> = selected_values.iter().map(|v| v.to_string()).collect();
                let res = expr.replace(value, &values[0]);
                let result = parse_expression(&res).unwrap();
                assert_results.push(result);
                println!("is_False: {:?}", result);
            } else {
                let result = parse_expression(&expr).unwrap();
                println!("is_False: {:?}", result);
                assert_results.push(result);
            }
        }

        if let Some(condition) = &assertion.is_empty {
            if condition.is_empty() {
                assert_results.push(true);
                println!("is_Empty: {:?}", true);
            } else {
                assert_results.push(false);
                println!("is_Empty: {:?}", false);
            }
        }
    }
    Ok(assert_results)
}

fn parse_expression(expr: &str) -> Result<bool, Box<dyn std::error::Error>> {
    let engine = Engine::new();

    let result = engine.eval_expression::<bool>(expr)?;

    Ok(result)
}

fn find_operator(input: &str) -> Option<(&str, usize)> {
    let operators = &["==", "!=", "<", ">", ">=", "<="];

    for operator in operators {
        if let Some(index) = input.find(operator) {
            return Some((operator, index));
        }
    }

    None
}
