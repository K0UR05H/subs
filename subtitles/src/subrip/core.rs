use super::format::Timecode;
use std::{error, result};

pub type Result<T> = result::Result<T, Box<dyn error::Error>>;

pub fn parse_position(line: String) -> Result<Option<usize>> {
    if line.is_empty() {
        Ok(None)
    } else {
        let position = line.parse()?;
        Ok(Some(position))
    }
}

pub fn parse_timecode(line: String) -> Result<Option<(Timecode, Timecode)>> {
    if line.is_empty() {
        Ok(None)
    } else {
        let line: Vec<&str> = line.split(&[':', ',', ' '][..]).collect();

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

pub fn parse_text(line: String) -> Option<String> {
    if line.is_empty() {
        None
    } else {
        Some(line)
    }
}

pub fn trim_newline(line: &mut String) {
    if line.ends_with('\n') {
        line.pop();
        if line.ends_with('\r') {
            line.pop();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_position() {
        let position = String::new();

        assert!(parse_position(position).unwrap().is_none());
    }

    #[test]
    fn wrong_position() {
        let position = String::from("1b");

        assert!(parse_position(position).is_err());
    }

    #[test]
    fn position() {
        let position = String::from("1433");

        assert_eq!(Some(1433), parse_position(position).unwrap());
    }

    #[test]
    fn empty_timecode() {
        let timecode = String::new();

        assert!(parse_timecode(timecode).unwrap().is_none());
    }

    #[test]
    fn bad_format_timecode() {
        let timecode = String::from("00:00:0,500 --> 00:00:2,00");

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
        let timecode = String::from("00:00:00,000");

        assert!(parse_timecode(timecode).is_err());
    }

    #[test]
    fn negative_timecode() {
        let timecode = String::from("00:-1:-58,-240 --> 00:-1:-55,-530");

        let expected_start = Timecode {
            hours: 0,
            minutes: -1,
            seconds: -58,
            milliseconds: -240,
        };
        let expected_end = Timecode {
            hours: 0,
            minutes: -1,
            seconds: -55,
            milliseconds: -530,
        };

        let (start, end) = parse_timecode(timecode).unwrap().unwrap();

        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn timecode() {
        let timecode = String::from("01:04:00,705 --> 01:04:02,145");

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
        let text = String::new();

        assert!(parse_text(text).is_none());
    }

    #[test]
    fn text() {
        let text = String::from("This is a test");

        assert_eq!("This is a test", parse_text(text).unwrap());
    }
}
