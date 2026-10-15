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
fn replaces_existing_content_and_keeps_the_new_mode() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    fs::write(&target, "old").unwrap();
    write_atomic(&target, b"new", 0o644, None).unwrap();
    assert_eq!(fs::read(&target).unwrap(), b"new");
    assert_eq!(fs::metadata(&target).unwrap().mode() & 0o777, 0o644);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_file_already_at_the_temp_name_is_left_alone() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    let squatter = dir.path().join(".data.tmp");
    fs::write(&squatter, "someone else's").unwrap();
    write_atomic(&target, b"new", 0o600, None).unwrap();
    assert_eq!(fs::read(&squatter).unwrap(), b"someone else's");
    assert_eq!(fs::read(&target).unwrap(), b"new");
}

#[test]
fn a_symlink_at_a_candidate_temp_name_is_never_followed() {
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim");
    fs::write(&victim, "keep me").unwrap();
    let link = dir.path().join(unique_temp_name("data"));
    std::os::unix::fs::symlink(&victim, &link).unwrap();
    assert!(open_exclusive(&link, 0o600).is_err());
    assert_eq!(fs::read(&victim).unwrap(), b"keep me");
}

#[test]
fn temp_names_differ_on_every_call() {
    assert_ne!(unique_temp_name("data"), unique_temp_name("data"));
}

#[test]
fn concurrent_writes_to_one_path_never_corrupt_it() {
    const WRITERS: usize = 8;
    const ROUNDS: usize = 25;
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    let payloads: Vec<Vec<u8>> = (0..WRITERS)
        .map(|index| format!("writer-{index}:").repeat(4096).into_bytes())
        .collect();
    std::thread::scope(|scope| {
        for payload in &payloads {
            let target = &target;
            scope.spawn(move || {
                for _ in 0..ROUNDS {
                    write_atomic(target, payload, 0o600, None).unwrap();
                }
            });
        }
        scope.spawn(|| {
            for _ in 0..ROUNDS {
                if let Ok(seen) = fs::read(&target) {
                    assert!(payloads.contains(&seen), "a reader saw a torn file");
                }
            }
        });
    });
    assert!(payloads.contains(&fs::read(&target).unwrap()));
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
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

#[test]
fn a_failed_fill_leaves_no_temp_file_behind() {
    let dir = tempfile::tempdir().unwrap();
    let mine = Owner::of(dir.path()).unwrap();
    if mine.uid == 0 {
        return;
    }
    let target = dir.path().join("data");
    let someone_else = Owner {
        uid: mine.uid + 1,
        gid: mine.gid,
    };
    assert!(write_atomic(&target, b"x", 0o600, Some(someone_else)).is_err());
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 0);
}

#[test]
fn a_failed_rename_of_an_existing_target_leaves_it_and_no_temp_file() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    fs::create_dir(&target).unwrap();
    fs::write(target.join("inner"), "kept").unwrap();
    assert!(write_atomic(&target, b"x", 0o600, None).is_err());
    assert_eq!(fs::read(target.join("inner")).unwrap(), b"kept");
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn stale_temp_files_of_this_target_are_removed_and_nothing_else() {
    let dir = tempfile::tempdir().unwrap();
    let target = dir.path().join("data");
    let stale = [".data.tmp", ".data.123.0.tmp", ".data.99999.42.tmp"];
    let keep = [
        "data",
        ".data.tmp.bak",
        ".data.abc.1.tmp",
        ".data.1.tmp",
        ".other.123.0.tmp",
        "data.123.0.tmp",
    ];
    for name in stale.iter().chain(&keep) {
        fs::write(dir.path().join(name), "x").unwrap();
    }
    assert_eq!(remove_stale_temps(&target), stale.len());
    let mut left: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    left.sort();
    let mut expected: Vec<String> = keep.iter().map(ToString::to_string).collect();
    expected.sort();
    assert_eq!(left, expected);
}

#[test]
fn removing_stale_temps_in_a_missing_directory_is_a_no_op() {
    let dir = tempfile::tempdir().unwrap();
    assert_eq!(remove_stale_temps(&dir.path().join("gone").join("data")), 0);
}
