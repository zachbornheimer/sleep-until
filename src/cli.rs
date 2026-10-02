use std::ffi::OsString;
use std::time::Duration;

use crate::failure::Failure;
use crate::interval::parse_interval;
use crate::time_of_day::TimeOfDay;

/// How long to wait: a fixed interval, or until the next local occurrence of a time of day.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schedule {
    After(Duration),
    At(TimeOfDay),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Help,
    Version,
    Wait(Schedule),
}

const UNTIL_FLAG: &str = "--until";

/// Turns the command line (without the program name) into a [`Request`].
///
/// # Errors
/// Returns a usage [`Failure`] for unknown options and malformed operands.
pub fn parse_request(args: impl IntoIterator<Item = OsString>) -> Result<Request, Failure> {
    let mut args = args.into_iter();
    let mut total: Option<Duration> = None;
    let mut until: Option<TimeOfDay> = None;
    let mut operands_only = false;

    while let Some(arg) = args.next() {
        let text = arg.to_string_lossy();
        if operands_only || !text.starts_with('-') || text == "-" {
            total = Some(add_operand(total, &arg)?);
        } else if text == "--" {
            operands_only = true;
        } else if text == "--help" {
            return Ok(Request::Help);
        } else if text == "--version" {
            return Ok(Request::Version);
        } else if text == UNTIL_FLAG {
            let value = args.next().ok_or(Failure::MissingUntilValue)?;
            until = Some(set_until(until, &value)?);
        } else if let Some(value) = text.strip_prefix("--until=") {
            until = Some(set_until(until, OsString::from(value).as_os_str())?);
        } else {
            return Err(Failure::InvalidOption(text.into_owned()));
        }
    }

    match (total, until) {
        (Some(_), Some(_)) => Err(Failure::UntilWithInterval),
        (Some(total), None) => Ok(Request::Wait(Schedule::After(total))),
        (None, Some(time)) => Ok(Request::Wait(Schedule::At(time))),
        (None, None) => Err(Failure::MissingOperand),
    }
}

fn add_operand(total: Option<Duration>, operand: &OsString) -> Result<Duration, Failure> {
    let text = operand.to_string_lossy();
    let interval = parse_interval(&text)?;
    Ok(total.unwrap_or_default().saturating_add(interval))
}

fn set_until(existing: Option<TimeOfDay>, value: &std::ffi::OsStr) -> Result<TimeOfDay, Failure> {
    if existing.is_some() {
        return Err(Failure::DuplicateUntil);
    }
    let text = value.to_string_lossy();
    TimeOfDay::parse(&text).ok_or_else(|| Failure::InvalidTimeOfDay(text.into_owned()))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(args: &[&str]) -> Result<Request, Failure> {
        parse_request(args.iter().map(OsString::from))
    }

    fn after(secs: u64) -> Request {
        Request::Wait(Schedule::After(Duration::from_secs(secs)))
    }

    #[test]
    fn operands_are_summed() {
        assert_eq!(parse(&["1m", "30"]), Ok(after(90)));
        assert_eq!(parse(&["--", "2"]), Ok(after(2)));
    }

    #[test]
    fn until_accepts_both_spellings() {
        let at = Ok(Request::Wait(Schedule::At(
            TimeOfDay::new(14, 39, 0).unwrap(),
        )));
        assert_eq!(parse(&["--until", "14:39"]), at);
        assert_eq!(parse(&["--until=14:39"]), at);
    }

    #[test]
    fn help_and_version_win_when_seen() {
        assert_eq!(parse(&["--help"]), Ok(Request::Help));
        assert_eq!(parse(&["1", "--version"]), Ok(Request::Version));
    }

    #[test]
    fn usage_failures() {
        assert_eq!(parse(&[]), Err(Failure::MissingOperand));
        assert_eq!(parse(&["--until"]), Err(Failure::MissingUntilValue));
        assert_eq!(parse(&["-1"]), Err(Failure::InvalidOption("-1".into())));
        assert_eq!(
            parse(&["--bogus"]),
            Err(Failure::InvalidOption("--bogus".into()))
        );
        assert_eq!(
            parse(&["--until", "25:00"]),
            Err(Failure::InvalidTimeOfDay("25:00".into()))
        );
        assert_eq!(
            parse(&["--until", "1:00", "--until", "2:00"]),
            Err(Failure::DuplicateUntil)
        );
        assert_eq!(
            parse(&["5", "--until", "1:00"]),
            Err(Failure::UntilWithInterval)
        );
        assert_eq!(parse(&["abc"]), Err(Failure::InvalidInterval("abc".into())));
    }
}
