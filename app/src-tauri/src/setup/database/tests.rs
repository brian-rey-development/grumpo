use std::fs;

use rusqlite::Connection;

use super::test_support::RecordingDialogs;
use super::*;
use crate::notify::test_support::RecordingNotifier;
use crate::storage::test_support::record;

const NOW: i64 = 1_800_000_000;

#[test]
fn a_healthy_database_opens_quietly() {
    let dir = tempfile::tempdir().unwrap();
    let (dialogs, notifier) = (RecordingDialogs::default(), RecordingNotifier::default());

    let store = open_database(&dir.path().join("focus.db"), &dialogs, &notifier, NOW);

    assert!(store.is_ok());
    assert_eq!(dialogs.alerts(), []);
    assert_eq!(notifier.notices(), []);
}

#[test]
fn a_corrupt_database_is_quarantined_and_the_app_starts_fresh_with_one_notice() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focus.db");
    fs::write(&path, b"definitely not a sqlite database, just text").unwrap();
    let (dialogs, notifier) = (RecordingDialogs::default(), RecordingNotifier::default());

    let store = open_database(&path, &dialogs, &notifier, NOW).unwrap();

    record(&store, "com.a", "A", None, (1, 2));
    assert!(dir.path().join(format!("focus.db.corrupt-{NOW}")).exists());
    assert_eq!(
        notifier.notices(),
        [Notice {
            title: RESET_TITLE.to_owned(),
            body: RESET_BODY.to_owned()
        }]
    );
    assert!(dialogs.alerts().is_empty(), "the app keeps running");
}

#[test]
fn a_newer_schema_shows_the_update_dialog_and_leaves_the_file_alone() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("focus.db");
    Connection::open(&path)
        .unwrap()
        .pragma_update(None, "user_version", 99)
        .unwrap();
    let (dialogs, notifier) = (RecordingDialogs::default(), RecordingNotifier::default());

    let error = open_database(&path, &dialogs, &notifier, NOW).unwrap_err();

    assert!(matches!(
        error,
        StorageError::SchemaTooNew { found: 99, .. }
    ));
    assert_eq!(
        dialogs.alerts(),
        [(NEWER_TITLE.to_owned(), NEWER_MESSAGE.to_owned())]
    );
    assert_eq!(notifier.notices(), []);
    assert!(path.exists());
    assert!(!dir.path().join(format!("focus.db.corrupt-{NOW}")).exists());
}

#[test]
fn another_failure_shows_the_generic_dialog() {
    let dir = tempfile::tempdir().unwrap();
    let blocker = dir.path().join("blocker");
    fs::write(&blocker, b"a file where a folder is needed").unwrap();
    let (dialogs, notifier) = (RecordingDialogs::default(), RecordingNotifier::default());

    let result = open_database(&blocker.join("focus.db"), &dialogs, &notifier, NOW);

    assert!(result.is_err());
    assert_eq!(
        dialogs.alerts(),
        [(FAILED_TITLE.to_owned(), FAILED_MESSAGE.to_owned())]
    );
}
