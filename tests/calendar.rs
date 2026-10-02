//! The real calendar adapter against fixed time zones. One test owns `TZ` for this process.

use std::time::Duration;

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

#[test]
fn resolves_in_the_local_zone_including_dst_days() {
    // SAFETY: this is the only test in this binary, so nothing else reads the environment.
    unsafe { std::env::set_var("TZ", "UTC") };
    assert_eq!(
        resolve(LocalDay::Today, 14, 39, NOON_UTC),
        NOON_UTC + 2 * 3600 + 39 * 60
    );
    assert_eq!(resolve(LocalDay::Tomorrow, 12, 0, NOON_UTC), NOON_UTC + DAY);

    // India has a +05:30 offset and no DST.
    unsafe { std::env::set_var("TZ", "Asia/Kolkata") };
    assert_eq!(resolve(LocalDay::Today, 17, 30, NOON_UTC), NOON_UTC);

    // New York springs forward 2026-03-08: 24h after local noon on the 7th is 13:00 local.
    unsafe { std::env::set_var("TZ", "America/New_York") };
    let noon_new_york_mar_7 = 1_772_902_800; // 2026-03-07T12:00:00-05:00
    assert_eq!(
        resolve(LocalDay::Tomorrow, 12, 0, noon_new_york_mar_7),
        noon_new_york_mar_7 + DAY - 3600
    );
}
