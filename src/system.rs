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
        match day {
            LocalDay::Today => {}
            LocalDay::Tomorrow => broken_down.tm_mday += 1, // mktime normalizes rollover
            LocalDay::On(date) => {
                broken_down.tm_year = libc::c_int::from(date.year()) - TM_YEAR_BASE;
                broken_down.tm_mon = libc::c_int::from(date.month()) - 1;
                broken_down.tm_mday = date.day().into();
            }
        }
        // SAFETY: `broken_down` is a fully initialized, exclusively borrowed `tm`.
        let resolved = unsafe { libc::mktime(&raw mut broken_down) };
        // Before the epoch is simply long past: the epoch is a faithful stand-in.
        let secs = u64::try_from(resolved).unwrap_or_default();
        Ok(WallInstant::since_epoch(Duration::from_secs(secs)))
    }

    fn format_local(&self, at: WallInstant) -> Result<String, CalendarUnavailable> {
        let seconds =
            libc::time_t::try_from(at.whole_seconds()).map_err(|_| CalendarUnavailable)?;
        format_broken_down(&local_broken_down(seconds)?)
    }
}

/// `tm_year` counts years since 1900.
const TM_YEAR_BASE: libc::c_int = 1900;
const LOCAL_FORMAT: &[u8] = b"%Y-%m-%d %H:%M:%S %z\0";
const FORMATTED_CAPACITY: usize = 64;

fn format_broken_down(broken_down: &libc::tm) -> Result<String, CalendarUnavailable> {
    let mut buffer = [0u8; FORMATTED_CAPACITY];
    // SAFETY: `buffer` is writable for its full length, `LOCAL_FORMAT` is NUL-terminated,
    // and `broken_down` is a valid `tm`.
    let written = unsafe {
        libc::strftime(
            buffer.as_mut_ptr().cast(),
            buffer.len(),
            LOCAL_FORMAT.as_ptr().cast(),
            broken_down,
        )
    };
    if written == 0 {
        return Err(CalendarUnavailable);
    }
    String::from_utf8(buffer[..written].to_vec()).map_err(|_| CalendarUnavailable)
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
