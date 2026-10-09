//! The real calendar adapter against fixed time zones. One test owns `TZ` for this process.

use std::time::Duration;

use slp::calendar_date::CalendarDate;
use slp::ports::{Calendar, LocalDay, WallInstant};
use slp::system::LocalCalendar;
use slp::time_of_day::TimeOfDay;

const DAY: u64 = 86_400;
/// 2026-01-15T12:00:00Z
const NOON_UTC: u64 = 1_768_478_400;

fn resolve(day: LocalDay, hour: u8, minute: u8, around: u64) -> u64 {
    let time = TimeOfDay::new(hour, minute, 0).unwrap();
    LocalCalendar
        .resolve(
            day,
            time,
            WallInstant::since_epoch(Duration::from_secs(around)),
        )
        .unwrap()
        .whole_seconds()
}

fn format_at(secs: u64) -> String {
    LocalCalendar
        .format_local(WallInstant::since_epoch(Duration::from_secs(secs)))
        .unwrap()
}

#[test]
fn resolves_in_the_local_zone_including_dst_days() {
    // SAFETY: this is the only test in this binary, so nothing else reads the environment.
    unsafe { std::env::set_var("TZ", "UTC") };
    assert_eq!(
        resolve(LocalDay::Today, 14, 39, NOON_UTC),
        NOON_UTC + 2 * 3600 + 39 * 60
    );
    assert_eq!(resolve(LocalDay::Tomorrow, 12, 0, NOON_UTC), NOON_UTC + DAY);
    assert_eq!(format_at(NOON_UTC), "2026-01-15 12:00:00 +0000");

    // India has a +05:30 offset and no DST.
    unsafe { std::env::set_var("TZ", "Asia/Kolkata") };
    assert_eq!(resolve(LocalDay::Today, 17, 30, NOON_UTC), NOON_UTC);
    assert_eq!(format_at(NOON_UTC), "2026-01-15 17:30:00 +0530");

    // New York springs forward 2026-03-08: 24h after local noon on the 7th is 13:00 local.
    unsafe { std::env::set_var("TZ", "America/New_York") };
    let noon_new_york_mar_7 = 1_772_902_800; // 2026-03-07T12:00:00-05:00
    assert_eq!(
        resolve(LocalDay::Tomorrow, 12, 0, noon_new_york_mar_7),
        noon_new_york_mar_7 + DAY - 3600
    );

    // An explicit date resolves independent of `around`, in the local zone with DST.
    let mar_8 = CalendarDate::new(2026, 3, 8).unwrap();
    assert_eq!(
        resolve(LocalDay::On(mar_8), 12, 0, NOON_UTC),
        noon_new_york_mar_7 + DAY - 3600
    );
    unsafe { std::env::set_var("TZ", "UTC") };
    let leap_day = CalendarDate::new(2028, 2, 29).unwrap();
    assert_eq!(
        resolve(LocalDay::On(leap_day), 0, 0, NOON_UTC),
        1_835_395_200 // 2028-02-29T00:00:00Z
    );
    // Dates before the epoch resolve to the epoch rather than failing.
    let old = CalendarDate::new(1960, 1, 1).unwrap();
    assert_eq!(resolve(LocalDay::On(old), 0, 0, NOON_UTC), 0);
}
