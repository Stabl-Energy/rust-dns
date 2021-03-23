//! [![crates.io version](https://img.shields.io/crates/v/temp-dir.svg)](https://crates.io/crates/temp-dir)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/temp-dir/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # temp-dir
//!
//! Provides a `TempDir` struct.
//!
//! ## Features
//! - Makes a directory in a system temporary directory
//! - Recursively deletes the directory and its contents on drop
//! - Deletes symbolic links and does not follow them.
//! - Optional name prefix
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Limitations
//! - Not security-hardened.
//!   For example, directory and file names are predictable.
//! - This crate uses
//!   [`std::fs::remove_dir_all`](https://doc.rust-lang.org/stable/std/fs/fn.remove_dir_all.html)
//!   which may be unreliable on Windows.
//!   See [rust#29497](https://github.com/rust-lang/rust/issues/29497) and
//!   [`remove_dir_all`](https://crates.io/crates/remove_dir_all) crate.
//!
//! ## Alternatives
//! - [`tempfile`](https://crates.io/crates/tempfile)
//!   - Popular and mature
//!   - Contains `unsafe`, dependencies full of `unsafe`
//!   - Heavy dependencies (libc, winapi, rand, etc.)
//! - [`test_dir`](https://crates.io/crates/test_dir)
//!   - Has a handy `TestDir` struct
//!   - Incomplete documentation
//! - [`temp_testdir`](https://crates.io/crates/temp_testdir)
//!   - Incomplete documentation
//! - [`mktemp`](https://crates.io/crates/mktemp)
//!   - Sets directory mode 0700 on unix
//!   - Contains `unsafe`
//!   - No readme or online docs
//!
//! ## Related Crates
//! - [`temp-file`](https://crates.io/crates/temp-file)
//!
//! ## Example
//! ```rust
//! use temp_dir::TempDir;
//! let d = TempDir::new().unwrap();
//! // Prints "/tmp/t1a9b0".
//! println!("{:?}", d.path());
//! let f = d.child("file1");
//! // Prints "/tmp/t1a9b0/file1".
//! println!("{:?}", f);
//! std::fs::write(&f, b"abc").unwrap();
//! assert_eq!(
//!     "abc",
//!     std::fs::read_to_string(&f).unwrap(),
//! );
//! // Prints "/tmp/t1a9b1".
//! println!(
//!     "{:?}", TempDir::new().unwrap().path());
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.9 - Increase test coverage
//! - v0.1.8 - Add [`leak`](https://docs.rs/temp-dir/latest/temp_dir/struct.TempDir.html#method.leak).
//! - v0.1.7 - Update docs:
//!   Warn about `std::fs::remove_dir_all` being unreliable on Windows.
//!   Warn about predictable directory and file names.
//!   Thanks to Reddit user
//!   [burntsushi](https://www.reddit.com/r/rust/comments/ma6y0x/tempdir_simple_temporary_directory_with_cleanup/gruo5iu/).
//! - v0.1.6 - Add
//!     [`TempDir::panic_on_cleanup_error`](https://docs.rs/temp-dir/latest/temp_dir/struct.TempDir.html#method.panic_on_cleanup_error).
//!     Thanks to Reddit users
//!     [`KhorneLordOfChaos`](https://www.reddit.com/r/rust/comments/ma6y0x/tempdir_simple_temporary_directory_with_cleanup/grsb5s3/)
//!     and
//!     [`dpc_pw`](https://www.reddit.com/r/rust/comments/ma6y0x/tempdir_simple_temporary_directory_with_cleanup/gru26df/)
//!     for their comments.
//! - v0.1.5 - Explain how it handles symbolic links.
//!   Thanks to Reddit user Mai4eeze for this
//!   [idea](https://www.reddit.com/r/rust/comments/ma6y0x/tempdir_simple_temporary_directory_with_cleanup/grsoz2g/).
//! - v0.1.4 - Update docs
//! - v0.1.3 - Minor code cleanup, update docs
//! - v0.1.2 - Update docs
//! - v0.1.1 - Fix license
//! - v0.1.0 - Initial version
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]
use core::sync::atomic::{AtomicU32, Ordering};
use std::path::{Path, PathBuf};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// The path of an existing writable directory in a system temporary directory.
///
/// Drop the struct to delete the directory and everything under it.
/// Deletes symbolic links and does not follow them.
///
/// Ignores any error while deleting.
/// See [`TempDir::panic_on_cleanup_error`](struct.TempDir.html#method.panic_on_cleanup_error).
///
/// # Example
/// ```rust
/// use temp_dir::TempDir;
/// let d = TempDir::new().unwrap();
/// // Prints "/tmp/t1a9b0".
/// println!("{:?}", d.path());
/// let f = d.child("file1");
/// // Prints "/tmp/t1a9b0/file1".
/// println!("{:?}", f);
/// std::fs::write(&f, b"abc").unwrap();
/// assert_eq!(
///     "abc",
///     std::fs::read_to_string(&f).unwrap(),
/// );
/// // Prints "/tmp/t1a9b1".
/// println!("{:?}", TempDir::new().unwrap().path());
/// ```
#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub struct TempDir {
    path_buf: Option<PathBuf>,
    panic_on_delete_err: bool,
}
impl TempDir {
    /// Create a new empty directory in a system temporary directory.
    ///
    /// Drop the struct to delete the directory and everything under it.
    /// Deletes symbolic links and does not follow them.
    ///
    /// Ignores any error while deleting.
    /// See [`TempDir::panic_on_cleanup_error`](struct.TempDir.html#method.panic_on_cleanup_error).
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the directory.
    ///
    /// # Example
    /// ```rust
    /// // Prints "/tmp/t1a9b0".
    /// println!("{:?}", temp_dir::TempDir::new().unwrap().path());
    /// ```
    pub fn new() -> Result<Self, String> {
        // Prefix with 't' to avoid name collisions with `temp-file` crate.
        Self::with_prefix("t")
    }

