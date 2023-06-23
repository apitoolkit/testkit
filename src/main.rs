mod base_cli;
mod base_request;
mod parser_yaml;

use base_cli::BaseCli;
use base_request::base_request;
#[tokio::main]

async fn main() {
    let base_cli = BaseCli::parse();

    env_logger::init();

    let parser = match parser_yaml::parse_yaml_file(&base_cli.file) {
        Ok(request_configs) => request_configs,
        Err(err) => {
            eprintln!("Failed to parse YAML file: {}", err);
            return;
        }
    };
    for test in parser {
        let result = base_request(&test).await;
        log::debug!("Test passed: {:?}", result);
    }
}
