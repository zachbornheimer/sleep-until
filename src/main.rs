use std::process::ExitCode;

use slp::app::run;
use slp::system::{LocalCalendar, SystemClock, ThreadSleeper};
use slp::wait::Dependencies;

fn main() -> ExitCode {
    let deps = Dependencies {
        clock: &SystemClock,
        calendar: &LocalCalendar,
        sleeper: &ThreadSleeper,
    };
    run(
        std::env::args_os().skip(1),
        &deps,
        &mut std::io::stdout().lock(),
        &mut std::io::stderr().lock(),
    )
}
