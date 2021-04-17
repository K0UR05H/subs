use super::{
    core::*,
    error::{Error, ErrorKind},
    format::SubRip,
};
use std::{
    io::{BufRead, BufReader, Read},
    result,
};

type ParseResult<T> = result::Result<T, Error>;

pub struct SubRipParser<T: Read> {
    subtitle: BufReader<T>,
    buffer: Vec<u8>,
}

impl<T: Read> SubRipParser<T> {
    fn parse_next(&mut self) -> ParseResult<Option<SubRip>> {
        let position = match self.read_line(true, |line| parse_position(line)) {
            Ok(Some(position)) => position,
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidPosition, err)),
        };

        let (start, end) = match self.read_line(true, |line| parse_timecode(line)) {
            Ok(Some((start, end))) => (start, end),
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidTimecode, err)),
        };

        let mut text = Vec::new();
        loop {
            match self.read_line(false, |line| Ok(parse_text(line))) {
                Ok(Some(t)) => text.push(t),
                Ok(None) => break,
                Err(err) => return Err(Error::new(ErrorKind::InvalidText, err)),
            }
        }

        Ok(Some(SubRip {
            position,
            start,
            end,
            text,
        }))
    }

    fn read_line<R, F: FnOnce(&[u8]) -> Result<R>>(
        &mut self,
        skip_non_ascii: bool,
        f: F,
    ) -> Result<R> {
        self.subtitle.read_until(b'\n', &mut self.buffer)?;
        self.trim_end();

        if skip_non_ascii {
            self.skip_non_ascii();
        }

        let result = f(&self.buffer);

        if result.is_err() {
            self.skip_to_next_subtitle();
        }

        self.buffer.clear();
        result
    }

    fn skip_non_ascii(&mut self) {
        if let Some(ascii_start) = self.buffer.iter().position(|x| x.is_ascii()) {
            if ascii_start > 0 {
                self.buffer = self.buffer.split_off(ascii_start);
            }
        }
    }

    fn trim_end(&mut self) {
        if self.buffer.ends_with(&[b'\n']) {
            self.buffer.pop();
            if self.buffer.ends_with(&[b'\r']) {
                self.buffer.pop();
            }
        }
    }

    fn skip_to_next_subtitle(&mut self) {
        while let Ok(read) = self.subtitle.read_until(b'\n', &mut self.buffer) {
            if (read == 0)
                | ((read == 1) && self.buffer.ends_with(&[b'\n']))
                | ((read == 2) && self.buffer.ends_with(&[b'\r', b'\n']))
            {
                break;
            }
        }
    }
}

impl<T: Read> From<T> for SubRipParser<T> {
    fn from(subtitle: T) -> Self {
        SubRipParser {
            subtitle: BufReader::new(subtitle),
            buffer: Vec::new(),
        }
    }
}

impl<T: Read> Iterator for SubRipParser<T> {
    type Item = ParseResult<SubRip>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next().transpose()
    }
}

#[cfg(test)]
mod tests {
    use super::{super::format::Timecode, *};
    use std::io::Cursor;

    #[test]
    fn skip_dom() {
        let bom: [u8; 3] = [0xef, 0xbb, 0xbf];
        let subtitle = "\
1433
01:04:00,705 --> 01:04:02,145
This is a
Test"
            .as_bytes();
        let subtitle = Cursor::new([&bom, subtitle].concat());

        let expected = SubRip {
            position: 1433,
            start: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 0,
                milliseconds: 705,
            },
            end: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 2,
                milliseconds: 145,
            },
            text: vec![
                String::from("This is a").into_bytes(),
                String::from("Test").into_bytes(),
            ],
        };

        let actual = SubRipParser::from(subtitle).next().unwrap().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parser_iteration() {
        let sub = "\
1433
01:04:00,705 --> 01:04:02,145
It's only after
we've lost everything

1434
01:04:02,170 --> 01:04:04,190
that we're free to do anything.";

        let mut parser = SubRipParser::from(sub.as_bytes());

        // First
        let expected = SubRip {
            position: 1433,
            start: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 0,
                milliseconds: 705,
            },
            end: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 2,
                milliseconds: 145,
            },
            text: vec![
                String::from("It's only after").into_bytes(),
                String::from("we've lost everything").into_bytes(),
            ],
        };
        assert_eq!(expected, parser.next().unwrap().unwrap());

        // Second
        let expected = SubRip {
            position: 1434,
            start: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 2,
                milliseconds: 170,
            },
            end: Timecode {
                hours: 1,
                minutes: 4,
                seconds: 4,
                milliseconds: 190,
            },
            text: vec![String::from("that we're free to do anything.").into_bytes()],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());

        // End
        assert!(parser.next().is_none());
    }

    #[test]
    fn skip_invalid_subtitle() {
        let sub = "\
1
00:00:00,000
Invalid

2
01:02:03,456 --> 07:08:09,101
This is a Test";
        let mut parser = SubRipParser::from(sub.as_bytes());

        assert!(parser.next().unwrap().is_err());

        let expected = SubRip {
            position: 2,
            start: Timecode {
                hours: 1,
                minutes: 2,
                seconds: 3,
                milliseconds: 456,
            },
            end: Timecode {
                hours: 7,
                minutes: 8,
                seconds: 9,
                milliseconds: 101,
            },
            text: vec![String::from("This is a Test").into_bytes()],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }

    #[test]
    fn parse_non_utf8_text() {
        let text = [b'\xff', b'\x74', b'\x65', b'\x73', b'\x74'];
        let subtitle = "\
1
01:02:03,456 --> 07:08:09,101
";
        let subtitle = [subtitle.as_bytes(), &text].concat();

        let mut parser = SubRipParser::from(Cursor::new(subtitle));

        let expected = SubRip {
            position: 1,
            start: Timecode {
                hours: 1,
                minutes: 2,
                seconds: 3,
                milliseconds: 456,
            },
            end: Timecode {
                hours: 7,
                minutes: 8,
                seconds: 9,
                milliseconds: 101,
            },
            text: vec![text.to_vec()],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }
}
