mod subfind;
mod options {
    pub const PATH: &str = "path";
    pub const PATTERN: &str = "pattern";
}

use clap::{App, Arg};
use regex::Regex;
use std::{env, error::Error};
use subfind::Config;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

fn main() -> Result<(), Box<dyn Error>> {
    let matches = App::new(NAME)
        .version(VERSION)
        .author(AUTHOR)
        .about(ABOUT)
        .arg(
            Arg::with_name(options::PATTERN)
                .value_name("PATTERN")
                .help("pattern to search for")
                .required(true)
                .takes_value(true),
        )
        .arg(
            Arg::with_name(options::PATH)
                .value_name("PATH")
                .help("subtitles path (standard input by default)")
                .default_value("-")
                .hide_default_value(true)
                .multiple(true),
        )
        .get_matches();

    let pattern = matches.value_of(options::PATTERN).unwrap();
    let regex = Regex::new(pattern)?;
    let paths = matches.values_of(options::PATH).unwrap().collect();

    let config = Config { regex, paths };
    subfind::run(config)
}
