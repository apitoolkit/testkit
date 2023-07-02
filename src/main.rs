mod base_cli;
mod base_request;
use base_cli::BaseCli;
use base_request::base_request;

#[tokio::main]
async fn main() -> Result((), anyhow::Error){
    let base_cli = BaseCli::parse();
    env_logger::init();

    let content = fs::read_to_string(file_path)?;
    let test_plans: Vec<base_request::TestPlan> = serde_yaml::from_str(&content)?;
    for test in parser {
        let result = base_request(&test).await;
        match result {
            Ok(res) => {
                log::debug!("Test passed: {:?}", res);
            }
            Err(err) => {
                println!("{}", err)
            }
        }
    }
}
