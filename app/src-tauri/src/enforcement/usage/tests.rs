use std::time::Duration;

use super::*;
use crate::enforcement::test_support::{app_target, browsing, in_app, site_target, T0};
use crate::platform::Browser;
use crate::storage::test_support::{record, store};
use crate::time::test_support::FixedCalendar;
use crate::time::Calendar;
use crate::tracking::timing::MAX_SAMPLE_GAP;

const STEAM: &str = "com.valvesoftware.steam";

fn day() -> LocalDay {
    FixedCalendar::utc().day_containing(T0)
}

fn steam_seconds(usage: &[(LimitTarget, u32)]) -> u32 {
    usage
        .iter()
        .find(|(target, _)| *target == app_target(STEAM))
        .map(|(_, seconds)| *seconds)
        .unwrap()
}

#[test]
fn nothing_is_read_when_there_are_no_limits() {
    let mut tracker = UsageTracker::default();

    assert_eq!(tracker.today(&store(), day(), &[], None, T0).len(), 0);
}

#[test]
fn stored_seconds_are_reported_per_target() {
    let store = store();
    record(&store, STEAM, "Steam", None, (T0 - 900, T0 - 300));
    let mut tracker = UsageTracker::default();

    let usage = tracker.today(&store, day(), &[app_target(STEAM)], None, T0);

    assert_eq!(steam_seconds(&usage), 600);
}

#[test]
fn the_store_is_read_again_only_after_ten_seconds_or_a_write() {
    let store = store();
    let targets = [app_target(STEAM)];
    record(&store, STEAM, "Steam", None, (T0 - 900, T0 - 600));
    let mut tracker = UsageTracker::default();
    assert_eq!(
        steam_seconds(&tracker.today(&store, day(), &targets, None, T0)),
        300
    );

    record(&store, STEAM, "Steam", None, (T0 - 500, T0 - 400));
    let cached = tracker.today(&store, day(), &targets, None, T0 + 9);
    assert_eq!(steam_seconds(&cached), 300);

    let refreshed = tracker.today(&store, day(), &targets, None, T0 + 10);
    assert_eq!(steam_seconds(&refreshed), 400);

    record(&store, STEAM, "Steam", None, (T0 - 300, T0 - 200));
    tracker.mark_written(T0 + 11);
    let after_write = tracker.today(&store, day(), &targets, None, T0 + 11);
    assert_eq!(steam_seconds(&after_write), 500);
}

#[test]
fn changed_targets_force_a_read() {
    let store = store();
    record(&store, STEAM, "Steam", None, (T0 - 900, T0 - 600));
    let mut tracker = UsageTracker::default();
    tracker.today(&store, day(), &[site_target("a.com")], None, T0);

    let usage = tracker.today(&store, day(), &[app_target(STEAM)], None, T0 + 1);

    assert_eq!(steam_seconds(&usage), 300);
}

#[test]
fn the_session_in_progress_is_credited_since_the_last_write() {
    let store = store();
    let mut tracker = UsageTracker::default();
    tracker.mark_written(T0);
    let steam = in_app(STEAM);

    let usage = tracker.today(&store, day(), &[app_target(STEAM)], Some(&steam), T0 + 7);

    assert_eq!(steam_seconds(&usage), 7);
}

#[test]
fn in_flight_time_is_capped_because_a_longer_trail_means_a_failed_write() {
    let store = store();
    let mut tracker = UsageTracker::default();
    tracker.mark_written(T0);
    let steam = in_app(STEAM);

    let usage = tracker.today(&store, day(), &[app_target(STEAM)], Some(&steam), T0 + 500);

    assert_eq!(steam_seconds(&usage), 30);
}

#[test]
fn sites_are_credited_for_subdomain_visits_and_other_activity_is_not() {
    let store = store();
    let mut tracker = UsageTracker::default();
    tracker.mark_written(T0);
    let targets = [site_target("reddit.com"), app_target(STEAM)];

    let on_reddit = browsing(Browser::Chrome, "old.reddit.com");
    let usage = tracker.today(&store, day(), &targets, Some(&on_reddit), T0 + 5);

    assert_eq!(
        usage,
        [(site_target("reddit.com"), 5), (app_target(STEAM), 0)]
    );
}

