//! [![crates.io version](https://img.shields.io/crates/v/permit.svg)](https://crates.io/crates/permit)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/permit/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # permit
//!
//! [`permit::Permit`](https://docs.rs/permit/latest/permit/struct.Permit.html)
//! is a struct for cancelling operations.
//!
//! ## Use Cases
//! - Graceful server shutdown
//! - Cancel operations that take too long
//! - Stop in-flight operations when revoking authorization
//!
//! ## Features
//! - Subordinate permits.
//!   Revoking a permit also revokes its subordinates, recursively.
//! - Drop a permit to revoke its subordinates, recursively.
//! - Wait for all subordinate permits to drop.
//! - Similar to Golang's [`context`](https://golang.org/pkg/context/)
//! - Depends only on `std`.
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Limitations
//! - Does not hold data values
//! - Allocates.  Uses [`alloc::sync::Arc`](https://doc.rust-lang.org/alloc/sync/struct.Arc.html).
//!
//! ## Alternatives
//! - [`async_ctx`](https://crates.io/crates/async_ctx)
//!   - Good API
//!   - Async only
//! - [`io-context`](https://crates.io/crates/io-context)
//!   - Holds [Any](https://doc.rust-lang.org/core/any/trait.Any.html) values
//!   - Unmaintained
//! - [`ctx`](https://crates.io/crates/ctx)
//!   - Holds an [Any](https://doc.rust-lang.org/core/any/trait.Any.html) value
//!   - API is a direct copy of Golang's
//!     [`context`](https://golang.org/pkg/context/),
//!     even where that doesn't make sense for Rust.
//!     For example, to cancel, one must copy the context and call
//!     a returned `Box<Fn>`.
//!   - Unmaintained
//!
//! ## Related Crates
//!
//! ## Example
//!
//! Graceful shutdown:
//! ```
//! # fn wait_for_shutdown_signal() { () }
//! let top_permit = permit::Permit::new();
//! // Start some worker threads.
//! for _ in 0..5 {
//!     let permit = top_permit.new_sub();
//!     std::thread::spawn(move || {
//!         while !permit.is_revoked() {
//!             // ...
//! #           std::thread::sleep(core::time::Duration::from_millis(1));
//!         }
//!     });
//! }
//! wait_for_shutdown_signal();
//! // Revoke all thread permits.
//! top_permit.revoke();
//! // Give the threads time to finish
//! // and drop their permits.
//! let _ = top_permit.try_wait_for(
//!     core::time::Duration::from_secs(3));
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.0 - Initial version
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::{Duration, Instant};

struct Inner {
    superior: Option<Arc<Inner>>,
    revoked: AtomicBool,
}
impl Inner {
    #[must_use]
    pub fn new() -> Arc<Self> {
        Arc::new(Inner {
            superior: None,
            revoked: AtomicBool::new(false),
        })
    }

    #[must_use]
    pub fn new_sub(self: &Arc<Self>) -> Arc<Self> {
        Arc::new(Self {
            superior: Some(self.clone()),
            revoked: AtomicBool::new(self.is_revoked()),
        })
    }

    #[must_use]
    pub fn is_revoked(&self) -> bool {
        if self.revoked.load(std::sync::atomic::Ordering::Relaxed) {
            return true;
        }
        #[allow(clippy::option_if_let_else)]
        if let Some(inner) = &self.superior {
            inner.is_revoked()
        } else {
            false
        }
    }

