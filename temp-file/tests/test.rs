#![forbid(unsafe_code)]
use core::sync::atomic::Ordering;
use safe_lock::SafeLock;
use std::collections::HashSet;
use std::io::ErrorKind;
use std::path::Path;
use temp_dir::TempDir;
use temp_file::{TempFile, TempFileBuilder, INTERNAL_COUNTER};

// The error tests require all tests to run single-threaded.
static LOCK: SafeLock = SafeLock::new();

fn get_file_len(temp_file: &TempFile) -> Result<u64, std::io::Error> {
    let path = temp_file.path();
    let metadata = std::fs::metadata(path)?;
    if metadata.is_file() {
        Ok(metadata.len())
    } else {
        Err(std::io::Error::new(
            ErrorKind::InvalidInput,
            format!("{path:?} is not a file"),
        ))
    }
}

fn path_exists(path: &Path) -> bool {
    !matches!(std::fs::metadata(path), Err(e) if e.kind() == ErrorKind::NotFound)
}

#[test]
fn empty() {
    let _guard = LOCK.lock();
    let temp_file = temp_file::empty();
    assert_eq!(0, get_file_len(&temp_file).unwrap());
    std::fs::write(temp_file.path(), b"abc").unwrap();
    assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
    let temp_file2 = temp_file::empty();
    assert_ne!(temp_file.path(), temp_file2.path());
}

#[test]
fn empty_error() {
    let _guard = LOCK.lock();
    let previous_counter_value = INTERNAL_COUNTER.load(Ordering::SeqCst);
    let _temp_file = temp_file::empty();
    INTERNAL_COUNTER.store(previous_counter_value, Ordering::SeqCst);
    let any = std::panic::catch_unwind(temp_file::empty).unwrap_err();
    let msg = any.downcast_ref::<String>().unwrap();
    assert!(
        msg.contains("error creating file"),
        "unexpected error {msg:?}",
    );
    assert!(msg.contains("AlreadyExists"), "unexpected error {msg:?}");
}

#[test]
fn with_contents() {
    let _guard = LOCK.lock();
    let temp_file = temp_file::with_contents(b"abc");
    assert_eq!(3, get_file_len(&temp_file).unwrap());
    assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
    std::fs::write(temp_file.path(), b"def").unwrap();
    assert_eq!("def", std::fs::read_to_string(temp_file.path()).unwrap());
}

#[test]
fn builder_empty() {
    let _guard = LOCK.lock();
    let temp_file = TempFileBuilder::new().build().unwrap();
    assert!(path_exists(temp_file.path()));
}

#[test]
fn builder_all_options() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::with_prefix("dir1").unwrap();
    let temp_file = TempFileBuilder::new()
        .in_dir(temp_dir.path())
        .prefix("prefix1")
        .suffix("suffix1")
        .build()
        .unwrap();
    assert!(path_exists(temp_file.path()));
    assert_eq!(
        Some(temp_dir.path()),
        temp_file.path().parent(),
        "{temp_file:?}",
    );
    let filename = temp_file.path().file_name().unwrap();
    assert!(
        filename.to_str().unwrap().starts_with("prefix1"),
        "{temp_file:?}",
    );
    assert!(
        filename.to_str().unwrap().ends_with("suffix1"),
        "{temp_file:?}",
    );
}

#[test]
fn temp_file_new() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    println!("{:?}", temp_file.path());
    println!("{:?}", TempFile::new().unwrap().path());
    assert_eq!(0, get_file_len(&temp_file).unwrap());
    std::fs::write(temp_file.path(), b"abc").unwrap();
    assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
    let temp_file2 = TempFile::new().unwrap();
    assert_ne!(temp_file.path(), temp_file2.path());
}

#[test]
fn temp_file_new_error() {
    let _guard = LOCK.lock();
    let previous_counter_value = INTERNAL_COUNTER.load(Ordering::SeqCst);
    let temp_file = TempFile::new().unwrap();
    INTERNAL_COUNTER.store(previous_counter_value, Ordering::SeqCst);
    let e = TempFile::new().unwrap_err();
    assert_eq!(std::io::ErrorKind::AlreadyExists, e.kind());
    assert!(
        e.to_string()
            .starts_with(&format!("error creating file {:?}", temp_file.path())),
        "unexpected error {e:?}",
    );
}

#[test]
fn temp_file_in_dir() {
    let _guard = LOCK.lock();
    let temp_dir = TempDir::with_prefix("dir1").unwrap();
    let temp_file = TempFile::in_dir(temp_dir.path()).unwrap();
    assert_eq!(
        Some(temp_dir.path()),
        temp_file.path().parent(),
        "{temp_file:?}",
    );
}

