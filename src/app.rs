//! The product operation: one command line in, one exit status out.

use std::ffi::OsString;
use std::io::Write;
use std::process::ExitCode;

use crate::cli::{parse_request, Request, Schedule};
use crate::failure::Failure;
use crate::wait::{wait, wake_time, Dependencies};

const PROGRAM: &str = "slp";
const NEVER: &str = "never";

const HELP: &str = "\
Usage: slp NUMBER[SUFFIX]...
  or:  slp --until HH:MM[:SS]
Pause for NUMBER seconds, or until the next local time HH:MM[:SS].

SUFFIX may be 's' for seconds (the default), 'm' for minutes, 'h' for hours
or 'd' for days. NUMBER may be a decimal number, or 'inf' to pause forever.
Given multiple arguments, pause for the amount of time specified by the sum.

--until HH:MM[:SS] pauses until that local time of day: today if it has not
passed yet, otherwise tomorrow. It cannot be combined with NUMBER arguments.

      --until TIME  pause until TIME in local time
  -n, --dry-run     print the local wake time ('wake at YYYY-MM-DD HH:MM:SS +ZZZZ',
                    or 'wake at never') and exit without pausing
      --help        display this help and exit
      --version     output version information and exit
";

/// Runs `slp` for `args` (without the program name) and reports the exit status.
pub fn run(
    args: impl IntoIterator<Item = OsString>,
    deps: &Dependencies,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> ExitCode {
    let outcome = parse_request(args).and_then(|request| match request {
        Request::Help => {
            print(out, HELP);
            Ok(())
        }
        Request::Version => {
            print(out, &version_line());
            Ok(())
        }
        Request::Wait(schedule) => wait(schedule, deps),
        Request::DryRun(schedule) => {
            let line = wake_line(schedule, deps)?;
            print(out, &line);
            Ok(())
        }
    });
    let Err(failure) = outcome else {
        return ExitCode::SUCCESS;
    };
    print(err, &format!("{PROGRAM}: {failure}\n"));
    if failure.is_usage() {
        print(
            err,
            &format!("Try '{PROGRAM} --help' for more information.\n"),
        );
    }
    ExitCode::FAILURE
}

fn wake_line(schedule: Schedule, deps: &Dependencies) -> Result<String, Failure> {
    let when = match wake_time(schedule, deps)? {
        Some(instant) => deps
            .calendar
            .format_local(instant)
            .map_err(|_| Failure::LocalTimeUnavailable)?,
        None => NEVER.to_owned(),
    };
    Ok(format!("wake at {when}\n"))
}

fn version_line() -> String {
    format!("{PROGRAM} {}\n", env!("CARGO_PKG_VERSION"))
}

/// Like `sleep`, output to a closed pipe is not a reason to change the exit status.
fn print(sink: &mut dyn Write, text: &str) {
    let _ = sink.write_all(text.as_bytes());
}
