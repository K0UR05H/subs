use super::format::{Line, Timecode};
use std::{borrow::Cow, error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub fn parse_position(line: &[u8]) -> Result<Option<usize>> {
    if line.is_empty() {
        Ok(None)
    } else {
        let position = String::from_utf8_lossy(line).parse()?;
        Ok(Some(position))
    }
}

pub fn parse_timecode(line: &[u8]) -> Result<Option<(Timecode, Timecode)>> {
    if line.is_empty() {
        Ok(None)
    } else {
        let line: Vec<Cow<str>> = line
            .split(|x| [b':', b',', b' '].contains(x))
            .map(|x| String::from_utf8_lossy(x))
            .collect();

        let err = "wrong timecode format";

        let start = Timecode {
            hours: line.get(0).ok_or(err)?.parse()?,
            minutes: line.get(1).ok_or(err)?.parse()?,
            seconds: line.get(2).ok_or(err)?.parse()?,
            milliseconds: line.get(3).ok_or(err)?.parse()?,
        };
        let end = Timecode {
            hours: line.get(5).ok_or(err)?.parse()?,
            minutes: line.get(6).ok_or(err)?.parse()?,
            seconds: line.get(7).ok_or(err)?.parse()?,
            milliseconds: line.get(8).ok_or(err)?.parse()?,
        };

        Ok(Some((start, end)))
    }
}

pub fn parse_text(line: &[u8]) -> Option<Line> {
    if line.is_empty() {
        None
    } else {
        Some(line.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_position() {
        let position = "".as_bytes();

        assert!(parse_position(position).unwrap().is_none());
    }

    #[test]
    fn wrong_position() {
        let position = "1b".as_bytes();

        assert!(parse_position(position).is_err());
    }

    #[test]
    fn position() {
        let position = "1433".as_bytes();

        assert_eq!(Some(1433), parse_position(position).unwrap());
    }

    #[test]
    fn empty_timecode() {
        let timecode = "".as_bytes();

        assert!(parse_timecode(timecode).unwrap().is_none());
    }

    #[test]
    fn bad_format_timecode() {
        let timecode = "00:00:0,500 --> 00:00:2,00".as_bytes();

        let expected_start = Timecode {
            hours: 0,
            minutes: 0,
            seconds: 0,
            milliseconds: 500,
        };
        let expected_end = Timecode {
            hours: 0,
            minutes: 0,
            seconds: 2,
            milliseconds: 0,
        };

        let (start, end) = parse_timecode(timecode).unwrap().unwrap();

        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn invalid_timecode() {
        let timecode = "00:00:00,000".as_bytes();

        assert!(parse_timecode(timecode).is_err());
    }

    #[test]
    fn timecode() {
        let timecode = "01:04:00,705 --> 01:04:02,145".as_bytes();

        let expected_start = Timecode {
            hours: 1,
            minutes: 4,
            seconds: 0,
            milliseconds: 705,
        };
        let expected_end = Timecode {
            hours: 1,
            minutes: 4,
            seconds: 2,
            milliseconds: 145,
        };

        let (start, end) = parse_timecode(timecode).unwrap().unwrap();

        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn empty_text() {
        let text = "".as_bytes();

        assert!(parse_text(text).is_none());
    }

    #[test]
    fn text() {
        let text = "This is a test".as_bytes();

        assert_eq!(text, parse_text(text).unwrap());
    }
}
