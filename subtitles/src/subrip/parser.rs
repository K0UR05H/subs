use super::{
    error::{Error, ErrorKind},
    format::{Line, SubRip, Timecode},
};
use std::{
    borrow::Cow,
    error,
    io::{BufRead, BufReader, Read},
    result,
};

type Result<T> = result::Result<T, Box<dyn error::Error>>;
type ParseResult<T> = result::Result<T, Error>;

pub struct SubRipParser<T: Read> {
    subtitle: BufReader<T>,
    buffer: Line,
}

impl<T: Read> SubRipParser<T> {
    fn parse_next(&mut self) -> ParseResult<Option<SubRip>> {
        let position = match self.parse_position() {
            Ok(Some(position)) => position,
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidPosition, err)),
        };

        let (start, end) = match self.parse_timecode() {
            Ok(Some((start, end))) => (start, end),
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidTimecode, err)),
        };

        let text = match self.parse_texts() {
            Some(text) => text,
            None => return Ok(None),
        };

        Ok(Some(SubRip {
            position,
            start,
            end,
            text,
        }))
    }

    fn parse_position(&mut self) -> Result<Option<usize>> {
        self.read_line(|buf| {
            if buf.is_empty() {
                Ok(None)
            } else {
                let position = String::from_utf8_lossy(buf).parse()?;
                Ok(Some(position))
            }
        })
    }

    fn parse_timecode(&mut self) -> Result<Option<(Timecode, Timecode)>> {
        self.read_line(|buf| {
            if buf.is_empty() {
                Ok(None)
            } else {
                let line: Vec<Cow<str>> = buf
                    .split(|x| [b':', b',', b' '].contains(x))
                    .map(|x| String::from_utf8_lossy(x))
                    .collect();

                let err = "wrong timecode";

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
        })
    }

    fn parse_texts(&mut self) -> Option<Vec<Line>> {
        let mut texts = Vec::new();
        while let Ok(Some(line)) = self.read_line(|buf| {
            if buf.is_empty() {
                Ok(None)
            } else {
                Ok(Some(buf.clone()))
            }
        }) {
            texts.push(line);
        }

        if texts.is_empty() {
            None
        } else {
            Some(texts)
        }
    }

    fn read_line<R, F: FnOnce(&Line) -> Result<R>>(&mut self, f: F) -> Result<R> {
        self.subtitle.read_until(b'\n', &mut self.buffer)?;
        self.trim_start();
        self.trim_newline();

        let result = f(&self.buffer);

        if result.is_err() {
            self.skip_to_next_subtitle();
        }

        self.buffer.clear();
        result
    }

    fn trim_start(&mut self) {
        if let Some(ascii_start) = self.buffer.iter().position(|x| x.is_ascii()) {
            if ascii_start > 0 {
                self.buffer = self.buffer.split_off(ascii_start);
            }
        }
    }

    fn trim_newline(&mut self) {
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
    use std::io::Cursor;

    use super::*;

    fn next<T: Read>(subtitle: T) -> Option<ParseResult<SubRip>> {
        let mut parser = SubRipParser::from(subtitle);
        parser.next()
    }

    fn position<T: Read>(position: T) -> Result<Option<usize>> {
        let mut parser = SubRipParser::from(position);
        parser.parse_position()
    }

    fn timecode<T: Read>(timecode: T) -> Result<Option<(Timecode, Timecode)>> {
        let mut parser = SubRipParser::from(timecode);
        parser.parse_timecode()
    }

    fn texts<T: Read>(t: T) -> Option<Vec<Line>> {
        let mut parser = SubRipParser::from(t);
        parser.parse_texts()
    }

    #[test]
    fn empty_position() {
        let pos = "\n".as_bytes();

        assert!(position(pos).unwrap().is_none());
    }

    #[test]
    fn wrong_position() {
        let pos = "1b\n".as_bytes();

        assert!(position(pos).is_err());
    }

    #[test]
    fn parse_position() {
        let pos = "1433\n".as_bytes();

        assert_eq!(Some(1433), position(pos).unwrap());
    }

    #[test]
    fn empty_timecode() {
        let t = "\n".as_bytes();

        assert!(timecode(t).unwrap().is_none());
    }

    #[test]
    fn parse_timecode() {
        let t = "01:04:00,705 --> 01:04:02,145\n".as_bytes();

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

        let (start, end) = timecode(t).unwrap().unwrap();

        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn bad_format_timecode() {
        let t = "00:00:0,500 --> 00:00:2,00\n".as_bytes();

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

        let (start, end) = timecode(t).unwrap().unwrap();

        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn invalid_timecode() {
        let t = "00:00:00,000\n".as_bytes();

        assert!(timecode(t).is_err());
    }

    #[test]
    fn empty_text() {
        let t = "".as_bytes();

        assert!(texts(t).is_none());
    }

    #[test]
    fn parse_texts() {
        let t = "This is a\nTest\n\n".as_bytes();

        let expected = vec![
            String::from("This is a").into_bytes(),
            String::from("Test").into_bytes(),
        ];
        let actual = texts(t).unwrap();

        assert_eq!(expected, actual);
    }

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

        let actual = next(subtitle).unwrap().unwrap();

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
}
