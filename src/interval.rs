use std::time::Duration;

use crate::failure::Failure;

/// The longest wait there is: `inf`, `infinity`, or anything too large to represent.
pub const FOREVER: Duration = Duration::MAX;

const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_HOUR: f64 = 3_600.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

const UNIT_SUFFIXES: [char; 4] = ['s', 'm', 'h', 'd'];

/// Parses one operand: a GNU-style `NUMBER[SUFFIX]`, or several `NUMBERSUFFIX` segments
/// such as `1h30m`, where SUFFIX is `s`, `m`, `h` or `d`.
///
/// # Errors
/// Returns [`Failure::InvalidInterval`] for text that is not a non-negative number.
pub fn parse_interval(text: &str) -> Result<Duration, Failure> {
    parse_single(text)
        .or_else(|| parse_compound(text))
        .ok_or_else(|| Failure::InvalidInterval(text.to_owned()))
}

fn parse_single(text: &str) -> Option<Duration> {
    let (number, unit_seconds) = split_suffix(text);
    parse_amount(number, unit_seconds)
}

/// Sums `NUMBERSUFFIX` segments; every segment needs its own suffix and a plain number.
fn parse_compound(text: &str) -> Option<Duration> {
    let mut rest = text;
    let mut total = Duration::ZERO;
    if rest.is_empty() {
        return None;
    }
    while !rest.is_empty() {
        let end = rest.find(UNIT_SUFFIXES)?;
        let (number, tail) = rest.split_at(end);
        let is_plain_number =
            number.starts_with(|c: char| c.is_ascii_digit() || c == '.' || c == '+');
        if !is_plain_number {
            return None;
        }
        total = total.saturating_add(parse_single(&rest[..=end])?);
        rest = &tail[1..];
    }
    Some(total)
}

fn parse_amount(number: &str, unit_seconds: f64) -> Option<Duration> {
    if number.starts_with('-') {
        return None;
    }
    let amount: f64 = number.parse().ok()?;
    if amount.is_nan() {
        return None;
    }
    Some(Duration::try_from_secs_f64(amount * unit_seconds).unwrap_or(FOREVER))
}

fn split_suffix(text: &str) -> (&str, f64) {
    let unit_seconds = match text.chars().next_back() {
        Some('s') => 1.0,
        Some('m') => SECONDS_PER_MINUTE,
        Some('h') => SECONDS_PER_HOUR,
        Some('d') => SECONDS_PER_DAY,
        _ => return (text, 1.0),
    };
    (&text[..text.len() - 1], unit_seconds)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn secs(text: &str) -> Duration {
        parse_interval(text).unwrap()
    }

    #[test]
    fn bare_number_is_seconds() {
        assert_eq!(secs("5"), Duration::from_secs(5));
        assert_eq!(secs("0.5"), Duration::from_millis(500));
        assert_eq!(secs(".5"), Duration::from_millis(500));
        assert_eq!(secs("1e1"), Duration::from_secs(10));
        assert_eq!(secs("+3"), Duration::from_secs(3));
    }

    #[test]
    fn suffixes_scale() {
        assert_eq!(secs("2s"), Duration::from_secs(2));
        assert_eq!(secs("2m"), Duration::from_secs(120));
        assert_eq!(secs("1.5h"), Duration::from_secs(5_400));
        assert_eq!(secs("1d"), Duration::from_secs(86_400));
    }

    #[test]
    fn infinity_and_overflow_wait_forever() {
        assert_eq!(secs("inf"), FOREVER);
        assert_eq!(secs("Infinity"), FOREVER);
        assert_eq!(secs("1e999"), FOREVER);
    }

    #[test]
    fn compound_segments_are_summed() {
        for (text, expected) in [
            ("1h30m", 5_400.0),
            ("2m30s", 150.0),
            ("1.5h", 5_400.0),
            ("1d12h", 129_600.0),
            ("1h1m1s", 3_661.0),
            ("0.5m0.5s", 30.5),
            ("1e1s1m", 70.0),
            ("1e-1h30s", 390.0),
            ("+1h+1m", 3_660.0),
        ] {
            assert_eq!(secs(text), Duration::from_secs_f64(expected), "{text:?}");
        }
    }

    #[test]
    fn compound_overflow_saturates() {
        assert_eq!(secs("1e999d1s"), FOREVER);
        assert_eq!(secs("1s1e999d"), FOREVER);
    }

    #[test]
    fn rejects_malformed_compounds() {
        for bad in [
            "1h30", "1h30x", "h30m", "1hm", "1h-5m", "1hinf", "infh1m", "1h 30m", "1h30m ", "1h.m",
            "1h30mm", "nanm1s",
        ] {
            assert_eq!(
                parse_interval(bad),
                Err(Failure::InvalidInterval(bad.to_owned())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn rejects_malformed_text() {
        for bad in [
            "", "s", "abc", "-1", "-0", "nan", "1x", "1 s", "1ss", "--1", "0x10",
        ] {
            assert_eq!(
                parse_interval(bad),
                Err(Failure::InvalidInterval(bad.to_owned())),
                "{bad:?}"
            );
        }
    }
}
