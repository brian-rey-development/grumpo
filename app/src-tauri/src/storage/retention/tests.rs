use super::*;
use crate::storage::test_support::{record, store};

const NOW: i64 = 400 * SECONDS_PER_DAY;

#[test]
fn days_are_limited_to_30_through_365() {
    for valid in [30, 90, 365] {
        assert_eq!(RetentionDays::new(valid).unwrap().get(), valid);
    }
    for invalid in [0, 29, 366, u16::MAX] {
        assert_eq!(
            RetentionDays::new(invalid),
            Err(RetentionOutOfRange(invalid))
        );
    }
    assert_eq!(RetentionDays::default().get(), 90);
}

#[test]
fn the_setting_defaults_then_round_trips() {
    let store = store();
    assert_eq!(store.retention_days().unwrap(), RetentionDays::default());
    store
        .set_retention_days(RetentionDays::new(45).unwrap(), NOW)
        .unwrap();
    assert_eq!(store.retention_days().unwrap().get(), 45);
}

#[test]
fn a_corrupt_setting_falls_back_to_the_default() {
    let store = store();
    for bad in ["abc", "5", "99999", "-3", ""] {
        store.set_setting(SETTING_KEY, bad).unwrap();
        assert_eq!(
            store.retention_days().unwrap(),
            RetentionDays::default(),
            "{bad:?}"
        );
    }
}

#[test]
fn prune_deletes_only_sessions_that_ended_before_the_cutoff() {
    let store = store();
    let cutoff = NOW - 90 * SECONDS_PER_DAY;
    record(&store, "com.old", "Old", None, (cutoff - 100, cutoff - 1));
    record(&store, "com.edge", "Edge", None, (cutoff - 100, cutoff));
    record(&store, "com.new", "New", None, (NOW - 10, NOW));

    assert_eq!(store.prune(NOW).unwrap(), 1);
    assert_eq!(store.prune(NOW).unwrap(), 0);
    let left: Vec<String> = store
        .with_conn(|c| {
            let mut s = c.prepare("SELECT app_id FROM sessions ORDER BY id")?;
            let rows = s.query_map([], |r| r.get(0))?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
        .unwrap();
    assert_eq!(left, ["com.edge", "com.new"]);
}

#[test]
fn shortening_the_retention_prunes_immediately() {
    let store = store();
    let forty_days_ago = NOW - 40 * SECONDS_PER_DAY;
    record(
        &store,
        "com.a",
        "A",
        None,
        (forty_days_ago, forty_days_ago + 5),
    );
    record(&store, "com.b", "B", None, (NOW - 10, NOW));
    assert_eq!(store.prune(NOW).unwrap(), 0);

    let deleted = store
        .set_retention_days(RetentionDays::new(30).unwrap(), NOW)
        .unwrap();

    assert_eq!(deleted, 1);
    let left: i64 = store
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(left, 1);
}

#[test]
fn lengthening_the_retention_deletes_nothing() {
    let store = store();
    let forty_days_ago = NOW - 40 * SECONDS_PER_DAY;
    record(
        &store,
        "com.a",
        "A",
        None,
        (forty_days_ago, forty_days_ago + 5),
    );
    let deleted = store
        .set_retention_days(RetentionDays::new(365).unwrap(), NOW)
        .unwrap();
    assert_eq!(deleted, 0);
}

#[cfg(feature = "ts")]
mod bindings {
    use focus_core::bindings::{export, render, Constant};

    use super::super::{DEFAULT_RETENTION_DAYS, MAX_RETENTION_DAYS, MIN_RETENTION_DAYS};

    const FILE: &str = "retention.ts";

    fn constants() -> Vec<Constant> {
        vec![
            Constant::number(
                "MIN_RETENTION_DAYS",
                "Fewest days of usage history the app keeps.",
                MIN_RETENTION_DAYS,
            ),
            Constant::number(
                "MAX_RETENTION_DAYS",
                "Most days of usage history the app keeps.",
                MAX_RETENTION_DAYS,
            ),
            Constant::number(
                "DEFAULT_RETENTION_DAYS",
                "Days of usage history kept until the person chooses otherwise.",
                DEFAULT_RETENTION_DAYS,
            ),
        ]
    }

    #[test]
    fn the_retention_bounds_follow_the_rust_values() {
        let text = render(&constants());
        assert!(text.contains("export const MIN_RETENTION_DAYS = 30;"));
        assert!(text.contains("export const MAX_RETENTION_DAYS = 365;"));
        assert!(text.contains("export const DEFAULT_RETENTION_DAYS = 90;"));
    }

    #[test]
    fn the_retention_bounds_are_written_for_the_interface() {
        export(FILE, &constants()).unwrap();
    }
}
