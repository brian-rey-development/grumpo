use focus_core::LimitTarget;

use super::*;
use crate::storage::test_support::{record, store};
use crate::storage::Store;
use focus_core::test_support::{app_target, site_target};

fn seconds(store: &Store, target: LimitTarget) -> u32 {
    store.usage_seconds(standard_day(), &[target]).unwrap()[0].1
}

#[test]
fn site_targets_match_the_hosts_their_block_covers() {
    let store = store();
    let day = standard_day();
    let chrome = "com.google.Chrome";
    record(
        &store,
        chrome,
        "Chrome",
        Some("youtube.com"),
        (day.start, day.start + 100),
    );
    record(
        &store,
        chrome,
        "Chrome",
        Some("m.youtube.com"),
        (day.start + 100, day.start + 150),
    );
    record(
        &store,
        chrome,
        "Chrome",
        Some("music.youtube.com"),
        (day.start + 150, day.start + 160),
    );
    record(
        &store,
        chrome,
        "Chrome",
        Some("notyoutube.com"),
        (day.start + 160, day.start + 460),
    );
    record(
        &store,
        chrome,
        "Chrome",
        Some("youtube.com.evil.io"),
        (day.start + 460, day.start + 560),
    );
    record(
        &store,
        chrome,
        "Chrome",
        None,
        (day.start + 560, day.start + 960),
    );

    assert_eq!(seconds(&store, site_target("youtube.com")), 150);
    assert_eq!(seconds(&store, site_target("m.youtube.com")), 50);
    assert_eq!(seconds(&store, site_target("notyoutube.com")), 300);
}

#[test]
fn app_targets_match_the_bundle_id_regardless_of_name() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.tinyspeck.slackmacgap",
        "Slack",
        None,
        (day.start, day.start + 90),
    );
    record(
        &store,
        "com.tinyspeck.slackmacgap",
        "Slack Beta",
        None,
        (day.start + 90, day.start + 100),
    );
    record(
        &store,
        "com.other",
        "Slack",
        None,
        (day.start + 100, day.start + 900),
    );

    assert_eq!(
        seconds(&store, app_target("com.tinyspeck.slackmacgap")),
        100
    );
}

#[test]
fn usage_is_clipped_to_the_range() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start - 500, day.start + 200),
    );
    record(&store, "com.a", "A", None, (day.end - 300, day.end + 700));

    assert_eq!(seconds(&store, app_target("com.a")), 500);
}

#[test]
fn results_keep_the_order_of_the_targets_and_zero_for_no_usage() {
    let store = store();
    let day = standard_day();
    record(&store, "com.a", "A", None, (day.start, day.start + 40));
    let targets = [site_target("example.com"), app_target("com.a")];

    let result = store.usage_seconds(day, &targets).unwrap();

    assert_eq!(result, [(targets[0].clone(), 0), (targets[1].clone(), 40)]);
    assert_eq!(store.usage_seconds(day, &[]).unwrap().len(), 0);
}

#[test]
fn the_total_clips_sessions_to_the_day_like_the_report_does() {
    let store = store();
    let day = standard_day();
    record(
        &store,
        "com.a",
        "A",
        None,
        (day.start - 100, day.start + 50),
    );
    record(
        &store,
        "com.b",
        "B",
        Some("x.com"),
        (day.start + 100, day.start + 400),
    );
    record(&store, "com.c", "C", None, (day.end - 30, day.end + 500));
    record(&store, "com.d", "D", None, (day.end + 10, day.end + 20));

    let total = store.total_seconds(day).unwrap();

    assert_eq!(total, 50 + 300 + 30);
    assert_eq!(total, store.day_usage(day).unwrap().total_seconds);
}

#[test]
fn the_total_of_an_empty_day_is_zero() {
    assert_eq!(store().total_seconds(standard_day()).unwrap(), 0);
}
