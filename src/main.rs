mod base_cli;
mod base_request;
use base_request::TestContext;
use log::LevelFilter;
use std::fs;
use std::str::FromStr;
use clap::Parser;
use std::path::PathBuf;
use base_cli::{Commands, Cli};
use dioxus::prelude::hot_reload_init;

mod app;
extern crate log;

#[tokio::main]
async fn main() {
    // hot_reload_init!();

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
        },
        Some(Commands::Test{file}) => {
            cli(file).await.unwrap()        
        }

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
