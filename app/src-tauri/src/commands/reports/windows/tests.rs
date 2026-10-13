use focus_core::test_support::{schedule, window};

use super::*;

fn monday() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
}

#[test]
fn keeps_only_windows_for_the_weekday() {
    let schedule = schedule(vec![
        window(&[Day::Mon, Day::Tue], 9 * 60, 12 * 60),
        window(&[Day::Sat], 10 * 60, 11 * 60),
    ]);
    assert_eq!(
        focus_windows(&schedule, monday()),
        [FocusWindow {
            start: 9 * 3600,
            end: 12 * 3600
        }]
    );
    let sunday = NaiveDate::from_ymd_opt(2026, 10, 18).unwrap();
    assert_eq!(focus_windows(&schedule, sunday), Vec::<FocusWindow>::new());
}

#[test]
fn a_window_may_end_at_midnight() {
    let schedule = schedule(vec![window(&[Day::Mon], 20 * 60, 24 * 60)]);
    assert_eq!(
        focus_windows(&schedule, monday()),
        [FocusWindow {
            start: 20 * 3600,
            end: 86_400
        }]
    );
}

#[test]
fn windows_are_sorted_and_overlaps_are_joined() {
    let schedule = schedule(vec![
        window(&[Day::Mon], 14 * 60, 16 * 60),
        window(&[Day::Mon], 9 * 60, 10 * 60),
        window(&[Day::Mon], 10 * 60, 11 * 60),
        window(&[Day::Mon], 15 * 60, 15 * 60 + 30),
    ]);
    assert_eq!(
        focus_windows(&schedule, monday()),
        [
            FocusWindow {
                start: 9 * 3600,
                end: 11 * 3600
            },
            FocusWindow {
                start: 14 * 3600,
                end: 16 * 3600
            },
        ]
    );
}

#[test]
fn only_today_gets_windows() {
    let schedule = schedule(vec![window(&[Day::Mon, Day::Tue], 9 * 60, 12 * 60)]);
    let yesterday = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
    let today = NaiveDate::from_ymd_opt(2026, 10, 13).unwrap();
    let tomorrow = NaiveDate::from_ymd_opt(2026, 10, 14).unwrap();

    assert_eq!(focus_windows_on(&schedule, today, today).len(), 1);
    assert_eq!(focus_windows_on(&schedule, yesterday, today), []);
    assert_eq!(focus_windows_on(&schedule, tomorrow, today), []);
}

#[test]
fn an_empty_schedule_has_no_windows() {
    assert_eq!(
        focus_windows(&Schedule::default(), monday()),
        Vec::<FocusWindow>::new()
    );
}
