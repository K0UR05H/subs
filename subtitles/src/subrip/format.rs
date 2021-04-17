use std::{borrow::Cow, fmt};

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
    pub fn text_from_utf8_lossy(&self) -> Vec<Cow<str>> {
        self.text
            .iter()
            .map(|x| String::from_utf8_lossy(x))
            .collect()
    }
}

impl fmt::Display for SubRip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = self.text_from_utf8_lossy().join("\n");

        write!(
            f,
            "\
{}
{:02}:{:02}:{:02},{:03} --> {:02}:{:02}:{:02},{:03}
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
