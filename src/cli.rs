use std::ffi::OsString;
use std::time::Duration;

use crate::calendar_date::{CalendarDate, looks_like_date_time, parse_date_time};
use crate::failure::Failure;
use crate::interval::parse_interval;
use crate::time_of_day::TimeOfDay;

/// How long to wait: a fixed interval, until the next local occurrence of a time of day,
/// or until a time on an explicit local date.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Schedule {
    After(Duration),
    At(TimeOfDay),
    On(CalendarDate, TimeOfDay),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Request {
    Help,
    Version,
    Wait(Schedule),
    /// Report when the schedule would end, without waiting.
    DryRun(Schedule),
}

const UNTIL_FLAG: &str = "--until";
const DRY_RUN_FLAGS: [&str; 2] = ["-n", "--dry-run"];

/// Turns the command line (without the program name) into a [`Request`].
///
/// # Errors
/// Returns a usage [`Failure`] for unknown options and malformed operands.
pub fn parse_request(args: impl IntoIterator<Item = OsString>) -> Result<Request, Failure> {
    let mut args = args.into_iter();
    let mut total: Option<Duration> = None;
    let mut until: Option<Schedule> = None;
    let mut operands_only = false;
    let mut dry_run = false;

    while let Some(arg) = args.next() {
        let text = arg.to_string_lossy();
        if operands_only || !text.starts_with('-') || text == "-" {
            total = Some(add_operand(total, &arg)?);
        } else if text == "--" {
            operands_only = true;
        } else if DRY_RUN_FLAGS.contains(&&*text) {
            dry_run = true;
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

    let schedule = match (total, until) {
        (Some(_), Some(_)) => return Err(Failure::UntilWithInterval),
        (Some(total), None) => Schedule::After(total),
        (None, Some(until)) => until,
        (None, None) => return Err(Failure::MissingOperand),
    };
    Ok(if dry_run {
        Request::DryRun(schedule)
    } else {
        Request::Wait(schedule)
    })
}

fn add_operand(total: Option<Duration>, operand: &OsString) -> Result<Duration, Failure> {
    let text = operand.to_string_lossy();
    let interval = parse_interval(&text)?;
    Ok(total.unwrap_or_default().saturating_add(interval))
}

fn set_until(existing: Option<Schedule>, value: &std::ffi::OsStr) -> Result<Schedule, Failure> {
    if existing.is_some() {
        return Err(Failure::DuplicateUntil);
    }
    let text = value.to_string_lossy();
    if looks_like_date_time(&text) {
        return parse_date_time(&text)
            .map(|(date, time)| Schedule::On(date, time))
            .ok_or_else(|| Failure::InvalidDateTime(text.into_owned()));
    }
    TimeOfDay::parse(&text)
        .map(Schedule::At)
        .ok_or_else(|| Failure::InvalidTimeOfDay(text.into_owned()))
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
    fn until_accepts_a_date_and_time() {
        let on = Ok(Request::Wait(Schedule::On(
            CalendarDate::new(2999, 12, 31).unwrap(),
            TimeOfDay::new(23, 59, 30).unwrap(),
        )));
        assert_eq!(parse(&["--until", "2999-12-31 23:59:30"]), on);
        assert_eq!(parse(&["--until=2999-12-31T23:59:30"]), on);
    }

    #[test]
    fn until_rejects_impossible_dates() {
        for bad in [
            "2999-13-01 10:00",
            "2999-02-30 10:00",
            "2999-12-31",
            "2999-12-31T",
        ] {
            assert_eq!(
                parse(&["--until", bad]),
                Err(Failure::InvalidDateTime(bad.into())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn until_text_that_is_not_year_led_keeps_the_time_of_day_error() {
        for bad in ["-5", "10:00 ", "T10:00", "25:00"] {
            assert_eq!(
                parse(&["--until", bad]),
                Err(Failure::InvalidTimeOfDay(bad.into())),
                "{bad:?}"
            );
        }
    }

    #[test]
    fn dry_run_wraps_the_schedule_wherever_the_flag_appears() {
        let dry = Ok(Request::DryRun(Schedule::After(Duration::from_secs(90))));
        assert_eq!(parse(&["--dry-run", "1m30s"]), dry);
        assert_eq!(parse(&["1m", "-n", "30"]), dry);
        assert_eq!(parse(&["-n", "--dry-run", "90"]), dry);
        assert_eq!(
            parse(&["-n", "--until", "14:39"]),
            Ok(Request::DryRun(Schedule::At(
                TimeOfDay::new(14, 39, 0).unwrap()
            )))
        );
        assert_eq!(parse(&["-n"]), Err(Failure::MissingOperand));
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
