mod base_cli;
mod base_request;
use std::fs;
use base_cli::BaseCli;
use base_request::base_request;

#[tokio::main]
async fn main() {
    env_logger::init();
    setup().await.unwrap()
}


async fn setup() -> Result<(), anyhow::Error> {
    let base_cli = BaseCli::parse();
    let content = fs::read_to_string(base_cli.file)?;
    let test_plans: Vec<base_request::TestPlan> = serde_yaml::from_str(&content)?;

    for test in test_plans {
        let result = base_request(&test).await;
        match result {
            Ok(res) => {
                log::debug!("Test passed: {:?}", res);
            }
            Err(err) => {
                log::error!("{}", err)
            }
        }
    }

    Ok(())
}
