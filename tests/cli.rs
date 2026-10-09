//! The built `slp` binary through its public command line.

use std::process::{Command, Output};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

fn slp(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_slp"))
        .env("TZ", "UTC")
        .args(args)
        .output()
        .unwrap()
}

fn text(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).into_owned()
}

#[test]
fn sleeps_the_requested_interval() {
    let started = Instant::now();
    let out = slp(&["0.3", "0.2s"]);
    assert!(out.status.success());
    assert!(started.elapsed() >= Duration::from_millis(500));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn sleeps_a_compound_interval_across_operands() {
    let started = Instant::now();
    let out = slp(&["0.1s0.1s", "0.1s"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(started.elapsed() >= Duration::from_millis(300));
    assert!(started.elapsed() < Duration::from_secs(3));
}

#[test]
fn trailing_bare_number_in_a_compound_is_invalid() {
    let out = slp(&["1h30"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(text(&out.stderr).contains("slp: invalid time interval '1h30'"));
}

#[test]
fn until_waits_for_the_next_matching_second() {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let target = (now + 2) % 86_400;
    let clock = format!(
        "{:02}:{:02}:{:02}",
        target / 3600,
        target / 60 % 60,
        target % 60
    );
    let started = Instant::now();
    let out = slp(&["--until", &clock]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    assert!(
        started.elapsed() >= Duration::from_secs(1),
        "returned too early"
    );
    assert!(started.elapsed() < Duration::from_secs(4), "slept too long");
}

#[test]
fn dry_run_of_forever_prints_never_and_does_not_sleep() {
    let started = Instant::now();
    for flag in ["--dry-run", "-n"] {
        let out = slp(&[flag, "inf"]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert_eq!(text(&out.stdout), "wake at never\n");
    }
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn dry_run_of_an_interval_prints_the_local_wake_time_without_sleeping() {
    let started = Instant::now();
    let out = slp(&["--dry-run", "1h30m"]);
    assert!(out.status.success(), "{}", text(&out.stderr));
    let line = text(&out.stdout);
    let stamp = line.strip_prefix("wake at ").unwrap().trim_end();
    let shape: Vec<usize> = stamp.split([' ', '-', ':']).map(str::len).collect();
    assert_eq!(shape, [4, 2, 2, 2, 2, 2, 5], "{line:?}");
    assert!(stamp.ends_with(" +0000"));
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn help_and_version_win_over_dry_run() {
    let help = slp(&["--dry-run", "5", "--help"]);
    assert!(text(&help.stdout).starts_with("Usage: slp"));
    let version = slp(&["-n", "5", "--version"]);
    assert!(text(&version.stdout).starts_with("slp "));
    assert!(text(&slp(&["--help"]).stdout).contains("--dry-run"));
}

#[test]
fn dated_until_in_the_past_is_a_runtime_error_without_usage_hint() {
    let out = slp(&["--until", "2020-01-01 00:00"]);
    assert_eq!(out.status.code(), Some(1));
    assert!(out.stdout.is_empty());
    assert_eq!(
        text(&out.stderr),
        "slp: --until 2020-01-01 00:00 has already passed\n"
    );
}

#[test]
fn dry_run_of_a_far_future_dated_until_is_deterministic() {
    for target in ["2999-12-31 12:00", "2999-12-31T12:00:00"] {
        let out = slp(&["-n", "--until", target]);
        assert!(out.status.success(), "{}", text(&out.stderr));
        assert_eq!(text(&out.stdout), "wake at 2999-12-31 12:00:00 +0000\n");
    }
}

#[test]
fn impossible_dates_are_usage_errors() {
    let out = slp(&["--until", "2999-02-30 10:00"]);
    assert_eq!(out.status.code(), Some(1));
    let stderr = text(&out.stderr);
    assert!(stderr.contains("slp: invalid date and time '2999-02-30 10:00'"));
    assert!(stderr.contains("Try 'slp --help'"));
}

#[test]
fn bad_input_exits_one_with_a_message_on_stderr() {
    for (args, message) in [
        (vec![], "slp: missing operand"),
        (vec!["abc"], "slp: invalid time interval 'abc'"),
        (vec!["-1"], "slp: invalid option '-1'"),
        (vec!["--until", "25:00"], "slp: invalid time of day '25:00'"),
        (vec!["--until", "1:00", "5"], "cannot be combined"),
    ] {
        let out = slp(&args);
        assert_eq!(out.status.code(), Some(1), "{args:?}");
        assert!(out.stdout.is_empty());
        let stderr = text(&out.stderr);
        assert!(stderr.contains(message), "{args:?}: {stderr}");
        assert!(stderr.contains("Try 'slp --help'"));
    }
}

#[test]
fn help_and_version_print_to_stdout() {
    let help = slp(&["--help"]);
    assert!(help.status.success());
    assert!(text(&help.stdout).starts_with("Usage: slp NUMBER[SUFFIX]..."));
    let version = slp(&["--version"]);
    assert_eq!(
        text(&version.stdout),
        format!("slp {}\n", env!("CARGO_PKG_VERSION"))
    );
}
