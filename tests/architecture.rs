//! Structural rules that tests of behavior cannot see.

use std::fs;
use std::path::Path;

fn production_sources() -> Vec<(String, String)> {
    let mut found = Vec::new();
    for entry in fs::read_dir(Path::new(env!("CARGO_MANIFEST_DIR")).join("src")).unwrap() {
        let path = entry.unwrap().path();
        let name = path.file_name().unwrap().to_string_lossy().into_owned();
        found.push((name, fs::read_to_string(&path).unwrap()));
    }
    found
}

#[test]
fn unsafe_code_stays_in_the_system_adapters() {
    for (name, text) in production_sources() {
        let uses_unsafe = text.lines().any(|line| {
            let code = line.split("//").next().unwrap_or("");
            code.contains("unsafe ") || code.contains("unsafe{")
        });
        assert!(
            !uses_unsafe || name == "system.rs",
            "{name} must not contain unsafe code"
        );
    }
}

#[test]
fn workflow_and_policy_do_not_touch_the_outside_world() {
    for (name, text) in production_sources() {
        if matches!(
            name.as_str(),
            "wait.rs" | "cli.rs" | "interval.rs" | "time_of_day.rs"
        ) {
            for forbidden in ["SystemTime", "std::thread", "libc::", "std::env", "std::fs"] {
                assert!(!text.contains(forbidden), "{name} must not use {forbidden}");
            }
        }
    }
}

#[test]
fn production_files_stay_under_the_size_barometer() {
    for (name, text) in production_sources() {
        let production = text.split("#[cfg(test)]").next().unwrap();
        assert!(
            production.lines().count() <= 250,
            "{name} exceeds 250 lines"
        );
    }
}
