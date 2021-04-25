use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Timecode {
    pub hours: i8,
    pub minutes: i8,
    pub seconds: i8,
    pub milliseconds: i16,
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
    pub text: Vec<String>,
}

impl fmt::Display for SubRip {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
            self.text.join("\n")
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
            text: vec![String::from("This is a"), String::from("Test")],
        };

        let expected = "\
1
01:02:03,456 --> 07:08:09,101
This is a
Test";

        assert_eq!(expected, format!("{}", sub));
    }
}
