use std::fs;
use std::os::unix::fs::MetadataExt;

use super::*;

#[test]
fn a_second_holder_is_refused_until_the_first_lets_go() {
    let dir = tempfile::tempdir().unwrap();
    let first = InstanceLock::acquire(dir.path()).unwrap();
    let error = InstanceLock::acquire(dir.path()).unwrap_err();
    assert!(matches!(error, LockError::Held(_)), "{error}");
    drop(first);
    assert!(InstanceLock::acquire(dir.path()).is_ok());
}

#[test]
fn the_lock_file_is_private_and_keeps_existing_content() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join(LOCK_FILE), "kept").unwrap();
    let _lock = InstanceLock::acquire(dir.path()).unwrap();
    let metadata = fs::metadata(dir.path().join(LOCK_FILE)).unwrap();
    assert_eq!(metadata.len(), 4);
    let fresh = tempfile::tempdir().unwrap();
    let _other = InstanceLock::acquire(fresh.path()).unwrap();
    let mode = fs::metadata(fresh.path().join(LOCK_FILE)).unwrap().mode();
    assert_eq!(mode & 0o777, LOCK_MODE);
}

#[test]
fn a_missing_directory_is_an_error_unless_the_caller_tolerates_it() {
    let dir = tempfile::tempdir().unwrap();
    let missing = dir.path().join("none");
    assert!(matches!(
        InstanceLock::acquire(&missing),
        Err(LockError::Io { .. })
    ));
    assert!(InstanceLock::acquire_if_initialized(&missing)
        .unwrap()
        .is_none());
    assert!(!missing.exists());
}

#[test]
fn a_held_lock_is_still_reported_by_the_tolerant_variant() {
    let dir = tempfile::tempdir().unwrap();
    let _held = InstanceLock::acquire(dir.path()).unwrap();
    assert!(matches!(
        InstanceLock::acquire_if_initialized(dir.path()),
        Err(LockError::Held(_))
    ));
}
