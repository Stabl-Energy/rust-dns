use assert_cmd::assert::OutputAssertExt;
use assert_cmd::cargo::CommandCargoExt;
use assert_cmd::output::OutputOkExt;
use chrono::{TimeZone, Timelike};
use core::ops::Range;
use core::sync::atomic::{AtomicU8, Ordering};
use core::time::Duration;
use rusty_fork::rusty_fork_test;
use spectral::assert_that;
use spectral::numeric::OrderedAssertions;
use std::convert::TryFrom;
use std::ffi::{OsStr, OsString};
use std::fmt::Debug;
use std::ops::RangeBounds;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;

static LOCK: safe_lock::SafeLock = safe_lock::SafeLock::new();

struct TempEnvVarChange {
    name: OsString,
    previous_value: Option<OsString>,
}
impl TempEnvVarChange {
    pub fn new(name: impl AsRef<OsStr>, value: impl AsRef<OsStr>) -> Self {
        let previous_value = std::env::var_os(name.as_ref());
        std::env::set_var(name.as_ref(), value.as_ref());
        Self {
            name: OsString::from(name.as_ref()),
            previous_value,
        }
    }
}
impl Drop for TempEnvVarChange {
    fn drop(&mut self) {
        if let Some(value) = self.previous_value.take() {
            std::env::set_var(&self.name, &value);
        }
    }
}

fn epoch_time() -> i64 {
    i64::try_from(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    )
    .unwrap_or(i64::MAX)
}

fn exec_cargo_bin(name: &str) -> String {
    String::from_utf8(
        std::process::Command::cargo_bin(name)
            .unwrap()
            .output()
            .unwrap()
            .ok()
            .unwrap()
            .stdout,
    )
    .unwrap()
}

fn expect_elapsed(before: Instant, range_ms: Range<u64>) {
    assert!(!range_ms.is_empty(), "invalid range {:?}", range_ms);
    let elapsed = before.elapsed();
    let duration_range = Duration::from_millis(range_ms.start)..Duration::from_millis(range_ms.end);
    assert!(
        duration_range.contains(&elapsed),
        "{:?} elapsed, out of range {:?}",
        elapsed,
        duration_range
    );
}

fn expect_in<T: Debug + PartialOrd>(
    value: &T,
    range: impl RangeBounds<T> + Debug,
) -> Result<(), String> {
    if !range.contains(value) {
        return Err(format!("value `{:?}` not in `{:?}`", value, range));
    }
    Ok(())
}

#[test]
fn once_i64() {
    use build_data::OnceI64;
    let value = OnceI64::new();
    assert_eq!(Ok(123), value.get(|| Ok(123)));
    assert_eq!(Ok(123), value.get(|| panic!()));
}

#[test]
fn once_i64_returns_min() {
    use build_data::OnceI64;
    let value = OnceI64::new();
    assert_eq!(Ok(i64::MIN), value.get(|| Ok(i64::MIN)));
    assert_eq!(Ok(123), value.get(|| Ok(123)));
    assert_eq!(Ok(123), value.get(|| panic!()));
}

#[test]
fn once_i64_static() {
    use build_data::OnceI64;
    static VALUE: OnceI64 = OnceI64::new();
    assert_eq!(Ok(123), VALUE.get(|| Ok(123)));
    assert_eq!(Ok(123), VALUE.get(|| panic!()));
}

#[test]
fn once_i64_concurrent() {
    use build_data::OnceI64;
    use spectral::iter::ContainingIntoIterAssertions;
    let value = Arc::new(OnceI64::new());
    let call_count = Arc::new(AtomicU8::new(0));
    let before = Instant::now();
    let mut handles = Vec::new();
    for n in &[111, 222] {
        let value_clone = value.clone();
        let call_count_clone = call_count.clone();
        let handle = std::thread::spawn(move || {
            value_clone.get(|| {
                call_count_clone.fetch_add(1, Ordering::Relaxed);
                std::thread::sleep(Duration::from_millis(100));
                Ok(*n)
            })
        });
        handles.push(handle);
    }
    let results: Vec<i64> = handles
        .drain(..)
        .map(|h| h.join().unwrap().unwrap())
        .collect();
    expect_elapsed(before, 50..150);
    value.get(|| panic!()).unwrap();
    assert_eq!(1, call_count.load(Ordering::Relaxed));
    assert_that(&[111, 222]).contains(&results[0]);
    for n in &results {
        assert_eq!(results[0], *n);
    }
}

