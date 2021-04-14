use std::{error, fmt};

#[derive(Debug)]
pub struct Error {
    kind: ErrorKind,
    error: Box<dyn error::Error>,
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorKind {
    InvalidPosition,
    InvalidTimecode,
    InvalidText,
}

impl ErrorKind {
    fn as_str(&self) -> &'static str {
        match self {
            ErrorKind::InvalidPosition => "invalid position",
            ErrorKind::InvalidTimecode => "invalid timecode",
            ErrorKind::InvalidText => "invalid text",
        }
    }
}

impl Error {
    pub fn new<E>(kind: ErrorKind, error: E) -> Error
    where
        E: Into<Box<dyn error::Error>>,
    {
        Error {
            kind,
            error: error.into(),
        }
    }

    pub fn kind(&self) -> ErrorKind {
        self.kind
    }
}

impl fmt::Display for Error {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(fmt, "{}: {}", self.kind.as_str(), self.error)
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        self.error.source()
    }
}
