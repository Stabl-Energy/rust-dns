//! temp-file
//! =========
//! [![crates.io version](https://img.shields.io/crates/v/temp-file.svg)](https://crates.io/crates/temp-file)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/temp-file/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! Provides a `TempFile` struct.
//!
//! # Features
//! - Makes a file in a system temporary directory
//! - Deletes the file on drop
//! - Optional file name prefix
//! - Optional file contents
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! # Limitations
//! - Not security-hardened. See
//!   [Secure Programming for Linux and Unix HOWTO - 7.10. Avoid Race Conditions](https://tldp.org/HOWTO/Secure-Programs-HOWTO/avoid-race.html)
//!   and [`mkstemp`](https://linux.die.net/man/3/mkstemp).
//!
//! # Alternatives
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
//! # Related Crates
//! - [`temp-dir`](https://crates.io/crates/temp-dir)
//!
//! # Example
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
//! # Cargo Geiger Safety Report
//!
//! # Changelog
//! - v0.1.7 - Add `in_dir`, `with_suffix`, and `TempFileBuilder`.
//! - v0.1.6
//!   - Return `std::io::Error` instead of `String`.
//!   - Add
//!     [`cleanup`](https://docs.rs/temp-file/latest/temp_file/struct.TempFile.html#method.cleanup).
//! - v0.1.5 - Increase test coverage
//! - v0.1.4 - Add
//!   [`leak`](https://docs.rs/temp-file/latest/temp_file/struct.TempFile.html#method.leak)
//!   and
//!   [`panic_on_cleanup_error`](https://docs.rs/temp-file/latest/temp_file/struct.TempFile.html#method.panic_on_cleanup_error).
//! - v0.1.3 - Update docs
//! - v0.1.2 - Update example
//! - v0.1.1 - Minor code cleanup, update docs
//! - v0.1.0 - Initial version
#![forbid(unsafe_code)]
// TODO(mleonhard) Implement features requested of `tempfile` crate:
//   https://github.com/Stebalien/tempfile/issues
use core::sync::atomic::{AtomicU32, Ordering};
use std::path::{Path, PathBuf};

static COUNTER: AtomicU32 = AtomicU32::new(0);

pub struct TempFileBuilder {
    dir_path: Option<PathBuf>,
    prefix: Option<String>,
    suffix: Option<String>,
}
impl TempFileBuilder {
    #[allow(clippy::new_without_default)]
    #[must_use]
    pub fn new() -> Self {
        Self {
            dir_path: None,
            prefix: None,
            suffix: None,
        }
    }

    #[must_use]
    pub fn in_dir(mut self, p: impl AsRef<Path>) -> Self {
        self.dir_path = Some(p.as_ref().to_path_buf());
        self
    }

    #[must_use]
    pub fn prefix(mut self, s: impl AsRef<str>) -> Self {
        self.prefix = Some(s.as_ref().to_string());
        self
    }

    #[must_use]
    pub fn suffix(mut self, s: impl AsRef<str>) -> Self {
        self.suffix = Some(s.as_ref().to_string());
        self
    }

    /// Creates the temp file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the file.
    pub fn build(self) -> Result<TempFile, std::io::Error> {
        TempFile::internal_new(
            self.dir_path.as_deref(),
            self.prefix.as_ref().map(AsRef::as_ref),
            self.suffix.as_ref().map(AsRef::as_ref),
        )
    }
}

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
#[derive(Clone, PartialOrd, Ord, PartialEq, Eq, Hash, Debug)]
pub struct TempFile {
    path_buf: PathBuf,
    delete_on_drop: bool,
    panic_on_delete_err: bool,
}
impl TempFile {
    fn internal_new(
        dir: Option<&Path>,
        prefix: Option<&str>,
        suffix: Option<&str>,
    ) -> Result<Self, std::io::Error> {
        let dir = dir.map_or_else(std::env::temp_dir, Path::to_path_buf);
        let filename = format!(
            "{}{:x}{:x}{}",
            prefix.unwrap_or(""),
            std::process::id(),
            COUNTER.fetch_add(1, Ordering::AcqRel),
            suffix.unwrap_or(""),
        );
        let file_path = dir.join(filename);
        let mut open_opts = std::fs::OpenOptions::new();
        open_opts.create_new(true);
        open_opts.write(true);
        open_opts.open(&file_path).map_err(|e| {
            std::io::Error::new(
                e.kind(),
                format!("error creating file {:?}: {}", &file_path, e),
            )
        })?;
        Ok(Self {
            path_buf: file_path,
            delete_on_drop: true,
            panic_on_delete_err: false,
        })
    }

