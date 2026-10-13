use super::*;

#[test]
fn an_untouched_log_is_not_rotated() {
    let dir = tempfile::tempdir().unwrap();
    let log = RotatingLog::open(dir.path().join("focusd.log")).unwrap();
    assert!(!log.is_rotated());
}

#[test]
fn a_renamed_log_is_detected_even_when_a_new_file_takes_its_place() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focusd.log");
    let log = RotatingLog::open(path.clone()).unwrap();
    fs::rename(&path, dir.path().join("focusd.log.0")).unwrap();
    assert!(log.is_rotated());
    fs::write(&path, "").unwrap();
    assert!(log.is_rotated());
}

#[test]
fn a_deleted_log_is_detected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focusd.log");
    let log = RotatingLog::open(path.clone()).unwrap();
    fs::remove_file(&path).unwrap();
    assert!(log.is_rotated());
}

#[test]
fn a_missing_directory_is_an_error() {
    let dir = tempfile::tempdir().unwrap();
    assert!(RotatingLog::open(dir.path().join("none").join("focusd.log")).is_err());
}

fn same_file(a: &File, b: &File) -> bool {
    let (a, b) = (a.metadata().unwrap(), b.metadata().unwrap());
    (a.dev(), a.ino()) == (b.dev(), b.ino())
}

#[test]
fn an_untouched_log_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let mut log = RotatingLog::open(dir.path().join("focusd.log")).unwrap();
    let reattached = log.reattach_with(|_| panic!("no redirect expected"));
    assert!(!reattached.unwrap());
}

#[test]
fn a_rotated_log_is_recreated_and_the_new_file_is_what_gets_redirected() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focusd.log");
    let mut log = RotatingLog::open(path.clone()).unwrap();
    fs::rename(&path, dir.path().join("focusd.log.0")).unwrap();

    let mut redirected = None;
    let reattached = log.reattach_with(|file| {
        redirected = Some(file.try_clone()?);
        Ok(())
    });

    assert!(reattached.unwrap());
    assert!(!log.is_rotated());
    assert!(same_file(&redirected.unwrap(), &File::open(&path).unwrap()));
    assert_eq!(fs::metadata(&path).unwrap().mode() & 0o777, LOG_MODE);
}

#[test]
fn a_failed_redirect_is_reported_and_retried_on_the_next_call() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focusd.log");
    let mut log = RotatingLog::open(path.clone()).unwrap();
    fs::remove_file(&path).unwrap();

    let failed = log.reattach_with(|_| Err(io::Error::other("dup2 failed")));
    assert!(failed.is_err());
    assert!(log.is_rotated());

    assert!(log.reattach_with(|_| Ok(())).unwrap());
    assert!(!log.is_rotated());
}

#[test]
fn a_log_that_cannot_be_recreated_stays_rotated_and_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let logs = dir.path().join("logs");
    fs::create_dir(&logs).unwrap();
    let mut log = RotatingLog::open(logs.join("focusd.log")).unwrap();
    fs::remove_dir_all(&logs).unwrap();
    let result = log.reattach_with(|_| panic!("no redirect without a new file"));
    assert!(result.is_err());
    assert!(log.is_rotated());
}
