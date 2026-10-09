//! Capabilities the sleeping workflow needs from the outside world.

use std::time::Duration;

use crate::calendar_date::CalendarDate;
use crate::time_of_day::TimeOfDay;

/// A point on the wall clock, as time since the Unix epoch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct WallInstant(Duration);

impl WallInstant {
    #[must_use]
    pub fn since_epoch(elapsed: Duration) -> Self {
        Self(elapsed)
    }

    #[must_use]
    pub fn as_duration_since_epoch(self) -> Duration {
        self.0
    }

    #[must_use]
    pub fn whole_seconds(self) -> u64 {
        self.0.as_secs()
    }

    /// This instant moved `later` ahead; `None` when that is not representable.
    #[must_use]
    pub fn checked_after(self, later: Duration) -> Option<Self> {
        self.0.checked_add(later).map(Self)
    }

    /// Time remaining until `later`; zero when `later` is not ahead.
    #[must_use]
    pub fn until(self, later: Self) -> Duration {
        later.0.saturating_sub(self.0)
    }
}

/// Which local calendar day a time of day is resolved on: relative to an instant, or explicit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocalDay {
    Today,
    Tomorrow,
    On(CalendarDate),
}

pub trait Clock {
    fn now(&self) -> WallInstant;
}

pub trait Calendar {
    /// The instant `time` occurs on `day`, in local time, where a relative `day` is relative to
    /// `around`. Dates before the Unix epoch resolve to the epoch.
    ///
    /// # Errors
    /// Fails when the local time zone cannot be resolved.
    fn resolve(
        &self,
        day: LocalDay,
        time: TimeOfDay,
        around: WallInstant,
    ) -> Result<WallInstant, CalendarUnavailable>;

    /// `at` as local time, formatted `YYYY-MM-DD HH:MM:SS +ZZZZ`.
    ///
    /// # Errors
    /// Fails when `at` cannot be expressed in the local time zone.
    fn format_local(&self, at: WallInstant) -> Result<String, CalendarUnavailable>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CalendarUnavailable;

pub trait Sleeper {
    fn sleep(&self, duration: Duration);
}
