use std::{borrow::Cow, fmt};

pub type Line = Vec<u8>;

#[derive(Debug, PartialEq)]
pub struct Timecode {
    pub hours: u8,
    pub minutes: u8,
    pub seconds: u8,
    pub milliseconds: u16,
}

/// Representing a SubRip (.srt) file
#[derive(Debug, PartialEq)]
pub struct SubRip {
    /// Subtitle position
    pub position: usize,
    /// The time that the subtitle should appear.
    pub start: Timecode,
    /// The time that the subtitle should disappear.
    pub end: Timecode,
    /// A list of lines in this subtitle.
    /// note that each line is a byte sequence and should be decoded.
    pub text: Vec<Line>,
}

impl SubRip {
    /// Decode subtitle text to a list of strings
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
