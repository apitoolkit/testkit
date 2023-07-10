mod base_cli;
mod base_request;
extern crate log;
use base_cli::BaseCli;
use base_request::TestContext;
use env_logger::Builder;
use log::LevelFilter;
use std::fs;
use std::str::FromStr;

#[tokio::main]
async fn main() {
    let base_cli = BaseCli::parse();
    let mut builder = Builder::from_default_env();

    builder
        .format_timestamp(None)
        .format_target(true)
        .filter_level(LevelFilter::from_str(&base_cli.log_level).unwrap_or(LevelFilter::Info))
        .filter_module("jsonpath_lib", LevelFilter::Info)
        .init();
    setup(base_cli).await.unwrap()
}

async fn setup(base_cli: BaseCli) -> Result<(), anyhow::Error> {
    let content = fs::read_to_string(base_cli.file.clone())?;
    let ctx = TestContext {
        file: base_cli.file.to_str().unwrap().into(),
        file_source: content.clone(),
        ..Default::default()
    };
    base_request::run(ctx, content).await
}
