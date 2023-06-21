pub use crate::base_request::RequestConfig;
use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

// parse yaml file
pub fn parse_yaml_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<RequestConfig>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let request_configs: Vec<RequestConfig> = serde_yaml::from_str(&content)?;
    Ok(request_configs)
}
