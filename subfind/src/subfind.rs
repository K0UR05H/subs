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
    let text = subtitle.text.join("\n");
    let mut last_uncolored = 0;

    for mat in regex.find_iter(&text) {
        print!(
            "{}{}",
            &text[last_uncolored..mat.start()],
            Green.paint(&text[mat.start()..mat.end()])
        );
        last_uncolored = mat.end();
    }

    if last_uncolored != 0 {
        println!("{}", &text[last_uncolored..]);
    }
}
