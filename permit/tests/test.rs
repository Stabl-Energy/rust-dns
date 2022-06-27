#![forbid(unsafe_code)]
use core::ops::Range;
use permit::{DeadlineExceeded, Permit};
use safina_async_test::async_test;
use std::time::{Duration, Instant};

fn expect_elapsed(before: Instant, range_ms: Range<u64>) -> Result<(), String> {
    if range_ms.is_empty() {
        return Err(format!("invalid range {:?}", range_ms));
    }
    let elapsed = before.elapsed();
    let duration_range = Duration::from_millis(range_ms.start)..Duration::from_millis(range_ms.end);
    if !duration_range.contains(&elapsed) {
        return Err(format!(
            "{:?} elapsed, out of range {:?}",
            elapsed, duration_range
        ));
    }
    Ok(())
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
fn debug() {
    let pmt = Permit::default();
    assert_eq!("Permit{revoked=false,num_subs=0}", &format!("{:?}", &pmt));
    let sub1 = pmt.new_sub();
    let _sub2 = pmt.new_sub();
    assert_eq!("Permit{revoked=false,num_subs=2}", &format!("{:?}", &pmt));
    assert_eq!("Permit{revoked=false,num_subs=0}", &format!("{:?}", &sub1));
    pmt.revoke();
    assert_eq!("Permit{revoked=true,num_subs=2}", &format!("{:?}", &pmt));
    assert_eq!("Permit{revoked=true,num_subs=0}", &format!("{:?}", &sub1));
}

#[test]
fn default() {
    let pmt = Permit::default();
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
    let top_permit = Permit::new();
    assert!(!top_permit.has_subs());
    let sub1 = top_permit.new_sub();
    assert!(top_permit.has_subs());
    drop(sub1);
    assert!(!top_permit.has_subs());

    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(permit);
        });
    }
    assert!(top_permit.has_subs());
    top_permit.wait();
    expect_elapsed(before, 50..100).unwrap();
    assert!(!top_permit.has_subs());
}

#[test]
fn wait() {
    let before = Instant::now();
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit.wait();
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
// https://gitlab.com/leonhard-llc/ops/-/issues/2
fn revoke_then_wait() {
    let before = Instant::now();
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit.revoke();
    top_permit.wait();
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
fn try_wait_for() {
    let before = Instant::now();
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit.try_wait_for(Duration::from_millis(100)).unwrap();
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
fn try_wait_until() {
    let before = Instant::now();
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(50));
            drop(permit);
        });
    }
    top_permit
        .try_wait_until(before + Duration::from_millis(100))
        .unwrap();
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
fn try_wait_for_deadline_exceeded() {
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            drop(permit);
        });
    }
    let before = Instant::now();
    assert_eq!(
        Err(DeadlineExceeded),
        top_permit.try_wait_for(Duration::from_millis(50))
    );
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
fn try_wait_until_deadline_exceeded() {
    let top_permit = Permit::new();
    for _ in 0..2 {
        let permit = top_permit.new_sub();
        std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(100));
            drop(permit);
        });
    }
    let before = Instant::now();
    assert_eq!(
        Err(DeadlineExceeded),
        top_permit.try_wait_until(before + Duration::from_millis(50))
    );
    expect_elapsed(before, 50..100).unwrap();
}

#[test]
fn deadline_exceeded() {
    assert_eq!("DeadlineExceeded", &format!("{}", DeadlineExceeded {}));
}

#[test]
fn await_revoked_returns_immediately() {
    let before = Instant::now();
    let permit = Permit::new();
    permit.revoke();
    safina::executor::block_on(async move { permit.await });
    expect_elapsed(before, 0..10).unwrap();
}

#[test]
fn await_timeout() {
    safina::timer::start_timer_thread();
    let permit = Permit::new();
    let sub = permit.new_sub();
    safina::executor::block_on(async move {
        safina::timer::with_timeout(sub, Duration::from_millis(50))
            .await
            .unwrap_err()
    });
}

#[test]
fn await_returns_when_revoked() {
    let before = Instant::now();
    let permit = Permit::new();
    let sub = permit.new_sub();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(50));
        drop(permit);
    });
    safina::executor::block_on(async move { sub.await });
    expect_elapsed(before, 50..100).unwrap();
}

#[async_test]
async fn await_many() {
    let before = Instant::now();
    let top_permit = Permit::new();
    let mut receivers = Vec::new();
    for _ in 0..100_000 {
        let permit = top_permit.new_sub();
        let (sender, receiver) = safina::sync::oneshot();
        receivers.push(receiver);
        safina::executor::spawn(async move {
            permit.await;
            sender.send(()).unwrap();
        });
    }
    safina::timer::sleep_for(Duration::from_millis(50)).await;
    top_permit.revoke();
    for mut receiver in receivers {
        receiver.async_recv().await.unwrap();
    }
    expect_elapsed(before, 0..10_000).unwrap();
}

#[async_test]
async fn await_loop() {
    let before = Instant::now();
    let top_permit = Permit::new();
    let permit = top_permit.new_sub();
    for _ in 0..5 {
        let sub = permit.new_sub();
        safina::timer::with_timeout(sub, Duration::from_millis(1))
            .await
            .unwrap_err();
    }
    expect_elapsed(before, 0..500).unwrap();
    drop(permit);
    drop(top_permit);
}
