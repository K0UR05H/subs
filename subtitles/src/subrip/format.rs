use std::str::Utf8Error;

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
    pub text: Vec<Vec<u8>>,
}

impl SubRip {
    pub fn text_from_utf8(&self) -> Vec<Result<&str, Utf8Error>> {
        self.text.iter().map(|x| std::str::from_utf8(x)).collect()
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
}
