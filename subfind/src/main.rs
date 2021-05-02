use clap::{App, Arg};
use crossterm::style::Colorize;
use regex::Regex;
use std::{
    env, error,
    ffi::OsStr,
    fs::{self, File},
    io::{self, Read},
    path::Path,
    result,
};
use subtitles::SubRip;

type Result<T> = result::Result<T, Box<dyn error::Error>>;

const NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");
const AUTHOR: &str = env!("CARGO_PKG_AUTHORS");
const ABOUT: &str = env!("CARGO_PKG_DESCRIPTION");

mod options {
    pub const PATH: &str = "path";
    pub const PATTERN: &str = "pattern";
}

fn main() -> Result<()> {
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
    let paths = matches.values_of(options::PATH).unwrap();

    for path in paths {
        if path == "-" {
            find_in_stdin(&regex);
        } else {
            find_in_path(&regex, path)?;
        }
    }

    Ok(())
}

fn find_in_stdin(regex: &Regex) {
    let stdin = io::stdin();
    let handle = stdin.lock();
    find_matches(&regex, handle);
}

fn find_in_path(regex: &Regex, path: impl AsRef<Path>) -> Result<()> {
    let file_type = fs::metadata(&path)?.file_type();

    if file_type.is_dir() {
        let entries = fs::read_dir(&path)?;
        for entry in entries {
            let entry = entry?;
            find_in_path(regex, entry.path())?;
        }
    } else if file_type.is_file() {
        print_file_name(path.as_ref());
        let file = File::open(path)?;
        find_matches(regex, file);
    }

    Ok(())
}

fn find_matches<T: Read>(regex: &Regex, subtitle: T) {
    let parser = subtitles::open(subtitle);

    for entry in parser {
        match entry {
            Ok(sub) => print_matches(sub, regex),
            Err(err) => eprintln!("{}: {}", "Error".red(), err),
        }
    }
}

fn print_file_name(path: &Path) {
    let file_name = path
        .file_stem()
        .unwrap_or_else(|| OsStr::new(""))
        .to_str()
        .unwrap_or("");

    println!("{}", file_name.blue());
}

fn print_matches(subtitle: SubRip, regex: &Regex) {
    for line in subtitle.text {
        if let Some(matched) = regex.find(&line) {
            println!(
                "{}: {}{}{}",
                subtitle.position,
                &line[..matched.start()],
                line[matched.start()..matched.end()].green(),
                &line[matched.end()..]
            );
        }
    }
}
