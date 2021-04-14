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
//!   - Date & time
//!   - Epoch time
//!   - Hostname
//!   - Rustc version
//! - Does all of its work in your
//!   [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html).
//! - No macros
//! - Depends only on `core::alloc` at runtime.
//! - Light build dependencies
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Alternatives
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
//! build-data = "0"
//!
//! [build-dependencies]
//! build-data-writer = "0"
//! ```
//!
//! Add a [`build.rs`](https://doc.rust-lang.org/cargo/reference/build-scripts.html)
//! file next to your `Cargo.toml`.
//! Call [`build_data_writer::write`](https://docs.rs/build-data-writer/latest/build_data_writer/fn.write.html)
//! to collect data and write it to the file.
//! ```
//! // build.rs
//! use std::env;
//! use std::path::Path;
//!
//! fn main() {
//! # }
//! # fn f() {
//!     build_data_writer::write(
//!         &Path::new(&env::var_os("OUT_DIR").unwrap())
//!         .join("build-data.txt")
//!     ).unwrap();
//!     build_data_writer::no_debug_rebuilds();
//! }
//! ```
//!
//! When you run `cargo build`, Cargo compiles and runs your `build.rs` which
//! writes the file:
//! ```text
//! // target/build-data.txt
//! GIT_BRANCH:release
//! GIT_COMMIT:a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
//! GIT_DIRTY:false
//! HOSTNAME:builder2
//! RUSTC_VERSION:rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
//! TIME:2021-04-14T06:25:59+00:00
//! TIME_SECONDS:1618381559
//! ```
//!
//! Include and parse the file in your program.
//! See [`include_str!`](https://doc.rust-lang.org/core/macro.include_str.html),
//! [`concat!`](https://doc.rust-lang.org/core/macro.concat.html),
//! [`env!`](https://doc.rust-lang.org/core/macro.env.html), and
//! [`build_data::BuildData::new`](https://docs.rs/build-data/latest/build_data/struct.BuildData.html#method.new).
//! ```
//! // src/bin/main.rs
//! # macro_rules! include_str (
//! #     ($arg:expr) => { "GIT_BRANCH:1\nGIT_COMMIT:abcdef123\nGIT_DIRTY:false\nHOSTNAME:h\nRUSTC_VERSION:2\nTIME:123\nTIME_SECONDS:123" };
//! # );
//! # macro_rules! concat (
//! #     ($arg:expr) => { "" };
//! # );
//! # macro_rules! env (
//! #     ($arg:expr) => { "" };
//! # );
//! # macro_rules! log (
//! #     ($fmt:expr, $arg:expr) => { ; };
//! # );
//! fn main() {
//!     let bd = build_data::BuildData::new(include_str!(
//!         concat!(env!("OUT_DIR"), "/build-data.txt")
//!     )).unwrap();
//!     // Built 2021-04-14T06:25:59+00:00 branch=release
//!     // commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
//!     // host=builder2
//!     // rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
//!     log!("{}", bd);
//! }
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.1 - Update docs.
//! - v0.1.0 - Initial version
//!
//! ## To Do
//! - Accept only `&'static str` and remove dependencyon `alloc`.
//! - See if we can make `new` into a `const fn`.
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

