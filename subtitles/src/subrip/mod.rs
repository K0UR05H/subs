mod core;
mod error;
pub mod format;
mod parser;

use parser::SubRipParser;
use std::io::Read;

/// Create a new parser for `subtitle`.
///
/// `subtitle` must be in SubRip (.srt) format.
pub fn open<T: Read>(subtitle: T) -> SubRipParser<T> {
    SubRipParser::from(subtitle)
}
