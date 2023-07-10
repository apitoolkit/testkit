use clap::{Arg, Command};
use std::path::PathBuf;

pub struct BaseCli {
    pub file: PathBuf,
    pub log_level: String,
}

impl BaseCli {
    pub fn parse() -> BaseCli {
        let matches = Command::new("Api Test")
            .version("0.1.0")
            .about("Api load testing CLI")
            .arg(
                Arg::new("file")
                    .short('f')
                    .long("file")
                    .value_name("FILE")
                    .help("Sets the YAML test configuration file")
                    .required(true),
            )
            .arg(
                Arg::new("log")
                    .short('l')
                    .long("log")
                    .value_name("LOG LEVEL")
                    .help("Sets the log level to be used. Eg trace, debug, warn, info, error"),
            )
            .get_matches();

        let file = matches.get_one::<String>("file").unwrap().to_owned();
        let file = PathBuf::from(file);

        let log_level = matches
            .get_one::<String>("log")
            .unwrap_or(&"info".to_string())
            .to_owned();
        BaseCli { file, log_level }
    }
}
