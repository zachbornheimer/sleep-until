//! The waiting workflow: observe, decide, execute.

use std::time::Duration;

use crate::calendar_date::CalendarDate;
use crate::cli::Schedule;
use crate::failure::Failure;
use crate::ports::{Calendar, Clock, LocalDay, Sleeper, WallInstant};
use crate::time_of_day::TimeOfDay;

/// The longest single sleep while waiting for a time of day.
///
/// Sleeps count monotonic time, which stops while the machine is suspended. Re-reading the
/// wall clock this often bounds how late `--until` can fire after a suspend.
pub const MAX_SLICE: Duration = Duration::from_secs(30);

pub struct Dependencies<'a> {
    pub clock: &'a dyn Clock,
    pub calendar: &'a dyn Calendar,
    pub sleeper: &'a dyn Sleeper,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Plan {
    Done,
    Sleep(Duration),
}

/// Blocks until `schedule` is satisfied.
///
/// # Errors
/// Fails only when a time of day cannot be resolved in the local time zone.
pub fn wait(schedule: Schedule, deps: &Dependencies) -> Result<(), Failure> {
    match schedule {
        Schedule::After(interval) => {
            execute(Plan::Sleep(interval), deps.sleeper);
            Ok(())
        }
        Schedule::At(time) => wait_until(next_occurrence(time, deps)?, deps),
        Schedule::On(date, time) => wait_until(dated_target(date, time, deps)?, deps),
    }
}

/// When `schedule` would end if started now; `None` means never.
///
/// # Errors
/// Fails only when a time of day cannot be resolved in the local time zone.
pub fn wake_time(schedule: Schedule, deps: &Dependencies) -> Result<Option<WallInstant>, Failure> {
    match schedule {
        Schedule::After(interval) => Ok(deps.clock.now().checked_after(interval)),
        Schedule::At(time) => next_occurrence(time, deps).map(Some),
        Schedule::On(date, time) => dated_target(date, time, deps).map(Some),
    }
}

fn wait_until(target: WallInstant, deps: &Dependencies) -> Result<(), Failure> {
    loop {
        let plan = decide(deps.clock.now(), target);
        if plan == Plan::Done {
            return Ok(());
        }
        execute(plan, deps.sleeper);
    }
}

/// Today's occurrence of `time`, or tomorrow's once today's second has passed.
fn next_occurrence(time: TimeOfDay, deps: &Dependencies) -> Result<WallInstant, Failure> {
    let now = deps.clock.now();
    let today = resolve_on(LocalDay::Today, time, now, deps)?;
    if today.whole_seconds() < now.whole_seconds() {
        resolve_on(LocalDay::Tomorrow, time, now, deps)
    } else {
        Ok(today)
    }
}

/// `time` on `date`; a moment whose second has already passed is an error, not a wrap-around.
fn dated_target(
    date: CalendarDate,
    time: TimeOfDay,
    deps: &Dependencies,
) -> Result<WallInstant, Failure> {
    let now = deps.clock.now();
    let target = resolve_on(LocalDay::On(date), time, now, deps)?;
    if target.whole_seconds() < now.whole_seconds() {
        Err(Failure::UntilPassed(date, time))
    } else {
        Ok(target)
    }
}

fn resolve_on(
    day: LocalDay,
    time: TimeOfDay,
    around: WallInstant,
    deps: &Dependencies,
) -> Result<WallInstant, Failure> {
    deps.calendar
        .resolve(day, time, around)
        .map_err(|_| Failure::LocalTimeUnavailable)
}

#[must_use]
pub fn decide(now: WallInstant, target: WallInstant) -> Plan {
    match now.until(target) {
        Duration::ZERO => Plan::Done,
        remaining => Plan::Sleep(remaining.min(MAX_SLICE)),
    }
}

fn execute(plan: Plan, sleeper: &dyn Sleeper) {
    if let Plan::Sleep(duration) = plan {
        sleeper.sleep(duration);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn at(secs: u64) -> WallInstant {
        WallInstant::since_epoch(Duration::from_secs(secs))
    }

    #[test]
    fn decide_is_done_at_or_past_target() {
        assert_eq!(decide(at(10), at(10)), Plan::Done);
        assert_eq!(decide(at(11), at(10)), Plan::Done);
    }

    #[test]
    fn decide_sleeps_the_remainder_capped_per_slice() {
        assert_eq!(decide(at(10), at(15)), Plan::Sleep(Duration::from_secs(5)));
        assert_eq!(decide(at(0), at(3_600)), Plan::Sleep(MAX_SLICE));
    }
}
