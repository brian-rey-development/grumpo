use std::os::unix::fs::PermissionsExt;

use super::test_support::{record, store};
use super::*;

fn mode(path: &Path) -> u32 {
    std::fs::metadata(path).unwrap().permissions().mode() & 0o777
}

#[test]
fn open_creates_private_directories_and_file_with_wal() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("nested").join("data").join("usage.db");
    let store = Store::open(&path).unwrap();

    assert_eq!(mode(path.parent().unwrap()), 0o700);
    assert_eq!(mode(path.parent().unwrap().parent().unwrap()), 0o700);
    assert_eq!(mode(&path), 0o600);
    let journal: String = store
        .with_conn(|c| Ok(c.pragma_query_value(None, "journal_mode", |r| r.get(0))?))
        .unwrap();
    assert_eq!(journal, "wal");

    record(&store, "com.a", "A", None, (1, 2));
    assert_eq!(mode(&dir.path().join("nested/data/usage.db-wal")), 0o600);
}

#[test]
fn connections_enforce_foreign_keys_and_a_busy_timeout() {
    let store = store();
    let (foreign_keys, timeout): (i64, i64) = store
        .with_conn(|c| {
            Ok((
                c.pragma_query_value(None, "foreign_keys", |r| r.get(0))?,
                c.pragma_query_value(None, "busy_timeout", |r| r.get(0))?,
            ))
        })
        .unwrap();
    assert_eq!(foreign_keys, 1);
    assert_eq!(timeout, 2_000);
}

#[test]
fn deleted_rows_are_overwritten_because_secure_delete_is_on() {
    let dir = tempfile::tempdir().unwrap();
    for store in [store(), Store::open(&dir.path().join("usage.db")).unwrap()] {
        let secure_delete: i64 = store
            .with_conn(|c| Ok(c.pragma_query_value(None, "secure_delete", |r| r.get(0))?))
            .unwrap();
        assert_eq!(secure_delete, 1);
    }
}

#[test]
fn data_survives_reopening() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usage.db");
    record(&Store::open(&path).unwrap(), "com.a", "A", None, (10, 20));

    let reopened = Store::open(&path).unwrap();
    let count: i64 = reopened
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(count, 1);
}

#[test]
fn a_database_from_a_newer_app_is_refused() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usage.db");
    let future = migrations::latest_version() + 1;
    Store::open(&path)
        .unwrap()
        .with_conn(|c| Ok(c.pragma_update(None, "user_version", future)?))
        .unwrap();

    let error = Store::open(&path).unwrap_err();
    assert!(matches!(error, StorageError::SchemaTooNew { .. }));
}

#[test]
fn a_poisoned_lock_is_recovered() {
    let store = std::sync::Arc::new(store());
    let clone = std::sync::Arc::clone(&store);
    let panicked = std::thread::spawn(move || {
        let _ = clone.with_conn(|_| -> Result<(), StorageError> { panic!("boom") });
    })
    .join();
    assert!(panicked.is_err());
    record(&store, "com.a", "A", None, (1, 2));
}
