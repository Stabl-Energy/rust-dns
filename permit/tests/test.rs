#![forbid(unsafe_code)]
use core::ops::Range;
use permit::{DeadlineExceeded, Permit};
use std::time::{Duration, Instant};

pub fn expect_elapsed(before: Instant, range_ms: Range<u64>) {
    if range_ms.is_empty() {
        panic!("invalid range {:?}", range_ms)
    }
    let elapsed = before.elapsed();
    let duration_range = Duration::from_millis(range_ms.start)..Duration::from_millis(range_ms.end);
    if !duration_range.contains(&elapsed) {
        panic!("{:?} elapsed, out of range {:?}", elapsed, duration_range);
    }
}

#[test]
fn new() {
    let pmt = Permit::new();
    assert!(!pmt.is_revoked());
    assert_eq!(Some(()), pmt.ok());
    pmt.revoke();
    assert!(pmt.is_revoked());
    assert_eq!(None, pmt.ok());
}

#[test]
fn default() {
    let pmt: Permit = Default::default();
    assert!(!pmt.is_revoked());
    pmt.revoke();
    assert!(pmt.is_revoked());
}

#[test]
fn revoke_superior() {
    let superior = Permit::new();
    let pmt = superior.new_sub();
    let sub = pmt.new_sub();
    assert!(!superior.is_revoked());
    assert!(!pmt.is_revoked());
    assert!(!sub.is_revoked());
    superior.revoke();
    assert!(superior.is_revoked());
    assert!(pmt.is_revoked());
    assert!(sub.is_revoked());
}

#[test]
fn revoke_sub() {
    let pmt = Permit::new();
    let sub = pmt.new_sub();
    assert!(!pmt.is_revoked());
    assert!(!sub.is_revoked());
    sub.revoke();
    assert!(!pmt.is_revoked());
    assert!(sub.is_revoked());
}

#[test]
fn revoke_then_sub() {
    let pmt = Permit::new();
    pmt.revoke();
    let sub = pmt.new_sub();
    assert!(pmt.is_revoked());
    assert!(sub.is_revoked());
}

#[test]
fn revoke_both() {
    let pmt = Permit::new();
    let sub = pmt.new_sub();
    assert!(!pmt.is_revoked());
    assert!(!sub.is_revoked());
    sub.revoke();
    pmt.revoke();
    assert!(pmt.is_revoked());
    assert!(sub.is_revoked());
}

#[test]
fn test_drop() {
    let pmt = Permit::new();
    let sub = pmt.new_sub();
    drop(pmt);
    assert!(sub.is_revoked());
}

#[test]
fn revoke_clone() {
    let pmt = Permit::new();
    let clone = pmt.clone();
    clone.revoke();
    assert!(!pmt.is_revoked());
    assert!(clone.is_revoked());
}

#[test]
fn revoke_clone_source() {
    let pmt = Permit::new();
    let clone = pmt.clone();
    pmt.revoke();
    assert!(pmt.is_revoked());
    assert!(!clone.is_revoked());
}

#[test]
fn revoke_clones_superior() {
    let superior = Permit::new();
    let pmt = superior.new_sub();
    let clone = pmt.clone();
    superior.revoke();
    assert!(superior.is_revoked());
    assert!(pmt.is_revoked());
    assert!(clone.is_revoked());
}

#[test]
fn clone_revoked() {
    let pmt = Permit::new();
    pmt.revoke();
    let clone = pmt.clone();
    assert!(pmt.is_revoked());
    assert!(clone.is_revoked());
}

#[test]
fn revoke_superior_and_clone() {
    let superior = Permit::new();
    let pmt = superior.new_sub();
    superior.revoke();
    let clone = pmt.clone();
    assert!(superior.is_revoked());
    assert!(pmt.is_revoked());
    assert!(clone.is_revoked());
}

#[test]
fn sync_and_send() {
    let superior = Permit::new();
    let pmt = superior.new_sub();
    let join_handle = std::thread::spawn(move || {
        for _ in 0..50 {
            if pmt.is_revoked() {
                return true;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        false
    });
    superior.revoke();
    assert!(join_handle.join().unwrap());
}

#[test]
fn has_subs() {
    let before = Instant::now();
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(50));
            drop(permit);
        });
    }
    assert!(top_permit.has_subs());
    top_permit.wait();
    expect_elapsed(before, 50..100);
    assert!(!top_permit.has_subs());
}

#[test]
fn wait() {
    let before = Instant::now();
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit.wait();
    expect_elapsed(before, 50..100);
}

#[test]
fn try_wait_for() {
    let before = Instant::now();
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit.try_wait_for(Duration::from_millis(100)).unwrap();
    expect_elapsed(before, 50..100);
}

#[test]
fn try_wait_until() {
    let before = Instant::now();
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit
        .try_wait_until(before + Duration::from_millis(100))
        .unwrap();
    expect_elapsed(before, 50..100);
}

#[test]
fn try_wait_for_deadline_exceeded() {
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(100));
            drop(permit);
        });
    }
    let before = Instant::now();
    assert_eq!(
        Err(DeadlineExceeded),
        top_permit.try_wait_for(Duration::from_millis(50))
    );
    expect_elapsed(before, 50..100);
}

#[test]
fn try_wait_until_deadline_exceeded() {
    let top_permit = permit::Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(core::time::Duration::from_millis(100));
            drop(permit);
        });
    }
    let before = Instant::now();
    assert_eq!(
        Err(DeadlineExceeded),
        top_permit.try_wait_until(before + Duration::from_millis(50))
    );
    expect_elapsed(before, 50..100);
}

#[test]
fn deadline_exceeded() {
    assert_eq!("DeadlineExceeded", &format!("{}", DeadlineExceeded {}));
}
