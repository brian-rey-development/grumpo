use std::os::unix::fs::MetadataExt;

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
fn a_missing_file_loads_defaults_without_creating_anything() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    assert_eq!(store.load(&now()).unwrap(), Snapshot::fresh(&now()));
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
    assert_eq!(store.load(&now()).unwrap(), state);
    let mode = fs::metadata(dir.path().join("state.json")).unwrap().mode();
    assert_eq!(mode & 0o777, 0o600);
    assert_eq!(fs::read_dir(dir.path()).unwrap().count(), 1);
}

#[test]
fn a_corrupt_file_is_moved_aside_and_defaults_load() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    fs::write(dir.path().join("state.json"), "{ not json").unwrap();
    assert_eq!(store.load(&now()).unwrap(), Snapshot::fresh(&now()));
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
    assert_eq!(store.load(&now()).unwrap(), Snapshot::fresh(&now()));
    assert_eq!(corrupt_files(dir.path()).len(), 1);
}

#[test]
fn rules_that_no_longer_validate_are_set_aside_and_defaults_load() {
    let dir = tempfile::tempdir().unwrap();
    let store = FileStateStore::open(dir.path()).unwrap();
    let mut document =
        serde_json::to_value(PersistedState::from(&Snapshot::fresh(&now()))).unwrap();
    document["rules"]["limits"] = serde_json::json!([
        { "target": { "kind": "site", "domain": "x.com" }, "dailyMinutes": 0 }
    ]);
    fs::write(dir.path().join("state.json"), document.to_string()).unwrap();
    assert_eq!(store.load(&now()).unwrap(), Snapshot::fresh(&now()));
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
