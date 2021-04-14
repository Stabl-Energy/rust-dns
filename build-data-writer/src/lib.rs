//! [![crates.io version](https://img.shields.io/crates/v/build-data-writer.svg)](https://crates.io/crates/build-data-writer)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/build-data-writer/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # build-data-writer
//!
//! Functions to to write `build-data.txt` from your
//! [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
//! Read the file with the
//! [`build-data`](https://crates.io/crates/build-data) crate.
//!
//! ## Features
//! - Saves build-time data:
//!   - Timestamp
//!   - Date-time string
//!   - Hostname
//!   - git commit, branch, and dirtiness
//!   - rustc version
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Alternatives
//! - [`build-info`](https://crates.io/crates/build-info)
//!   - Mature & popular
//!   - Confusing API
//!   - Uses procedural macros
//!
//! ## Example
//! See [`build-data`](https://crates.io/crates/build-data) crate docs.
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.1 - Initial version
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]

// https://doc.rust-lang.org/cargo/reference/build-scripts.html
// https://doc.rust-lang.org/cargo/reference/build-script-examples.html

use chrono::TimeZone;
use std::convert::TryFrom;
use std::ffi::OsStr;

/// Executes `cmd` with `args` as parameters, waits for it to exit, and
/// returns its stdout, trimmed, and escaped with
/// [`escape_ascii`](#method.escape_ascii).
///
/// # Errors
/// Returns a descriptive error string if it fails to execute the command
/// or if the command exits with a non-zero status.
///
/// # Panics
/// Panics if the process writes non-UTF bytes to stdout.
pub fn exec(cmd: impl AsRef<OsStr>, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(cmd.as_ref())
        .args(args)
        .output()
        .map_err(|e| {
            format!(
                "error executing '{} {}': {}",
                cmd.as_ref().to_string_lossy(),
                args.join(" "),
                e
            )
        })?;
    if !output.status.success() {
        return Err(format!(
            "command '{} {}' failed: status={} stdout='{}' stderr='{}'",
            cmd.as_ref().to_string_lossy(),
            args.join(" "),
            output.status,
            escape_ascii(output.stdout),
            escape_ascii(output.stderr)
        ));
    }
    Ok(escape_ascii(std::str::from_utf8(&output.stdout).unwrap().trim()).replace('"', "\\"))
}

/// Converts a byte slice into a string using
/// [`core::ascii::escape_default`](https://doc.rust-lang.org/core/ascii/fn.escape_default.html)
/// to escape each byte.
///
/// # Example
/// ```
/// use build_data_writer::escape_ascii;
/// assert_eq!("abc", escape_ascii(b"abc"));
/// assert_eq!("abc\\n", escape_ascii(b"abc\n"));
/// assert_eq!(
///     "Euro sign: \\xe2\\x82\\xac",
///     escape_ascii("Euro sign: \u{20AC}".as_bytes())
/// );
/// assert_eq!("\\x01\\x02\\x03", escape_ascii(&[1, 2, 3]));
/// ```
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn escape_ascii(input: impl AsRef<[u8]>) -> String {
    let mut result = String::new();
    for byte in input.as_ref() {
        for ascii_byte in core::ascii::escape_default(*byte) {
            result.push_str(core::str::from_utf8(&[ascii_byte]).unwrap());
        }
    }
    result
}

/// Gets the number of seconds since 1970-01-01T00:00:00Z.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn get_seconds_since_epoch() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

/// Formats the time as an ISO-8601 or RFC-3339 string.
/// Example: `2021-04-14T06:25:59+00:00`
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn format_iso8601(seconds_since_epoch: u64) -> String {
    chrono::Utc
        .timestamp(i64::try_from(seconds_since_epoch).unwrap(), 0)
        .to_rfc3339()
}

/// Gets the name of the current machine.
///
/// # Errors
/// Returns an error if it fails to execute the `hostname` command.
pub fn get_hostname() -> Result<String, String> {
    // Cargo doesn't pass the `HOSTNAME` env var to build scripts.
    exec("hostname", &[])
}

/// Gets the latest git commit of the source code directory.
///
/// Example: `"a5547bfb1edb9712588f0f85d3e2c8ba618ac51f"`
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn get_git_commit() -> Result<String, String> {
    exec("git", &["rev-parse", "HEAD"])
}

/// Gets the current branch of the source code directory.
///
/// Example: `"release"`
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn get_git_branch() -> Result<String, String> {
    exec("git", &["branch", "--show-current"])
}

/// Returns `true` if the source directory contains uncommitted changes.
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn is_git_dirty() -> Result<bool, String> {
    Ok(!exec("git", &["status", "-s"])?.is_empty())
}

