//! [![crates.io version](https://img.shields.io/crates/v/permit.svg)](https://crates.io/crates/permit)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/permit/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! [`permit::Permit`](https://docs.rs/permit/latest/permit/struct.Permit.html)
//! is a struct for cancelling operations.
//!
//! # Use Cases
//! - Graceful server shutdown
//! - Cancel operations that take too long
//! - Stop in-flight operations when revoking authorization
//!
//! # Features
//! - Subordinate permits.
//!   Revoking a permit also revokes its subordinates, recursively.
//! - Drop a permit to revoke its subordinates, recursively.
//! - Wait for all subordinate permits to drop.
//! - Implements `Future`.  You can `await` a permit and return when it is revoked.
//! - Similar to Golang's [`context`](https://golang.org/pkg/context/)
//! - Depends only on `std`.
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! # Limitations
//! - Does not hold data values
//! - Allocates.  Uses [`alloc::sync::Arc`](https://doc.rust-lang.org/alloc/sync/struct.Arc.html).
//!
//! # Alternatives
//! - [`async_ctx`](https://crates.io/crates/async_ctx)
//!   - Good API
//!   - Async only
//! - [`stopper`](https://crates.io/crates/stopper/)
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
//! # Related Crates
//!
//! # Example
//!
//! Graceful shutdown:
//! ```
//! # use core::time::Duration;
//! # fn wait_for_shutdown_signal() { () }
//! let top_permit = permit::Permit::new();
//! // Start some worker threads.
//! for _ in 0..5 {
//!     let permit = top_permit.new_sub();
//!     std::thread::spawn(move || {
//!         while !permit.is_revoked() {
//!             // ...
//! #           std::thread::sleep(Duration::from_millis(1));
//!         }
//!     });
//! }
//! wait_for_shutdown_signal();
//! // Revoke all thread permits and wait for them to
//! // finish and drop their permits.
//! top_permit
//!     .revoke()
//!     .try_wait_for(Duration::from_secs(3))
//!     .unwrap();
//! ```
//!
//! # Cargo Geiger Safety Report
//! # Changelog
//! - v0.1.5 - Implement `Debug`
//! - v0.1.4 - Fix [bug](https://gitlab.com/leonhard-llc/ops/-/issues/2)
//!   where `revoke()` and then `wait()` would not wait.
//! - v0.1.3
//!   - Don't keep or wake stale
//!     [`std::task::Waker`](https://doc.rust-lang.org/std/task/struct.Waker.html) structs.
//!   - Eliminate race that causes unnecessary wake.
//! - v0.1.2 - Implement `Future`
//! - v0.1.1 - Make `revoke` return `&Self`
//! - v0.1.0 - Initial version
#![forbid(unsafe_code)]
use core::fmt::{Debug, Formatter};
use core::hash::{Hash, Hasher};
use core::pin::Pin;
use core::sync::atomic::AtomicBool;
use core::task::{Context, Poll, Waker};
use core::time::Duration;
use std::collections::HashSet;
use std::future::Future;
use std::sync::{Arc, Mutex, Weak};
use std::time::Instant;

// This code was beautiful before implementing `Future`:
// https://gitlab.com/leonhard-llc/ops/-/blob/26adc04aec12ac083fda358f176f0ef5130cda60/permit/src/lib.rs
//
// How can we simplify it?

struct ArcNode(Arc<Node>);
impl PartialEq for ArcNode {
    fn eq(&self, other: &Self) -> bool {
        Arc::as_ptr(&self.0).eq(&Arc::as_ptr(&other.0))
    }
}
impl Eq for ArcNode {}
impl Hash for ArcNode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        Arc::as_ptr(&self.0).hash(state);
    }
}
// impl Debug for ArcNode {
//     fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
//         write!(f, "ArcNode({:?})", Arc::as_ptr(&self.0))
//     }
// }

// #[derive(Debug)]
struct Inner {
    revoked: bool,
    opt_waker: Option<Waker>,
    subs: HashSet<ArcNode>,
}
impl Inner {
    #[must_use]
    pub fn new(revoked: bool) -> Self {
        Inner {
            revoked,
            opt_waker: None,
            subs: HashSet::new(),
        }
    }

    pub fn add_sub(&mut self, node: &Arc<Node>) {
        if !self.revoked {
            self.subs.insert(ArcNode(Arc::clone(node)));
        }
    }

    pub fn remove_sub(&mut self, node: &Arc<Node>) {
        let arc_node = ArcNode(Arc::clone(node));
        self.subs.remove(&arc_node);
    }

