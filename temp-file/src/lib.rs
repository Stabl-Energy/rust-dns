//! [![crates.io version](https://img.shields.io/crates/v/temp-file.svg)](https://crates.io/crates/temp-file)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/temp-file/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # temp-file
//!
//! Provides a `TempFile` struct.
//!
//! ## Features
//! - Makes a file in a system temporary directory
//! - Deletes the file on drop
//! - Optional file name prefix
//! - Optional file contents
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//!
//! ## Limitations
//! - Not security-hardened. See
//!   [Secure Programming for Linux and Unix HOWTO - 7.10. Avoid Race Conditions](https://tldp.org/HOWTO/Secure-Programs-HOWTO/avoid-race.html)
//!   and [`mkstemp`](https://linux.die.net/man/3/mkstemp).
//!
//! ## Alternatives
//! - [`tempfile`](https://crates.io/crates/tempfile)
//!   - Popular and mature
//!   - Supports some security-sensitive use cases
//!   - Contains `unsafe`, dependencies full of `unsafe`
//!   - Heavy dependencies (libc, winapi, rand, etc.)
//! - [`test-temp-file`](https://crates.io/crates/test-temp-file)
//!   - Depends on crates which contain `unsafe`
//!   - Incomplete documentation
//! - [`temp_file_name`](https://crates.io/crates/temp_file_name)
//!   - Does not delete file
//!   - Usage is not straightforward.  Missing example.
//! - [`mktemp`](https://crates.io/crates/mktemp)
//!   - Sets file mode 0600 on unix
//!   - Contains `unsafe`
//!   - No readme or online docs
//!
//! ## Related Crates
//! - [`temp-dir`](https://crates.io/crates/temp-dir)
//!
//! ## Example
//! ```rust
//! let t = temp_file::with_contents(b"abc");
//! // Prints "/tmp/1a9b0".
//! println!("{:?}", t.path());
//! assert_eq!(
//!   "abc",
//!   std::fs::read_to_string(t.path()).unwrap(),
//! );
//! // Prints "/tmp/1a9b1".
//! println!("{:?}", temp_file::empty().path());
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.4 - Add
//!   [`leak`](https://docs.rs/temp-file/latest/temp_file/struct.TempFile.html#method.leak)
//!   and
//!   [`panic_on_cleanup_error`](https://docs.rs/temp-file/latest/temp_file/struct.TempFile.html#method.panic_on_cleanup_error).
//! - v0.1.3 - Update docs
//! - v0.1.2 - Update example
//! - v0.1.1 - Minor code cleanup, update docs
//! - v0.1.0 - Initial version
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]
// TODO(mleonhard) Implement features requested of `tempfile` crate:
//   https://github.com/Stebalien/tempfile/issues
use core::sync::atomic::{AtomicU32, Ordering};
use std::path::{Path, PathBuf};

static COUNTER: AtomicU32 = AtomicU32::new(0);

/// The path of an existing writable file in a system temporary directory.
///
/// Deletes the file on drop.  Ignores errors deleting the file.
///
/// # Example
/// ```rust
/// use temp_file::TempFile;
/// let t = TempFile::new()
///   .unwrap()
///   .with_contents(b"abc")
///   .unwrap();
/// // Prints "/tmp/1a9b0".
/// println!("{:?}", t.path());
/// assert_eq!(
///   "abc",
///   std::fs::read_to_string(t.path()).unwrap(),
/// );
/// // Prints "/tmp/1a9b1".
/// println!("{:?}", TempFile::new().unwrap().path());
/// ```
#[derive(Clone, PartialOrd, PartialEq, Debug)]
pub struct TempFile {
    path_buf: Option<PathBuf>,
    panic_on_delete_err: bool,
}
impl TempFile {
    /// Create a new empty file in a system temporary directory.
    ///
    /// Drop the returned struct to delete the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the file.
    ///
    /// # Example
    /// ```rust
    /// // Prints "/tmp/1a9b0".
    /// println!("{:?}", temp_file::TempFile::new().unwrap().path());
    /// ```
    pub fn new() -> Result<Self, String> {
        Self::with_prefix("")
    }

