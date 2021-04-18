//! [![crates.io version](https://img.shields.io/crates/v/build-data.svg)](https://crates.io/crates/build-data)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/build-data/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # build-data
//!
//! Include build data in your program.
//!
//! ## Features
//! - Saves build-time data:
//!   - Git commit, branch, and dirtiness
//!   - Source modification date & time
//!   - Rustc version
//!   - Rust channel (stable, nightly, or beta)
//! - Does all of its work in your
//!   [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
//! - Sets environment variables.
//!   Use [`env!`](https://doc.rust-lang.org/core/macro.env.html) to use them
//!   in your program.
//! - No macros
//! - No runtime dependencies
//! - Light build dependencies
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Alternatives
//! - Environment variables that cargo sets for crates:
//!   - `CARGO_PKG_NAME`
//!   - `CARGO_PKG_VERSION`
//!   - `CARGO_BIN_NAME`
//!   - [others](https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates)
//! - [`vergen`](https://crates.io/crates/vergen)
//!   - Mature & very popular
//!   - Good API, needs only `env!` to retrieve values
//!   - Excellent test coverage
//!   - Heavy build dependencies
//! - [`build-info`](https://crates.io/crates/build-info)
//!   - Mature
//!   - Confusing API
//!   - Uses procedural macros
//!
//! # Example
//!
//! ```toml
//! // Cargo.toml
//! [dependencies]
//!
//! [build-dependencies]
//! build-data = "0"
//! ```
//!
//! Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
//! file next to your `Cargo.toml`.
//! Call [`build_data::set_*`](https://docs.rs/build-data/) functions to
//! set variables.
//! ```
//! // build.rs
//!
//! fn main() {
//! # }
//! # fn f() {
//!     build_data::set_GIT_BRANCH();
//!     build_data::set_GIT_COMMIT();
//!     build_data::set_GIT_DIRTY();
//!     build_data::set_SOURCE_TIMESTAMP();
//!     build_data::no_debug_rebuilds();
//! }
//! ```
//!
//! Use [`env!`](https://doc.rust-lang.org/core/macro.env.html) to access the
//! variables in your program:
//! ```
//! // src/bin/main.rs
//! # macro_rules! env (
//! #     ($arg:expr) => { "" };
//! # );
//! fn main() {
//!     // Built from branch=release
//!     // commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
//!     // dirty=false
//!     // source_timestamp=2021-04-14T06:25:59+00:00
//!     println!("Built from branch={} commit={} dirty={} source_timestamp={}",
//!         env!("GIT_BRANCH"),
//!         env!("GIT_COMMIT"),
//!         env!("GIT_DIRTY"),
//!         env!("SOURCE_TIMESTAMP"),
//!     );
//! }
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.3 - Update docs.
//! - v0.1.2 - Rewrote based on
//!     [feedback](https://www.reddit.com/r/rust/comments/mqnbvw/)
//!     from r/rust.
//! - v0.1.1 - Update docs.
//! - v0.1.0 - Initial version
//!
//! ## To Do
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]
#![allow(non_snake_case)]

// https://doc.rust-lang.org/cargo/reference/build-scripts.html
// https://doc.rust-lang.org/cargo/reference/build-script-examples.html

use safe_lock::SafeLock;
use std::convert::TryFrom;
use std::ffi::OsStr;
use std::sync::atomic::{AtomicI64, Ordering};

/// Caches an i64.
pub struct OnceI64 {
    cached_value: AtomicI64,
    lock: safe_lock::SafeLock,
}
impl OnceI64 {
    /// Makes a new empty struct.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            cached_value: AtomicI64::new(i64::MIN),
            lock: SafeLock::new(),
        }
    }
    /// Gets the value if it was already created, otherwise calls `f` to create it.
    ///
    /// Thread-safe.
    ///
    /// # Errors
    /// Returns any error returned by `f`.
    #[allow(clippy::missing_panics_doc)]
    pub fn get(&self, f: impl FnOnce() -> Result<i64, String>) -> Result<i64, String> {
        let _guard = self.lock.lock().unwrap();
        let value = self.cached_value.load(Ordering::Relaxed);
        if value != i64::MIN {
            return Ok(value);
        }
        let new_value = (f)()?;
        self.cached_value.store(new_value, Ordering::Relaxed);
        Ok(new_value)
    }
}

