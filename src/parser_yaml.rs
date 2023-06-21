use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize)]
pub struct TestConfig {
    pub url: String,
    pub method: String,
    pub headers: Option<Vec<Header>>,
    pub body: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    pub name: String,
    pub value: String,
}

pub fn parse_yaml_file<P: AsRef<Path>>(file_path: P) -> Result<Vec<TestConfig>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let test_configs: Vec<TestConfig> = serde_yaml::from_str(&content)?;
    Ok(test_configs)
}
