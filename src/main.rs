mod base_cli;
mod base_request;
use base_cli::Commands;
use base_request::TestContext;
extern crate dotenv;
use clap::Parser;
use dotenv::dotenv;
use env_logger::Builder;
use log::LevelFilter;
use std::{env, fs, path::PathBuf, str::FromStr};

mod app;
extern crate log;

#[tokio::main]
async fn main() {
    dotenv().ok();
    let cli_instance = base_cli::Cli::parse();

    let mut builder = env_logger::Builder::from_default_env();
    builder
        .format_timestamp(None)
        .format_target(true)
        .filter_level(LevelFilter::from_str(&cli_instance.log_level).unwrap_or(LevelFilter::Info))
        .filter_module("jsonpath_lib", LevelFilter::Info)
        .init();

    match cli_instance.command {
        None | Some(Commands::App {}) => {
            app::app_init();
        }
        Some(Commands::Test { file }) => cli(file).await.unwrap(),
    }
}

async fn cli(file: PathBuf) -> Result<(), anyhow::Error> {
    let content = fs::read_to_string(file.clone())?;
    let ctx = TestContext {
        file: file.to_str().unwrap().into(),
        file_source: content.clone(),
        ..Default::default()
    };
    base_request::run(ctx, content).await
}