/// Converts a byte slice into a string using
/// [`core::ascii::escape_default`](https://doc.rust-lang.org/core/ascii/fn.escape_default.html)
/// to escape each byte.
///
/// # Example
/// ```
/// use build_data::escape_ascii;
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
            "command '{} {}' failed: exit={} stdout='{}' stderr='{}'",
            cmd.as_ref().to_string_lossy(),
            args.join(" "),
            output
                .status
                .code()
                .map_or_else(|| String::from("signal"), |c| c.to_string()),
            escape_ascii(output.stdout),
            escape_ascii(output.stderr)
        ));
    }
    let stdout = std::str::from_utf8(&output.stdout).map_err(|_| {
        format!(
            "command '{} {}' wrote non-utf8 bytes to stdout",
            cmd.as_ref().to_string_lossy(),
            args.join(" ")
        )
    })?;
    Ok(escape_ascii(stdout.trim()).replace('"', "\\"))
}

/// Formats the epoch timestamp as a UTC date like `"2021-05-04Z"`.
#[must_use]
pub fn format_date(epoch: i64) -> String {
    chrono::TimeZone::timestamp(&chrono::Utc, epoch, 0)
        .format("%Y-%m-%dZ")
        .to_string()
}

/// Formats the epoch timestamp as a UTC time like `"13:02:59Z"`.
#[must_use]
pub fn format_time(epoch: i64) -> String {
    chrono::TimeZone::timestamp(&chrono::Utc, epoch, 0)
        .format("%H:%M:%SZ")
        .to_string()
}

/// Formats the epoch timestamp as a UTC timestamp like `"20201-05-04T13:02:59Z"`.
#[must_use]
pub fn format_timestamp(epoch: i64) -> String {
    chrono::TimeZone::timestamp(&chrono::Utc, epoch, 0)
        .to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
}

/// Gets the current time as an epoch timestamp.
#[allow(clippy::missing_panics_doc)]
#[must_use]
pub fn now() -> i64 {
    static CACHED_VALUE: OnceI64 = OnceI64::new();
    CACHED_VALUE
        .get(|| {
            Ok(i64::try_from(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            )
            .unwrap())
        })
        .unwrap()
}

/// Gets the environment variable named `name` if it is set.
///
/// Returns `None` if the variable is unset, is empty, or contains only whitespace.
///
/// Trims whitespace from the start and end of the value before returning it.
///
/// # Errors
/// Returns an error if the environment variable value is not valid utf-8.
pub fn get_env(name: &str) -> Result<Option<String>, String> {
    let value = match std::env::var(name) {
        Ok(value) => value,
        Err(std::env::VarError::NotPresent) => return Ok(None),
        Err(std::env::VarError::NotUnicode(_)) => {
            return Err(format!("env var '{}' contains non-utf8 bytes", name))
        }
    };
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Ok(None);
    }
    Ok(Some(trimmed.to_string()))
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

/// Gets the latest git commit of the source code directory.
/// Returns the truncated hash.
///
/// Example: `"a5547bf"`
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn get_git_commit_short() -> Result<String, String> {
    let long = get_git_commit()?;
    if long.len() < 7 {
        return Err(format!("got malformed commit hash from git: '{}'", long));
    }
    let short = &long[0..7];
    Ok(short.to_string())
}

/// Gets the current branch of the source code directory.
///
/// Example: `"release"`
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn get_git_branch() -> Result<String, String> {
    exec("git", &["rev-parse", "--abbrev-ref=loose", "HEAD"])
}

