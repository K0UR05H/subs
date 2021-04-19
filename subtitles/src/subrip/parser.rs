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
    buffer: Vec<u8>,
    decoder: Decoder,
}

impl<T: Read> SubRipParser<T> {
    fn parse_next(&mut self) -> ParseResult<Option<SubRip>> {
        let position = match self.read_line(parse_position) {
            Ok(Some(position)) => position,
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidPosition, err)),
        };

        let (start, end) = match self.read_line(parse_timecode) {
            Ok(Some((start, end))) => (start, end),
            Ok(None) => return Ok(None),
            Err(err) => return Err(Error::new(ErrorKind::InvalidTimecode, err)),
        };

        let mut text = Vec::new();
        loop {
            match self.read_line(|line| Ok(parse_text(line))) {
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

    fn read_line<R, F>(&mut self, f: F) -> Result<R>
    where
        F: FnOnce(String) -> Result<R>,
    {
        let line = self.next_line_string()?;
        let result = f(line);

        if result.is_err() {
            self.skip_to_next_subtitle()?;
        }

        result
    }

    fn skip_to_next_subtitle(&mut self) -> Result<()> {
        loop {
            if self.next_line_string()?.is_empty() {
                break;
            }
        }
        Ok(())
    }

    fn next_line_string(&mut self) -> Result<String> {
        let buf = self.next_line()?;
        Ok(self.decode_buf(&buf))
    }

    fn next_line(&mut self) -> Result<Vec<u8>> {
        let newline = match self.buffer.iter().position(|x| *x == b'\n') {
            Some(n) => n + 1,
            None => {
                self.fill_buf()?;
                self.buffer.len()
            }
        };

        let buf = self.buffer.drain(..newline).collect();
        Ok(buf)
    }

    fn fill_buf(&mut self) -> Result<()> {
        self.subtitle.read_until(b'\n', &mut self.buffer)?;
        if self.decoder.encoding() == UTF_16LE {
            self.read_byte();
        }

        Ok(())
    }

    fn read_byte(&mut self) {
        let mut byte = [0u8; 1];
        let _ = self.subtitle.read_exact(&mut byte);
        self.buffer.extend_from_slice(&byte);
    }

    fn decode_buf(&mut self, buf: &[u8]) -> String {
        let mut line = String::with_capacity(buf.len());
        let _ = self.decoder.decode_to_string(&buf, &mut line, false);

        trim_newline(line)
    }
}

impl<T: Read> From<T> for SubRipParser<T> {
    fn from(subtitle: T) -> Self {
        let mut subtitle = BufReader::new(subtitle);

        let mut bom = [0u8; 3];
        let _ = subtitle.read_exact(&mut bom);

        let (encoding, _) = Encoding::for_bom(&bom).unwrap_or((UTF_8, 3));

        SubRipParser {
            subtitle,
            buffer: bom.to_vec(),
            decoder: Encoding::new_decoder_with_bom_removal(encoding),
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
            text: vec![String::from("This is a Test")],
        };

        assert_eq!(expected, parser.next().unwrap().unwrap());
    }
}