#[test]
fn escape_ascii() {
    use build_data::escape_ascii;
    assert_eq!("", escape_ascii(b""));
    assert_eq!("abc", escape_ascii(b"abc"));
    assert_eq!("\\r\\n", escape_ascii(b"\r\n"));
    assert_eq!(
        "\\xe2\\x82\\xac",
        escape_ascii(/* Euro sign */ "\u{20AC}".as_bytes())
    );
    assert_eq!("\\x01", escape_ascii(b"\x01"));
}

#[test]
fn exec() {
    use spectral::string::StrAssertions;
    let _guard = LOCK.lock().unwrap();
    assert_that(&build_data::exec("nonexistent", &[]).unwrap_err().as_str())
        .contains("error executing");
    let err =
        build_data::exec("bash", &["-c", "echo stdout1; echo stderr1 >&2; exit 1"]).unwrap_err();
    assert_that(&err).contains("exit=1");
    assert_that(&err).contains("stdout='stdout1\\n'");
    assert_that(&err).contains("stderr='stderr1\\n'");
    assert_that(&build_data::exec("bash", &["-c", "kill $$"]).unwrap_err()).contains("exit=signal");
    assert_that(&build_data::exec("bash", &["-c", "echo -e '\\xc3\\x28'"]).unwrap_err())
        .contains("non-utf8");
    assert_eq!(
        "hello1",
        build_data::exec("bash", &["-c", "echo hello1"]).unwrap()
    );
    assert_eq!(
        "hello1",
        build_data::exec("bash", &["-c", "echo ' hello1 '"]).unwrap()
    );
}

#[test]
#[allow(clippy::unreadable_literal)]
fn format_date() {
    assert_eq!("2021-04-14Z", build_data::format_date(1618370707));
}

#[test]
#[allow(clippy::unreadable_literal)]
fn format_time() {
    assert_eq!("03:25:07Z", build_data::format_time(1618370707));
}

#[test]
#[allow(clippy::unreadable_literal)]
fn format_timestamp() {
    assert_eq!(
        "2021-04-14T03:25:07Z",
        build_data::format_timestamp(1618370707)
    );
}

#[test]
fn test_now() {
    let before = epoch_time();
    let value: i64 = build_data::now();
    let after = epoch_time();
    assert_eq!(value, build_data::now());
    expect_in(&value, before..=after).unwrap();
}

#[test]
fn get_env() {
    use spectral::string::StrAssertions;
    use std::os::unix::ffi::OsStringExt;
    assert_eq!(None, build_data::get_env("NONEXISTENT_ENV_VAR").unwrap());
    std::env::set_var("TEST_GET_ENV__EMPTY", "");
    assert_eq!(None, build_data::get_env("TEST_GET_ENV__EMPTY").unwrap());
    std::env::set_var("TEST_GET_ENV__WHITESPACE", " ");
    assert_eq!(
        None,
        build_data::get_env("TEST_GET_ENV__WHITESPACE").unwrap()
    );
    std::env::set_var("TEST_GET_ENV__VALUE", "value1");
    assert_eq!(
        "value1",
        &build_data::get_env("TEST_GET_ENV__VALUE").unwrap().unwrap()
    );
    std::env::set_var("TEST_GET_ENV__TRIM", " value1 ");
    assert_eq!(
        "value1",
        &build_data::get_env("TEST_GET_ENV__TRIM").unwrap().unwrap()
    );
    let non_utf8: OsString = OsString::from_vec(vec![0xC3_u8, 0x28]);
    std::env::set_var("TEST_GET_ENV__VAR_NON_UTF8", non_utf8);
    assert_that(&build_data::get_env("TEST_GET_ENV__VAR_NON_UTF8").unwrap_err())
        .contains("non-utf8");
}

