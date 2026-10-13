use std::fs;

use rusqlite::Connection;

use super::*;
use crate::storage::test_support::record;

const NOW: i64 = 1_800_000_000;

fn database_in(dir: &tempfile::TempDir) -> PathBuf {
    dir.path().join("focus.db")
}

fn names(dir: &tempfile::TempDir) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(dir.path())
        .unwrap()
        .map(|entry| entry.unwrap().file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    names
}

#[test]
fn a_healthy_database_opens_untouched() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    record(&Store::open(&path).unwrap(), "com.a", "A", None, (1, 2));

    let opened = open_or_recover(&path, NOW).unwrap();

    let Opened::Ready(store) = opened else {
        panic!("a healthy database must not be replaced");
    };
    assert_eq!(store.total_seconds(crate::time::today()).unwrap(), 0);
    assert!(!names(&dir).iter().any(|name| name.contains("corrupt")));
}

#[test]
fn a_file_that_is_not_a_database_is_kept_aside_and_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    fs::write(
        &path,
        b"this is not a database, only some text that is long enough",
    )
    .unwrap();

    let opened = open_or_recover(&path, NOW).unwrap();

    let Opened::Recovered { store, kept_as } = opened else {
        panic!("a damaged database must be replaced");
    };
    assert_eq!(kept_as, dir.path().join(format!("focus.db.corrupt-{NOW}")));
    assert!(fs::read(&kept_as).unwrap().starts_with(b"this is not"));
    record(&store, "com.a", "A", None, (1, 2));
}

#[test]
fn the_wal_and_shm_files_move_with_the_database_under_the_same_stamp() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    fs::write(&path, b"garbage garbage garbage garbage garbage garbage").unwrap();
    fs::write(dir.path().join("focus.db-wal"), b"wal").unwrap();
    fs::write(dir.path().join("focus.db-shm"), b"shm").unwrap();

    quarantine(&path, NOW).unwrap();

    let kept = names(&dir);
    for expected in [
        format!("focus.db.corrupt-{NOW}"),
        format!("focus.db-wal.corrupt-{NOW}"),
        format!("focus.db-shm.corrupt-{NOW}"),
    ] {
        assert!(kept.contains(&expected), "{expected} in {kept:?}");
    }
}

#[test]
fn two_recoveries_within_one_second_keep_both_files() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    fs::write(&path, b"first damaged database, long enough to be text").unwrap();

    let Opened::Recovered { kept_as: first, .. } = open_or_recover(&path, NOW).unwrap() else {
        panic!("a damaged database must be replaced");
    };
    fs::write(&path, b"second damaged database, long enough to be text").unwrap();
    let Opened::Recovered {
        kept_as: second, ..
    } = open_or_recover(&path, NOW).unwrap()
    else {
        panic!("a damaged database must be replaced");
    };

    assert_ne!(first, second);
    assert!(fs::read(&first).unwrap().starts_with(b"first damaged"));
    assert!(fs::read(&second).unwrap().starts_with(b"second damaged"));
    assert_eq!(second, dir.path().join(format!("focus.db.corrupt-{NOW}-1")));
}

#[test]
fn a_taken_name_on_a_sidecar_alone_moves_the_whole_set_to_the_next_stamp() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    fs::write(&path, b"garbage garbage garbage garbage garbage garbage").unwrap();
    fs::write(dir.path().join("focus.db-wal"), b"wal").unwrap();
    fs::write(
        dir.path().join(format!("focus.db-wal.corrupt-{NOW}")),
        b"old",
    )
    .unwrap();

    let kept_as = quarantine(&path, NOW).unwrap();

    assert_eq!(
        kept_as,
        dir.path().join(format!("focus.db.corrupt-{NOW}-1"))
    );
    assert_eq!(
        fs::read(dir.path().join(format!("focus.db-wal.corrupt-{NOW}"))).unwrap(),
        b"old"
    );
    assert_eq!(
        fs::read(dir.path().join(format!("focus.db-wal.corrupt-{NOW}-1"))).unwrap(),
        b"wal"
    );
}

#[test]
fn a_database_with_damaged_pages_is_replaced_too() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    {
        let store = Store::open(&path).unwrap();
        for index in 0..2_000 {
            let start = index * 10;
            record(
                &store,
                "com.a",
                "A",
                Some("example.com"),
                (start, start + 5),
            );
        }
    }
    let mut bytes = fs::read(&path).unwrap();
    let damage_from = bytes.len() / 2;
    bytes[damage_from..damage_from + 8_192].fill(0xAB);
    fs::write(&path, bytes).unwrap();
    for sidecar in ["focus.db-wal", "focus.db-shm"] {
        let _ = fs::remove_file(dir.path().join(sidecar));
    }

    let opened = open_or_recover(&path, NOW).unwrap();

    assert!(matches!(opened, Opened::Recovered { .. }));
}

#[test]
fn a_newer_schema_is_reported_as_it_is_and_nothing_is_moved() {
    let dir = tempfile::tempdir().unwrap();
    let path = database_in(&dir);
    {
        let conn = Connection::open(&path).unwrap();
        conn.pragma_update(None, "user_version", 99).unwrap();
    }

    let error = open_or_recover(&path, NOW).unwrap_err();

    assert!(matches!(
        error,
        StorageError::SchemaTooNew { found: 99, .. }
    ));
    assert_eq!(names(&dir), ["focus.db"]);
}

#[test]
fn damage_is_told_apart_from_other_failures() {
    assert!(StorageError::Corrupt("page 4".into()).is_damaged());
    assert!(!StorageError::SchemaTooNew {
        found: 2,
        supported: 1
    }
    .is_damaged());
    assert!(!StorageError::SessionNotFound(1).is_damaged());
}
