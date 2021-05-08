use super::{
    core::*,
    error::{Error, ErrorKind},
    format::SubRip,
};
use encoding_rs::{Decoder, Encoding, UTF_16LE, UTF_8};
use std::{
    io::{BufRead, BufReader, Read},
    result,
};

type ParseResult<T> = result::Result<T, Error>;

pub struct SubRipParser<T: Read> {
    subtitle: BufReader<T>,
    decoder: Option<Decoder>,
}

impl<T: Read> SubRipParser<T> {
    fn parse_next(&mut self) -> ParseResult<Option<SubRip>> {
        // Parse position
        let line = match self.skip_empty_lines() {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidPosition, err)),
        };
        let position =
            parse_position(line).map_err(|err| Error::new(ErrorKind::InvalidPosition, err))?;

        // Parse timecode
        let line = match self.skip_empty_lines() {
            Ok(Some(line)) => line,
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidTimecode, err)),
        };
        let (start, end) =
            parse_timecode(line).map_err(|err| Error::new(ErrorKind::InvalidTimecode, err))?;

        // Parse text
        let mut text = Vec::new();
        loop {
            match self.next_line() {
                Ok(Some(line)) => {
                    if line.is_empty() {
                        break;
                    } else {
                        text.push(line)
                    }
                }
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

    fn skip_empty_lines(&mut self) -> Result<Option<String>> {
        loop {
            match self.next_line()? {
                Some(line) => {
                    if !line.is_empty() {
                        break Ok(Some(line));
                    }
                }
                None => break Ok(None),
            }
        }
    }

    fn next_line(&mut self) -> Result<Option<String>> {
        let mut buf = Vec::new();
        self.subtitle.read_until(b'\n', &mut buf)?;

        let decoder = self.decoder.get_or_insert_with(|| {
            let (encoding, _) = Encoding::for_bom(&buf).unwrap_or((UTF_8, 3));
            Encoding::new_decoder_with_bom_removal(encoding)
        });

        // in this case new line character is \x0A\x00
        // and we have already read until \x0A
        if decoder.encoding() == UTF_16LE {
            self.subtitle.read_until(b'\x00', &mut buf)?;
        }

        if buf.is_empty() {
            Ok(None)
        } else {
            let mut line = String::with_capacity(buf.len());
            let _ = decoder.decode_to_string(&buf, &mut line, false);
            trim_newline(&mut line);

            Ok(Some(line))
        }
    }
}

impl<T: Read> From<T> for SubRipParser<T> {
    fn from(subtitle: T) -> Self {
        SubRipParser {
            subtitle: BufReader::new(subtitle),
            decoder: None,
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
    fn utf_8_with_bom() {
        let subtitle = b"\
\xEF\xBB\xBF\
1433
01:04:00,705 --> 01:04:02,145
This is a
Test";
        let subtitle = Cursor::new(subtitle);

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
            text: vec![String::from("This is a"), String::from("Test")],
        };

        let actual = SubRipParser::from(subtitle).next().unwrap().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn utf_16be_with_bom() {
        let mut bom = vec![b'\xFE', b'\xFF'];
        let subtitle: Vec<u8> = "\
1433
01:04:00,705 --> 01:04:02,145
This is ą
Tęst"
            .encode_utf16()
            .map(|x| x.to_be_bytes().to_vec())
            .flatten()
            .collect();

        bom.extend(subtitle);
        let subtitle = Cursor::new(bom);

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
            text: vec![String::from("This is ą"), String::from("Tęst")],
        };

        let actual = SubRipParser::from(subtitle).next().unwrap().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn utf_16le_with_bom() {
        let mut bom = vec![b'\xFF', b'\xFE'];
        let subtitle: Vec<u8> = "\
1433
01:04:00,705 --> 01:04:02,145
This is ą
Tęst"
            .encode_utf16()
            .map(|x| x.to_le_bytes().to_vec())
            .flatten()
            .collect();

        bom.extend(subtitle);
        let subtitle = Cursor::new(bom);

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
            text: vec![String::from("This is ą"), String::from("Tęst")],
        };

        let actual = SubRipParser::from(subtitle).next().unwrap().unwrap();

        assert_eq!(expected, actual);
    }

    #[test]
    fn parse_subtitle() {
        let sub = "\
1
01:02:03,456 --> 07:08:09,101
This is a Test";
        let mut parser = SubRipParser::from(sub.as_bytes());

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
            text: vec![String::from("This is a Test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }

    #[test]
    fn parse_subtitle_cr_lf() {
        let sub = "\
1\r\n\
01:02:03,456 --> 07:08:09,101\r\n\
This is a Test";
        let mut parser = SubRipParser::from(sub.as_bytes());

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
            text: vec![String::from("This is a Test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
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
                String::from("It's only after"),
                String::from("we've lost everything"),
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
            text: vec![String::from("that we're free to do anything.")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());

        // End
        assert!(parser.next().is_none());
    }

    #[test]
    fn invalid_subtitle() {
        let sub = "\
1
00:00:00,000

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
            text: vec![String::from("This is a Test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }

    #[test]
    fn empty_lines() {
        let sub = "\
1
00:00:00,000 --> 00:00:01,000
test



2
00:00:01,000 --> 00:00:02,000
test";

        let mut parser = SubRipParser::from(sub.as_bytes());

        // First
        let expected = SubRip {
            position: 1,
            start: Timecode {
                hours: 0,
                minutes: 0,
                seconds: 0,
                milliseconds: 0,
            },
            end: Timecode {
                hours: 0,
                minutes: 0,
                seconds: 1,
                milliseconds: 0,
            },
            text: vec![String::from("test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());

        // Second
        let expected = SubRip {
            position: 2,
            start: Timecode {
                hours: 0,
                minutes: 0,
                seconds: 1,
                milliseconds: 0,
            },
            end: Timecode {
                hours: 0,
                minutes: 0,
                seconds: 2,
                milliseconds: 0,
            },
            text: vec![String::from("test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }
}
