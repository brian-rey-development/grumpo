use std::cell::Cell;

use super::*;
use crate::launch_at_login::test_support::FakeLaunch;
use crate::storage::test_support::{record, store};

const NOW: i64 = 1_000 * 86_400;
const DAY: i64 = 86_400;

fn wanted(retention_days: u16, launch_at_login: bool) -> Settings {
    Settings {
        retention_days,
        launch_at_login,
        auto_update: true,
    }
}

fn apply_quietly(
    store: &Store,
    launch: &FakeLaunch,
    wanted: Settings,
) -> Result<Settings, AppError> {
    apply(store, launch, wanted, NOW, || {})
}

fn age_sessions(store: &Store) {
    record(
        store,
        "com.a",
        "A",
        None,
        (NOW - 60 * DAY, NOW - 60 * DAY + 10),
    );
    record(store, "com.a", "A", None, (NOW - DAY, NOW - DAY + 10));
}

#[test]
fn defaults_are_ninety_days_no_launch_and_updates_on() {
    let settings = read(&store(), &FakeLaunch::default()).unwrap();
    assert_eq!(settings, wanted(90, false));
    assert!(settings.auto_update);
}

#[test]
fn updates_round_trip() {
    let store = store();
    let launch = FakeLaunch::default();

    let applied = apply_quietly(&store, &launch, wanted(120, true)).unwrap();
    assert_eq!(applied, wanted(120, true));
    assert_eq!(read(&store, &launch).unwrap(), wanted(120, true));

    let applied = apply_quietly(&store, &launch, wanted(120, false)).unwrap();
    assert_eq!(applied, wanted(120, false));
    assert!(!launch.is_enabled().unwrap());
}

#[test]
fn automatic_updates_are_on_by_default_and_can_be_turned_off() {
    let store = store();
    let launch = FakeLaunch::default();
    let off = Settings {
        auto_update: false,
        ..wanted(90, false)
    };

    assert!(read(&store, &launch).unwrap().auto_update);
    assert_eq!(apply_quietly(&store, &launch, off).unwrap(), off);
    assert!(!read(&store, &launch).unwrap().auto_update);
}

#[test]
fn shortening_retention_prunes_and_reports_it() {
    let store = store();
    age_sessions(&store);
    let pruned = Cell::new(0);

    apply(
        &store,
        &FakeLaunch::default(),
        wanted(30, false),
        NOW,
        || {
            pruned.set(pruned.get() + 1);
        },
    )
    .unwrap();

    assert_eq!(pruned.get(), 1);
}

#[test]
fn out_of_range_retention_is_rejected_before_any_change() {
    let store = store();
    let launch = FakeLaunch::default();

    for days in [0, 29, 366, u16::MAX] {
        let error = apply_quietly(&store, &launch, wanted(days, true)).unwrap_err();
        assert!(matches!(error, AppError::InvalidInput(_)), "{days}");
    }
    assert!(!launch.is_enabled().unwrap());
    assert_eq!(store.retention_days().unwrap().get(), 90);
}

#[test]
fn a_refused_launch_change_leaves_the_retention_applied_and_announced() {
    let store = store();
    age_sessions(&store);
    let pruned = Cell::new(false);

    let error = apply(
        &store,
        &FakeLaunch::failing(),
        wanted(30, true),
        NOW,
        || {
            pruned.set(true);
        },
    )
    .unwrap_err();

    assert!(matches!(error, AppError::LaunchAtLogin(_)));
    assert_eq!(store.retention_days().unwrap().get(), 30);
    assert!(pruned.get(), "the pruned history must still reach the UI");
}

#[test]
fn an_unchanged_launch_setting_is_not_touched() {
    let store = store();
    let launch = FakeLaunch::failing();
    assert!(apply_quietly(&store, &launch, wanted(45, false)).is_ok());
    assert_eq!(launch.writes(), 0);
}

#[test]
fn a_changed_launch_setting_is_written_once() {
    let store = store();
    let launch = FakeLaunch::default();

    apply_quietly(&store, &launch, wanted(45, true)).unwrap();
    apply_quietly(&store, &launch, wanted(45, true)).unwrap();

    assert_eq!(launch.writes(), 1);
}
