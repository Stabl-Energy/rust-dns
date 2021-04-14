use assert_cmd::prelude::*;
use safe_lock::SafeLock;
use safe_regex::{regex, Matcher7};
use spectral::assert_that;
use spectral::numeric::OrderedAssertions;
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};

static ENV_VARS_LOCK: SafeLock = SafeLock::new();

struct TempCurrentDirChange {
    previous_value: Option<PathBuf>,
}
impl TempCurrentDirChange {
    pub fn new(path: &Path) -> Self {
        let previous_value = std::env::current_dir().unwrap();
        std::env::set_current_dir(path).unwrap();
        Self {
            previous_value: Some(previous_value),
        }
    }
}
impl Drop for TempCurrentDirChange {
    fn drop(&mut self) {
        if let Some(value) = self.previous_value.take() {
            std::env::set_current_dir(&value).unwrap();
        }
    }
}

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

struct ParsedFile {
    git_branch: String,
    git_commit: String,
    git_dirty: bool,
    hostname: String,
    rustc_version: String,
    time: String,
    time_seconds: u64,
}

fn parse_file(path: &Path) -> Result<ParsedFile, Box<dyn std::error::Error>> {
    let contents = std::fs::read(path)?;
    // println!("file contents:\n>>>>\n{}\n<<<<", std::str::from_utf8(&contents).unwrap());
    let matcher: Matcher7<_> = regex!(
        br#"GIT_BRANCH:([^\n]*)
GIT_COMMIT:([^\n]*)
GIT_DIRTY:(true|false)
HOSTNAME:([^\n]*)
RUSTC_VERSION:([^\n]*)
TIME:([^\n]*)
TIME_SECONDS:([0-9]+)
"#
    );
    let (
        git_branch_bytes,
        git_commit_bytes,
        git_dirty_bytes,
        hostname_bytes,
        rustc_version_bytes,
        time_bytes,
        time_seconds_bytes,
    ) = matcher.match_slices(&contents).unwrap();
    Ok(ParsedFile {
        time_seconds: std::str::from_utf8(time_seconds_bytes)
            .unwrap()
            .parse()
            .unwrap(),
        time: String::from_utf8(time_bytes.to_vec()).unwrap(),
        hostname: String::from_utf8(hostname_bytes.to_vec()).unwrap(),
        git_commit: String::from_utf8(git_commit_bytes.to_vec()).unwrap(),
        git_branch: String::from_utf8(git_branch_bytes.to_vec()).unwrap(),
        git_dirty: git_dirty_bytes == "true".as_bytes(),
        rustc_version: String::from_utf8(rustc_version_bytes.to_vec()).unwrap(),
    })
}

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
fn exec_fails() {
    build_data_writer::exec("nonexistent-binary", &[]).unwrap_err();
}

#[test]
fn exec_nonzero_exit() {
    build_data_writer::exec("bash", &["-c", "echo err1; exit 1"]).unwrap_err();
}

