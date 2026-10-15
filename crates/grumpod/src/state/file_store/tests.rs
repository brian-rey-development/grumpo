use std::os::unix::fs::{symlink, MetadataExt};

use focus_core::{Domain, Rules};

use super::*;
use focus_core::test_support::now_at;

fn now() -> Now {
    now_at(2026, 10, 12, 9, 0)
}

fn corrupt_files(dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir)
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .filter(|name| name.contains(".corrupt-"))
        .collect();
    names.sort();
    names
}

#[test]
fn open_creates_a_private_directory_and_tightens_an_existing_one() {
    let root = tempfile::tempdir().unwrap();
    let nested = root.path().join("a").join("focus");
    FileStateStore::open(&nested).unwrap();
    assert_eq!(fs::metadata(&nested).unwrap().mode() & 0o777, 0o700);
    fs::set_permissions(&nested, Permissions::from_mode(0o755)).unwrap();
    FileStateStore::open(&nested).unwrap();
    assert_eq!(fs::metadata(&nested).unwrap().mode() & 0o777, 0o700);
}

#[test]
fn a_missing_file_loads_nothing_without_creating_anything() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    assert_eq!(store.load(&now()).unwrap(), None);
    assert!(!store.exists());
}

#[test]
fn save_then_load_round_trips_with_private_mode() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let mut state = Snapshot::fresh(&now());
    state.rules = Rules {
        blocked_sites: [Domain::parse("youtube.com").unwrap()].into(),
        ..Rules::default()
    }
    .validate()
    .unwrap();
    store.save(&state).unwrap();
    assert_eq!(store.load(&now()).unwrap(), Some(state));
    let mode = fs::metadata(dir.path().join("state.json")).unwrap().mode();
    assert_eq!(mode & 0o777, 0o600);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_corrupt_file_is_moved_aside_and_nothing_loads() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::write(dir.path().join("state.json"), "{ not json").unwrap();
    assert_eq!(store.load(&now()).unwrap(), None);
    assert!(!store.exists());
    let kept = corrupt_files(dir.path());
    assert_eq!(kept.len(), 1);
    assert!(kept[0].starts_with("state.json.corrupt-"));
    let content = fs::read_to_string(dir.path().join(&kept[0])).unwrap();
    assert_eq!(content, "{ not json");
}

#[test]
fn repeated_corruption_in_the_same_second_keeps_every_copy() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    for text in ["first", "second"] {
        fs::write(dir.path().join("state.json"), text).unwrap();
        store.load(&now()).unwrap();
    }
    assert_eq!(corrupt_files(dir.path()).len(), 2);
}

#[test]
fn a_newer_schema_is_an_error_and_the_file_stays_where_it_is() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let text = r#"{"schemaVersion":99}"#;
    fs::write(dir.path().join("state.json"), text).unwrap();
    let error = store.load(&now()).unwrap_err();
    assert!(
        matches!(error, StateError::UnsupportedVersion { found: 99, .. }),
        "{error}"
    );
    assert_eq!(corrupt_files(dir.path()), Vec::<String>::new());
    assert_eq!(
        fs::read_to_string(dir.path().join("state.json")).unwrap(),
        text
    );
}

#[test]
fn an_unreadable_file_is_an_error_and_is_not_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::create_dir(dir.path().join("state.json")).unwrap();
    let error = store.load(&now()).unwrap_err();
    assert!(matches!(error, StateError::Io { .. }), "{error}");
    assert_eq!(corrupt_files(dir.path()), Vec::<String>::new());
    assert!(dir.path().join("state.json").is_dir());
}

#[test]
fn a_file_with_no_readable_version_is_set_aside() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::write(dir.path().join("state.json"), r#"{"rules":{}}"#).unwrap();
    assert_eq!(store.load(&now()).unwrap(), None);
    assert_eq!(corrupt_files(dir.path()).len(), 1);
}

#[test]
fn rules_that_no_longer_validate_are_set_aside_and_nothing_loads() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let mut document =
        serde_json::to_value(PersistedState::from(&Snapshot::fresh(&now()))).unwrap();
    document["rules"]["limits"] = serde_json::json!([
        { "target": { "kind": "site", "domain": "x.com" }, "dailyMinutes": 0 }
    ]);
    fs::write(dir.path().join("state.json"), document.to_string()).unwrap();
    assert_eq!(store.load(&now()).unwrap(), None);
    assert_eq!(corrupt_files(dir.path()).len(), 1);
}

