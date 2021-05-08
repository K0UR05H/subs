use clap::{App, Arg};
use std::{error::Error, fs::File};

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
            Arg::with_name("file")
                .short("f")
                .long("file")
                .value_name("FILE")
                .required(true),
        )
        .get_matches();

    let path = matches.value_of("file").unwrap();
    let file = File::open(path)?;

    let parser = subtitles::open(file);
    for entry in parser {
        match entry {
            Ok(sub) => {
                for line in sub.text {
                    println!("{}", line);
                }
            }
            Err(err) => eprintln!("Error: {}", err),
        }
    }

    Ok(())
}