    pub fn poll(&mut self, cx: &mut Context<'_>) -> Poll<()> {
        if self.revoked {
            Poll::Ready(())
        } else {
            self.opt_waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }

    pub fn revoke(&mut self) -> (Option<Waker>, HashSet<ArcNode>) {
        self.revoked = true;
        (self.opt_waker.take(), core::mem::take(&mut self.subs))
    }
}

// #[derive(Debug)]
struct Node {
    superior: Weak<Node>,
    atomic_revoked: AtomicBool,
    inner: Mutex<Inner>,
}
impl Node {
    #[must_use]
    pub fn new(revoked: bool, superior: Weak<Self>) -> Self {
        Node {
            superior,
            atomic_revoked: AtomicBool::new(revoked),
            inner: Mutex::new(Inner::new(revoked)),
        }
    }

    #[must_use]
    pub fn new_apex() -> Self {
        Self::new(false, Weak::new())
    }

    #[must_use]
    pub fn new_sub(self: &Arc<Self>) -> Arc<Self> {
        let node = Arc::new(Self::new(self.is_revoked(), Arc::downgrade(self)));
        self.inner.lock().unwrap().add_sub(&node);
        node
    }

    #[must_use]
    pub fn new_clone(self: &Arc<Self>) -> Arc<Self> {
        let node = Arc::new(Self::new(self.is_revoked(), Weak::clone(&self.superior)));
        if let Some(superior) = self.superior.upgrade() {
            superior.add_sub(&node);
        }
        node
    }

    pub fn add_sub(self: &Arc<Self>, node: &Arc<Node>) {
        self.inner.lock().unwrap().add_sub(node);
    }

    fn remove_sub(&self, node: &Arc<Node>) {
        self.inner.lock().unwrap().remove_sub(node);
    }

    #[must_use]
    pub fn has_subs(self: &Arc<Self>) -> bool {
        Arc::weak_count(self) != 0
    }

    #[must_use]
    pub fn is_revoked(&self) -> bool {
        self.atomic_revoked
            .load(std::sync::atomic::Ordering::Relaxed)
    }

    pub fn poll(self: &Arc<Self>, cx: &mut Context<'_>) -> Poll<()> {
        self.inner.lock().unwrap().poll(cx)
    }

    fn revoke(self: &Arc<Self>, wake: bool) {
        self.atomic_revoked
            .store(true, std::sync::atomic::Ordering::Relaxed);
        let (opt_waker, subs) = self.inner.lock().unwrap().revoke();
        if wake {
            if let Some(waker) = opt_waker {
                waker.wake();
            }
        }
        for sub in subs {
            sub.0.revoke(true);
        }
    }

    pub fn revoke_and_remove_from_superior(self: &Arc<Self>) {
        if let Some(superior) = self.superior.upgrade() {
            superior.remove_sub(self);
        }
        self.revoke(false);
    }
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct DeadlineExceeded;
impl core::fmt::Display for DeadlineExceeded {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
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
/// // Revoke all thread permits and wait for them to
/// // finish and drop their permits.
/// top_permit
///     .revoke()
///     .try_wait_for(core::time::Duration::from_secs(3))
///     .unwrap();
/// ```
pub struct Permit {
    node: Arc<Node>,
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
            node: Arc::new(Node::new_apex()),
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
            node: self.node.new_sub(),
        }
    }

    /// Returns `true` if [`revoke()`](#method.revoke) has previously been called
    /// on this permit or any of its superiors.
    #[must_use]
    pub fn is_revoked(&self) -> bool {
        self.node.is_revoked()
    }

    /// Returns `Some(())` if [`revoke()`](#method.revoke) has not been called
    /// on this permit or any of its superiors.
    #[must_use]
    pub fn ok(&self) -> Option<()> {
        if self.node.is_revoked() {
            None
        } else {
            Some(())
        }
    }

    /// Revokes this permit and all subordinate permits.
    #[allow(clippy::must_use_candidate)]
    pub fn revoke(&self) -> &Self {
        self.node.revoke_and_remove_from_superior();
        self
    }

    /// Returns `true` if this permit has any subordinate permits that have not
    /// been dropped.
    ///
    /// This includes direct subordinates and their subordinates, recursively.
    #[must_use]
    pub fn has_subs(&self) -> bool {
        self.node.has_subs()
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
    pub fn try_wait_for(&self, duration: Duration) -> Result<(), DeadlineExceeded> {
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
    pub fn try_wait_until(&self, deadline: Instant) -> Result<(), DeadlineExceeded> {
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
        self.node.revoke_and_remove_from_superior();
    }
}
impl Clone for Permit {
    fn clone(&self) -> Self {
        Self {
            node: self.node.new_clone(),
        }
    }
}
impl Debug for Permit {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), core::fmt::Error> {
        write!(
            f,
            "Permit{{revoked={},num_subs={}}}",
            self.is_revoked(),
            Arc::weak_count(&self.node)
        )
    }
}
impl Default for Permit {
    fn default() -> Self {
        Self::new()
    }
}
impl Future for Permit {
    type Output = ();

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.node.poll(cx)
    }
}
