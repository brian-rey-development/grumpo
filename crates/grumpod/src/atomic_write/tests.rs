use super::*;

#[test]
fn writes_content_with_the_requested_mode_and_leaves_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    write_atomic(&target, b"hello", 0o600, None).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"hello");
    assert_eq!(fs::metadata(&target).unwrap().mode() & 0o777, 0o600);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn replaces_existing_content_and_a_stale_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    fs::write(&target, "old").unwrap();
    let stale = dir.path().join(".data.tmp");
    fs::write(&stale, "junk").unwrap();
    write_atomic(&target, b"new", 0o644, None).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(fs::metadata(&target).unwrap().mode() & 0o777, 0o644);
    assert!(!stale.exists());
}

#[test]
fn keeps_the_original_when_the_directory_is_missing() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("missing").join("data");
    assert!(write_atomic(&target, b"x", 0o600, None).is_err());
}

#[test]
fn accepts_the_current_owner_without_chown() {
    let dir = tempfile::tempdir().unwrap();
    let owner = Owner::of(dir.path()).unwrap();
    let target = dir.path().join("data");
    write_atomic(&target, b"x", 0o600, Some(owner)).unwrap();
    assert_eq!(Owner::of(&target).unwrap(), owner);
}

#[test]
fn a_failed_directory_sync_after_the_rename_is_not_an_error() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    let outcome = write_atomic_with(&target, b"new", 0o600, None, |_| {
        Err(io::Error::other("fsync failed"))
    });
    assert!(outcome.is_ok());
    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_failed_write_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    fs::create_dir(&target).unwrap();
    assert!(write_atomic(&target, b"x", 0o600, None).is_err());
    assert!(!dir.path().join(".data.tmp").exists());
}
