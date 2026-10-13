use super::*;
use crate::test_support::{hm, local};

fn at(hour: u16, minute: u16) -> TimeOfDay {
    TimeOfDay::new(hm(hour, minute)).unwrap()
}

#[test]
fn position_counts_minutes_from_monday_midnight() {
    assert_eq!(WeekMask::position(local(2026, 10, 12, 0, 0)), 0);
    assert_eq!(WeekMask::position(local(2026, 10, 12, 1, 30)), 90);
    assert_eq!(WeekMask::position(local(2026, 10, 13, 0, 0)), 1440);
    assert_eq!(
        WeekMask::position(local(2026, 10, 18, 23, 59)),
        MINUTES_PER_WEEK - 1
    );
}

#[test]
fn fill_marks_start_inclusive_and_end_exclusive() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Tue, at(9, 0), at(10, 0));
    let origin = MINUTES_PER_DAY_USIZE;
    assert!(!mask.is_set(origin + 539));
    assert!(mask.is_set(origin + 540));
    assert!(mask.is_set(origin + 599));
    assert!(!mask.is_set(origin + 600));
}

#[test]
fn fill_to_end_of_day_stops_at_the_day_boundary() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Mon, at(23, 0), TimeOfDay::END_OF_DAY);
    assert!(mask.is_set(1439));
    assert!(!mask.is_set(1440));
}

#[test]
fn fill_ignores_empty_and_inverted_ranges() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Mon, at(10, 0), at(10, 0));
    mask.fill(Day::Mon, at(11, 0), at(10, 0));
    assert_eq!(mask, WeekMask::default());
}

#[test]
fn is_set_wraps_around_the_week() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Mon, TimeOfDay::MIDNIGHT, at(1, 0));
    assert!(mask.is_set(MINUTES_PER_WEEK));
    assert!(mask.is_set(MINUTES_PER_WEEK + 59));
    assert!(!mask.is_set(MINUTES_PER_WEEK + 60));
}

#[test]
fn uncovered_by_counts_only_minutes_missing_from_the_other() {
    let mut wide = WeekMask::default();
    wide.fill(Day::Mon, at(8, 0), at(12, 0));
    let mut narrow = WeekMask::default();
    narrow.fill(Day::Mon, at(9, 0), at(11, 0));
    narrow.fill(Day::Tue, at(9, 0), at(11, 0));
    assert_eq!(wide.uncovered_by(&narrow), 120);
    assert_eq!(narrow.uncovered_by(&wide), 120);
    assert_eq!(wide.uncovered_by(&wide), 0);
    assert_eq!(wide.uncovered_by(&WeekMask::default()), 240);
}

#[test]
fn fill_span_wraps_around_the_week() {
    let mut mask = WeekMask::default();
    mask.fill_span(MINUTES_PER_WEEK - 30, 90);
    assert!(mask.is_set(MINUTES_PER_WEEK - 1));
    assert!(mask.is_set(59));
    assert!(!mask.is_set(60));
    assert!(!mask.is_set(MINUTES_PER_WEEK - 31));
}

#[test]
fn run_from_measures_contiguous_marked_minutes() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Mon, at(8, 0), at(9, 0));
    assert_eq!(mask.run_from(480), Some(60));
    assert_eq!(mask.run_from(500), Some(40));
    assert_eq!(mask.run_from(540), Some(0));
    assert_eq!(mask.run_from(0), Some(0));
}

#[test]
fn run_from_wraps_across_the_end_of_the_week() {
    let mut mask = WeekMask::default();
    mask.fill(Day::Sun, at(23, 0), TimeOfDay::END_OF_DAY);
    mask.fill(Day::Mon, TimeOfDay::MIDNIGHT, at(1, 0));
    let sunday_23 = 6 * MINUTES_PER_DAY_USIZE + 23 * 60;
    assert_eq!(mask.run_from(sunday_23), Some(120));
}

#[test]
fn run_from_is_none_when_the_whole_week_is_marked() {
    let mut mask = WeekMask::default();
    for day in Day::ALL {
        mask.fill(day, TimeOfDay::MIDNIGHT, TimeOfDay::END_OF_DAY);
    }
    assert_eq!(mask.run_from(12_345), None);
    assert_eq!(mask.uncovered_by(&WeekMask::default()), 10_080);
}