#[test]
fn test_escape_ascii() {
    use build_data_writer::escape_ascii;
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
fn test_get_seconds_since_epoch() {
    let before = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let value = build_data_writer::get_seconds_since_epoch();
    let after = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert_that(&value).is_greater_than_or_equal_to(before);
    assert_that(&value).is_less_than_or_equal_to(after);
}

#[test]
fn test_format_iso8601() {
    assert_eq!(
        "2021-04-14T03:25:07+00:00",
        build_data_writer::format_iso8601(1618370707)
    );
}

#[test]
fn test_get_hostname() {
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
    assert_eq!(
        &expected_hostname,
        &build_data_writer::get_hostname().unwrap()
    );
}

#[test]
fn test_write_bad_filenames() {
    let dir = temp_dir::TempDir::new().unwrap();
    build_data_writer::write(&Path::new("")).unwrap_err();
    build_data_writer::write(dir.path()).unwrap_err();
    build_data_writer::write(&dir.path().join("nonexistent/build-data.txt")).unwrap_err();
}

#[test]
fn test_write_errors() {
    let _guard = ENV_VARS_LOCK.lock().unwrap();
    let dir = temp_dir::TempDir::new().unwrap();
    // Change the current directory to a directory that is not a git repository.
    // This will make the 'git' commands fail.
    let _cwd_guard = TempCurrentDirChange::new(dir.path());
    // Clear PATH so the 'hostname' command fails.
    let _change_guard = TempEnvVarChange::new("PATH", "");
    // Clear RUSTC so the 'rustc --version' command fails.
    let _change_guard = TempEnvVarChange::new("RUSTC", "");
    let file_path = dir.path().join("build-data.txt");
    let before = now();
    build_data_writer::write(&file_path).unwrap();
    let after = now();
    let bd = build_data::BuildData::new(std::fs::read_to_string(&file_path).unwrap()).unwrap();
    assert_eq!(None, bd.git_branch);
    assert_eq!(None, bd.git_commit);
    assert_eq!(None, bd.git_dirty);
    assert_eq!(None, bd.hostname);
    assert_eq!(None, bd.rustc_version);
    assert_eq!(build_data_writer::format_iso8601(bd.time_seconds), bd.time);
    assert_that(&bd.time_seconds).is_greater_than_or_equal_to(before);
    assert_that(&bd.time_seconds).is_less_than_or_equal_to(after);
}

#[test]
fn test_write() {
    // Set RUSTC env var.
    let rustc_path = rustc_path();
    let _guard = ENV_VARS_LOCK.lock().unwrap();
    let _change_guard = TempEnvVarChange::new("RUSTC", &rustc_path);

    let before: u64 = now();
    let file = temp_file::empty();
    build_data_writer::write(file.path()).unwrap();
    let after = now();

    let parsed_file = parse_file(file.path()).unwrap();
    let expected_time =
        chrono::TimeZone::timestamp(&chrono::Utc, parsed_file.time_seconds as i64, 0).to_rfc3339();

    assert_eq!(exec("git", "branch --show-current"), parsed_file.git_branch);
    assert_eq!(exec("git", "rev-parse HEAD"), parsed_file.git_commit);
    assert_eq!(!exec("git", "status -s").is_empty(), parsed_file.git_dirty);
    assert_eq!(exec("hostname", ""), parsed_file.hostname);
    assert_eq!(
        exec(rustc_path.to_str().unwrap(), "--version"),
        parsed_file.rustc_version
    );
    assert_eq!(expected_time, parsed_file.time);
    assert_that(&parsed_file.time_seconds).is_greater_than_or_equal_to(before);
    assert_that(&parsed_file.time_seconds).is_less_than_or_equal_to(after);

    let bd = build_data::BuildData::new(std::fs::read_to_string(file.path()).unwrap()).unwrap();
    assert_eq!(Some(parsed_file.git_branch), bd.git_branch);
    assert_eq!(Some(parsed_file.git_commit), bd.git_commit);
    assert_eq!(Some(parsed_file.git_dirty), bd.git_dirty);
    assert_eq!(Some(parsed_file.hostname), bd.hostname);
    assert_eq!(Some(parsed_file.rustc_version), bd.rustc_version);
    assert_eq!(parsed_file.time, bd.time);
    assert_eq!(parsed_file.time_seconds, bd.time_seconds);
}

#[test]
fn test_no_debug_rebuilds_debug() {
    let output = std::process::Command::cargo_bin("test_no_debug_rebuilds_debug")
        .unwrap()
        .output()
        .unwrap()
        .ok()
        .unwrap();
    assert_eq!(
        "cargo:rerun-if-env-changed=PROFILE\n",
        &String::from_utf8(output.stdout.clone()).unwrap()
    );
}

#[test]
fn test_no_debug_rebuilds_release() {
    let output = std::process::Command::cargo_bin("test_no_debug_rebuilds_release")
        .unwrap()
        .output()
        .unwrap()
        .ok()
        .unwrap();
    assert_eq!("", &String::from_utf8(output.stdout.clone()).unwrap());
}