#[test]
fn get_git_branch() {
    let _guard = LOCK.lock().unwrap();
    let value: String = build_data::get_git_branch().unwrap();
    let matcher: safe_regex::Matcher0<_> = safe_regex::regex!(br"[-_.+a-zA-Z0-9]+");
    assert!(matcher.is_match(value.as_bytes()), "{:?}", value);

    assert_eq!(
        format!("cargo:rustc-env=GIT_BRANCH={}\n", value),
        exec_cargo_bin("test_set_git_branch")
    );
}

#[test]
fn get_git_commit() {
    let _guard = LOCK.lock().unwrap();
    let value: String = build_data::get_git_commit().unwrap();
    assert!(safe_regex::regex!(br"[0-9a-f]{40}").is_match(value.as_bytes()));
    assert_eq!(
        format!("cargo:rustc-env=GIT_COMMIT={}\n", value),
        exec_cargo_bin("test_set_git_commit")
    );
}

#[test]
fn get_git_commit_short() {
    let _guard = LOCK.lock().unwrap();
    let value: String = build_data::get_git_commit_short().unwrap();
    assert!(safe_regex::regex!(br"[0-9a-f]{7}").is_match(value.as_bytes()));
    assert_eq!(
        format!("cargo:rustc-env=GIT_COMMIT_SHORT={}\n", value),
        exec_cargo_bin("test_set_git_commit_short")
    );
}

#[test]
fn get_git_dirty() {
    let _guard = LOCK.lock().unwrap();
    let value = build_data::get_git_dirty().unwrap();
    assert_eq!(
        format!("cargo:rustc-env=GIT_DIRTY={}\n", value),
        exec_cargo_bin("test_set_git_dirty")
    );
    if value {
        return;
    }
    let path = std::env::current_dir()
        .unwrap()
        .join("test_get_git_dirty.tmp");
    std::fs::write(&path, "a").unwrap();
    let value = build_data::get_git_dirty().unwrap();
    std::fs::remove_file(&path).unwrap();
    assert!(value);
}

#[test]
fn get_hostname() {
    let _guard = LOCK.lock().unwrap();
    let expected_hostname = String::from_utf8(
        std::process::Command::new("bash")
            .arg("-lc")
            .arg("echo $HOSTNAME")
            .assert()
            .success()
            .get_output()
            .stdout
            .clone(),
    )
    .unwrap()
    .trim()
    .to_string();
    assert_eq!(&expected_hostname, &build_data::get_hostname().unwrap());

    assert_eq!(
        format!("cargo:rustc-env=BUILD_HOSTNAME={}\n", expected_hostname),
        exec_cargo_bin("test_set_build_hostname")
    );
}

#[test]
fn get_rustc_version() {
    let _guard = LOCK.lock().unwrap();
    let _change_guard = TempEnvVarChange::new(
        "RUSTC",
        &Path::new(&std::env::var_os("CARGO").unwrap())
            .parent()
            .unwrap()
            .join("rustc"),
    );
    let value: String = build_data::get_rustc_version().unwrap();
    // rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
    let matcher: safe_regex::Matcher0<_> =
        safe_regex::regex!(br"rustc [0-9]+\.[0-9]+\.[0-9]+(?:-nightly|-beta)?(?: .*)?");
    assert!(matcher.is_match(value.as_bytes()));

    assert_eq!(
        format!("cargo:rustc-env=RUSTC_VERSION={}\n", value),
        exec_cargo_bin("test_set_rustc_version")
    );
    assert_eq!(
        format!(
            "cargo:rustc-env=RUSTC_VERSION_SEMVER={}\n",
            build_data::parse_rustc_semver(&value).unwrap()
        ),
        exec_cargo_bin("test_set_rustc_version_semver")
    );
    assert_eq!(
        format!(
            "cargo:rustc-env=RUST_CHANNEL={}\n",
            build_data::parse_rustc_channel(&value).unwrap()
        ),
        exec_cargo_bin("test_set_rust_channel")
    );

    let _change_guard = TempEnvVarChange::new("RUSTC", "");
    build_data::get_rustc_version().unwrap_err();
}

