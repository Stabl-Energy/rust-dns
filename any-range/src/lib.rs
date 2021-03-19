//! [![crates.io version](https://img.shields.io/crates/v/any-range.svg)](https://crates.io/crates/any-range)
//! [![license: Apache 2.0](https://gitlab.com/leonhard-llc/ops/-/raw/main/license-apache-2.0.svg)](https://gitlab.com/leonhard-llc/ops/-/raw/main/any-range/LICENSE)
//! [![unsafe forbidden](https://gitlab.com/leonhard-llc/ops/-/raw/main/unsafe-forbidden.svg)](https://github.com/rust-secure-code/safety-dance/)
//! [![pipeline status](https://gitlab.com/leonhard-llc/ops/badges/main/pipeline.svg)](https://gitlab.com/leonhard-llc/ops/-/pipelines)
//!
//! # any-range
//!
//! `AnyRange<T>` enum can hold any `Range*<T>` type.
//!
//! ## Use Cases
//! - Store any kind of range in a struct without adding a type parameter
//!
//! ## Features
//! - `no_std`, depends only on `core`
//! - `forbid(unsafe_code)`
//!
//! ## Limitations
//! - Uses more bytes than a plain `Range<T>`.
//!   The alignment of `T` determines how many extra bytes the enum uses.
//!
//! ## Alternatives
//! - [`anyrange`](https://crates.io/crates/anyrange)
//!   - Should be called `ToRange`
//!   - Doesn't support `RangeInclusive` or `RangeToInclusive`
//!   - Unmaintained
//!
//! ## Example
//! ```
//! use any_range::AnyRange;
//! let range: AnyRange<u8> = (3..5).into();
//! assert!(range.contains(&3));
//! ```
//!
//! ## Cargo Geiger Safety Report
//!
#![forbid(unsafe_code)]
use core::fmt::Debug;
use core::ops::{Bound, Range, RangeFrom, RangeFull, RangeInclusive, RangeTo, RangeToInclusive};
use std::ops::RangeBounds;

/// An enum that can hold any Range* type.
///
/// # Example
/// ```
/// use any_range::AnyRange;
/// let range: AnyRange<u8> = (3..5).into();
/// assert!(range.contains(&3));
/// ```
#[derive(Clone, PartialEq, Eq)]
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

#[cfg(test)]
mod tests {
    use crate::AnyRange;
    use std::ops::Bound;

    #[test]
    fn test() {
        let range: AnyRange<u8> = (3..5).into();
        let range_from: AnyRange<u8> = (3..).into();
        let range_full: AnyRange<u8> = (..).into();
        let range_inclusive: AnyRange<u8> = (3..=5).into();
        let range_to: AnyRange<u8> = (..5).into();
        let range_to_inclusive: AnyRange<u8> = (..=5).into();

        let range_clone = range.clone();
        assert_eq!(range, range_clone);

        assert_eq!(Bound::Included(&3), range.start_bound());
        assert_eq!(Bound::Included(&3), range_from.start_bound());
        assert_eq!(Bound::Unbounded, range_full.start_bound());
        assert_eq!(Bound::Included(&3), range_inclusive.start_bound());
        assert_eq!(Bound::Unbounded, range_to.start_bound());
        assert_eq!(Bound::Unbounded, range_to_inclusive.start_bound());

        assert_eq!(Bound::Excluded(&5), range.end_bound());
        assert_eq!(Bound::Unbounded, range_from.end_bound());
        assert_eq!(Bound::Unbounded, range_full.end_bound());
        assert_eq!(Bound::Included(&5), range_inclusive.end_bound());
        assert_eq!(Bound::Excluded(&5), range_to.end_bound());
        assert_eq!(Bound::Included(&5), range_to_inclusive.end_bound());

        assert!(range.contains(&3));
        assert!(range.contains(&4));
        assert!(!range.contains(&5));
        assert!(range_from.contains(&3));
        assert!(range_from.contains(&100));
        assert!(range_full.contains(&0));
        assert!(range_full.contains(&100));
        assert!(range_inclusive.contains(&3));
        assert!(range_inclusive.contains(&4));
        assert!(range_inclusive.contains(&5));
        assert!(!range_inclusive.contains(&6));
        assert!(range_to.contains(&0));
        assert!(range_to.contains(&4));
        assert!(!range_to.contains(&5));
        assert!(range_to_inclusive.contains(&0));
        assert!(range_to_inclusive.contains(&4));
        assert!(range_to_inclusive.contains(&5));
        assert!(!range_to_inclusive.contains(&6));
    }
}
