use std::collections::HashMap;

use log;
use reqwest::{Client, Method, Response};
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
    // outputs: Option<RequestResult>,
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
    status_code: u16,
    headers: Option<HashMap<String, String>>,
}

pub async fn base_request(stage: &TestPlan) -> Result<(), Box<dyn std::error::Error>> {
    println!("================================================================================================================");

    log::info!("Executing Test: {}", stage.name);
    println!("================================================================================================================");

    let client = Client::new();
    //  let mut results: Vec<_> = Vec::new();

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
        let status_code = response.status().as_u16();
        let body = response.text().await?;
        let response_json: Value = serde_json::from_str(&body)?;

        // let assert_result = check_assertion(&stage.asserts, response_json);
        // println!("{:?}\n",response_json)
    }
    println!("================================================================================================================");
    Ok(())
}

// pub fn check_assertion(asserts: &[Assert], response: Value)  {
   

    
//     for assert in asserts{
//         if let Some(expr) = &assert.is_true {
//             // println!("{:?}",expr);
//             let result = parse_expression(expr, &response.);
//             println!("{:?}",result)
//             // assert_results.push(result);
//         }    
    

//     }

// }

