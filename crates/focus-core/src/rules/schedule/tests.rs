use chrono::Timelike;

use super::*;
use crate::test_support::{hm, local, schedule};

fn window(days: &[Day], start: (u16, u16), end: (u16, u16)) -> Window {
    crate::test_support::window(days, hm(start.0, start.1), hm(end.0, end.1))
}

const WEEKDAYS: [Day; 5] = [Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri];

fn legacy_like() -> Schedule {
    schedule(vec![
        window(&WEEKDAYS, (0, 0), (19, 0)),
        window(&[Day::Sat], (0, 0), (13, 0)),
    ])
}

#[test]
fn empty_schedule_is_never_active() {
    let schedule = Schedule::default();
    assert!(!schedule.active_at(local(2026, 10, 12, 10, 0)));
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 10, 0)),
        Coverage::Inactive
    );
}

#[test]
fn active_at_has_inclusive_start_and_exclusive_end() {
    let schedule = schedule(vec![window(&[Day::Mon], (9, 0), (17, 0))]);
    let cases = [
        (local(2026, 10, 12, 8, 59), false),
        (local(2026, 10, 12, 9, 0), true),
        (local(2026, 10, 12, 12, 0), true),
        (local(2026, 10, 12, 16, 59), true),
        (local(2026, 10, 12, 17, 0), false),
        (local(2026, 10, 13, 12, 0), false),
    ];
    for (at, expected) in cases {
        assert_eq!(schedule.active_at(at), expected, "at {at}");
    }
}

#[test]
fn seconds_do_not_move_the_boundaries() {
    let schedule = schedule(vec![window(&[Day::Mon], (9, 0), (17, 0))]);
    let before_start = local(2026, 10, 12, 8, 59).with_second(59).unwrap();
    let before_end = local(2026, 10, 12, 16, 59).with_second(59).unwrap();
    assert!(!schedule.active_at(before_start));
    assert!(schedule.active_at(before_end));
}

#[test]
fn a_window_ending_at_end_of_day_covers_the_last_minute() {
    let schedule = schedule(vec![window(&[Day::Mon], (22, 0), (24, 0))]);
    assert!(schedule.active_at(local(2026, 10, 12, 23, 59)));
    assert!(!schedule.active_at(local(2026, 10, 13, 0, 0)));
}

#[test]
fn legacy_like_schedule_covers_weekday_mornings_and_saturday_until_one() {
    let schedule = legacy_like();
    let cases = [
        (local(2026, 10, 12, 0, 0), true),
        (local(2026, 10, 12, 18, 59), true),
        (local(2026, 10, 12, 19, 0), false),
        (local(2026, 10, 16, 18, 59), true),
        (local(2026, 10, 17, 12, 59), true),
        (local(2026, 10, 17, 13, 0), false),
        (local(2026, 10, 18, 9, 0), false),
    ];
    for (at, expected) in cases {
        assert_eq!(schedule.active_at(at), expected, "at {at}");
    }
}

#[test]
fn active_until_is_the_end_of_the_current_window() {
    let schedule = schedule(vec![window(&[Day::Mon], (9, 0), (17, 0))]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 9, 0)),
        Coverage::Until(local(2026, 10, 12, 17, 0))
    );
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 16, 59)),
        Coverage::Until(local(2026, 10, 12, 17, 0))
    );
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 17, 0)),
        Coverage::Inactive
    );
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 8, 59)),
        Coverage::Inactive
    );
}

#[test]
fn active_until_keeps_the_end_aligned_to_the_minute() {
    let schedule = schedule(vec![window(&[Day::Mon], (9, 0), (17, 0))]);
    let mid_minute = local(2026, 10, 12, 10, 0).with_second(42).unwrap();
    assert_eq!(
        schedule.active_until(mid_minute),
        Coverage::Until(local(2026, 10, 12, 17, 0))
    );
}

#[test]
fn active_until_merges_adjacent_windows() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (9, 0), (12, 0)),
        window(&[Day::Mon], (12, 0), (15, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 10, 0)),
        Coverage::Until(local(2026, 10, 12, 15, 0))
    );
}

#[test]
fn active_until_merges_overlapping_windows_in_any_order() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (11, 0), (16, 0)),
        window(&[Day::Mon], (9, 0), (13, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 9, 0)),
        Coverage::Until(local(2026, 10, 12, 16, 0))
    );
}

