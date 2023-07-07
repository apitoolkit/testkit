mod base_cli;
mod base_request;
use base_cli::BaseCli;
use base_request::{TestContext, base_request};
use std::fs;

#[tokio::main]
async fn main() {
    env_logger::init();
    setup().await.unwrap()
}

async fn setup() -> Result<(), anyhow::Error> {
    let base_cli = BaseCli::parse();
    let content = fs::read_to_string(base_cli.file.clone())?;
    let test_plans: Vec<base_request::TestPlan> = serde_yaml::from_str(&content)?;

    let ctx = TestContext{
        file: base_cli.file.to_str().unwrap().into(),
        file_source: content,
        ..Default::default()
    };

    for test in test_plans {
        let result = base_request(ctx.clone(), &test).await;
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