/// Returns `true` if the source directory contains uncommitted changes.
///
/// # Errors
/// Returns an error if it fails to execute the `git` command.
pub fn get_git_dirty() -> Result<bool, String> {
    Ok(!exec("git", &["status", "-s"])?.is_empty())
}

/// Gets the name of the current machine.
///
/// Cargo doesn't pass the `HOSTNAME` env var to build scripts.
/// Uses the `hostname` command.
///
/// # Errors
/// Returns an error if it fails to execute the `hostname` command.
pub fn get_hostname() -> Result<String, String> {
    exec("hostname", &[])
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

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Debug)]
pub enum RustChannel {
    Stable,
    Beta,
    Nightly,
}
impl core::fmt::Display for RustChannel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            RustChannel::Stable => write!(f, "stable"),
            RustChannel::Beta => write!(f, "beta"),
            RustChannel::Nightly => write!(f, "nightly"),
        }
    }
}

/// Parses the output of `rustc --version`.
///
/// # Errors
/// Returns an error if it fails to execute `rustc`,
/// the process exits with non-zero status,
/// or it fails to parse the output.
///
/// # Panics
/// Panics if the process writes non-UTF bytes to stdout.
#[allow(clippy::if_not_else, clippy::range_plus_one, clippy::assign_op_pattern)]
pub fn parse_rustc_version(version: impl AsRef<str>) -> Result<(String, RustChannel), String> {
    let matcher: safe_regex::Matcher3<_> =
        safe_regex::regex!(br"(?:rustc )?([0-9]+\.[0-9]+\.[0-9]+)(?:(-beta)|(-nightly))?(?: .*)?");
    let (semver_bytes, beta, nightly) = matcher
        .match_slices(version.as_ref().trim().as_bytes())
        .ok_or_else(|| format!("failed parsing rustc version: '{}'", version.as_ref()))?;
    let semver = String::from_utf8(semver_bytes.to_vec()).unwrap();
    let channel = if !beta.is_empty() {
        RustChannel::Beta
    } else if !nightly.is_empty() {
        RustChannel::Nightly
    } else {
        RustChannel::Stable
    };
    Ok((semver, channel))
}

/// Gets the dotted-numeric version from the rustc version string.
///
/// Example: `"1.53.0"`
///
/// # Errors
/// Returns an error if it fails to parse `version`.
#[allow(clippy::missing_panics_doc)]
pub fn parse_rustc_semver(version: impl AsRef<str>) -> Result<String, String> {
    let (semver, _channel) = parse_rustc_version(version)?;
    Ok(semver)
}

/// Gets the channel from the rustc version string.
///
/// # Errors
/// Returns an error if it fails to parse `version`.
#[allow(clippy::missing_panics_doc)]
pub fn parse_rustc_channel(version: impl AsRef<str>) -> Result<RustChannel, String> {
    let (_semver, channel) = parse_rustc_version(version)?;
    Ok(channel)
}

/// Gets the modification time of the source code.
///
/// Reads the
/// [`SOURCE_DATE_EPOCH`](https://reproducible-builds.org/docs/source-date-epoch/)
/// env var if set.  Otherwise, runs `git` to get the value.
///
/// # Errors
/// Returns an error when:
/// - `SOURCE_DATE_EPOCH` is non-empty and cannot be parsed as an `i64`.
/// - it failed to execute `git`
/// - `git` exited with non-zero status
/// - `git` wrote stdout data that cannot be parsed as an `i64`.
///
/// # Panics
/// Panics if `git` writes non-UTF bytes to stdout.
pub fn get_source_time() -> Result<i64, String> {
    static CACHED_VALUE: OnceI64 = OnceI64::new();
    CACHED_VALUE.get(|| {
        if let Some(value) = get_env("SOURCE_DATE_EPOCH").unwrap() {
            return value.parse().map_err(|_| {
                format!(
                    "failed parsing env var as i64: SOURCE_DATE_EPOCH='{}'",
                    value
                )
            });
        }
        let stdout = exec("git", &["log", "-1", "--pretty=%ct"])?;
        stdout.parse().map_err(|_| {
            format!(
                "failed parsing output of 'git log -1 --pretty=%ct' as i64: {}",
                stdout
            )
        })
    })
}