/// Gets the version of the Rust compiler used to build the build script.
///
/// Example: `"rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)"`
///
/// # Errors
/// Returns an error if it fails to execute the `rustc` command.
pub fn get_rustc_version() -> Result<String, String> {
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
    let rustc_var = std::env::var_os("RUSTC")
        .filter(|s| !s.is_empty())
        .ok_or_else(|| String::from("RUSTC env var is not set"))?;
    exec(rustc_var, &["--version"])
}

/// Writes `build-data.txt`.
///
/// Uses `"unknown"` when it fails to get a particular value.
///
/// # Example File
/// ```text
/// GIT_BRANCH:release
/// GIT_COMMIT:a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
/// GIT_DIRTY:false
/// HOSTNAME:builder2
/// RUSTC_VERSION:rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
/// TIME:2021-04-14T06:25:59+00:00
/// TIME_SECONDS:1618381559
/// ```
///
/// # Example
/// Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
/// file next to your `Cargo.toml`:
/// ```
/// // build.rs
/// use std::env;
/// use std::path::Path;
///
/// fn main() {
/// # }
/// # fn f() {
///     build_data_writer::write(
///         &Path::new(&env::var_os("OUT_DIR").unwrap()).join("build-data.txt")
///     ).unwrap();
/// }
/// ```
///
/// Include the file in your program and parse it.
/// See [`include_str!`](https://doc.rust-lang.org/core/macro.include_str.html),
/// [`concat!`](https://doc.rust-lang.org/core/macro.concat.html),
/// [`env!`](https://doc.rust-lang.org/core/macro.env.html), and
/// [`build_data::BuildData::new`](https://docs.rs/build-data/latest/build_data/struct.BuildData.html#method.new).
/// ```
/// # macro_rules! include_str (
/// #     ($arg:expr) => { "GIT_BRANCH:1\nGIT_COMMIT:abcdef123\nGIT_DIRTY:false\nHOSTNAME:h\nRUSTC_VERSION:2\nTIME:123\nTIME_SECONDS:123" };
/// # );
/// # macro_rules! concat (
/// #     ($arg:expr) => { "" };
/// # );
/// # macro_rules! env (
/// #     ($arg:expr) => { "" };
/// # );
/// fn main() {
///     let bd = build_data::BuildData::new(
///         include_str!(concat!(env!("OUT_DIR"), "/build-data.txt"))
///     ).unwrap();
///     // ...
/// }
/// ```
///
/// # Errors
/// Returns an error if it fails to write the file.
pub fn write(path: &std::path::Path) -> Result<(), std::io::Error> {
    let mut contents = String::new();
    match get_git_branch() {
        Ok(value) => contents.push_str(&format!("GIT_BRANCH:{}\n", value)),
        Err(e) => eprint!(
            "WARNING build-data-writer: failed getting git branch: {}",
            e
        ),
    }
    match get_git_commit() {
        Ok(value) => contents.push_str(&format!("GIT_COMMIT:{}\n", value)),
        Err(e) => eprint!(
            "WARNING build-data-writer: failed getting git commit: {}",
            e
        ),
    }
    match is_git_dirty() {
        Ok(value) => contents.push_str(&format!("GIT_DIRTY:{}\n", value)),
        Err(e) => eprint!(
            "WARNING build-data-writer: failed getting git dirtiness: {}",
            e
        ),
    }
    match get_hostname() {
        Ok(value) => contents.push_str(&format!("HOSTNAME:{}\n", value)),
        Err(e) => eprint!("WARNING build-data-writer: failed getting hostname: {}", e),
    }
    match get_rustc_version() {
        Ok(value) => contents.push_str(&format!("RUSTC_VERSION:{}\n", value)),
        Err(e) => eprint!("WARNING build-data-writer: failed rustc version: {}", e),
    }
    let seconds_since_epoch = get_seconds_since_epoch();
    contents.push_str(&format!("TIME:{}\n", format_iso8601(seconds_since_epoch)));
    contents.push_str(&format!("TIME_SECONDS:{}\n", seconds_since_epoch));
    std::fs::write(&path, contents)
}

/// Tells cargo not to rebuild `build.rs` during debug builds when other files
/// change.
/// This speeds up development builds.
pub fn no_debug_rebuilds() {
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
    // "PROFILE — release for release builds, debug for other builds."
    if std::env::var_os("PROFILE")
        .filter(|s| s == "debug")
        .is_some()
    {
        // https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection
        // "The rerun-if-env-changed instruction tells Cargo to re-run the
        //  build script if the value of an environment variable of the
        //  given name has changed."
        println!("cargo:rerun-if-env-changed=PROFILE");
    }
}