#[test]
fn temp_file_with_prefix() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::with_prefix("prefix1").unwrap();
    let filename = temp_file.path().file_name().unwrap();
    assert!(
        filename.to_str().unwrap().starts_with("prefix1"),
        "{temp_file:?}",
    );
}

#[test]
fn temp_file_with_suffix() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::with_suffix("suffix1").unwrap();
    let filename = temp_file.path().file_name().unwrap();
    assert!(
        filename.to_str().unwrap().ends_with("suffix1"),
        "{temp_file:?}",
    );
}

#[test]
fn temp_file_with_contents() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap().with_contents(b"abc").unwrap();
    assert_eq!(3, get_file_len(&temp_file).unwrap());
    assert_eq!("abc", std::fs::read_to_string(temp_file.path()).unwrap());
    std::fs::write(temp_file.path(), b"def").unwrap();
    assert_eq!("def", std::fs::read_to_string(temp_file.path()).unwrap());
}

#[test]
fn temp_file_with_contents_error() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    std::fs::remove_file(temp_file.path()).unwrap();
    let temp_file_path = temp_file.path().to_path_buf();
    std::fs::create_dir(&temp_file_path).unwrap();
    let result = temp_file.with_contents(b"abc");
    std::fs::remove_dir(&temp_file_path).unwrap();
    let e = result.unwrap_err();
    assert!(
        e.to_string()
            .starts_with(&format!("error writing file {temp_file_path:?}")),
        "unexpected error {e:?}",
    );
}

#[test]
fn cleanup() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.cleanup().unwrap();
    assert!(!path_exists(&path));
}

#[test]
fn leak_then_cleanup() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    temp_file.cleanup().unwrap();
    assert!(!path_exists(&path));
}

#[test]
fn cleanup_already_deleted() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    std::fs::remove_file(&path).unwrap();
    temp_file.cleanup().unwrap();
    assert!(!path_exists(&path));
}

#[test]
fn cleanup_error() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    std::fs::remove_file(temp_file.path()).unwrap();
    let path = temp_file.path().to_path_buf();
    std::fs::create_dir(&path).unwrap();
    let result = temp_file.cleanup();
    std::fs::remove_dir(&path).unwrap();
    let e = result.unwrap_err();
    assert!(
        e.to_string()
            .starts_with(&format!("error removing file {path:?}")),
        "unexpected error {e:?}",
    );
}

#[test]
fn test_drop() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    let path = temp_file.path().to_path_buf();
    TempFile::new().unwrap();
    drop(temp_file);
    assert!(!path_exists(&path));
}

#[test]
fn drop_already_deleted() {
    let _guard = LOCK.lock();
    let temp_file = TempFile::new().unwrap();
    std::fs::remove_file(temp_file.path()).unwrap();
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
    let f = temp_file::empty();
    let path = f.path().to_path_buf();
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    drop(f);
    std::fs::metadata(&path).unwrap();
    std::fs::remove_dir(&path).unwrap();
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
    let f = temp_file::empty().panic_on_cleanup_error();
    let path = f.path().to_path_buf();
    std::fs::remove_file(&path).unwrap();
    std::fs::create_dir(&path).unwrap();
    let result = std::panic::catch_unwind(move || drop(f));
    std::fs::metadata(&path).unwrap();
    std::fs::remove_dir(&path).unwrap();
    let msg = result.unwrap_err().downcast::<String>().unwrap();
    assert!(
        msg.contains("error removing file "),
        "unexpected panic message {msg:?}",
    );
}

#[test]
fn leak() {
    let _guard = LOCK.lock();
    let f = temp_file::empty();
    let path = f.path().to_path_buf();
    f.leak();
    std::fs::metadata(&path).unwrap();
    std::fs::remove_file(&path).unwrap();
}

#[test]
fn test_derived() {
    let _guard = LOCK.lock();
    INTERNAL_COUNTER.store(100, Ordering::SeqCst);
    let t1 = TempFile::new().unwrap();
    let t2 = TempFile::new().unwrap();
    // Clone
    let t1_clone = t1.clone();
    // Debug
    assert!(format!("{t2:?}").contains("TempFile"));
    // PartialEq
    assert_eq!(t1, t1_clone);
    // Ord
    assert!(t1 < t2);
    // Hash
    let mut set = HashSet::new();
    set.insert(t1);
    set.insert(t1_clone);
    assert_eq!(1, set.len());
}
