use super::*;
use crate::storage::test_support::{record, store};
use crate::time::test_support::fixed_day;
use crate::time::units::DAYS_PER_WEEK;
use crate::time::LocalDay;

fn week() -> [LocalDay; DAYS_PER_WEEK] {
    std::array::from_fn(|i| {
        let offset = i64::try_from(i).unwrap() * 24 * HOUR;
        let day = u32::try_from(i).unwrap() + 1;
        fixed_day(date(day), DAY_START + offset)
    })
}

#[test]
fn an_empty_week_has_seven_zero_days() {
    let usage = store().week_usage(&week()).unwrap();
    assert_eq!(usage.days.len(), 7);
    assert!(usage.days.iter().all(|d| d.total_seconds == 0));
    assert_eq!(usage.total_seconds, 0);
}

#[test]
fn days_total_separately_and_sum_to_the_week() {
    let store = store();
    let days = week();
    record(
        &store,
        "com.a",
        "A",
        None,
        (days[0].start + 10, days[0].start + 110),
    );
    record(
        &store,
        "com.a",
        "A",
        None,
        (days[2].start, days[2].start + 50),
    );
    record(
        &store,
        "com.b",
        "B",
        Some("x.com"),
        (days[6].end - 30, days[6].end + 500),
    );
    record(
        &store,
        "com.a",
        "A",
        None,
        (days[0].start - 1_000, days[0].start - 500),
    );

    let usage = store.week_usage(&days).unwrap();

    let totals: Vec<_> = usage.days.iter().map(|d| d.total_seconds).collect();
    assert_eq!(totals, [100, 0, 50, 0, 0, 0, 30]);
    assert_eq!(usage.days[2].date, date(3));
    assert_eq!(usage.total_seconds, 180);
    assert_eq!(usage.top_apps[0].key, "com.a");
    assert_eq!(usage.top_apps[0].seconds, 150);
    assert_eq!(usage.top_sites.len(), 1);
    assert_eq!(usage.top_sites[0].seconds, 30);
}

#[test]
fn a_session_crossing_midnight_counts_once_per_day_it_touches() {
    let store = store();
    let days = week();
    record(
        &store,
        "com.a",
        "A",
        None,
        (days[1].end - 40, days[1].end + 60),
    );

    let usage = store.week_usage(&days).unwrap();

    assert_eq!(
        (usage.days[1].total_seconds, usage.days[2].total_seconds),
        (40, 60)
    );
    assert_eq!(usage.total_seconds, 100);
    assert_eq!(usage.top_apps[0].seconds, 100);
}
