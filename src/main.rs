mod base_cli;
mod base_request;
use base_cli::BaseCli;
use base_request::{base_request, TestContext};
use std::fs;

#[tokio::main]
async fn main() {
    env_logger::init();
    setup().await.unwrap()
}

async fn setup() -> Result<(), anyhow::Error> {
    let base_cli = BaseCli::parse();
    let content = fs::read_to_string(base_cli.file.clone())?;
    let ctx = TestContext {
        file: base_cli.file.to_str().unwrap().into(),
        file_source: content.clone(),
        ..Default::default()
    };
    base_request::run(ctx, content).await
}
