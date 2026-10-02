//! The waiting workflow against port fakes: no real clock, zone, or sleep.

use std::cell::{Cell, RefCell};
use std::time::Duration;

use slp::cli::Schedule;
use slp::failure::Failure;
use slp::ports::{Calendar, CalendarUnavailable, Clock, LocalDay, Sleeper, WallInstant};
use slp::time_of_day::TimeOfDay;
use slp::wait::{Dependencies, MAX_SLICE, wait};

const TODAY_TARGET: u64 = 1_000;
const TOMORROW_TARGET: u64 = 1_000 + 86_400;

/// Sleeping advances the clock, so the workflow sees time pass.
struct Timeline {
    now: Cell<Duration>,
    slept: RefCell<Vec<Duration>>,
}

impl Timeline {
    fn starting_at(secs: f64) -> Self {
        Self {
            now: Cell::new(Duration::from_secs_f64(secs)),
            slept: RefCell::default(),
        }
    }
}

impl Clock for Timeline {
    fn now(&self) -> WallInstant {
        WallInstant::since_epoch(self.now.get())
    }
}

impl Sleeper for Timeline {
    fn sleep(&self, duration: Duration) {
        self.slept.borrow_mut().push(duration);
        self.now.set(self.now.get() + duration);
    }
}

struct FixedCalendar(Result<(), CalendarUnavailable>);

impl Calendar for FixedCalendar {
    fn resolve(
        &self,
        day: LocalDay,
        _: TimeOfDay,
        _: WallInstant,
    ) -> Result<WallInstant, CalendarUnavailable> {
        self.0?;
        let secs = if day == LocalDay::Today {
            TODAY_TARGET
        } else {
            TOMORROW_TARGET
        };
        Ok(WallInstant::since_epoch(Duration::from_secs(secs)))
    }
}

fn until_target(timeline: &Timeline, calendar: &FixedCalendar) -> Result<(), Failure> {
    let deps = Dependencies {
        clock: timeline,
        calendar,
        sleeper: timeline,
    };
    wait(Schedule::At(TimeOfDay::new(0, 16, 40).unwrap()), &deps)
}

fn total(timeline: &Timeline) -> Duration {
    timeline.slept.borrow().iter().sum()
}

#[test]
fn interval_sleeps_exactly_once() {
    let timeline = Timeline::starting_at(0.0);
    let calendar = FixedCalendar(Ok(()));
    let deps = Dependencies {
        clock: &timeline,
        calendar: &calendar,
        sleeper: &timeline,
    };
    wait(Schedule::After(Duration::from_secs(90)), &deps).unwrap();
    assert_eq!(*timeline.slept.borrow(), [Duration::from_secs(90)]);
}

#[test]
fn until_waits_for_a_target_later_today_in_slices() {
    let timeline = Timeline::starting_at(900.0);
    until_target(&timeline, &FixedCalendar(Ok(()))).unwrap();
    assert_eq!(total(&timeline), Duration::from_secs(100));
    assert!(
        timeline
            .slept
            .borrow()
            .iter()
            .all(|slice| *slice <= MAX_SLICE)
    );
}

#[test]
fn until_rolls_to_tomorrow_once_today_has_passed() {
    let timeline = Timeline::starting_at(1_001.0);
    until_target(&timeline, &FixedCalendar(Ok(()))).unwrap();
    assert_eq!(
        total(&timeline),
        Duration::from_secs(TOMORROW_TARGET - 1_001)
    );
}

#[test]
fn until_the_current_second_returns_without_sleeping_a_day() {
    let timeline = Timeline::starting_at(1_000.5);
    until_target(&timeline, &FixedCalendar(Ok(()))).unwrap();
    assert!(total(&timeline) <= Duration::from_secs(1));
}

#[test]
fn unavailable_calendar_is_a_failure_not_a_hang() {
    let timeline = Timeline::starting_at(0.0);
    let result = until_target(&timeline, &FixedCalendar(Err(CalendarUnavailable)));
    assert_eq!(result, Err(Failure::LocalTimeUnavailable));
    assert!(timeline.slept.borrow().is_empty());
}
