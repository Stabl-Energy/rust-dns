use crate::{TempDir, COUNTER};
use core::sync::atomic::Ordering;
use safe_lock::SafeLock;
use std::io::ErrorKind;
use std::path::Path;

// The error tests require all tests to run single-threaded.
static LOCK: SafeLock = SafeLock::new();

fn expect_not_found(path: impl AsRef<Path>) {
    match std::fs::metadata(&path) {
        Ok(_) => panic!("exists {:?}", path.as_ref()),
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => panic!("error getting metadata of {:?}: {}", path.as_ref(), e),
    }
}

#[test]
fn new() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    println!("{:?}", temp_dir);
    println!("{:?}", TempDir::new().unwrap());
    let metadata = std::fs::metadata(temp_dir.path()).unwrap();
    assert!(metadata.is_dir());
    let temp_dir2 = TempDir::new().unwrap();
    assert_ne!(temp_dir.path(), temp_dir2.path());
}

#[test]
fn new_error() {
    let _guard = LOCK.lock();
    let previous_counter_value = COUNTER.load(Ordering::SeqCst);
    let temp_dir = TempDir::new().unwrap();
    COUNTER.store(previous_counter_value, Ordering::SeqCst);
    assert_eq!(
        Err(format!(
            "error creating directory {:?}: File exists (os error 17)",
            temp_dir.path()
        )),
        TempDir::new()
    );
}

#[test]
fn with_prefix() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::with_prefix("prefix1").unwrap();
    let name = temp_dir.path().file_name().unwrap();
    assert!(
        name.to_str().unwrap().starts_with("prefix1"),
        "{:?}",
        temp_dir
    );
    let metadata = std::fs::metadata(temp_dir.path()).unwrap();
    assert!(metadata.is_dir());
    let temp_dir2 = TempDir::new().unwrap();
    assert_ne!(temp_dir.path(), temp_dir2.path());
}

#[test]
fn with_prefix_error() {
    let _guard = LOCK.lock();
    let previous_counter_value = COUNTER.load(Ordering::SeqCst);
    let temp_dir = TempDir::with_prefix("prefix1").unwrap();
    COUNTER.store(previous_counter_value, Ordering::SeqCst);
    assert_eq!(
        Err(format!(
            "error creating directory {:?}: File exists (os error 17)",
            temp_dir.path()
        )),
        TempDir::with_prefix("prefix1")
    );
}

#[test]
fn child() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    let file1_path = temp_dir.child("file1");
    assert!(
        file1_path.ends_with("file1"),
        "{:?}",
        file1_path.to_string_lossy()
    );
    assert!(
        file1_path.starts_with(temp_dir.path()),
        "{:?}",
        file1_path.to_string_lossy()
    );
    std::fs::write(&file1_path, b"abc").unwrap();
}

#[test]
fn test_drop() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    let file1_path = temp_dir.child("file1");
    std::fs::write(&file1_path, b"abc").unwrap();
    TempDir::new().unwrap();
    drop(temp_dir);
    expect_not_found(&dir_path);
    expect_not_found(&file1_path);
}

#[test]
fn drop_already_deleted() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    std::fs::remove_dir(temp_dir.path()).unwrap();
}

#[cfg(unix)]
#[test]
fn drop_error_ignored() {
    // On Gitlab's shared CI runners, the cleanup always succeeds and the
    // test fails.  So we skip this test when it's running on Gitlab CI.
    if std::env::current_dir().unwrap().starts_with("/builds/") {
        println!("Running on Gitlab CI.  Skipping test.");
        return;
    }
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    let file1_path = temp_dir.child("file1");
    std::fs::write(&file1_path, b"abc").unwrap();
    assert!(std::process::Command::new("chmod")
        .arg("-w")
        .arg(temp_dir.path())
        .status()
        .unwrap()
        .success());
    drop(temp_dir);
    std::fs::metadata(&dir_path).unwrap();
    std::fs::metadata(&file1_path).unwrap();
    assert!(std::process::Command::new("chmod")
        .arg("u+w")
        .arg(&dir_path)
        .status()
        .unwrap()
        .success());
    std::fs::remove_dir_all(&dir_path).unwrap();
}

#[cfg(unix)]
#[test]
fn drop_error_panic() {
    // On Gitlab's shared CI runners, the cleanup always succeeds and the
    // test fails.  So we skip this test when it's running on Gitlab CI.
    if std::env::current_dir().unwrap().starts_with("/builds/") {
        println!("Running on Gitlab CI.  Skipping test.");
        return;
    }
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap().panic_on_cleanup_error();
    let dir_path = temp_dir.path().to_path_buf();
    let file1_path = temp_dir.child("file1");
    std::fs::write(&file1_path, b"abc").unwrap();
    assert!(std::process::Command::new("chmod")
        .arg("-w")
        .arg(temp_dir.path())
        .status()
        .unwrap()
        .success());
    let result = std::panic::catch_unwind(move || drop(temp_dir));
    std::fs::metadata(&dir_path).unwrap();
    std::fs::metadata(&file1_path).unwrap();
    assert!(std::process::Command::new("chmod")
        .arg("u+w")
        .arg(&dir_path)
        .status()
        .unwrap()
        .success());
    std::fs::remove_dir_all(&dir_path).unwrap();
    match result {
        Ok(_) => panic!("expected panic"),
        Err(any) => {
            let e = any.downcast::<String>().unwrap();
            assert!(
                e.starts_with(&format!(
                    "error removing directory and contents {:?}: ",
                    dir_path
                )),
                "unexpected error {:?}",
                e
            );
        }
    }
}

#[test]
fn leak() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::new().unwrap();
    let dir_path = temp_dir.path().to_path_buf();
    let file1_path = temp_dir.child("file1");
    std::fs::write(&file1_path, b"abc").unwrap();
    temp_dir.leak();
    std::fs::metadata(&dir_path).unwrap();
    std::fs::metadata(&file1_path).unwrap();
    std::fs::remove_dir_all(&dir_path).unwrap();
}
