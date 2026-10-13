use std::os::unix::fs::PermissionsExt;

use serde_json::{json, Value};

use crate::storage::retention::RetentionDays;
use crate::storage::test_support::{record, store};

#[test]
fn export_round_trips_sessions_and_settings() {
    let store = store();
    record(
        &store,
        "com.google.Chrome",
        "Chrome",
        Some("youtube.com"),
        (100, 200),
    );
    record(&store, "com.apple.dt.Xcode", "Xcode", None, (300, 400));
    store
        .set_retention_days(RetentionDays::new(60).unwrap(), 0)
        .unwrap();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("export.json");

    store.export_json(&path, 999).unwrap();

    let value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(
        value,
        json!({
            "exportedAt": 999,
            "schemaVersion": 1,
            "settings": { "retention_days": "60" },
            "sessions": [
                { "startedAt": 100, "endedAt": 200, "appId": "com.google.Chrome",
                  "appName": "Chrome", "domain": "youtube.com" },
                { "startedAt": 300, "endedAt": 400, "appId": "com.apple.dt.Xcode",
                  "appName": "Xcode", "domain": null }
            ]
        })
    );
}

#[test]
fn export_is_owner_only_and_leaves_no_temp_file() {
    let store = store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("export.json");
    store.export_json(&path, 1).unwrap();

    let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    let names: Vec<_> = std::fs::read_dir(dir.path()).unwrap().collect();
    assert_eq!(names.len(), 1);
}

#[test]
fn export_replaces_an_existing_file_and_survives_a_stale_temp() {
    let store = store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("export.json");
    std::fs::write(&path, "old").unwrap();
    std::fs::write(dir.path().join("export.json.tmp"), "stale").unwrap();

    store.export_json(&path, 5).unwrap();

    let value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["exportedAt"], 5);
}

#[test]
fn a_failed_export_keeps_the_previous_file() {
    let store = store();
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("missing-dir").join("export.json");
    assert!(store.export_json(&path, 1).is_err());
    assert!(!path.exists());
}
