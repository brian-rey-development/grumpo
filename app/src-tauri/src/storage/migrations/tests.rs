use super::*;

fn table_names(conn: &Connection) -> Vec<String> {
    let mut stmt = conn
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name NOT LIKE 'sqlite_%' ORDER BY name")
        .unwrap();
    stmt.query_map([], |row| row.get(0))
        .unwrap()
        .collect::<Result<_, _>>()
        .unwrap()
}

#[test]
fn a_fresh_database_gets_the_latest_schema() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn).unwrap();
    assert_eq!(current_version(&conn).unwrap(), latest_version());
    assert_eq!(table_names(&conn), ["sessions", "settings"]);
}

#[test]
fn migrating_twice_changes_nothing() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn).unwrap();
    conn.execute(
        "INSERT INTO sessions (started_at, ended_at, app_id, app_name) VALUES (1, 2, 'a.b', 'A')",
        [],
    )
    .unwrap();
    migrate(&mut conn).unwrap();
    let rows: i64 = conn
        .query_row("SELECT COUNT(*) FROM sessions", [], |row| row.get(0))
        .unwrap();
    assert_eq!(rows, 1);
    assert_eq!(current_version(&conn).unwrap(), latest_version());
}

#[test]
fn a_newer_schema_is_rejected_and_left_untouched() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn).unwrap();
    let future = latest_version() + 1;
    conn.pragma_update(None, "user_version", future).unwrap();
    let error = migrate(&mut conn).unwrap_err();
    assert!(matches!(
        error,
        StorageError::SchemaTooNew { found, supported }
            if found == future && supported == latest_version()
    ));
    assert_eq!(current_version(&conn).unwrap(), future);
    assert_eq!(table_names(&conn), ["sessions", "settings"]);
}

#[test]
fn a_failing_migration_rolls_back_completely() {
    let mut conn = Connection::open_in_memory().unwrap();
    let result = apply(
        &mut conn,
        "CREATE TABLE t (x); SELECT * FROM missing_table;",
        1,
    );
    assert!(result.is_err());
    assert_eq!(current_version(&conn).unwrap(), 0);
    assert_eq!(table_names(&conn).len(), 0);
}

#[test]
fn the_schema_rejects_sessions_that_end_before_they_start() {
    let mut conn = Connection::open_in_memory().unwrap();
    migrate(&mut conn).unwrap();
    let result = conn.execute(
        "INSERT INTO sessions (started_at, ended_at, app_id, app_name) VALUES (10, 5, 'a.b', 'A')",
        [],
    );
    assert!(result.is_err());
}
