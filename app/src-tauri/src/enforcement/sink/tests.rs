use std::thread;
use std::time::{Duration, Instant};

use focus_core::test_support::eventually;

use crate::enforcement::test_support::{
    app_block, blocking, first_shown, in_app, FakeControl, Rig, T0,
};
use crate::tracking::sampler::SampleSink;

const STEAM: &str = "com.valvesoftware.steam";

#[test]
fn a_sample_reaches_the_worker_which_quits_the_blocked_app_and_announces_it() {
    let rig = Rig::new(blocking(&[STEAM], &[]));
    let (sink, _handle) = rig.sink();

    sink.on_sample(Some(&in_app(STEAM)), T0);

    eventually(|| !rig.surface.shown().is_empty());
    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(rig.surface.shown(), [first_shown(app_block(STEAM))]);
}

#[test]
fn a_written_usage_notice_announces_changed_usage() {
    let rig = Rig::new(blocking(&[], &[]));
    let (sink, _handle) = rig.sink();

    sink.on_usage_written();

    eventually(|| rig.events.count() >= 1);
}

#[test]
fn a_burst_of_usage_writes_is_coalesced() {
    let rig = Rig::new(blocking(&[], &[]));
    let (sink, handle) = rig.sink();

    for _ in 0..200 {
        sink.on_usage_written();
    }
    eventually(|| rig.events.count() >= 1);
    thread::sleep(Duration::from_millis(1_200));
    drop(handle);

    assert!(
        (1..=2).contains(&rig.events.count()),
        "leading event plus at most one trailing, got {}",
        rig.events.count()
    );
}

#[test]
fn posting_never_waits_for_a_busy_worker() {
    let rig = Rig::with_control(
        blocking(&[STEAM], &[]),
        FakeControl::slow(Duration::from_millis(400)),
    );
    let (sink, _handle) = rig.sink();
    sink.on_sample(Some(&in_app(STEAM)), T0);
    thread::sleep(Duration::from_millis(50));
    let started = Instant::now();

    for tick in 0..5_000 {
        sink.on_sample(Some(&in_app(STEAM)), T0 + 100 + tick);
        sink.on_usage_written();
    }

    assert!(
        started.elapsed() < Duration::from_millis(200),
        "{:?}",
        started.elapsed()
    );
}

#[test]
fn dropping_the_handle_stops_the_worker() {
    let rig = Rig::new(blocking(&[STEAM], &[]));
    let (sink, handle) = rig.sink();

    drop(handle);
    sink.on_sample(Some(&in_app(STEAM)), T0);
    sink.on_usage_written();
    thread::sleep(Duration::from_millis(100));

    assert_eq!(rig.control.quits().len(), 0);
    assert_eq!(rig.events.count(), 0);
}

#[test]
fn a_failing_platform_does_not_kill_the_worker() {
    let rig = Rig::with_control(blocking(&[STEAM], &[]), FakeControl::failing());
    let (sink, _handle) = rig.sink();

    sink.on_sample(Some(&in_app(STEAM)), T0);
    eventually(|| !rig.control.quits().is_empty());
    sink.on_sample(Some(&in_app(STEAM)), T0 + 10);

    eventually(|| rig.control.quits().len() == 2);
}
