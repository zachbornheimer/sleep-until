use std::time::Duration;

use crate::failure::Failure;

/// The longest wait there is: `inf`, `infinity`, or anything too large to represent.
pub const FOREVER: Duration = Duration::MAX;

const SECONDS_PER_MINUTE: f64 = 60.0;
const SECONDS_PER_HOUR: f64 = 3_600.0;
const SECONDS_PER_DAY: f64 = 86_400.0;

/// Parses one GNU-style `NUMBER[SUFFIX]` operand, where SUFFIX is `s`, `m`, `h` or `d`.
///
/// # Errors
/// Returns [`Failure::InvalidInterval`] for text that is not a non-negative number.
pub fn parse_interval(text: &str) -> Result<Duration, Failure> {
    let invalid = || Failure::InvalidInterval(text.to_owned());
    let (number, unit_seconds) = split_suffix(text);
    if number.starts_with('-') {
        return Err(invalid());
    }
    let amount: f64 = number.parse().map_err(|_| invalid())?;
    if amount.is_nan() {
        return Err(invalid());
    }
    Ok(Duration::try_from_secs_f64(amount * unit_seconds).unwrap_or(FOREVER))
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
