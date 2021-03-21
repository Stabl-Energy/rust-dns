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
//! - Optional name prefix
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//!
//! ## Limitations
//! - Not security-hardened.
//!
//! ## Alternatives
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
/// Recursively deletes the directory and its contents on drop.
/// Ignores any error while deleting.
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
    path_buf: PathBuf,
}
impl TempDir {
    /// Create a new empty directory in a system temporary directory.
    ///
    /// Drop the returned struct to delete the directory and everything under it.
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
    /// Drop the returned struct to delete the directory and everything under it.
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
        Ok(Self { path_buf })
    }

    /// The path to the directory.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path_buf
    }

    /// The path to `name` under the directory.
    #[must_use]
    pub fn child(&self, name: impl AsRef<str>) -> PathBuf {
        let mut result = self.path_buf.clone();
        result.push(name.as_ref());
        result
    }
}
impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path_buf);
    }
}

#[cfg(test)]
mod test {
    use crate::{TempDir, COUNTER};
    use core::sync::atomic::Ordering;
    use safe_lock::SafeLock;
    use std::io::ErrorKind;
    use std::path::Path;

    // The error tests require all tests to run single-threaded.
    static LOCK: SafeLock = SafeLock::new();

    fn expect_not_found(path: impl AsRef<Path>) {
        match std::fs::metadata(&path) {
            Ok(_) => panic!("exists {:?}", path.as_ref()),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => panic!("error getting metadata of {:?}: {}", path.as_ref(), e),
        }
    }

    #[test]
    fn test_new() {
        let _guard = LOCK.lock();
        let temp_dir = TempDir::new().unwrap();
        println!("{:?}", temp_dir);
        println!("{:?}", TempDir::new().unwrap());
        let metadata = std::fs::metadata(temp_dir.path()).unwrap();
        assert!(metadata.is_dir());
        let temp_dir2 = TempDir::new().unwrap();
        assert_ne!(temp_dir.path(), temp_dir2.path());
    }

    #[test]
    fn test_new_error() {
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
    fn test_with_prefix() {
        let _guard = LOCK.lock();
        let temp_dir = TempDir::with_prefix("prefix1").unwrap();
        let name = temp_dir.path().file_name().unwrap();
        assert!(
            name.to_str().unwrap().starts_with("prefix1"),
            "{:?}",
            temp_dir
        );
        let metadata = std::fs::metadata(temp_dir.path()).unwrap();
        assert!(metadata.is_dir());
        let temp_dir2 = TempDir::new().unwrap();
        assert_ne!(temp_dir.path(), temp_dir2.path());
    }

    #[test]
    fn test_with_prefix_error() {
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

    #[test]
    fn test_child() {
        let _guard = LOCK.lock();
        let temp_dir = TempDir::new().unwrap();
        let file1_path = temp_dir.child("file1");
        assert!(
            file1_path.ends_with("file1"),
            "{:?}",
            file1_path.to_string_lossy()
        );
        assert!(
            file1_path.starts_with(temp_dir.path()),
            "{:?}",
            file1_path.to_string_lossy()
        );
        std::fs::write(&file1_path, b"abc").unwrap();
    }

    #[test]
    fn test_drop() {
        let _guard = LOCK.lock();
        let dir_path;
        let file1_path;
        {
            let temp_dir = TempDir::new().unwrap();
            dir_path = temp_dir.path().to_path_buf();
            file1_path = temp_dir.child("file1");
            std::fs::write(&file1_path, b"abc").unwrap();
            TempDir::new().unwrap();
        }
        expect_not_found(&dir_path);
        expect_not_found(&file1_path);
    }

    #[test]
    fn test_drop_already_deleted() {
        let _guard = LOCK.lock();
        let temp_dir = TempDir::new().unwrap();
        std::fs::remove_dir(temp_dir.path()).unwrap();
    }
}