#[test]
fn the_summary_never_quotes_the_state_file() {
    let invalid = migrate(serde_json::json!({
        "schemaVersion": 1,
        "rules": {
            "blockedSites": [], "blockedApps": [], "schedule": { "windows": [] },
            "limits": [{ "target": { "kind": "site", "domain": "secret-site.com" }, "dailyMinutes": 0 }]
        },
        "unlock": { "phase": { "kind": "locked" }, "unlocksToday": 0, "day": "2026-10-12" },
        "exhausted": { "day": "2026-10-12", "targets": [] }
    }))
    .unwrap_err();
    assert!(invalid.to_string().contains("secret-site.com"));
    assert!(!invalid.summary().contains("secret-site.com"));
    let unknown = migrate(serde_json::json!({ "schemaVersion": 1, "secret-field": 1 }));
    assert!(!unknown.unwrap_err().summary().contains("secret-field"));
}

fn corrupt_names_taken(dir: &Path, unix: i64) {
    fs::write(dir.join(format!("state.json.corrupt-{unix}")), "old").unwrap();
    for n in 1..MAX_QUARANTINE_NAMES {
        fs::write(dir.join(format!("state.json.corrupt-{unix}-{n}")), "old").unwrap();
    }
}

#[test]
fn a_full_quarantine_still_keeps_one_copy_in_the_last_slot() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    corrupt_names_taken(dir.path(), now().utc().timestamp());
    let slot = dir.path().join("state.json.corrupt-last");

    for text in ["{ first", "{ second"] {
        fs::write(dir.path().join("state.json"), text).unwrap();
        assert_eq!(store.load(&now()).unwrap(), None);
        assert_eq!(fs::read_to_string(&slot).unwrap(), text);
    }

    assert_eq!(fs::metadata(&slot).unwrap().mode() & 0o777, 0o600);
}

#[test]
fn a_failed_rename_still_keeps_one_copy_in_the_last_slot() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::write(dir.path().join("state.json"), "{ broken").unwrap();

    store.set_aside_with(&now(), |_, _| Err(io::Error::other("rename refused")));

    let slot = dir.path().join("state.json.corrupt-last");
    assert_eq!(fs::read_to_string(slot).unwrap(), "{ broken");
    assert_eq!(corrupt_files(dir.path()), ["state.json.corrupt-last"]);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 2);
}

#[test]
fn a_successful_set_aside_does_not_touch_the_last_slot() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::write(dir.path().join("state.json"), "{ broken").unwrap();
    store.load(&now()).unwrap();
    assert!(!dir.path().join("state.json.corrupt-last").exists());
}

#[test]
fn a_symlinked_state_directory_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let real = root.path().join("real");
    fs::create_dir(&real).unwrap();
    let link = root.path().join("link");
    symlink(&real, &link).unwrap();

    let error = FileStateStore::open(&link).unwrap_err();

    assert!(matches!(error, StateError::Symlink(_)), "{error}");
    assert_eq!(fs::read_dir(&real).unwrap().count(), 0);
}

#[test]
fn a_dangling_symlink_in_place_of_the_directory_is_refused() {
    let root = tempfile::tempdir().unwrap();
    let link = root.path().join("link");
    symlink(root.path().join("nowhere"), &link).unwrap();

    let error = FileStateStore::open(&link).unwrap_err();

    assert!(matches!(error, StateError::Symlink(_)), "{error}");
    assert!(!root.path().join("nowhere").exists());
}

#[test]
fn a_symlinked_state_file_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let elsewhere = dir.path().join("elsewhere");
    fs::write(&elsewhere, "{}").unwrap();
    symlink(&elsewhere, dir.path().join("state.json")).unwrap();

    let error = FileStateStore::open(dir.path()).unwrap_err();

    assert!(matches!(error, StateError::Symlink(_)), "{error}");
}

/// The test cannot create a directory owned by another user without root, so the uid the
/// daemon expects is the one that differs.
#[test]
fn a_directory_owned_by_someone_else_is_refused_and_left_as_it_is() {
    let dir = tempfile::tempdir().unwrap();
    fs::set_permissions(dir.path(), Permissions::from_mode(0o755)).unwrap();
    let owner = fs::metadata(dir.path()).unwrap().uid();

    let error = FileStateStore::open_as(dir.path(), owner + 1).unwrap_err();

    assert!(
        matches!(error, StateError::ForeignOwner { found, expected, .. } if found == owner && expected == owner + 1),
        "{error}"
    );
    assert_eq!(fs::metadata(dir.path()).unwrap().mode() & 0o777, 0o755);
    assert!(FileStateStore::open_as(dir.path(), owner).is_ok());
}

#[test]
fn open_uses_the_effective_uid_of_the_process() {
    let dir = tempfile::tempdir().unwrap();
    assert!(FileStateStore::open(dir.path()).is_ok());
}

#[test]
fn stale_temporary_files_of_a_save_are_cleared_without_touching_the_state() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    store.save(&Snapshot::fresh(&now())).unwrap();
    fs::write(dir.path().join(".state.json.31337.2.tmp"), "half").unwrap();

    store.remove_stale_temps();

    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
    assert!(store.load(&now()).unwrap().is_some());
}
