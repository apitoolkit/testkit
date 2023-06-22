use std::collections::HashMap;

use reqwest::{Client, header::{HeaderMap, HeaderValue, CONTENT_TYPE, AUTHORIZATION}, Method};
use serde::{Deserialize, Serialize};
use serde_yaml::Value;
use log;


#[derive(Debug, Serialize, Deserialize)]
pub struct  ApiPlan{
    name: String,
    request: RequestConfig,
    response: Option<RequestResult>,
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<HashMap<String, String>>,
    json: Option<Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RequestResult {
    status_code: u16,
    headers: Option<HashMap<String, String>>,
    }

pub async fn base_request(
    stage: &ApiPlan,
) -> Result<(), Box<dyn std::error::Error>> {
     log::info!("Executing stage: {}", stage.name);

    let client = Client::new();

    let method = match stage.request.method.as_str() {
        "GET" => Method::GET,
        "POST" => Method::POST,
        "PUT" => Method::PUT,
        "DELETE" => Method::DELETE,
        _ => panic!("Unsupported HTTP method"),
    };


    let mut request_builder = client.request(method, &stage.request.url);

    if let Some(headers) = &stage.request.headers {
        let mut header_map = HeaderMap::new();
        for (name, value) in headers {
            let header_value = HeaderValue::from_str(value)?;
                match name.as_str() {
                "Content-Type" =>header_map.insert(CONTENT_TYPE, header_value),
               "AUTHORIZATION" => header_map.insert(AUTHORIZATION, header_value),
               _ => panic!("Unsupported HTTP header"),

            };

        }
        request_builder = request_builder.headers(header_map);
    }

    let response = request_builder.send().await?;
    let status_code = response.status().as_u16();
    // let body = response.json().await?;
    match &stage.response {
        Some(response_config) => {
            if status_code != response_config.status_code {
                return Err(format!(
                    "Received unexpected status code. Expected: {}, Received: {}",
                    response_config.status_code, status_code
                )
                .into());
            }

            log::info!("Stage executed successfully!");
        }
        None => {
            log::info!("Stage executed successfully!");
        }
    } 

    Ok(())

}