    pub fn revoke(&self) {
        self.revoked
            .store(true, std::sync::atomic::Ordering::Relaxed)
    }
}
impl Clone for Inner {
    fn clone(&self) -> Self {
        Inner {
            superior: self.superior.clone(),
            revoked: AtomicBool::new(self.is_revoked()),
        }
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeadlineExceeded;
impl core::fmt::Display for DeadlineExceeded {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(f, "DeadlineExceeded")
    }
}
impl std::error::Error for DeadlineExceeded {}

/// A struct for cancelling operations.
///
/// Use [`new_sub()`](#method.new_sub) to make a subordinate permit.
/// Call [`revoke()`](#method.revoke) to revoke a permit
/// and its subordinate permits, recursively.
///
/// # Example
///
/// Graceful shutdown:
/// ```
/// # fn wait_for_shutdown_signal() { () }
/// let top_permit = permit::Permit::new();
/// // Start some worker threads.
/// for _ in 0..5 {
///     let permit = top_permit.new_sub();
///     std::thread::spawn(move || {
///         while !permit.is_revoked() {
///             // ...
/// #           std::thread::sleep(core::time::Duration::from_millis(1));
///         }
///     });
/// }
/// wait_for_shutdown_signal();
/// // Revoke all thread permits.
/// top_permit.revoke();
/// // Give the threads time to finish
/// // and drop their permits.
/// let _ = top_permit.try_wait_for(
///     core::time::Duration::from_secs(3));
/// ```
pub struct Permit {
    inner: Arc<Inner>,
}
impl Permit {
    /// Makes a new permit.
    ///
    /// This permit is not subordinate to any other permit.
    /// It has no superior.
    ///
    /// Dropping the permit revokes it and any subordinate permits.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Inner::new(),
        }
    }

    /// Make a new permit that is subordinate to this permit.
    ///
    /// Call [`revoke()`](#method.revoke) to revoke a permit
    /// and its subordinate permits, recursively.
    ///
    /// Dropping the permit revokes it and any subordinate permits.
    #[must_use]
    pub fn new_sub(&self) -> Self {
        Self {
            inner: self.inner.new_sub(),
        }
    }

    /// Returns `true` if [`revoke()`](#method.revoke) has previously been called
    /// on this permit or any of its superiors.
    #[must_use]
    pub fn is_revoked(&self) -> bool {
        self.inner.is_revoked()
    }

    /// Returns `Some(())` if [`revoke()`](#method.revoke) has not been called
    /// on this permit or any of its superiors.
    #[must_use]
    pub fn ok(&self) -> Option<()> {
        if self.inner.is_revoked() {
            None
        } else {
            Some(())
        }
    }

    /// Revokes this permit and all subordinate permits.
    pub fn revoke(&self) {
        self.inner.revoke()
    }

    /// Returns `true` if this permit has any subordinate permits that have not
    /// been dropped.
    ///
    /// This includes direct subordinates and their subordinates, recursively.
    #[must_use]
    pub fn has_subs(&self) -> bool {
        Arc::strong_count(&self.inner) != 1
    }

    /// Wait indefinitely for all subordinate permits to drop.
    ///
    /// This waits for all direct subordinates and their subordinates,
    /// recursively.
    pub fn wait(&self) {
        while self.try_wait_for(Duration::from_secs(3600)).is_err() {}
    }

    /// Wait for all subordinate permits to drop.
    ///
    /// This waits for all direct subordinates and their subordinates,
    /// recursively.
    ///
    /// # Errors
    /// Returns [`DeadlineExceeded`](struct.DeadlineExceeded.html) if the subordinate permits
    /// are not all dropped before `duration` passes.
    pub fn try_wait_for(&self, duration: core::time::Duration) -> Result<(), DeadlineExceeded> {
        self.try_wait_until(Instant::now() + duration)
    }

    /// Wait for all subordinate permits to drop.
    ///
    /// This waits for all direct subordinates and their subordinates,
    /// recursively.
    ///
    /// # Errors
    /// Returns [`DeadlineExceeded`](struct.DeadlineExceeded.html) if the subordinate permits
    /// are not all dropped before `deadline` passes.
    pub fn try_wait_until(&self, deadline: std::time::Instant) -> Result<(), DeadlineExceeded> {
        while Instant::now() < deadline {
            if !self.has_subs() {
                return Ok(());
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        Err(DeadlineExceeded {})
    }
}
impl Drop for Permit {
    fn drop(&mut self) {
        self.inner.revoke()
    }
}
impl Clone for Permit {
    fn clone(&self) -> Self {
        Self {
            inner: Arc::new(self.inner.as_ref().clone()),
        }
    }
}
impl Default for Permit {
    fn default() -> Self {
        Self::new()
    }
}