    fn remove_file(path: &Path) -> Result<(), std::io::Error> {
        match std::fs::remove_file(path) {
            Ok(()) => Ok(()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            Err(e) => Err(std::io::Error::new(
                e.kind(),
                format!("error removing file {:?}: {}", path, e),
            )),
        }
    }

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
    pub fn new() -> Result<Self, std::io::Error> {
        Self::internal_new(None, None, None)
    }

    /// Create a new empty file in `dir`.
    ///
    /// Drop the returned struct to delete the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the file.
    ///
    /// # Example
    /// ```
    /// // Prints "/tmp/temp_uploads/1a9b0".
    /// let dir = std::env::temp_dir().join("temp_uploads");
    /// # std::fs::create_dir(&dir).unwrap();
    /// println!("{:?}", temp_file::TempFile::in_dir(&dir).unwrap().path());
    /// # std::fs::remove_dir(&dir).unwrap();
    /// ```
    pub fn in_dir(dir: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        Self::internal_new(Some(dir.as_ref()), None, None)
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
    pub fn with_prefix(prefix: impl AsRef<str>) -> Result<Self, std::io::Error> {
        Self::internal_new(None, Some(prefix.as_ref()), None)
    }

    /// Create a new empty file in a system temporary directory.
    /// Use `suffix` as the last part of the file's name.
    ///
    /// You can use this to give the filename a particular extension.
    ///
    /// Drop the returned struct to delete the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to create the file.
    ///
    /// # Example
    /// ```rust
    /// // Prints "/tmp/1a9b0.txt".
    /// println!("{:?}", temp_file::TempFile::with_suffix(".txt").unwrap().path());
    /// ```
    pub fn with_suffix(suffix: impl AsRef<str>) -> Result<Self, std::io::Error> {
        Self::internal_new(None, None, Some(suffix.as_ref()))
    }

    /// Write `contents` to the file.
    ///
    /// # Errors
    /// Returns `Err` when it fails to write all of `contents` to the file.
    #[allow(clippy::missing_panics_doc)]
    pub fn with_contents(self, contents: &[u8]) -> Result<Self, std::io::Error> {
        let path = self.path_buf.as_path();
        std::fs::write(path, contents).map_err(|e| {
            std::io::Error::new(e.kind(), format!("error writing file {:?}: {}", path, e))
        })?;
        Ok(self)
    }

    /// Remove the file now.  Do nothing later on drop.
    ///
    /// # Errors
    /// Returns an error if the file exists and we fail to remove it.
    #[allow(clippy::missing_panics_doc)]
    pub fn cleanup(mut self) -> Result<(), std::io::Error> {
        let result = Self::remove_file(&self.path_buf);
        if result.is_ok() {
            self.delete_on_drop = false;
        }
        result
    }

    /// Make the struct panic on drop if it hits an error while
    /// removing the file.
    #[must_use]
    pub fn panic_on_cleanup_error(mut self) -> Self {
        self.panic_on_delete_err = true;
        self
    }

    /// Do not delete the file.
    ///
    /// This is useful when debugging a test.
    pub fn leak(mut self) {
        self.delete_on_drop = false;
    }

    /// The path to the file.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn path(&self) -> &Path {
        self.path_buf.as_path()
    }
}
impl Drop for TempFile {
    fn drop(&mut self) {
        if self.delete_on_drop {
            let result = Self::remove_file(self.path_buf.as_path());
            if self.panic_on_delete_err {
                if let Err(e) = result {
                    panic!("{}", e);
                }
            }
        }
    }
}

/// Create a new empty file in a system temporary directory.
///
/// # Panics
/// Panics if it cannot create the file.
#[must_use]
pub fn empty() -> TempFile {
    TempFile::new().unwrap()
}

/// Create a new  file in a system temporary directory
/// and writes `contents` to it.
///
/// # Panics
/// Panics if it fails to create the file or fails to write all of `contents`.
#[must_use]
pub fn with_contents(contents: &[u8]) -> TempFile {
    TempFile::new().unwrap().with_contents(contents).unwrap()
}

#[cfg(test)]
mod test;