/// Tells cargo not to rebuild `build.rs` during debug builds when other files
/// change.
///
/// This speeds up development builds.
#[allow(clippy::missing_panics_doc)]
pub fn no_debug_rebuilds() {
    // https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-build-scripts
    // "PROFILE — release for release builds, debug for other builds."
    if &get_env("PROFILE")
        .unwrap()
        .expect("PROFILE env var not set")
        == "debug"
    {
        // https://doc.rust-lang.org/cargo/reference/build-scripts.html#change-detection
        // "The rerun-if-env-changed instruction tells Cargo to re-run the
        //  build script if the value of an environment variable of the
        //  given name has changed."
        println!("cargo:rerun-if-env-changed=PROFILE");
    }
}

/// Sets the `SOURCE_DATE` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"2021-04-14Z"`
///
/// # Panics
/// Panics if `SOURCE_DATE_EPOCH` env var is set to a non-integer value.
/// Panics if it fails to get the timestamp from `git`.
pub fn set_SOURCE_DATE() {
    println!(
        "cargo:rustc-env=SOURCE_DATE={}",
        format_date(get_source_time().unwrap())
    );
}

/// Sets the `SOURCE_TIME` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"03:25:07Z"`
///
/// # Panics
/// Panics if `SOURCE_DATE_EPOCH` env var is set to a non-integer value.
/// Panics if it fails to get the timestamp from `git`.
pub fn set_SOURCE_TIME() {
    println!(
        "cargo:rustc-env=SOURCE_TIME={}",
        format_time(get_source_time().unwrap())
    );
}

/// Sets the `SOURCE_TIMESTAMP` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"2021-04-14T03:25:07Z"`
///
/// # Panics
/// Panics if `SOURCE_DATE_EPOCH` env var is set to a non-integer value.
/// Panics if it fails to get the timestamp from `git`.
pub fn set_SOURCE_TIMESTAMP() {
    println!(
        "cargo:rustc-env=SOURCE_TIMESTAMP={}",
        format_timestamp(get_source_time().unwrap())
    );
}

/// Sets the `SOURCE_EPOCH_TIME` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"1618370707"`
///
/// # Panics
/// Panics if `SOURCE_DATE_EPOCH` env var is set to a non-integer value.
/// Panics if it fails to get the timestamp from `git`.
pub fn set_SOURCE_EPOCH_TIME() {
    println!(
        "cargo:rustc-env=SOURCE_EPOCH_TIME={}",
        get_source_time().unwrap()
    );
}

/// Sets the `BUILD_DATE` env variable with the current date, in UTC.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Calling this will make your build
/// [non-reproducible](https://reproducible-builds.org/docs/timestamps/).
///
/// Example value: `"2021-04-14Z"`
pub fn set_BUILD_DATE() {
    println!("cargo:rustc-env=BUILD_DATE={}", format_date(now()));
}

/// Sets the `BUILD_TIME` env variable, with the current time, in UTC.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Calling this will make your build
/// [non-reproducible](https://reproducible-builds.org/docs/timestamps/).
///
/// Example value: `"03:25:07Z"`
pub fn set_BUILD_TIME() {
    println!("cargo:rustc-env=BUILD_TIME={}", format_time(now()));
}

/// Sets the `BUILD_TIMESTAMP` env variable, with the current date & time, in UTC.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Calling this will make your build
/// [non-reproducible](https://reproducible-builds.org/docs/timestamps/).
///
/// Example value: `"2021-04-14T03:25:07Z"`
pub fn set_BUILD_TIMESTAMP() {
    println!(
        "cargo:rustc-env=BUILD_TIMESTAMP={}",
        format_timestamp(now())
    );
}

