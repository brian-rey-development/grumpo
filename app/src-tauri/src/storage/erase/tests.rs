use crate::time::test_support::fixed_day;
use focus_core::test_support::site_target;
use serde_json::{json, Value};

use crate::storage::retention::RetentionDays;
use crate::storage::test_support::{record, store};
use crate::storage::Store;

const MIDNIGHT: i64 = 1_000_000;

fn spans(store: &Store) -> Vec<(i64, i64)> {
    store
        .with_conn(|c| {
            let mut s = c.prepare("SELECT started_at, ended_at FROM sessions ORDER BY id")?;
            let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
        .unwrap()
}

#[test]
fn deleting_everything_removes_all_sessions_but_keeps_settings() {
    let store = store();
    record(&store, "com.a", "A", Some("example.com"), (1, 9));
    record(&store, "com.a", "A", None, (MIDNIGHT + 10, MIDNIGHT + 50));
    store
        .set_retention_days(RetentionDays::new(60).unwrap(), 0)
        .unwrap();

    store.delete_all(MIDNIGHT, false).unwrap();

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("after.json");
    store.export_json(&path, 2).unwrap();
    let value: Value = serde_json::from_slice(&std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(value["sessions"], json!([]));
    assert_eq!(value["settings"], json!({ "retention_days": "60" }));
}

#[test]
fn keeping_today_removes_the_past_only() {
    let store = store();
    record(&store, "com.a", "A", None, (MIDNIGHT - 500, MIDNIGHT - 100));
    record(&store, "com.a", "A", None, (MIDNIGHT - 5, MIDNIGHT));
    record(&store, "com.b", "B", None, (MIDNIGHT + 10, MIDNIGHT + 70));

    store.delete_all(MIDNIGHT, true).unwrap();

    assert_eq!(spans(&store), [(MIDNIGHT + 10, MIDNIGHT + 70)]);
}

#[test]
fn keeping_today_cuts_a_session_that_crosses_midnight() {
    let store = store();
    record(&store, "com.a", "A", None, (MIDNIGHT - 600, MIDNIGHT + 300));

    store.delete_all(MIDNIGHT, true).unwrap();

    assert_eq!(spans(&store), [(MIDNIGHT, MIDNIGHT + 300)]);
}

#[test]
fn keeping_today_keeps_the_usage_that_daily_limits_count() {
    let store = store();
    let limit_target = site_target("example.com");
    record(
        &store,
        "com.a",
        "A",
        Some("example.com"),
        (MIDNIGHT + 10, MIDNIGHT + 910),
    );
    let day = fixed_day(
        chrono::NaiveDate::from_ymd_opt(2026, 3, 1).unwrap(),
        MIDNIGHT,
    );

    store.delete_all(MIDNIGHT, true).unwrap();

    let used = store
        .usage_seconds(day, std::slice::from_ref(&limit_target))
        .unwrap();
    assert_eq!(used, [(limit_target, 900)]);
}

#[test]
fn delete_all_works_on_a_file_database_and_leaves_it_usable() {
    let dir = tempfile::tempdir().unwrap();
    let store = Store::open(&dir.path().join("usage.db")).unwrap();
    record(&store, "com.a", "A", None, (1, 9));
    store.delete_all(MIDNIGHT, false).unwrap();
    record(&store, "com.b", "B", None, (2, 3));
    assert_eq!(spans(&store), [(2, 3)]);
}

#[test]
fn deleted_text_is_overwritten_on_disk() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("usage.db");
    let store = Store::open(&path).unwrap();
    record(
        &store,
        "com.secret.marker-app",
        "Secret Marker",
        Some("marker-site.example"),
        (1, 9),
    );
    store.delete_all(MIDNIGHT, false).unwrap();
    drop(store);

    let bytes = std::fs::read(&path).unwrap();
    let leaked = bytes
        .windows(b"marker-site".len())
        .any(|w| w == b"marker-site");
    assert!(!leaked, "deleted domain survived in the database file");
}
