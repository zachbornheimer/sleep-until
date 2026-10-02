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
