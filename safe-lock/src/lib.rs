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
//! ```ignore
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
#![forbid(unsafe_code)]
use core::sync::atomic::{AtomicBool, Ordering};

/// A handle to the acquired lock.  Drop this to release the lock.
pub struct SafeLockGuard<'x> {
    inner: &'x SafeLock,
}
impl<'x> Drop for SafeLockGuard<'x> {
    fn drop(&mut self) {
        if !self.inner.locked.swap(false, Ordering::SeqCst) {
            eprintln!("lock released twice")
            // unreachable!()
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
/// ```ignore
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
    #[must_use]
    pub fn lock(&self) -> Option<SafeLockGuard> {
        loop {
            if self
                .locked
                .compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::Acquire)
                .is_ok()
            {
                return Some(SafeLockGuard { inner: &self });
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

#[cfg(test)]
mod tests {
    use crate::SafeLock;
    use core::time::Duration;
    use std::sync::atomic::{AtomicBool, Ordering};

    static _STATIC_SAFE_LOCK: SafeLock = SafeLock::new();

    #[test]
    fn sequential() {
        let lock = SafeLock::new();
        {
            let _guard = lock.lock();
        }
        {
            let _guard = lock.lock();
        }
    }

    #[test]
    fn quick_unblock() {
        static LOCK: SafeLock = SafeLock::new();
        let before = std::time::Instant::now();
        std::thread::spawn(|| {
            let _guard = LOCK.lock();
            std::thread::sleep(Duration::from_millis(100));
        });
        std::thread::sleep(Duration::from_millis(50));
        let _guard = LOCK.lock();
        let elapsed = std::time::Instant::now().saturating_duration_since(before);
        assert!((100..150).contains(&elapsed.as_millis()), elapsed);
    }

    #[test]
    fn multiple_threads() {
        static LOCK: SafeLock = SafeLock::new();
        static FLAG: AtomicBool = AtomicBool::new(false);
        let mut join_handles = Vec::new();
        for _ in 0..5 {
            join_handles.push(std::thread::spawn(|| {
                let _guard = LOCK.lock();
                assert!(!FLAG.swap(true, Ordering::Acquire));
                std::thread::sleep(Duration::from_millis(100));
                assert!(FLAG.swap(false, Ordering::Release));
                true
            }));
        }
        for join_handle in join_handles.drain(..) {
            join_handle.join().unwrap();
        }
    }
}
