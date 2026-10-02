//! Host adapters: the real clock, calendar, and sleeper.
//!
//! This is the only module allowed to contain `unsafe` (checked in `tests/architecture.rs`).

use std::mem::MaybeUninit;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use crate::ports::{Calendar, CalendarUnavailable, Clock, LocalDay, Sleeper, WallInstant};
use crate::time_of_day::TimeOfDay;

unsafe extern "C" {
    /// POSIX; the `libc` crate does not bind it on every target.
    fn tzset();
}

pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> WallInstant {
        let elapsed = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default();
        WallInstant::since_epoch(elapsed)
    }
}

pub struct ThreadSleeper;

impl Sleeper for ThreadSleeper {
    fn sleep(&self, duration: Duration) {
        std::thread::sleep(duration);
    }
}

/// Resolves local times through the C library's time zone database.
pub struct LocalCalendar;

impl Calendar for LocalCalendar {
    fn resolve(
        &self,
        day: LocalDay,
        time: TimeOfDay,
        around: WallInstant,
    ) -> Result<WallInstant, CalendarUnavailable> {
        let seconds =
            libc::time_t::try_from(around.whole_seconds()).map_err(|_| CalendarUnavailable)?;
        let mut broken_down = local_broken_down(seconds)?;
        broken_down.tm_hour = time.hour().into();
        broken_down.tm_min = time.minute().into();
        broken_down.tm_sec = time.second().into();
        broken_down.tm_isdst = -1; // let the C library choose standard or daylight time
        if day == LocalDay::Tomorrow {
            broken_down.tm_mday += 1; // mktime normalizes month and year rollover
        }
        // SAFETY: `broken_down` is a fully initialized, exclusively borrowed `tm`.
        let resolved = unsafe { libc::mktime(&raw mut broken_down) };
        u64::try_from(resolved)
            .map(|secs| WallInstant::since_epoch(Duration::from_secs(secs)))
            .map_err(|_| CalendarUnavailable)
    }
}

fn local_broken_down(seconds: libc::time_t) -> Result<libc::tm, CalendarUnavailable> {
    let mut slot = MaybeUninit::<libc::tm>::uninit();
    // SAFETY: both pointers are valid for the call; `localtime_r` initializes `slot` on success.
    // `tzset` is required first because POSIX does not make `localtime_r` read `TZ`.
    let filled = unsafe {
        tzset();
        libc::localtime_r(&raw const seconds, slot.as_mut_ptr())
    };
    if filled.is_null() {
        return Err(CalendarUnavailable);
    }
    // SAFETY: `localtime_r` returned non-null, so `slot` is initialized.
    Ok(unsafe { slot.assume_init() })
}
