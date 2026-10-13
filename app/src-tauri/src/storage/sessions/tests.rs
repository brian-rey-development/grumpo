use super::*;
use crate::storage::test_support::store;
use focus_core::test_support::{named_app, site};

fn row(store: &Store, id: SessionId) -> (i64, i64, String, String, Option<String>) {
    store
        .with_conn(|c| {
            Ok(c.query_row(
                "SELECT started_at, ended_at, app_id, app_name, domain FROM sessions WHERE id = ?1",
                [id.get()],
                |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?)),
            )?)
        })
        .unwrap()
}

#[test]
fn opening_stores_a_zero_length_session() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.google.Chrome", "Google Chrome"),
            domain: Some(site("https://www.YouTube.com/watch?v=x")),
            started_at: 1_000,
        })
        .unwrap();
    assert_eq!(
        row(&store, id),
        (
            1_000,
            1_000,
            "com.google.Chrome".into(),
            "Google Chrome".into(),
            Some("youtube.com".into())
        )
    );
}

#[test]
fn sessions_without_a_domain_store_null() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.apple.dt.Xcode", "Xcode"),
            domain: None,
            started_at: 5,
        })
        .unwrap();
    assert_eq!(row(&store, id).4, None);
}

#[test]
fn extending_moves_the_end_forward_only() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.a", "A"),
            domain: None,
            started_at: 100,
        })
        .unwrap();
    store.extend_session(id, 130).unwrap();
    assert_eq!(row(&store, id).1, 130);
    store.extend_session(id, 120).unwrap();
    assert_eq!(row(&store, id).1, 130);
}

#[test]
fn extending_a_missing_session_is_a_typed_error() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.a", "A"),
            domain: None,
            started_at: 1,
        })
        .unwrap();
    store.delete_all(0, false).unwrap();
    assert!(matches!(
        store.extend_session(id, 5),
        Err(StorageError::SessionNotFound(found)) if found == id.get()
    ));
}

#[test]
fn ending_sets_the_end_exactly_even_backwards() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.a", "A"),
            domain: None,
            started_at: 100,
        })
        .unwrap();
    store.extend_session(id, 300).unwrap();
    store.end_session(id, 180).unwrap();
    assert_eq!(row(&store, id).1, 180);
    store.end_session(id, 50).unwrap();
    assert_eq!(row(&store, id).1, 100, "never before the start");
}

#[test]
fn ending_a_missing_session_is_a_typed_error() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.a", "A"),
            domain: None,
            started_at: 1,
        })
        .unwrap();
    store.delete_all(0, false).unwrap();
    assert!(matches!(
        store.end_session(id, 5),
        Err(StorageError::SessionNotFound(_))
    ));
}

#[test]
fn hostile_text_is_stored_verbatim_not_executed() {
    let store = store();
    let id = store
        .open_session(&NewSession {
            app: named_app("com.a", "x'); DROP TABLE sessions;--"),
            domain: None,
            started_at: 1,
        })
        .unwrap();
    assert_eq!(row(&store, id).3, "x'); DROP TABLE sessions;--");
}
