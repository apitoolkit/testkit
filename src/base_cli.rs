use clap::{Arg,Command};
use std::path::PathBuf;

pub struct BaseCli {
    pub file:PathBuf ,
}

impl BaseCli {
    pub fn parse() -> BaseCli {

        let matches = Command::new("Api Test").arg(
            Arg::new("file")
                .short('f')
                .long("file")
                .value_name("FILE")
                .help("Sets the YAML test configuration file")
                .required(true),
        )
        .get_matches();

        let file = matches.get_one::<String>("file").unwrap().to_owned();
        let file = PathBuf::from(file);
        BaseCli { file }
    }
}
