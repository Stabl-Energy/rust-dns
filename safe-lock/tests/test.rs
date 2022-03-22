use core::time::Duration;
use safe_lock::SafeLock;
use std::sync::atomic::{AtomicBool, Ordering};

static _STATIC_SAFE_LOCK: SafeLock = SafeLock::new();

#[test]
fn default() {
    let lock = SafeLock::default();
    let _guard = lock.lock();
}

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
    assert!((100..150).contains(&elapsed.as_millis()), "{:?}", elapsed);
}

#[test]
fn multiple_threads() {
    static LOCK: SafeLock = SafeLock::new();
    static FLAG: AtomicBool = AtomicBool::new(false);
    let mut join_handles = Vec::new();
    for _ in 0..10 {
        join_handles.push(std::thread::spawn(|| {
            for _ in 0..100 {
                let _guard = LOCK.lock();
                assert!(!FLAG.swap(true, Ordering::SeqCst));
                std::thread::sleep(Duration::from_millis(1));
                assert!(FLAG.swap(false, Ordering::SeqCst));
            }
        }));
    }
    for join_handle in join_handles.drain(..) {
        join_handle.join().unwrap();
    }
}
