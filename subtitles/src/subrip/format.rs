use std::{fmt, str::Utf8Error};

pub type Line = Vec<u8>;

#[derive(Debug, PartialEq)]
pub struct Timecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

#[derive(Debug, PartialEq)]
pub struct SubRip {
    pub position: usize,
    pub start: Timecode,
    pub end: Timecode,
    pub text: Vec<Line>,
}

impl SubRip {
    pub fn text_from_utf8(&self) -> Vec<Result<&str, Utf8Error>> {
        self.text.iter().map(|x| std::str::from_utf8(x)).collect()
    }
}

impl fmt::Display for SubRip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text.iter().fold(String::new(), |acc, x| {
            acc + "\n" + &*String::from_utf8_lossy(x)
        });

        write!(
            f,
            "\
{}
{:02}:{:02}:{:02},{:03} --> {:02}:{:02}:{:02},{:03}\
{}",
            self.position,
            self.start.hours,
            self.start.minutes,
            self.start.seconds,
            self.start.milliseconds,
            self.end.hours,
            self.end.minutes,
            self.end.seconds,
            self.end.milliseconds,
            text
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn text_from_utf8() {
        let text = String::from("test").into_bytes();
        let sub = SubRip {
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
                seconds: 0,
                milliseconds: 0,
            },
            text: vec![text],
        };

        assert_eq!(vec![Ok("test")], sub.text_from_utf8());
    }

    #[test]
    fn text_from_invalid_utf8() {
        let text = vec![b'\xff'];
        let sub = SubRip {
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
                seconds: 0,
                milliseconds: 0,
            },
            text: vec![text],
        };

        assert_eq!(1, sub.text_from_utf8().len());
        assert!(sub.text_from_utf8().first().unwrap().is_err())
    }

    #[test]
    fn display() {
        let sub = SubRip {
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
            text: vec![
                String::from("This is a").into_bytes(),
                String::from("Test").into_bytes(),
            ],
        };

        let expected = "\
1
01:02:03,456 --> 07:08:09,101
This is a
Test";

        assert_eq!(expected, format!("{}", sub));
    }
}