#[test]
fn private_and_idle_activity_is_not_credited() {
    let store = store();
    let mut tracker = UsageTracker::default();
    tracker.mark_written(T0);
    let targets = [app_target(STEAM)];
    let mut private = in_app(STEAM);
    private.private = true;
    let mut idle = in_app(STEAM);
    idle.engaged = false;
    idle.idle = Duration::from_secs(300);

    assert_eq!(
        steam_seconds(&tracker.today(&store, day(), &targets, Some(&private), T0 + 5)),
        0
    );
    assert_eq!(
        steam_seconds(&tracker.today(&store, day(), &targets, Some(&idle), T0 + 5)),
        0
    );
}

#[test]
fn with_no_write_seen_nothing_is_credited() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let steam = in_app(STEAM);

    let usage = tracker.today(&store, day(), &[app_target(STEAM)], Some(&steam), T0);

    assert_eq!(steam_seconds(&usage), 0);
}

fn private_in(bundle_id: &str) -> Activity {
    let mut activity = in_app(bundle_id);
    activity.private = true;
    activity
}

fn sample_privately(
    tracker: &mut UsageTracker,
    store: &Store,
    activity: &Activity,
    at: i64,
) -> u32 {
    steam_seconds(&tracker.today(store, day(), &[app_target(STEAM)], Some(activity), at))
}

#[test]
fn engaged_private_time_accumulates_by_the_gap_between_samples() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let private = private_in(STEAM);

    assert_eq!(sample_privately(&mut tracker, &store, &private, T0), 0);
    assert_eq!(sample_privately(&mut tracker, &store, &private, T0 + 2), 2);
    assert_eq!(sample_privately(&mut tracker, &store, &private, T0 + 4), 4);
}

#[test]
fn private_time_is_added_to_what_the_store_holds() {
    let store = store();
    record(&store, STEAM, "Steam", None, (T0 - 900, T0 - 600));
    let mut tracker = UsageTracker::default();
    let private = private_in(STEAM);

    sample_privately(&mut tracker, &store, &private, T0);
    let total = sample_privately(&mut tracker, &store, &private, T0 + 2);

    assert_eq!(total, 300 + 2);
}

#[test]
fn a_gap_longer_than_a_run_counts_only_up_to_the_cap() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let private = private_in(STEAM);

    sample_privately(&mut tracker, &store, &private, T0);
    let total = sample_privately(&mut tracker, &store, &private, T0 + 3_600);

    assert_eq!(total, u32::try_from(MAX_SAMPLE_GAP).unwrap());
}

#[test]
fn only_engaged_private_time_on_the_target_counts() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let mut idle = private_in(STEAM);
    idle.engaged = false;
    let elsewhere = private_in("com.apple.Notes");

    sample_privately(&mut tracker, &store, &idle, T0);
    assert_eq!(sample_privately(&mut tracker, &store, &idle, T0 + 2), 0);
    sample_privately(&mut tracker, &store, &elsewhere, T0 + 4);
    assert_eq!(
        sample_privately(&mut tracker, &store, &elsewhere, T0 + 6),
        0
    );
}

#[test]
fn ordinary_time_does_not_enter_the_private_tally() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let steam = in_app(STEAM);

    tracker.today(&store, day(), &[app_target(STEAM)], Some(&steam), T0);
    tracker.today(&store, day(), &[app_target(STEAM)], Some(&steam), T0 + 2);

    assert!(tracker.private.is_empty());
}

#[test]
fn private_time_is_forgotten_by_a_reset_and_by_a_restart() {
    let store = store();
    let private = private_in(STEAM);
    let mut tracker = UsageTracker::default();
    sample_privately(&mut tracker, &store, &private, T0);
    sample_privately(&mut tracker, &store, &private, T0 + 2);

    tracker.reset();
    assert_eq!(sample_privately(&mut tracker, &store, &private, T0 + 4), 0);

    let mut restarted = UsageTracker::default();
    assert_eq!(
        sample_privately(&mut restarted, &store, &private, T0 + 6),
        0
    );
}

#[test]
fn private_time_is_tallied_per_target_and_sites_follow_subdomains() {
    let store = store();
    let mut tracker = UsageTracker::default();
    let mut visit = browsing(Browser::Chrome, "old.reddit.com");
    visit.private = true;
    let targets = [site_target("reddit.com"), site_target("example.com")];

    tracker.today(&store, day(), &targets, Some(&visit), T0);
    let usage = tracker.today(&store, day(), &targets, Some(&visit), T0 + 2);

    assert_eq!(
        usage,
        [
            (site_target("reddit.com"), 2),
            (site_target("example.com"), 0)
        ]
    );
}
