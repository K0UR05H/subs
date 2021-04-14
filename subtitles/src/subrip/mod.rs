mod core;
mod error;
pub mod format;
mod parser;

use parser::SubRipParser;
use std::io::Read;

pub fn open<T: Read>(subtitle: T) -> SubRipParser<T> {
    SubRipParser::from(subtitle)
}
