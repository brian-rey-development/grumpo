use super::*;
use crate::storage::sessions::NewSession;
use focus_core::test_support::app;

fn writer() -> (SessionWriter, Arc<Store>) {
    let store = Arc::new(Store::open_in_memory().unwrap());
    (SessionWriter::new(Arc::clone(&store)), store)
}

fn opened(at: i64) -> SessionEvent {
    SessionEvent::Opened(NewSession {
        app: app("com.a"),
        domain: None,
        started_at: at,
    })
}

fn stored(store: &Store) -> Vec<(i64, i64)> {
    store
        .with_conn(|c| {
            let mut s = c.prepare("SELECT started_at, ended_at FROM sessions ORDER BY id")?;
            let rows = s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
        .unwrap()
}

#[test]
fn opening_writes_immediately() {
    let (mut writer, store) = writer();
    assert!(writer.apply(opened(100), 100));
    assert_eq!(stored(&store), [(100, 100)]);
}

#[test]
fn extends_are_batched_until_the_flush_interval() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    for at in [102, 104, 106, 108] {
        assert!(!writer.apply(SessionEvent::Extended { ended_at: at }, at));
    }
    assert_eq!(stored(&store), [(100, 100)]);

    assert!(writer.apply(SessionEvent::Extended { ended_at: 110 }, 110));
    assert_eq!(stored(&store), [(100, 110)]);
    assert!(!writer.apply(SessionEvent::Extended { ended_at: 112 }, 112));
    assert_eq!(stored(&store), [(100, 110)]);
}

#[test]
fn closing_flushes_the_latest_end() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 104 }, 104);
    assert!(writer.apply(SessionEvent::Closed, 104));
    assert_eq!(stored(&store), [(100, 104)]);
    assert!(!writer.apply(SessionEvent::Closed, 106));
}

#[test]
fn an_explicit_flush_writes_pending_and_is_idempotent() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 102 }, 102);
    assert!(writer.flush(102));
    assert!(!writer.flush(102));
    assert_eq!(stored(&store), [(100, 102)]);
}

#[test]
fn a_new_session_flushes_the_previous_one_first() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 102 }, 102);
    writer.apply(opened(102), 102);
    assert_eq!(stored(&store), [(100, 102), (102, 102)]);
}

#[test]
fn trimming_moves_the_end_back_and_survives_the_following_close() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 110 }, 110);
    assert_eq!(stored(&store), [(100, 110)]);

    assert!(writer.apply(SessionEvent::Trimmed { ended_at: 104 }, 112));
    assert_eq!(stored(&store), [(100, 104)]);
    assert!(!writer.apply(SessionEvent::Closed, 112));
    assert_eq!(stored(&store), [(100, 104)]);
}

#[test]
fn trimming_discards_a_pending_end_that_was_never_written() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 106 }, 106);
    writer.apply(SessionEvent::Trimmed { ended_at: 102 }, 108);
    writer.apply(SessionEvent::Closed, 108);
    assert_eq!(stored(&store), [(100, 102)]);
}

#[test]
fn trimming_without_an_open_row_does_nothing() {
    let (mut writer, store) = writer();
    assert!(!writer.apply(SessionEvent::Trimmed { ended_at: 5 }, 5));
    assert_eq!(stored(&store).len(), 0);
}

#[test]
fn deleting_usage_mid_session_keeps_recording_into_a_new_row() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    store.delete_all(0, false).unwrap();

    writer.apply(SessionEvent::Extended { ended_at: 110 }, 110);
    assert_eq!(stored(&store), [(110, 110)]);

    writer.apply(SessionEvent::Extended { ended_at: 130 }, 130);
    writer.apply(SessionEvent::Closed, 130);
    assert_eq!(stored(&store), [(110, 130)]);
}

#[test]
fn the_new_row_describes_the_same_app_and_domain() {
    let (mut writer, store) = writer();
    writer.apply(
        SessionEvent::Opened(NewSession {
            app: app("com.a"),
            domain: Some(focus_core::Domain::parse("example.com").unwrap()),
            started_at: 100,
        }),
        100,
    );
    store.delete_all(0, false).unwrap();
    writer.apply(SessionEvent::Extended { ended_at: 120 }, 120);
    writer.flush(120);

    let identity: (String, Option<String>) = store
        .with_conn(|c| {
            Ok(c.query_row("SELECT app_id, domain FROM sessions", [], |r| {
                Ok((r.get(0)?, r.get(1)?))
            })?)
        })
        .unwrap();
    assert_eq!(
        identity,
        ("com.a".to_owned(), Some("example.com".to_owned()))
    );
}

#[test]
fn closing_after_a_deletion_leaves_no_row_behind() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 104 }, 104);
    store.delete_all(0, false).unwrap();
    assert!(!writer.apply(SessionEvent::Closed, 104));
    assert_eq!(stored(&store).len(), 0);
}

#[test]
fn a_session_row_survives_a_delete_that_keeps_today() {
    let (mut writer, store) = writer();
    writer.apply(opened(100), 100);
    writer.apply(SessionEvent::Extended { ended_at: 110 }, 110);
    store.delete_all(50, true).unwrap();
    writer.apply(SessionEvent::Extended { ended_at: 120 }, 120);
    writer.apply(SessionEvent::Closed, 120);
    assert_eq!(stored(&store), [(100, 120)]);
}
