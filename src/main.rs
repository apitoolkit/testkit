mod base_cli;
mod base_request;
extern crate log;
use base_cli::BaseCli;
use base_request::TestContext;
use env_logger::Builder;
use log::LevelFilter;
use std::fs;

#[tokio::main]
async fn main() {
    let mut builder = Builder::from_default_env();

    builder
        .format_timestamp(None)
        .format_target(false)
        .filter(None, LevelFilter::Info)
        .init();
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
