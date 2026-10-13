use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use focus_core::test_support::eventually;

use super::test_support::*;
use super::*;
use crate::platform::Browser;
use crate::tracking::test_support::CHROME;

#[test]
fn a_run_of_samples_becomes_one_stored_session() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.apple.dt.Xcode"));
    rig.tick_every_two_seconds(4);
    rig.sampler.finish();
    assert_eq!(
        rig.rows(),
        [(T0, T0 + 6, "com.apple.dt.Xcode".to_owned(), None)]
    );
}

#[test]
fn browser_domains_are_stored_and_a_domain_change_splits_sessions() {
    let mut rig = rig();
    rig.probe.set_foreground(Some(CHROME));
    rig.probe
        .set_host(Browser::Chrome, Some("https://www.youtube.com/watch?v=1"));
    rig.tick_every_two_seconds(3);
    rig.probe.set_host(Browser::Chrome, Some("docs.rs"));
    rig.tick_every_two_seconds(3);
    rig.sampler.finish();

    assert_eq!(
        rig.rows(),
        [
            (
                T0,
                T0 + 8,
                CHROME.to_owned(),
                Some("youtube.com".to_owned())
            ),
            (
                T0 + 8,
                T0 + 10,
                CHROME.to_owned(),
                Some("docs.rs".to_owned())
            ),
        ]
    );
}

#[test]
fn extends_are_batched_and_flushed_at_most_every_ten_seconds() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(5);
    assert_eq!(rig.rows(), [(T0, T0, "com.a".to_owned(), None)]);

    rig.tick_every_two_seconds(1);
    assert_eq!(rig.rows(), [(T0, T0 + 10, "com.a".to_owned(), None)]);
}

#[test]
fn finish_flushes_pending_usage() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(3);
    assert_eq!(rig.rows()[0].1, T0);
    rig.sampler.finish();
    assert_eq!(rig.rows()[0].1, T0 + 4);
}

#[test]
fn deadlines_follow_a_fixed_grid_and_skip_missed_slots() {
    let start = Instant::now();
    let slot = |n: u32| start + SAMPLE_INTERVAL * n;
    assert_eq!(
        next_deadline(slot(0), slot(0) + Duration::from_millis(300)),
        slot(1)
    );
    assert_eq!(next_deadline(slot(1), slot(1)), slot(2));
    assert_eq!(
        next_deadline(slot(1), slot(4) + Duration::from_millis(1)),
        slot(5)
    );
}

#[test]
fn a_sleep_gap_ends_the_session_at_its_last_seen_time() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(3);
    rig.clock.advance(3_600);
    rig.tick_every_two_seconds(2);
    rig.sampler.finish();

    assert_eq!(
        rig.rows(),
        [
            (T0, T0 + 4, "com.a".to_owned(), None),
            (T0 + 3_606, T0 + 3_608, "com.a".to_owned(), None),
        ]
    );
}

#[test]
fn the_sink_sees_every_sample_and_usage_writes() {
    let mut rig = rig();
    rig.tick_every_two_seconds(1);
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(2);

    let samples = rig.sink.samples.lock().unwrap();
    assert_eq!(samples.len(), 3);
    assert!(samples[0].0.is_none());
    assert_eq!(
        samples[1].0.as_ref().map(|a| a.app.bundle_id()),
        Some("com.a")
    );
    assert_eq!(samples[2].1, T0 + 4);
    assert_eq!(rig.sink.usage_writes.load(Ordering::SeqCst), 1);
}

#[test]
fn old_sessions_are_pruned_on_the_first_tick() {
    let mut rig = rig();
    let old = T0 - 120 * 86_400;
    crate::storage::test_support::record(&rig.store, "com.old", "Old", None, (old, old + 10));
    rig.tick_every_two_seconds(1);
    assert_eq!(rig.rows().len(), 0);
}

#[test]
fn the_thread_samples_immediately_and_flushes_on_stop() {
    let rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    let (store, sink, clock) = (
        Arc::clone(&rig.store),
        Arc::clone(&rig.sink),
        Arc::clone(&rig.clock),
    );
    let handle = rig.sampler.start().unwrap();

    eventually(|| !sink.samples.lock().unwrap().is_empty());
    clock.advance(2);
    handle.stop();

    let ended: i64 = store
        .with_conn(|c| Ok(c.query_row("SELECT COUNT(*) FROM sessions", [], |r| r.get(0))?))
        .unwrap();
    assert_eq!(ended, 1);
    assert_eq!(sink.samples.lock().unwrap().len(), 1);
}
