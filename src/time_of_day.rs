/// A wall-clock time of day, validated: `00:00:00` through `23:59:59`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TimeOfDay {
    hour: u8,
    minute: u8,
    second: u8,
}

impl TimeOfDay {
    /// Parses `H:MM` or `H:MM:SS` (one or two digits per field).
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let mut fields = text.split(':');
        let hour = field(fields.next()?)?;
        let minute = field(fields.next()?)?;
        let second = fields.next().map_or(Some(0), field)?;
        if fields.next().is_some() {
            return None;
        }
        Self::new(hour, minute, second)
    }

    #[must_use]
    pub fn new(hour: u8, minute: u8, second: u8) -> Option<Self> {
        (hour < 24 && minute < 60 && second < 60).then_some(Self {
            hour,
            minute,
            second,
        })
    }

    #[must_use]
    pub fn hour(self) -> u8 {
        self.hour
    }

    #[must_use]
    pub fn minute(self) -> u8 {
        self.minute
    }

    #[must_use]
    pub fn second(self) -> u8 {
        self.second
    }
}

fn field(text: &str) -> Option<u8> {
    let valid = matches!(text.len(), 1..=2) && text.bytes().all(|b| b.is_ascii_digit());
    if valid { text.parse().ok() } else { None }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_minutes_and_seconds_forms() {
        assert_eq!(TimeOfDay::parse("14:39"), TimeOfDay::new(14, 39, 0));
        assert_eq!(TimeOfDay::parse("9:05"), TimeOfDay::new(9, 5, 0));
        assert_eq!(TimeOfDay::parse("00:00:07"), TimeOfDay::new(0, 0, 7));
        assert_eq!(TimeOfDay::parse("23:59:59"), TimeOfDay::new(23, 59, 59));
    }

    #[test]
    fn rejects_out_of_range_and_malformed() {
        for bad in [
            "", "14", "24:00", "12:60", "12:00:60", "1:2:3:4", "ab:cd", "-1:00", "+1:00", "123:00",
            "12:", ":30",
        ] {
            assert_eq!(TimeOfDay::parse(bad), None, "{bad:?}");
        }
    }
}