#[test]
fn rust_channel() {
    assert_eq!("stable", &format!("{}", build_data::RustChannel::Stable));
    assert_eq!("beta", &format!("{}", build_data::RustChannel::Beta));
    assert_eq!("nightly", &format!("{}", build_data::RustChannel::Nightly));
}

#[test]
fn parse_rustc_version() {
    use build_data::RustChannel;
    build_data::parse_rustc_version("").unwrap_err();
    build_data::parse_rustc_version("not a rustc version").unwrap_err();
    build_data::parse_rustc_version("rustc1.2.3").unwrap_err();
    build_data::parse_rustc_version("other 1.2.3").unwrap_err();
    build_data::parse_rustc_version("1").unwrap_err();
    build_data::parse_rustc_version("1.2").unwrap_err();
    build_data::parse_rustc_version("other 1..3").unwrap_err();
    build_data::parse_rustc_version("1.2.3-invalid").unwrap_err();
    build_data::parse_rustc_version("1.2.3x").unwrap_err();
    build_data::parse_rustc_version("1.2.3-nightlyX").unwrap_err();
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Stable),
        build_data::parse_rustc_version("rustc 1.53.0 (07e0e2ec2 2021-03-24)").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Beta),
        build_data::parse_rustc_version("rustc 1.53.0-beta (07e0e2ec2 2021-03-24)").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Nightly),
        build_data::parse_rustc_version("rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Stable),
        build_data::parse_rustc_version("1.53.0 (07e0e2ec2 2021-03-24)").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Stable),
        build_data::parse_rustc_version("rustc 1.53.0").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Stable),
        build_data::parse_rustc_version("1.53.0").unwrap()
    );
    assert_eq!(
        (String::from("1.53.0"), RustChannel::Nightly),
        build_data::parse_rustc_version("1.53.0-nightly").unwrap()
    );
}

#[test]
fn parse_rustc_semver() {
    assert_eq!(
        String::from("1.53.0"),
        build_data::parse_rustc_semver("rustc 1.53.0 (07e0e2ec2 2021-03-24)").unwrap()
    );
}

#[test]
fn parse_rustc_channel() {
    assert_eq!(
        build_data::RustChannel::Beta,
        build_data::parse_rustc_channel("rustc 1.53.0-beta (07e0e2ec2 2021-03-24)").unwrap()
    );
}

#[allow(clippy::unreadable_literal)]
fn get_source_time_() {
    use spectral::string::StrAssertions;
    let _source_date_epoch_guard = TempEnvVarChange::new("SOURCE_DATE_EPOCH", "");
    let path_guard = TempEnvVarChange::new(
        "PATH",
        std::env::current_dir()
            .unwrap()
            .join("tests")
            .join("fake-bin"),
    );
    let err = build_data::get_source_time().unwrap_err();
    assert_that(&err).contains("failed parsing");
    assert_that(&err).contains("git");
    drop(path_guard);
    let value: i64 = build_data::get_source_time().unwrap();
    assert_that(&value).is_greater_than(1618400000);
    std::thread::sleep(Duration::from_millis(1100));
    assert_eq!(value, build_data::get_source_time().unwrap());
}
rusty_fork_test! {
    #[test]
    fn get_source_time() { get_source_time_() }
}

