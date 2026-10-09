use std::fmt;

use crate::time_of_day::TimeOfDay;

const DATE_TIME_SEPARATORS: [char; 2] = [' ', 'T'];
const DATE_FIELD_SEPARATOR: char = '-';
const YEAR_DIGITS: usize = 4;
const MONTH_OR_DAY_DIGITS: usize = 2;
const FEBRUARY: u8 = 2;

/// A day on the Gregorian calendar, validated: real month, real day, leap years honored.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarDate {
    year: u16,
    month: u8,
    day: u8,
}

impl CalendarDate {
    #[must_use]
    pub fn new(year: u16, month: u8, day: u8) -> Option<Self> {
        let in_range = year >= 1 && (1..=12).contains(&month) && day >= 1;
        (in_range && day <= days_in_month(year, month)).then_some(Self { year, month, day })
    }

    #[must_use]
    pub fn year(self) -> u16 {
        self.year
    }

    #[must_use]
    pub fn month(self) -> u8 {
        self.month
    }

    #[must_use]
    pub fn day(self) -> u8 {
        self.day
    }
}

impl fmt::Display for CalendarDate {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:04}-{:02}-{:02}", self.year, self.month, self.day)
    }
}

/// Whether `text` is meant as a date and time rather than a bare time of day.
#[must_use]
pub fn looks_like_date_time(text: &str) -> bool {
    text.contains(DATE_TIME_SEPARATORS) || text.contains(DATE_FIELD_SEPARATOR)
}

/// Parses `YYYY-MM-DD HH:MM[:SS]` or `YYYY-MM-DDTHH:MM[:SS]`.
#[must_use]
pub fn parse_date_time(text: &str) -> Option<(CalendarDate, TimeOfDay)> {
    let (date, time) = text.split_once(DATE_TIME_SEPARATORS)?;
    Some((parse_date(date)?, TimeOfDay::parse(time)?))
}

fn parse_date(text: &str) -> Option<CalendarDate> {
    let mut fields = text.split(DATE_FIELD_SEPARATOR);
    let year = digits(fields.next()?, YEAR_DIGITS)?;
    let month = digits(fields.next()?, MONTH_OR_DAY_DIGITS)?;
    let day = digits(fields.next()?, MONTH_OR_DAY_DIGITS)?;
    if fields.next().is_some() {
        return None;
    }
    CalendarDate::new(year, u8::try_from(month).ok()?, u8::try_from(day).ok()?)
}

fn digits(text: &str, width: usize) -> Option<u16> {
    let valid = text.len() == width && text.bytes().all(|b| b.is_ascii_digit());
    if valid {
        text.parse().ok()
    } else {
        None
    }
}

fn days_in_month(year: u16, month: u8) -> u8 {
    match month {
        FEBRUARY if is_leap_year(year) => 29,
        FEBRUARY => 28,
        4 | 6 | 9 | 11 => 30,
        _ => 31,
    }
}

fn is_leap_year(year: u16) -> bool {
    (year % 4 == 0 && year % 100 != 0) || year % 400 == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date(year: u16, month: u8, day: u8) -> CalendarDate {
        CalendarDate::new(year, month, day).unwrap()
    }

    fn at(hour: u8, minute: u8, second: u8) -> TimeOfDay {
        TimeOfDay::new(hour, minute, second).unwrap()
    }

    #[test]
    fn parses_space_and_t_separated_forms() {
        let expected = Some((date(2999, 12, 31), at(23, 59, 0)));
        assert_eq!(parse_date_time("2999-12-31 23:59"), expected);
        assert_eq!(parse_date_time("2999-12-31T23:59"), expected);
        assert_eq!(
            parse_date_time("2030-01-02T03:04:05"),
            Some((date(2030, 1, 2), at(3, 4, 5)))
        );
        assert_eq!(
            parse_date_time("2030-01-02 3:04"),
            Some((date(2030, 1, 2), at(3, 4, 0)))
        );
    }

    #[test]
    fn leap_days_exist_only_in_leap_years() {
        assert!(CalendarDate::new(2024, 2, 29).is_some());
        assert!(CalendarDate::new(2000, 2, 29).is_some());
        assert!(CalendarDate::new(2023, 2, 29).is_none());
        assert!(CalendarDate::new(1900, 2, 29).is_none());
    }

    #[test]
    fn month_lengths_are_enforced() {
        assert!(CalendarDate::new(2026, 4, 30).is_some());
        assert!(CalendarDate::new(2026, 4, 31).is_none());
        assert!(CalendarDate::new(2026, 12, 31).is_some());
        assert!(CalendarDate::new(2026, 2, 30).is_none());
        assert!(CalendarDate::new(2026, 13, 1).is_none());
        assert!(CalendarDate::new(2026, 0, 1).is_none());
        assert!(CalendarDate::new(2026, 1, 0).is_none());
        assert!(CalendarDate::new(0, 1, 1).is_none());
    }

    #[test]
    fn rejects_malformed_dates_and_times() {
        for bad in [
            "",
            "2999-12-31",
            "2999-12-31 ",
            "2999-12-31 25:00",
            "2999-13-01 10:00",
            "2999-02-30 10:00",
            "2023-02-29 10:00",
            "2999-1-01 10:00",
            "2999-01-1 10:00",
            "299-01-01 10:00",
            "29999-01-01 10:00",
            "2999/01/01 10:00",
            "2999-01-01  10:00",
            "2999-01-01x10:00",
            "10:00",
            "+999-01-01 10:00",
        ] {
            assert_eq!(parse_date_time(bad), None, "{bad:?}");
        }
    }

    #[test]
    fn displays_the_way_it_is_typed() {
        assert_eq!(date(2026, 3, 9).to_string(), "2026-03-09");
    }
}
