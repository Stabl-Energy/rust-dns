//! [![crates.io version](https://img.shields.io/crates/v/safe-lock.svg)](https://crates.io/crates/safe-lock)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/safe-lock/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # safe-lock
//!
//! A simple `SafeLock` struct.
//!
//! ## Use Cases
//! - Run tests sequentially
//! - Prevent concurrent operations on atomic values
//! - Prevent concurrent operations on data and systems outside the Rust runtime
//!
//! ## Features
//! - Const constructor
//! - Depends only on `std`
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! ## Limitations
//! - Not a `Mutex<T>`.  Does not contain a value.
//! - Unoptimized.  Uses
//!   [`AtomicBool`](https://doc.rust-lang.org/core/sync/atomic/struct.AtomicBool.html)
//!   in a spinlock, not fast OS locks.
//! - Not a fair lock.  If multiple threads acquire the lock in loops,
//!   some may never acquire it.
//!
//! ## Alternatives
//! - [`rusty-fork`](https://crates.io/crates/rusty-fork)
//!   - Run tests in separate processes
//! - [`std::sync::Mutex`](https://doc.rust-lang.org/std/sync/struct.Mutex.html)
//!   - Part of the Rust standard library: well reviewed, well tested, and well maintained.
//!   - Uses fast OS locks
//!   - Has no const constructor.  See [rust#66806](https://github.com/rust-lang/rust/issues/66806)
//!     and [const-eval#3](https://github.com/rust-lang/const-eval/issues/3).
//!     You can work around this with unstable
//!     [`core::lazy::OnceCell`](https://doc.rust-lang.org/core/lazy/struct.OnceCell.html)
//!     or various `unsafe` crates:
//!     [`lazy_static`](https://crates.io/crates/lazy_static),
//!     [`once_cell`](https://crates.io/crates/once_cell),
//!     [`lazycell`](https://crates.io/crates/lazycell), and
//!     [`conquer-once`](https://crates.io/crates/conquer-once).
//! - [`parking_lot`](https://crates.io/crates/parking_lot)
//!   - Well written code.
//!     Many hope that it will end up in the Rust standard library someday.
//!   - Contains plenty of `unsafe`
//! - [`try-lock`](https://crates.io/crates/try-lock)
//!   - Popular
//!   - No dependencies, `no_std`
//!   - Uses `unsafe`
//! - [`ruspiro-lock`](https://crates.io/crates/ruspiro-lock)
//!   - Sync and async locks
//!   - No dependencies, `no_std`
//!   - Uses `unsafe`
//! - [`flexible-locks`](https://crates.io/crates/flexible-locks)
//!   - Lots of `unsafe`
//!   - Uses fast OS locks
//!   - Unmaintained
//!
//! ## Related Crates
//! - [`safina-sync`](https://crates.io/crates/safina-sync)
//!   provides a safe async `Mutex`
//!
//! ## Example
//!
//! Make some tests run sequentially so they don't interfere with each other:
//! ```unknown
//! use safe_lock::SafeLock;
//! static LOCK: SafeLock = SafeLock::new();
//!
//! [#test]
//! fn test1() {
//!     let _guard = LOCK.lock();
//!     // ...
//! }
//!
//! [#test]
//! fn test2() {
//!     let _guard = LOCK.lock();
//!     // ...
//! }
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
//! ## Changelog
//! - v0.1.3 - Increase test coverage
//! - v0.1.2 - Use `Acquire` and `Release` ordering
//! - v0.1.1 - Update docs
//! - v0.1.0 - Initial version
//!
//! ## Happy Contributors 🙂
//! Fixing bugs and adding features is easy and fast.
//! Send us a pull request and we intend to:
//! - Always respond within 24 hours
//! - Provide clear & concrete feedback
//! - Immediately make a new release for your accepted change
#![forbid(unsafe_code)]
use core::sync::atomic::{AtomicBool, Ordering};

/// A handle to the acquired lock.  Drop this to release the lock.
pub struct SafeLockGuard<'x> {
    inner: &'x SafeLock,
}
impl<'x> Drop for SafeLockGuard<'x> {
    fn drop(&mut self) {
        if !self.inner.locked.swap(false, Ordering::Release) {
            unreachable!();
        }
    }
}

#[cfg(test)]
#[test]
fn test_unreachable() {
    let lock: SafeLock = Default::default();
    let guard1 = lock.lock();
    let guard2 = SafeLockGuard { inner: &lock };
    drop(guard1);
    match std::panic::catch_unwind(move || drop(guard2)) {
        Ok(_) => panic!("expected panic"),
        Err(any) => {
            assert_eq!(
                "internal error: entered unreachable code",
                *any.downcast::<&'static str>().unwrap()
            );
        }
    }
}

/// A lock.
///
/// See [`lock`](#method.lock).
///
/// This is not a fair lock.
/// If multiple threads acquire the lock in loops,
/// some may never acquire it.
///
/// # Example
///
/// Make some tests run sequentially so they don't interfere with each other:
/// ```unknown
/// use safe_lock::SafeLock;
/// static LOCK: SafeLock = SafeLock::new();
///
/// [#test]
/// fn test1() {
///     let _guard = LOCK.lock();
///     // ...
/// }
///
/// [#test]
/// fn test2() {
///     let _guard = LOCK.lock();
///     // ...
/// }
/// ```
pub struct SafeLock {
    locked: AtomicBool,
}
impl SafeLock {
    #[must_use]
    pub const fn new() -> SafeLock {
        SafeLock {
            locked: AtomicBool::new(false),
        }
    }

    /// Waits until the lock is free, then acquires the lock.
    ///
    /// Multiple threads can call `lock` but only one will acquire the lock
    /// and return.
    ///
    /// Drop the returned `SafeLockGuard` to release the lock.
    ///
    /// This is not a fair lock.
    /// If multiple threads acquire the lock in loops,
    /// some may never acquire it.
    ///
    /// Uses
    /// [`Ordering::Acquire`](https://doc.rust-lang.org/stable/std/sync/atomic/enum.Ordering.html#variant.Acquire)
    /// to acquire the lock and
    /// [`Ordering::Release`](https://doc.rust-lang.org/stable/std/sync/atomic/enum.Ordering.html#variant.Release)
    /// to release it, so the lock orders operations on other atomic values.
    #[must_use]
    pub fn lock(&self) -> Option<SafeLockGuard> {
        loop {
            // We could use `Ordering::Relaxed` here:
            // "Typical use for relaxed memory ordering is incrementing
            // counters, such as the reference counters of `std::shared_ptr`,
            // since this only requires atomicity, but not ordering or
            // synchronization (note that decrementing the `shared_ptr`
            // counters requires acquire-release synchronization with the
            // destructor)"
            // https://en.cppreference.com/w/cpp/atomic/memory_order#Relaxed_ordering
            //
            // But one use-case for `SafeLock` is to prevent concurrent
            // operations on atomic values.  So we need ordering between
            // acquiring the lock and seeing changes to other atomic values.
            // Therefore we use Ordering::Acquire and Ordering::Release.
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::Acquire, Ordering::Relaxed)
                .is_ok()
            {
                return Some(SafeLockGuard { inner: self });
            }
            std::thread::yield_now();
        }
    }
}
impl Default for SafeLock {
    fn default() -> Self {
        Self::new()
    }
}