    /// Create a new empty file in a system temporary directory.
    /// Use `prefix` as the first part of the file's name.
    ///
    /// Drop the returned struct to delete the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the file.
    ///
    /// # Example
    /// ```rust
    /// // Prints "/tmp/ok1a9b0".
    /// println!("{:?}", temp_file::TempFile::with_prefix("ok").unwrap().path());
    /// ```
    pub fn with_prefix(prefix: impl AsRef<str>) -> Result<Self, String> {
        let mut open_opts = std::fs::OpenOptions::new();
        open_opts.create_new(true);
        open_opts.write(true);
        let path_buf = std::env::temp_dir().join(format!(
            "{}{:x}{:x}",
            prefix.as_ref(),
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::AcqRel),
        ));
        open_opts
            .open(&path_buf)
            .map_err(|e| format!("error creating file {:?}: {}", &path_buf, e))?;
        Ok(Self {
            path_buf: Some(path_buf),
            panic_on_delete_err: false,
        })
    }

    /// Write `contents` to the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to write all of `contents` to the file.
    pub fn with_contents(self, contents: &[u8]) -> Result<Self, String> {
        let path = self.path_buf.as_ref().unwrap();
        std::fs::write(path, contents)
            .map_err(|e| format!("error writing file {:?}: {}", path, e))?;
        Ok(self)
    }

    /// Make the struct panic on Drop if it hits an error while
    /// removing the file.
    #[must_use]
    pub fn panic_on_cleanup_error(mut self) -> Self {
        Self {
            path_buf: self.path_buf.take(),
            panic_on_delete_err: true,
        }
    }

    /// Do not delete the file.
    ///
    /// This is useful when debugging a test.
    pub fn leak(mut self) -> () {
        self.path_buf.take();
    }

    /// The path to the file.
    #[must_use]
    pub fn path(&self) -> &Path {
        self.path_buf.as_ref().unwrap()
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        if let Some(path) = &self.path_buf {
            let result = std::fs::remove_file(path);
            if self.panic_on_delete_err {
                if let Err(e) = result {
                    panic!("error removing file {:?}: {}", path, e);
                }
            }
        }
    }
}

/// Create a new empty file in a system temporary directory.
///
/// Panics if it cannot create the file.
#[must_use]
pub fn empty() -> TempFile {
    TempFile::new().unwrap()
}

/// Create a new  file in a system temporary directory
/// and writes `contents` to it.
///
/// Panics if it fails to create the file or fails to write all of `contents`.
#[must_use]
pub fn with_contents(contents: &[u8]) -> TempFile {
    TempFile::new().unwrap().with_contents(contents).unwrap()
}

#[cfg(test)]
mod test {
    use crate::{TempFile, COUNTER};
    use core::sync::atomic::Ordering;
    use safe_lock::SafeLock;
    use std::io::ErrorKind;
    use std::path::Path;

    // The error tests require all tests to run single-threaded.
    static LOCK: SafeLock = SafeLock::new();

    fn expect_not_found(path: impl AsRef<Path>) {
        match std::fs::metadata(&path) {
            Ok(_) => panic!("exists: {}", path.as_ref().to_string_lossy()),
            Err(e) if e.kind() == ErrorKind::NotFound => {}
            Err(e) => panic!(
                "error getting metadata of {}: {}",
                path.as_ref().to_string_lossy(),
                e
            ),
        }
    }

