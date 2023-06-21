mod base_cli;
mod parser_yaml;
// mod base_request;

use base_cli::BaseCli;
use parser_yaml::parse_yaml_file;
// use base_request::BaseRequest;


fn main() {
    let base_cli = BaseCli::parse();

    let parser = match parse_yaml_file(&base_cli.file) {
        Ok(test_configs) => test_configs,
        Err(err) => {
            eprintln!("Failed to parse YAML file: {}", err);
            return;
        }
    };

    for test in parser {

        println!("Test passed: {:?}", test.url);
        // let result = BaseRequest(&test).await;

        // match result {
        //     Ok(test_result) => {
        //         println!("Test passed: {:?}", test_result);
        //     }
        //     Err(err) => {
        //         eprintln!("Test failed: {}", err);
        //     }
        // }
    }

}
