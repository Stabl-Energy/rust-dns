use any_range::AnyRange;
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

    assert_eq!("AnyRange(3..5)", &format!("{range:?}"));
    assert_eq!("AnyRange(3..)", &format!("{range_from:?}"));
    assert_eq!("AnyRange(..)", &format!("{range_full:?}"));
    assert_eq!("AnyRange(3..=5)", &format!("{range_inclusive:?}"));
    assert_eq!("AnyRange(..5)", &format!("{range_to:?}"));
    assert_eq!("AnyRange(..=5)", &format!("{range_to_inclusive:?}"));

    assert_eq!(AnyRange::Range(0..2), AnyRange::Range(0..2));
    assert!(AnyRange::Range(0..1) < AnyRange::Range(0..2));
    assert!(AnyRange::Range(0..2) < AnyRange::Range(1..2));
    assert_eq!(AnyRange::RangeFrom(0..), AnyRange::RangeFrom(0..));
    assert!(AnyRange::RangeFrom(0..) < AnyRange::RangeFrom(1..));
    assert_eq!(AnyRange::RangeFull::<u8>(..), AnyRange::RangeFull(..));
    assert_eq!(
        AnyRange::RangeInclusive(0..=2),
        AnyRange::RangeInclusive(0..=2)
    );
    assert!(AnyRange::RangeInclusive(0..=1) < AnyRange::RangeInclusive(0..=2));
    assert!(AnyRange::RangeInclusive(0..=2) < AnyRange::RangeInclusive(1..=2));
    assert_eq!(AnyRange::RangeTo(..0), AnyRange::RangeTo(..0));
    assert!(AnyRange::RangeTo(..0) < AnyRange::RangeTo(..1));
    assert_eq!(
        AnyRange::RangeToInclusive(..=0),
        AnyRange::RangeToInclusive(..=0)
    );
    assert!(AnyRange::RangeToInclusive(..=0) < AnyRange::RangeToInclusive(..=1));
    assert!(range < range_from);
    assert!(range_from < range_full);
    assert!(range_full < range_inclusive);
    assert!(range_inclusive < range_to);
    assert!(range_to < range_to_inclusive);
}
