use chrono::NaiveDate;

use super::*;
use crate::storage::test_support::{record, store};
use crate::time::{LocalDay, HOURS_PER_DAY};

fn march(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 3, day).unwrap()
}

fn hour(count: usize) -> i64 {
    i64::try_from(count).unwrap() * HOUR
}

/// Local 02:00 does not exist, so local 02:00 and 03:00 begin at the same instant and the
/// day is 23 real hours long.
fn spring_forward_day() -> LocalDay {
    let hour_starts = std::array::from_fn(|h| DAY_START + hour(if h < 3 { h } else { h - 1 }));
    LocalDay::new(march(8), hour_starts, DAY_START + hour(23))
}

/// Local 01:00 happens twice, so local hour 1 spans two real hours and the day is 25 real
/// hours long.
fn fall_back_day() -> LocalDay {
    let hour_starts = std::array::from_fn(|h| DAY_START + hour(if h < 2 { h } else { h + 1 }));
    LocalDay::new(march(1), hour_starts, DAY_START + hour(25))
}

#[test]
fn the_skipped_spring_forward_hour_is_an_empty_bucket() {
    let store = store();
    let day = spring_forward_day();
    record(&store, "com.a", "A", None, (day.start, day.end));

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.total_seconds, 23 * 3_600);
    assert_eq!(usage.hourly[2], 0);
    for (clock_hour, seconds) in usage.hourly.iter().enumerate() {
        if clock_hour != 2 {
            assert_eq!(*seconds, 3_600, "local hour {clock_hour}");
        }
    }
}

#[test]
fn time_after_the_spring_forward_gap_lands_in_its_local_hour() {
    let store = store();
    let day = spring_forward_day();
    let before_gap = day.start + hour(2) - 10;
    record(&store, "com.a", "A", None, (before_gap, before_gap + 30));

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.hourly[1], 10, "01:59:50 is still local hour 1");
    assert_eq!(usage.hourly[3], 20, "03:00:00 is local hour 3");
    assert_eq!(usage.hourly[2], 0);
}

#[test]
fn the_repeated_fall_back_hour_folds_into_one_local_hour() {
    let store = store();
    let day = fall_back_day();
    record(&store, "com.a", "A", None, (day.start, day.end));

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.total_seconds, 25 * 3_600);
    assert_eq!(usage.hourly[1], 2 * 3_600);
    for (clock_hour, seconds) in usage.hourly.iter().enumerate() {
        if clock_hour != 1 {
            assert_eq!(*seconds, 3_600, "local hour {clock_hour}");
        }
    }
    assert_eq!(usage.hourly.iter().sum::<u32>(), usage.total_seconds);
}

#[test]
fn both_occurrences_of_the_repeated_hour_count_toward_local_hour_one() {
    let store = store();
    let day = fall_back_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start + hour(1) + 5, day.start + hour(1) + 15),
    );
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start + hour(2) + 5, day.start + hour(2) + 25),
    );

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.hourly[1], 30);
    assert_eq!(usage.hourly[2], 0);
}

#[test]
fn a_session_crossing_into_the_last_hour_of_a_long_day_is_split_correctly() {
    let store = store();
    let day = fall_back_day();
    let last_hour = day.end - hour(1);
    record(&store, "com.a", "A", None, (last_hour - 10, last_hour + 10));

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.hourly[HOURS_PER_DAY - 2], 10);
    assert_eq!(usage.hourly[HOURS_PER_DAY - 1], 10);
}

#[test]
fn careless_hour_starts_are_made_monotonic_and_stay_inside_the_day() {
    let mut hour_starts = [DAY_START; HOURS_PER_DAY];
    hour_starts[5] = DAY_START + hour(40);
    hour_starts[6] = DAY_START + hour(3);
    let day = LocalDay::new(march(1), hour_starts, DAY_START + hour(24));

    let bounds = day.hour_bounds();

    assert!(bounds.windows(2).all(|pair| pair[0] <= pair[1]));
    assert_eq!(bounds[HOURS_PER_DAY], DAY_START + hour(24));
}