    #[test]
    fn empty() {
        let _guard = LOCK.lock();
        let temp_file = crate::empty();
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.is_file());
        assert_eq!(0, metadata.len());
        std::fs::write(temp_file.path(), b"abc").unwrap();
        assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
        let temp_file2 = crate::empty();
        assert_ne!(temp_file.path(), temp_file2.path());
    }

    #[test]
    fn empty_error() {
        let _guard = LOCK.lock();
        let previous_counter_value = COUNTER.load(Ordering::SeqCst);
        let temp_file = crate::empty();
        COUNTER.store(previous_counter_value, Ordering::SeqCst);
        let e = if let Err(e) = std::panic::catch_unwind(|| crate::empty()) {
            e
        } else {
            panic!("expected panic");
        };
        assert_eq!(
            &format!(
                "called `Result::unwrap()` on an `Err` value: {:?}",
                format!(
                    "error creating file {:?}: File exists (os error 17)",
                    temp_file.path()
                )
            ),
            e.downcast_ref::<String>().unwrap()
        );
    }

    #[test]
    fn with_contents() {
        let _guard = LOCK.lock();
        let temp_file = crate::with_contents(b"abc");
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.is_file());
        assert_eq!(3, metadata.len());
        assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
        std::fs::write(temp_file.path(), b"def").unwrap();
        assert_eq!("def", std::fs::read_to_string(temp_file.path()).unwrap());
    }

    #[test]
    fn temp_file_new() {
        let _guard = LOCK.lock();
        let temp_file = TempFile::new().unwrap();
        println!("{:?}", temp_file.path());
        println!("{:?}", TempFile::new().unwrap().path());
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.is_file());
        assert_eq!(0, metadata.len());
        std::fs::write(temp_file.path(), b"abc").unwrap();
        assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
        let temp_file2 = TempFile::new().unwrap();
        assert_ne!(temp_file.path(), temp_file2.path());
    }

    #[test]
    fn temp_file_new_error() {
        let _guard = LOCK.lock();
        let previous_counter_value = COUNTER.load(Ordering::SeqCst);
        let temp_file = TempFile::new().unwrap();
        COUNTER.store(previous_counter_value, Ordering::SeqCst);
        assert_eq!(
            Err(format!(
                "error creating file {:?}: File exists (os error 17)",
                temp_file.path()
            )),
            TempFile::new()
        );
    }

    #[test]
    fn temp_file_with_prefix() {
        let _guard = LOCK.lock();
        let temp_file = TempFile::with_prefix("prefix1").unwrap();
        let name = temp_file.path().file_name().unwrap();
        assert!(
            name.to_str().unwrap().starts_with("prefix1"),
            "{:?}",
            temp_file
        );
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.is_file());
        assert_eq!(0, metadata.len());
        std::fs::write(temp_file.path(), b"abc").unwrap();
        assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
        let temp_file2 = TempFile::new().unwrap();
        assert_ne!(temp_file.path(), temp_file2.path());
    }

    #[test]
    fn temp_file_with_prefix_error() {
        let _guard = LOCK.lock();
        let previous_counter_value = COUNTER.load(Ordering::SeqCst);
        let temp_file = TempFile::with_prefix("prefix1").unwrap();
        COUNTER.store(previous_counter_value, Ordering::SeqCst);
        assert_eq!(
            Err(format!(
                "error creating file {:?}: File exists (os error 17)",
                temp_file.path()
            )),
            TempFile::with_prefix("prefix1")
        );
    }

    #[test]
    fn temp_file_with_contents() {
        let _guard = LOCK.lock();
        let temp_file = TempFile::new().unwrap().with_contents(b"abc").unwrap();
        let metadata = std::fs::metadata(temp_file.path()).unwrap();
        assert!(metadata.is_file());
        assert_eq!(3, metadata.len());
        assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
        std::fs::write(temp_file.path(), b"def").unwrap();
        assert_eq!("def", std::fs::read_to_string(temp_file.path()).unwrap());
    }

    #[test]
    fn temp_file_with_contents_error() {
        let _guard = LOCK.lock();
        let temp_file = TempFile::new().unwrap();
        std::fs::remove_file(temp_file.path()).unwrap();
        let temp_file_path = temp_file.path().to_path_buf();
        std::fs::create_dir(&temp_file_path).unwrap();
        assert_eq!(
            Err(format!(
                "error writing file {:?}: Is a directory (os error 21)",
                temp_file.path()
            )),
            temp_file.with_contents(b"abc")
        );
        std::fs::remove_dir(&temp_file_path).unwrap();
    }

    #[test]
    fn test_drop() {
        let _guard = LOCK.lock();
        let path_copy;
        {
            let temp_file = TempFile::new().unwrap();
            path_copy = temp_file.path().to_path_buf();
            TempFile::new().unwrap();
        }
        expect_not_found(&path_copy);
    }

    #[test]
    fn drop_already_deleted() {
        let _guard = LOCK.lock();
        let temp_file = TempFile::new().unwrap();
        std::fs::remove_file(temp_file.path()).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn drop_error_ignored() {
        // On Gitlab's shared CI runners, the cleanup always succeeds and the
        // test fails.  So we skip this test when it's running on Gitlab CI.
        if std::env::current_dir().unwrap().starts_with("/builds/") {
            println!("Running on Gitlab CI.  Skipping test.");
            return;
        }
        let _guard = LOCK.lock();
        let f = crate::empty();
        let path = f.path().to_path_buf();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        drop(f);
        std::fs::metadata(&path).unwrap();
        std::fs::remove_dir(&path).unwrap();
    }

    #[cfg(unix)]
    #[test]
    fn drop_error_panic() {
        // On Gitlab's shared CI runners, the cleanup always succeeds and the
        // test fails.  So we skip this test when it's running on Gitlab CI.
        if std::env::current_dir().unwrap().starts_with("/builds/") {
            println!("Running on Gitlab CI.  Skipping test.");
            return;
        }
        let _guard = LOCK.lock();
        let f = crate::empty().panic_on_cleanup_error();
        let path = f.path().to_path_buf();
        std::fs::remove_file(&path).unwrap();
        std::fs::create_dir(&path).unwrap();
        let result = std::panic::catch_unwind(move || drop(f));
        std::fs::metadata(&path).unwrap();
        std::fs::remove_dir(&path).unwrap();
        match result {
            Ok(_) => panic!("expected panic"),
            Err(any) => {
                let e = any.downcast::<String>().unwrap();
                assert!(
                    e.starts_with(&format!("error removing file {:?}: ", path)),
                    "unexpected error {:?}",
                    e
                );
            }
        }
    }

    #[test]
    fn leak() {
        let _guard = LOCK.lock();
        let f = crate::empty();
        let path = f.path().to_path_buf();
        f.leak();
        std::fs::metadata(&path).unwrap();
        std::fs::remove_file(&path).unwrap();
    }
}