#[test]
fn active_until_stops_at_a_gap() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (9, 0), (12, 0)),
        window(&[Day::Mon], (12, 1), (15, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 10, 0)),
        Coverage::Until(local(2026, 10, 12, 12, 0))
    );
}

#[test]
fn active_until_continues_across_midnight() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (20, 0), (24, 0)),
        window(&[Day::Tue], (0, 0), (7, 30)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 12, 21, 0)),
        Coverage::Until(local(2026, 10, 13, 7, 30))
    );
}

#[test]
fn active_until_continues_across_the_week_wrap() {
    let schedule = schedule(vec![
        window(&[Day::Sun], (22, 0), (24, 0)),
        window(&[Day::Mon], (0, 0), (6, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 18, 23, 0)),
        Coverage::Until(local(2026, 10, 19, 6, 0))
    );
}

#[test]
fn active_until_runs_through_consecutive_full_days() {
    let schedule = schedule(vec![window(&WEEKDAYS, (0, 0), (24, 0))]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 14, 10, 0)),
        Coverage::Until(local(2026, 10, 17, 0, 0))
    );
}

#[test]
fn active_until_reports_always_when_every_minute_is_covered() {
    let schedule = schedule(vec![window(&Day::ALL, (0, 0), (24, 0))]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 14, 10, 0)),
        Coverage::Always
    );
}

#[test]
fn a_week_covered_in_pieces_is_always() {
    let schedule = schedule(vec![
        window(&Day::ALL, (0, 0), (12, 0)),
        window(&Day::ALL, (12, 0), (24, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 14, 3, 0)),
        Coverage::Always
    );
}

#[test]
fn a_single_missing_minute_ends_an_otherwise_full_week() {
    let schedule = schedule(vec![
        window(&Day::ALL, (0, 0), (12, 0)),
        window(&Day::ALL, (12, 1), (24, 0)),
    ]);
    assert_eq!(
        schedule.active_until(local(2026, 10, 14, 3, 0)),
        Coverage::Until(local(2026, 10, 14, 12, 0))
    );
}

#[test]
fn invalid_windows_cover_nothing() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (10, 0), (10, 0)),
        window(&[Day::Mon], (12, 0), (11, 0)),
        window(&[], (0, 0), (24, 0)),
    ]);
    assert!(!schedule.active_at(local(2026, 10, 12, 10, 0)));
    assert_eq!(schedule.mask(), WeekMask::default());
}

#[test]
fn serializes_in_camel_case() {
    let schedule = schedule(vec![window(&[Day::Mon, Day::Fri], (9, 0), (17, 0))]);
    let json = serde_json::to_string(&schedule).unwrap();
    assert_eq!(
        json,
        r#"{"windows":[{"days":["mon","fri"],"start":540,"end":1020}]}"#
    );
    assert_eq!(serde_json::from_str::<Schedule>(&json).unwrap(), schedule);
}

#[test]
fn through_day_extends_a_window_that_continues_past_midnight() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (23, 0), (24, 0)),
        window(&[Day::Tue], (0, 0), (2, 0)),
    ]);
    assert_eq!(
        schedule.active_until_through_day(local(2026, 10, 12, 20, 0)),
        Coverage::Until(local(2026, 10, 13, 2, 0))
    );
}

#[test]
fn through_day_without_windows_ends_at_the_next_midnight() {
    let empty = Schedule::default();
    assert_eq!(
        empty.active_until_through_day(local(2026, 10, 12, 20, 15)),
        Coverage::Until(local(2026, 10, 13, 0, 0))
    );
    assert_eq!(
        empty.active_until_through_day(local(2026, 10, 12, 23, 59)),
        Coverage::Until(local(2026, 10, 13, 0, 0))
    );
}

#[test]
fn through_day_is_always_when_windows_and_the_rest_of_today_fill_the_week() {
    let schedule = schedule(vec![
        window(&[Day::Mon], (0, 0), (13, 0)),
        window(&Day::ALL[1..], (0, 0), (24, 0)),
    ]);
    let at = local(2026, 10, 12, 12, 30);
    assert_eq!(
        schedule.active_until(at),
        Coverage::Until(local(2026, 10, 12, 13, 0))
    );
    assert_eq!(schedule.active_until_through_day(at), Coverage::Always);
}
