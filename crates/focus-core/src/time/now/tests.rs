use chrono::TimeZone;

use super::*;
use crate::test_support::local;

fn at(hour: u32, minute: u32) -> NaiveDateTime {
    local(2026, 10, 12, hour, minute)
}

#[test]
fn exposes_both_views_of_the_instant() {
    let utc = Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap();
    let now = Now::new(utc, at(9, 0));
    assert_eq!(now.utc(), utc);
    assert_eq!(now.local(), at(9, 0));
}

#[test]
fn local_to_utc_applies_the_current_offset() {
    let utc = Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap();
    let now = Now::new(utc, at(9, 0));
    let expected = Utc.with_ymd_and_hms(2026, 10, 12, 15, 30, 0).unwrap();
    assert_eq!(now.local_to_utc(at(12, 30)), Some(expected));
}

#[test]
fn local_to_utc_goes_backwards_for_earlier_times() {
    let utc = Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap();
    let now = Now::new(utc, at(9, 0));
    let expected = Utc.with_ymd_and_hms(2026, 10, 12, 11, 0, 0).unwrap();
    assert_eq!(now.local_to_utc(at(8, 0)), Some(expected));
}

#[test]
fn local_to_utc_reports_overflow_as_none() {
    let now = Now::new(DateTime::<Utc>::MAX_UTC, at(9, 0));
    assert_eq!(now.local_to_utc(NaiveDateTime::MAX), None);
}

#[test]
fn plus_minutes_adds_and_saturates() {
    let start = Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap();
    assert_eq!(
        plus_minutes(start, 90),
        Utc.with_ymd_and_hms(2026, 10, 12, 13, 30, 0).unwrap()
    );
    assert_eq!(plus_minutes(start, 0), start);
    assert_eq!(
        plus_minutes(DateTime::<Utc>::MAX_UTC, 5),
        DateTime::<Utc>::MAX_UTC
    );
    assert_eq!(
        plus_minutes(start, u16::MAX),
        Utc.with_ymd_and_hms(2026, 10, 12, 12, 0, 0).unwrap() + chrono::Duration::minutes(65_535)
    );
}
