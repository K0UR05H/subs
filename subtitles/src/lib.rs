#![deny(missing_docs)]

//! A simple library for parsing subtitles.
//!
//! # Usage
//!
//! ```no_run
//! # use std::io::Error;
//! use std::fs::File;
//!
//! let file = File::open("/path/to/subtitle.srt")?;
//! let parser = subtitles::open(file);
//!
//! for subtitle in parser {
//!     match subtitle {
//!         Ok(sub) => println!("{}", sub),
//!         Err(err) => eprintln!("{}", err),
//!     }
//! }
//! # Ok::<(), Error>(())
//! ```

mod subrip;

pub use subrip::format::SubRip;
pub use subrip::open;
