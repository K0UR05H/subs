use super::format::{Line, SubRip, Timecode};
use std::{
    io::{BufRead, BufReader, Read},
    str::Utf8Error,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct SubRipParser<T: Read> {
    subtitle: BufReader<T>,
    buffer: Line,
}

impl<T: Read> SubRipParser<T> {
    pub fn new(subtitle: T) -> SubRipParser<T> {
        SubRipParser {
            subtitle: BufReader::new(subtitle),
            buffer: Vec::new(),
        }
    }

    fn parse_next(&mut self) -> Result<Option<SubRip>> {
        let position = match self.parse_position()? {
            Some(position) => position,
            None => return Ok(None),
        };
        let (start, end) = match self.parse_timecode()? {
            Some((start, end)) => (start, end),
            None => return Ok(None),
        };
        let text = match self.parse_text() {
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
                let position = buf
                    .iter()
                    .filter(|&x| x.is_ascii_digit())
                    .map(|x| (x - b'0').to_string())
                    .fold(String::new(), |acc, x| acc + &x)
                    .parse()?;
                Ok(Some(position))
            }
        })
    }

    fn parse_timecode(&mut self) -> Result<Option<(Timecode, Timecode)>> {
        self.read_line(|buf| {
            if buf.is_empty() {
                Ok(None)
            } else {
                let line: Vec<std::result::Result<&str, Utf8Error>> = buf
                    .split(|x| [b':', b',', b' '].contains(x))
                    .map(|x| std::str::from_utf8(x))
                    .collect();

                let start = Timecode {
                    hours: line[0]?.parse()?,
                    minutes: line[1]?.parse()?,
                    seconds: line[2]?.parse()?,
                    milliseconds: line[3]?.parse()?,
                };
                let end = Timecode {
                    hours: line[5]?.parse()?,
                    minutes: line[6]?.parse()?,
                    seconds: line[7]?.parse()?,
                    milliseconds: line[8]?.parse()?,
                };

                Ok(Some((start, end)))
            }
        })
    }

    fn parse_text(&mut self) -> Option<Vec<Line>> {
        let mut text = Vec::new();
        while let Ok(Some(line)) = self.read_line(|buf| {
            if buf.is_empty() {
                Ok(None)
            } else {
                Ok(Some(buf.clone()))
            }
        }) {
            text.push(line);
        }

        if text.is_empty() {
            None
        } else {
            Some(text)
        }
    }

    fn read_line<R, F: FnOnce(&Line) -> Result<R>>(&mut self, f: F) -> Result<R> {
        self.subtitle.read_until(b'\n', &mut self.buffer)?;
        self.trim_newline();

        let result = f(&self.buffer);

        self.buffer.clear();
        result
    }

    fn trim_newline(&mut self) {
        if self.buffer.ends_with(&[b'\n']) {
            self.buffer.pop();
            if self.buffer.ends_with(&[b'\r']) {
                self.buffer.pop();
            }
        }
    }
}

impl<T: Read> Iterator for SubRipParser<T> {
    type Item = Result<SubRip>;

    fn next(&mut self) -> Option<Self::Item> {
        self.parse_next().transpose()
    }
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use super::*;

    #[test]
    fn skip_dom() {
        let bom: [u8; 3] = [0xef, 0xbb, 0xbf];
        let subtitle = "\
1433
01:04:00,705 --> 01:04:02,145
It's only after
we've lost everything"
            .as_bytes();
        let subtitle = [&bom, subtitle].concat();
        let mut parser = SubRipParser::new(Cursor::new(subtitle));

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
    }

    #[test]
    fn empty_position() {
        let position = "\n".as_bytes();
        let mut parser = SubRipParser::new(position);

        assert!(parser.parse_position().unwrap().is_none());
    }

    #[test]
    fn parse_position() {
        let position = "1433\n".as_bytes();
        let mut parser = SubRipParser::new(position);

        assert_eq!(Some(1433), parser.parse_position().unwrap());
    }

    #[test]
    fn empty_timecode() {
        let timecode = "\n".as_bytes();
        let mut parser = SubRipParser::new(timecode);

        assert!(parser.parse_timecode().unwrap().is_none());
    }

    #[test]
    fn parse_timecode() {
        let timecode = "01:04:00,705 --> 01:04:02,145\n".as_bytes();
        let mut parser = SubRipParser::new(timecode);

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

        let (start, end) = parser.parse_timecode().unwrap().unwrap();
        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn bad_format_timecode() {
        let timecode = "00:00:0,500 --> 00:00:2,00\n".as_bytes();
        let mut parser = SubRipParser::new(timecode);

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

        let (start, end) = parser.parse_timecode().unwrap().unwrap();
        assert_eq!(expected_start, start);
        assert_eq!(expected_end, end);
    }

    #[test]
    fn empty_text() {
        let text = "".as_bytes();
        let mut parser = SubRipParser::new(text);

        assert!(parser.parse_text().is_none());
    }

    #[test]
    fn parse_text() {
        let text = "It's only after\nwe've lost everything\n\n".as_bytes();
        let mut parser = SubRipParser::new(text);
        let result = parser.parse_text().unwrap();
        let expected = vec![
            String::from("It's only after").into_bytes(),
            String::from("we've lost everything").into_bytes(),
        ];

        assert_eq!(expected, result);
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

        let mut parser = SubRipParser::new(sub.as_bytes());

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
}