    /// Create a new empty directory in a system temporary directory.
    /// Use `prefix` as the first part of the directory's name.
    ///
    /// Drop the struct to delete the directory and everything under it.
    /// Deletes symbolic links and does not follow them.
    ///
    /// Ignores any error while deleting.
    /// See [`TempDir::panic_on_cleanup_error`](struct.TempDir.html#method.panic_on_cleanup_error).
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the directory.
    ///
    /// # Example
    /// ```rust
    /// // Prints "/tmp/ok1a9b0".
    /// println!("{:?}", temp_dir::TempDir::with_prefix("ok").unwrap().path());
    /// ```
    pub fn with_prefix(prefix: impl AsRef<str>) -> Result<Self, String> {
        let path_buf = std::env::temp_dir().join(format!(
            "{}{:x}-{:x}",
            prefix.as_ref(),
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::AcqRel),
        ));
        std::fs::create_dir(&path_buf)
            .map_err(|e| format!("error creating directory {:?}: {}", &path_buf, e))?;
        Ok(Self {
            path_buf: Some(path_buf),
            panic_on_delete_err: false,
        })
    }

    /// Make the struct panic on Drop if it hits an error while
    /// removing the directory or its contents.
    #[must_use]
    pub fn panic_on_cleanup_error(mut self) -> Self {
        Self {
            path_buf: self.path_buf.take(),
            panic_on_delete_err: true,
        }
    }

    /// Do not delete the directory or its contents.
    ///
    /// This is useful when debugging a test.
    pub fn leak(mut self) {
        self.path_buf.take();
    }

    /// The path to the directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path_buf.as_ref().unwrap()
    }

    /// The path to `name` under the directory.
    #[must_use]
    pub fn child(&self, name: impl AsRef<str>) -> PathBuf {
        let mut result = self.path_buf.as_ref().unwrap().clone();
        result.push(name.as_ref());
        result
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        if let Some(path) = &self.path_buf {
            let result = std::fs::remove_dir_all(path);
            if self.panic_on_delete_err {
                if let Err(e) = result {
                    panic!("error removing directory and contents {:?}: {}", path, e);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{TempDir, COUNTER};
    use core::sync::atomic::Ordering;
    use safe_lock::SafeLock;
    use std::io::ErrorKind;
    use std::path::Path;

    // These tests must run single-threaded.
    static LOCK: SafeLock = SafeLock::new();

    #[test]
    fn new_error() {
        let _guard = LOCK.lock();
        let previous_counter_value = COUNTER.load(Ordering::SeqCst);
        let temp_dir = TempDir::new().unwrap();
        COUNTER.store(previous_counter_value, Ordering::SeqCst);
        assert_eq!(
            Err(format!(
                "error creating directory {:?}: File exists (os error 17)",
                temp_dir.path()
            )),
            TempDir::new()
        );
    }

    #[test]
    fn with_prefix_error() {
        let _guard = LOCK.lock();
        let previous_counter_value = COUNTER.load(Ordering::SeqCst);
        let temp_dir = TempDir::with_prefix("prefix1").unwrap();
        COUNTER.store(previous_counter_value, Ordering::SeqCst);
        assert_eq!(
            Err(format!(
                "error creating directory {:?}: File exists (os error 17)",
                temp_dir.path()
            )),
            TempDir::with_prefix("prefix1")
        );
    }
}
