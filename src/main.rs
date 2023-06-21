mod base_cli;
mod base_request;
mod parser_yaml;

use base_cli::BaseCli;
use base_request::base_request;
#[tokio::main]

async fn main() {
    let base_cli = BaseCli::parse();

    let parser = match parser_yaml::parse_yaml_file(&base_cli.file) {
        Ok(request_configs) => request_configs,
        Err(err) => {
            eprintln!("Failed to parse YAML file: {}", err);
            return;
        }
    };

    for test in parser {
        let result = base_request(&test).await;

        match result {
            Ok(test_result) => {
                println!("Test passed: {:?}", test_result);
            }
            Err(err) => {
                eprintln!("Test failed: {}", err);
            }
        }
    }
}
