#![forbid(unsafe_code)]

use assert_cmd::prelude::*;
use spectral::assert_that;
use spectral::numeric::OrderedAssertions;
use std::path::{Path, PathBuf};

fn exec(cmd: impl AsRef<str>, params: impl AsRef<str>) -> String {
    let mut command = std::process::Command::new(cmd.as_ref());
    String::from_utf8(
        if params.as_ref().is_empty() {
            &mut command
        } else {
            command.args(params.as_ref().split(" "))
        }
        .assert()
        .success()
        .get_output()
        .stdout
        .clone(),
    )
    .unwrap()
    .trim()
    .to_string()
}

fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn rustc_path() -> PathBuf {
    Path::new(&std::env::var_os("CARGO").unwrap())
        .parent()
        .unwrap()
        .join("rustc")
}

#[test]
fn test() {
    let bd = build_data::BuildData::new(include_str!(concat!(env!("OUT_DIR"), "/build-data.txt")))
        .unwrap();
    let after = now();
    let before = after - 4 * 60 * 60;
    let expected_time =
        chrono::TimeZone::timestamp(&chrono::Utc, bd.time_seconds as i64, 0).to_rfc3339();
    assert_eq!(
        Some(exec("git", "rev-parse --abbrev-ref=loose HEAD")),
        bd.git_branch
    );
    assert_eq!(Some(exec("git", "rev-parse HEAD")), bd.git_commit);
    assert_eq!(Some(!exec("git", "status -s").is_empty()), bd.git_dirty);
    assert_eq!(Some(exec("hostname", "")), bd.hostname);
    assert_eq!(
        Some(exec(rustc_path().to_str().unwrap(), "--version")),
        bd.rustc_version
    );
    assert_eq!(expected_time, bd.time);
    assert_that(&bd.time_seconds).is_greater_than_or_equal_to(before);
    assert_that(&bd.time_seconds).is_less_than_or_equal_to(after);
}
