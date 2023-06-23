pub use crate::base_request::TestPlan;
use serde_yaml;
use std::fs;
use std::path::Path;

// parse yaml file
pub fn parse_yaml_file<P: AsRef<Path>>(
    file_path: P,
) -> Result<Vec<TestPlan>, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(file_path)?;
    let request_configs: Vec<TestPlan> = serde_yaml::from_str(&content)?;
    Ok(request_configs)
}
