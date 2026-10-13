use super::*;
use crate::storage::test_support::{record, store};
use crate::time::test_support::fixed_day;

#[test]
fn an_empty_day_is_all_zeroes() {
    let usage = store().day_usage(standard_day()).unwrap();
    assert_eq!(usage.total_seconds, 0);
    assert_eq!(usage.hourly, [0; 24]);
    assert_eq!(
        usage.timeline.len() + usage.top_apps.len() + usage.top_sites.len(),
        0
    );
}

#[test]
fn sessions_are_clipped_to_the_day_on_both_sides() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start - 600, day.start + 300),
    );
    record(&store, "com.b", "B", None, (day.end - 200, day.end + 900));
    record(
        &store,
        "com.c",
        "C",
        None,
        (day.start - 900, day.start - 100),
    );
    record(&store, "com.d", "D", None, (day.end, day.end + 50));

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.total_seconds, 500);
    let keys: Vec<_> = usage
        .top_apps
        .iter()
        .map(|e| (e.key.as_str(), e.seconds))
        .collect();
    assert_eq!(keys, [("com.a", 300), ("com.b", 200)]);
}

#[test]
fn sessions_spanning_midnight_split_between_the_two_days() {
    let store = store();
    let first = standard_day();
    let second = fixed_day(date(2), first.end);
    record(
        &store,
        "com.a",
        "A",
        None,
        (first.end - 1_000, first.end + 2_500),
    );

    assert_eq!(store.day_usage(first).unwrap().total_seconds, 1_000);
    assert_eq!(store.day_usage(second).unwrap().total_seconds, 2_500);
}

#[test]
fn hourly_buckets_follow_hours_since_the_day_start() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start + HOUR - 100, day.start + HOUR + 200),
    );
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start + 23 * HOUR, day.start + 23 * HOUR + 60),
    );

    let usage = store.day_usage(day).unwrap();

    assert_eq!(usage.hourly[0], 100);
    assert_eq!(usage.hourly[1], 200);
    assert_eq!(usage.hourly[23], 60);
    assert_eq!(usage.hourly.iter().sum::<u32>(), usage.total_seconds);
}

#[test]
fn top_lists_split_apps_from_sites() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.google.Chrome",
        "Chrome",
        Some("youtube.com"),
        (day.start, day.start + 600),
    );
    record(
        &store,
        "com.google.Chrome",
        "Chrome",
        Some("docs.rs"),
        (day.start + 700, day.start + 1_000),
    );
    record(
        &store,
        "com.apple.dt.Xcode",
        "Xcode",
        None,
        (day.start + 2_000, day.start + 2_400),
    );

    let usage = store.day_usage(day).unwrap();

    let apps: Vec<_> = usage
        .top_apps
        .iter()
        .map(|e| (e.label.as_str(), e.seconds))
        .collect();
    assert_eq!(apps, [("Chrome", 900), ("Xcode", 400)]);
    let sites: Vec<_> = usage
        .top_sites
        .iter()
        .map(|e| (e.key.as_str(), e.seconds))
        .collect();
    assert_eq!(sites, [("youtube.com", 600), ("docs.rs", 300)]);
}

#[test]
fn the_timeline_comes_back_in_order_with_domains() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start + 500, day.start + 700),
    );
    record(
        &store,
        "com.google.Chrome",
        "Chrome",
        Some("x.com"),
        (day.start, day.start + 400),
    );

    let timeline = store.day_usage(day).unwrap().timeline;

    assert_eq!(timeline.len(), 2);
    assert_eq!(timeline[0].domain.as_deref(), Some("x.com"));
    assert_eq!(timeline[1].app_id, "com.a");
}
