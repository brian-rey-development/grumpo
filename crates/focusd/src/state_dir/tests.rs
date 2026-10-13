use std::fs;
use std::os::unix::fs::MetadataExt;

use super::*;

#[test]
fn creates_the_directory_privately_and_holds_the_lock() {
    let root = tempfile::tempdir().unwrap();
    let directory = root.path().join("focus");

    let (_store, _lock) = ensure_state_dir(&directory).unwrap();

    assert_eq!(fs::metadata(&directory).unwrap().mode() & 0o777, 0o700);
    assert!(matches!(
        ensure_state_dir(&directory),
        Err(StateDirError::Lock(LockError::Held(_)))
    ));
}

#[test]
fn the_lock_is_released_with_the_returned_value() {
    let root = tempfile::tempdir().unwrap();
    drop(ensure_state_dir(root.path()).unwrap());
    assert!(ensure_state_dir(root.path()).is_ok());
}
