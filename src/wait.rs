//! The waiting workflow: observe, decide, execute.

use std::time::Duration;

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
        Schedule::At(time) => wait_until(time, deps),
    }
}

fn wait_until(time: TimeOfDay, deps: &Dependencies) -> Result<(), Failure> {
    let target = observe_target(time, deps)?;
    loop {
        let plan = decide(deps.clock.now(), target);
        if plan == Plan::Done {
            return Ok(());
        }
        execute(plan, deps.sleeper);
    }
}

/// Today's occurrence of `time`, or tomorrow's once today's second has passed.
fn observe_target(time: TimeOfDay, deps: &Dependencies) -> Result<WallInstant, Failure> {
    let now = deps.clock.now();
    let resolve = |day| {
        deps.calendar
            .resolve(day, time, now)
            .map_err(|_| Failure::LocalTimeUnavailable)
    };
    let today = resolve(LocalDay::Today)?;
    if today.whole_seconds() < now.whole_seconds() {
        resolve(LocalDay::Tomorrow)
    } else {
        Ok(today)
    }
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
