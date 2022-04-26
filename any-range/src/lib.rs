//! any-range
//! =========
//! [![crates.io version](https://img.shields.io/crates/v/any-range.svg)](https://crates.io/crates/any-range)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/any-range/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! `AnyRange<T>` enum can hold any `Range*<T>` type.
//!
//! # Use Cases
//! - Store any kind of range in a struct without adding a type parameter
//!
//! # Features
//! - `no_std`, depends only on `core`
//! - `forbid(unsafe_code)`
//! - 100% test coverage
//!
//! # Limitations
//! - Uses more bytes than a plain `Range<T>`.
//!   The alignment of `T` determines how many extra bytes the enum uses.
//!
//! # Alternatives
//! - [`anyrange`](https://crates.io/crates/anyrange)
//!   - Should be called `ToRange`
//!   - Doesn't support `RangeInclusive` or `RangeToInclusive`
//!   - Unmaintained
//!
//! # Example
//! ```
//! use any_range::AnyRange;
//! let range: AnyRange<u8> = (3..5).into();
//! assert!(range.contains(&3));
//! ```
//!
//! # Cargo Geiger Safety Report
//! # Changelog
//! - v0.1.3 - Implement `Hash`, `PartialOrd`, `Ord`
//! - v0.1.2 - Increase test coverage
//! - v0.1.1 - Update docs
//! - v0.1.0 - Initial version
#![forbid(unsafe_code)]
use core::fmt::Debug;
use core::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::cmp::Ordering;
use std::ops::RangeBounds;

/// An enum that can hold any Range* type.
///
/// # Example
/// ```
/// use any_range::AnyRange;
/// let range: AnyRange<u8> = (3..5).into();
/// assert!(range.contains(&3));
/// ```
#[derive(Clone, PartialEq, Eq, Hash)]
pub enum AnyRange<T: Clone + PartialOrd + PartialEq> {
    Range(Range<T>),
    RangeFrom(RangeFrom<T>),
    RangeFull(RangeFull),
    RangeInclusive(RangeInclusive<T>),
    RangeTo(RangeTo<T>),
    RangeToInclusive(RangeToInclusive<T>),
}
impl<T: Clone + PartialOrd + PartialEq> AnyRange<T> {
    /// Returns true if item is contained in the range.
    pub fn contains(&self, value: &T) -> bool {
        RangeBounds::contains(self, value)
    }
    /// Returns the start value as a Bound.
    pub fn start_bound(&self) -> Bound<&T> {
        RangeBounds::start_bound(self)
    }
    /// Returns the end value as a Bound.
    pub fn end_bound(&self) -> Bound<&T> {
        RangeBounds::end_bound(self)
    }
    fn order(&self) -> u8 {
        match self {
            Self::Range(_) => 0,
            Self::RangeFrom(_) => 1,
            Self::RangeFull(_) => 2,
            Self::RangeInclusive(_) => 3,
            Self::RangeTo(_) => 4,
            Self::RangeToInclusive(_) => 5,
        }
    }
}
impl<T: Clone + PartialOrd + PartialEq> RangeBounds<T> for AnyRange<T> {
    fn start_bound(&self) -> Bound<&T> {
        match self {
            Self::Range(r) => r.start_bound(),
            Self::RangeFrom(r) => r.start_bound(),
            Self::RangeFull(r) => r.start_bound(),
            Self::RangeInclusive(r) => r.start_bound(),
            Self::RangeTo(r) => r.start_bound(),
            Self::RangeToInclusive(r) => r.start_bound(),
        }
    }
    fn end_bound(&self) -> Bound<&T> {
        match self {
            Self::Range(r) => r.end_bound(),
            Self::RangeFrom(r) => r.end_bound(),
            Self::RangeFull(r) => r.end_bound(),
            Self::RangeInclusive(r) => r.end_bound(),
            Self::RangeTo(r) => r.end_bound(),
            Self::RangeToInclusive(r) => r.end_bound(),
        }
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<Range<T>> for AnyRange<T> {
    fn from(r: Range<T>) -> Self {
        Self::Range(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<RangeFrom<T>> for AnyRange<T> {
    fn from(r: RangeFrom<T>) -> Self {
        Self::RangeFrom(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<RangeFull> for AnyRange<T> {
    fn from(r: RangeFull) -> Self {
        Self::RangeFull(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<RangeInclusive<T>> for AnyRange<T> {
    fn from(r: RangeInclusive<T>) -> Self {
        Self::RangeInclusive(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<RangeTo<T>> for AnyRange<T> {
    fn from(r: RangeTo<T>) -> Self {
        Self::RangeTo(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq> From<RangeToInclusive<T>> for AnyRange<T> {
    fn from(r: RangeToInclusive<T>) -> Self {
        Self::RangeToInclusive(r)
    }
}
impl<T: Clone + PartialOrd + PartialEq + Debug> Debug for AnyRange<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> Result<(), core::fmt::Error> {
        match self {
            AnyRange::Range(r) => write!(f, "AnyRange({:?})", r),
            AnyRange::RangeFrom(r) => write!(f, "AnyRange({:?})", r),
            AnyRange::RangeFull(r) => write!(f, "AnyRange({:?})", r),
            AnyRange::RangeInclusive(r) => write!(f, "AnyRange({:?})", r),
            AnyRange::RangeTo(r) => write!(f, "AnyRange({:?})", r),
            AnyRange::RangeToInclusive(r) => write!(f, "AnyRange({:?})", r),
        }
    }
}
impl<T: Clone + PartialOrd + PartialEq + Debug> PartialOrd for AnyRange<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match (self, other) {
            (AnyRange::Range(a), AnyRange::Range(b)) if a.start == b.start => {
                a.end.partial_cmp(&b.end)
            }
            (AnyRange::Range(a), AnyRange::Range(b)) => a.start.partial_cmp(&b.start),
            (AnyRange::RangeFrom(a), AnyRange::RangeFrom(b)) => a.start.partial_cmp(&b.start),
            (AnyRange::RangeFull(_), AnyRange::RangeFull(_)) => Some(Ordering::Equal),
            (AnyRange::RangeInclusive(a), AnyRange::RangeInclusive(b))
                if a.start() == b.start() =>
            {
                a.end().partial_cmp(b.end())
            }
            (AnyRange::RangeInclusive(a), AnyRange::RangeInclusive(b)) => {
                a.start().partial_cmp(b.start())
            }
            (AnyRange::RangeTo(a), AnyRange::RangeTo(b)) => a.end.partial_cmp(&b.end),
            (AnyRange::RangeToInclusive(a), AnyRange::RangeToInclusive(b)) => {
                a.end.partial_cmp(&b.end)
            }
            (a, b) => a.order().partial_cmp(&b.order()),
        }
    }
}
impl<T: Clone + PartialOrd + PartialEq + Eq + Debug> Ord for AnyRange<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        PartialOrd::partial_cmp(self, other).unwrap()
    }
}
