use ansi_term::Color::{Blue, Green, Red};
use regex::Regex;
use std::{
    error,
    fs::{self, File},
    io::{self, Read},
    path::Path,
    result,
};
use subtitles::SubRip;

type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub struct Config<'a> {
    pub regex: Regex,
    pub paths: Vec<&'a str>,
}

pub fn run(config: Config) -> Result<()> {
    for path in config.paths {
        if path == "-" {
            find(io::stdin(), &config.regex);
        } else {
            find_in_path(path, &config.regex)?;
        }
    }
    Ok(())
}

fn find_in_path(path: impl AsRef<Path>, regex: &Regex) -> Result<()> {
    let file_type = fs::metadata(&path)?.file_type();

    if file_type.is_dir() {
        for entry in fs::read_dir(&path)? {
            let entry = entry?;
            find_in_path(entry.path(), regex)?;
        }
    } else if file_type.is_file() {
        print_file_name(path.as_ref());
        find(File::open(path)?, regex);
    }

    Ok(())
}

fn print_file_name(path: &Path) {
    if let Some(stem) = path.file_stem() {
        if let Some(stem_str) = stem.to_str() {
            println!("{}", Blue.paint(stem_str))
        }
    }
}

fn find<T: Read>(subtitle: T, regex: &Regex) {
    let parser = subtitles::open(subtitle);

    for entry in parser {
        match entry {
            Ok(sub) => print_matches(sub, regex),
            Err(err) => eprintln!("{}: {}", Red.paint("Error"), err),
        }
    }
}

fn print_matches(subtitle: SubRip, regex: &Regex) {
    for line in subtitle.text {
        let mut last_match = 0;
        for reg_match in regex.find_iter(&line) {
            let unmatched = &line[last_match..reg_match.start()];
            let matched = reg_match.as_str();
            print!("{}{}", unmatched, Green.paint(matched));

            last_match = reg_match.end();
        }

        if last_match > 0 {
            println!("{}", &line[last_match..]);
        }
    }
}