fn source_date_epoch_() {
    use spectral::string::StrAssertions;
    let change_guard = TempEnvVarChange::new("SOURCE_DATE_EPOCH", "not-digits");
    assert_that(&build_data::get_source_time().unwrap_err()).contains("failed parsing");
    drop(change_guard);
    let _change_guard = TempEnvVarChange::new("SOURCE_DATE_EPOCH", "123");
    assert_eq!(123, build_data::get_source_time().unwrap());
    assert_eq!(123, build_data::get_source_time().unwrap());

    assert_eq!(
        "cargo:rustc-env=SOURCE_DATE=1970-01-01Z\n",
        exec_cargo_bin("test_set_source_date")
    );
    assert_eq!(
        "cargo:rustc-env=SOURCE_TIME=00:02:03Z\n",
        exec_cargo_bin("test_set_source_time")
    );
    assert_eq!(
        "cargo:rustc-env=SOURCE_TIMESTAMP=1970-01-01T00:02:03Z\n",
        exec_cargo_bin("test_set_source_timestamp")
    );
    assert_eq!(
        "cargo:rustc-env=SOURCE_EPOCH_TIME=123\n",
        exec_cargo_bin("test_set_source_epoch_time")
    );
}
rusty_fork_test! {
    #[test]
    fn source_date_epoch() { source_date_epoch_() }
}

#[test]
fn no_debug_rebuilds_debug() {
    let _guard = LOCK.lock().unwrap();
    assert_eq!(
        "cargo:rerun-if-env-changed=PROFILE\n",
        exec_cargo_bin("test_no_debug_rebuilds_debug")
    );
}

#[test]
fn no_debug_rebuilds_release() {
    let _guard = LOCK.lock().unwrap();
    assert_eq!("", exec_cargo_bin("test_no_debug_rebuilds_release"));
}

#[test]
fn set_build_date() {
    use spectral::iter::ContainingIntoIterAssertions;
    let _guard = LOCK.lock().unwrap();
    let before = chrono::Utc.timestamp(epoch_time(), 0).date().naive_utc();
    let stdout = exec_cargo_bin("test_set_build_date");
    let after = chrono::Utc.timestamp(epoch_time(), 0).date().naive_utc();
    let value =
        chrono::NaiveDate::parse_from_str(&stdout, "cargo:rustc-env=BUILD_DATE=%Y-%m-%dZ\n")
            .map_err(|e| {
                format!(
                    "error parsing output '{}': {}",
                    build_data::escape_ascii(&stdout),
                    e
                )
            })
            .unwrap();
    assert_that(&[before, after]).contains(&value);
}

#[test]
fn set_build_time() {
    let _guard = LOCK.lock().unwrap();
    let before = epoch_time();
    let stdout = exec_cargo_bin("test_set_build_time");
    let after = epoch_time();
    let time = chrono::NaiveTime::parse_from_str(&stdout, "cargo:rustc-env=BUILD_TIME=%H:%M:%SZ\n")
        .map_err(|e| {
            format!(
                "error parsing output '{}': {}",
                build_data::escape_ascii(&stdout),
                e
            )
        })
        .unwrap();
    let value = chrono::Utc
        .timestamp(if time.hour() == 0 { after } else { before }, 0)
        .date()
        .and_time(time)
        .unwrap()
        .timestamp();
    expect_in(&value, before..=after).unwrap();
}

#[test]
fn set_build_timestamp() {
    let _guard = LOCK.lock().unwrap();
    let before = epoch_time();
    let stdout = exec_cargo_bin("test_set_build_timestamp");
    let after = epoch_time();
    let value = chrono::Utc
        .datetime_from_str(
            &stdout,
            "cargo:rustc-env=BUILD_TIMESTAMP=%Y-%m-%dT%H:%M:%SZ\n",
        )
        .map_err(|e| {
            format!(
                "error parsing output '{}': {}",
                build_data::escape_ascii(&stdout),
                e
            )
        })
        .unwrap()
        .timestamp();
    expect_in(&value, before..=after).unwrap();
}

#[test]
fn set_build_epoch_time() {
    let _guard = LOCK.lock().unwrap();
    let before = epoch_time();
    let stdout = exec_cargo_bin("test_set_build_epoch_time");
    let after = epoch_time();
    let value = chrono::Utc
        .datetime_from_str(&stdout, "cargo:rustc-env=BUILD_EPOCH_TIME=%s\n")
        .map_err(|e| {
            format!(
                "error parsing output '{}': {}",
                build_data::escape_ascii(&stdout),
                e
            )
        })
        .unwrap()
        .timestamp();
    expect_in(&value, before..=after).unwrap();
}
