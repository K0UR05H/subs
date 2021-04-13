pub mod format;
mod parser;
mod error;

use parser::SubRipParser;
use std::io::Read;

pub fn open<T: Read>(subtitle: T) -> SubRipParser<T> {
    SubRipParser::from(subtitle)
}