/// Sets the `BUILD_EPOCH_TIME` env variable, with the current time.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Calling this will make your build
/// [non-reproducible](https://reproducible-builds.org/docs/timestamps/).
///
/// Example value: `"1618370707"`
pub fn set_BUILD_EPOCH_TIME() {
    println!("cargo:rustc-env=BUILD_EPOCH_TIME={}", now());
}

/// Sets the `BUILD_HOSTNAME` env variable, with the hostname of the machine executing the build.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Calling this will make your build
/// [non-reproducible](https://reproducible-builds.org/docs/timestamps/).
///
/// Example value: `"builder2"`
///
/// Executes the `hostname` command.
///
/// # Panics
/// Panics if it fails to get the timestamp from `hostname`.
pub fn set_BUILD_HOSTNAME() {
    println!("cargo:rustc-env=BUILD_HOSTNAME={}", get_hostname().unwrap());
}

/// Sets the `GIT_BRANCH` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Executes the `git` command.
///
/// Example value: `"release"`
///
/// # Panics
/// Panics if it fails to get the value from `git`.
pub fn set_GIT_BRANCH() {
    println!("cargo:rustc-env=GIT_BRANCH={}", get_git_branch().unwrap());
}

/// Sets the `GIT_COMMIT` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Executes the `git` command.
///
/// Example value: `"a5547bfb1edb9712588f0f85d3e2c8ba618ac51f"`
///
/// # Panics
/// Panics if it fails to get the value from `git`.
pub fn set_GIT_COMMIT() {
    println!("cargo:rustc-env=GIT_COMMIT={}", get_git_commit().unwrap());
}

/// Sets the `GIT_COMMIT_SHORT` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Executes the `git` command.
///
/// Example value: `"a5547bf"`
///
/// # Panics
/// Panics if it fails to get the value from `git`.
pub fn set_GIT_COMMIT_SHORT() {
    println!(
        "cargo:rustc-env=GIT_COMMIT_SHORT={}",
        get_git_commit_short().unwrap()
    );
}

/// Sets the `GIT_DIRTY` env variable.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Executes the `git` command.
///
/// Sets the variable to `"true"` if the git repository contains uncommitted
/// changes.  Otherwise, sets it to `"false"`.
///
/// # Panics
/// Panics if it fails to get the value from `git`.
pub fn set_GIT_DIRTY() {
    println!("cargo:rustc-env=GIT_DIRTY={}", get_git_dirty().unwrap());
}

/// Sets the `RUSTC_VERSION` env variable to the output of `rustc --version`.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)"`
///
/// Executes the `rustc` command.
///
/// # Panics
/// Panics if it fails to get the value from `rustc`.
pub fn set_RUSTC_VERSION() {
    println!(
        "cargo:rustc-env=RUSTC_VERSION={}",
        get_rustc_version().unwrap()
    );
}

/// Sets the `RUSTC_VERSION_SEMVER` to the dotted version number of the `rustc`
/// used by the current build.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Example value: `"1.53.0"`
///
/// Executes the `rustc` command.
///
/// # Panics
/// Panics if it fails to get the value from `rustc`.
pub fn set_RUSTC_VERSION_SEMVER() {
    println!(
        "cargo:rustc-env=RUSTC_VERSION_SEMVER={}",
        parse_rustc_semver(get_rustc_version().unwrap()).unwrap()
    );
}

/// Sets the `RUST_CHANNEL` env variable to Rust channel used by the current build.
///
/// Call this from `build.rs`.
/// Use `env!` in your `main.rs` to use the variable.
///
/// Possible values:
/// - `"stable"`
/// - `"beta"`
/// - `"nightly"`
///
/// Executes the `rustc` command.
///
/// # Panics
/// Panics if it fails to get the value from `rustc`.
pub fn set_RUST_CHANNEL() {
    println!(
        "cargo:rustc-env=RUST_CHANNEL={}",
        parse_rustc_channel(get_rustc_version().unwrap()).unwrap()
    );
}