/// Build data parser and holder.  Example in [module docs](index.html).
pub struct BuildData {
    pub git_branch: Option<String>,
    pub git_commit: Option<String>,
    pub git_dirty: Option<bool>,
    pub hostname: Option<String>,
    pub rustc_version: Option<String>,
    pub time: String,
    pub time_seconds: u64,
}
impl BuildData {
    /// Parses `build-data.txt` written by
    /// [`build_data_writer::write`](https://docs.rs/build-data-writer/latest/build_data_writer/fn.write.html).
    ///
    /// See [module docs](index.html) for example and expected file format.
    ///
    /// Ignores malformed fields.
    ///
    /// Treats empty fields as missing.
    ///
    /// Trims whitespace from field names and values.
    ///
    /// If a field appears multiple times, uses the last value.
    ///
    /// # Errors
    /// Returns an error when:
    /// - `contents` is empty
    /// - `contents` is missing `TIME` or `TIME_SECONDS` fields
    /// - `contents` has a `TIME_SECONDS` field that is not all decimal digits
    ///    convertible to a u64
    #[allow(clippy::missing_panics_doc)]
    pub fn new(contents: impl AsRef<str>) -> Result<Self, &'static str> {
        if contents.as_ref().is_empty() {
            return Err("build data string is empty");
        }
        let mut git_branch = None;
        let mut git_commit = None;
        let mut git_dirty = None;
        let mut hostname = None;
        let mut rustc_version = None;
        let mut time = None;
        let mut time_seconds = None;
        for line in contents.as_ref().split(|c| c == '\r' || c == '\n') {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let mut parts = line.splitn(2, ':');
            let key = parts.next().unwrap().trim();
            let value = parts.next().map(str::trim).filter(|s| !s.is_empty());
            match (key, value) {
                ("", _) | (_, Some("")) => eprintln!(
                    "WARNING build data string contains malformed line: '{}'",
                    line
                ),
                (_, None) => continue,
                ("GIT_BRANCH", Some(value)) => {
                    git_branch = Some(String::from(value));
                }
                ("GIT_COMMIT", Some(value)) => {
                    git_commit = Some(String::from(value));
                }
                ("GIT_DIRTY", Some("true")) => {
                    git_dirty = Some(true);
                }
                ("GIT_DIRTY", Some("false")) => {
                    git_dirty = Some(false);
                }
                ("GIT_DIRTY", Some(other)) => {
                    eprintln!(
                        "WARNING build data string contains malformed GIT_DIRTY value: '{}'",
                        other
                    )
                }
                ("HOSTNAME", Some(value)) => {
                    hostname = Some(String::from(value));
                }
                ("RUSTC_VERSION", Some(value)) => {
                    rustc_version = Some(String::from(value));
                }
                ("TIME", Some(value)) => {
                    time = Some(String::from(value));
                }
                ("TIME_SECONDS", Some(value)) => {
                    time_seconds = Some(value.parse().map_err(|_| {
                        "error parsing build data TIME_SECONDS value as an integer"
                    })?);
                }
                _ => eprintln!(
                    "WARNING build data string contains unknown field: '{}'",
                    line
                ),
            }
        }
        Ok(Self {
            git_branch,
            git_commit,
            git_dirty,
            hostname,
            rustc_version,
            time: time.ok_or("build data string has no TIME line")?,
            time_seconds: time_seconds.ok_or("build data string has no TIME_SECONDS line")?,
        })
    }
}
impl core::fmt::Debug for BuildData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        // BuildData{2021-04-14T06:25:59+00:00 1618381559 branch=release
        // commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f dirty=false
        // host=builder2 rustc=rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)}
        write!(
            f,
            "BuildData{{{} {} branch={} commit={} dirty={} hostname={} rustc={}}}",
            self.time,
            self.time_seconds,
            self.git_branch.as_ref().unwrap_or(&String::new()),
            self.git_commit.as_ref().unwrap_or(&String::new()),
            self.git_dirty
                .map_or("", |b| if b { "true" } else { "false" }),
            self.hostname.as_ref().unwrap_or(&String::new()),
            self.rustc_version.as_ref().unwrap_or(&String::new()),
        )
    }
}
impl core::fmt::Display for BuildData {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        // Built 2021-04-13T08:17:32+00:00 branch=release commit=a5547bfb1edb9712588f0f85d3e2c8ba618ac51f
        // host=builder2 rustc 1.53.0-nightly (07e0e2ec2 2021-03-24)
        write!(f, "Built {}", self.time)?;
        if let Some(ref value) = self.git_branch {
            write!(f, " {}", value)?;
        }
        if let Some(ref value) = self.git_commit {
            write!(f, " {}", value)?;
        }
        if let Some(true) = self.git_dirty {
            write!(f, " GIT-DIRTY")?;
        }
        if let Some(ref value) = self.hostname {
            write!(f, " {}", value)?;
        }
        if let Some(ref value) = self.rustc_version {
            write!(f, " {}", value)?;
        }
        Ok(())
    }
}
