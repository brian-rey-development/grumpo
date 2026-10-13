use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Instant;

use focus_core::test_support::eventually;

use super::*;

const LONG: Duration = Duration::from_secs(60);

#[test]
fn a_stop_ends_a_long_wait_at_once() {
    let finished = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&finished);
    let mut thread = StoppableThread::spawn_signalled("test-wait", move |signal| {
        assert!(signal.wait(LONG));
        flag.store(true, Ordering::SeqCst);
    })
    .unwrap();

    let started = Instant::now();
    thread.stop();

    assert!(finished.load(Ordering::SeqCst));
    assert!(started.elapsed() < LONG);
}

#[test]
fn a_wait_that_times_out_reports_no_stop() {
    let outcome = Arc::new(AtomicBool::new(true));
    let result = Arc::clone(&outcome);
    let mut thread = StoppableThread::spawn_signalled("test-timeout", move |signal| {
        result.store(signal.wait(Duration::from_millis(1)), Ordering::SeqCst);
    })
    .unwrap();

    eventually(|| !outcome.load(Ordering::SeqCst));
    thread.stop();
}

#[test]
fn the_stop_hook_runs_once_however_often_stop_is_called() {
    let hooks = Arc::new(AtomicU32::new(0));
    let counted = Arc::clone(&hooks);
    let mut thread = StoppableThread::spawn(
        "test-hook",
        move || {
            counted.fetch_add(1, Ordering::SeqCst);
        },
        || {},
    )
    .unwrap();

    thread.stop();
    thread.stop();
    drop(thread);

    assert_eq!(hooks.load(Ordering::SeqCst), 1);
}

#[test]
fn dropping_the_handle_stops_and_joins() {
    let finished = Arc::new(AtomicBool::new(false));
    let flag = Arc::clone(&finished);
    let thread = StoppableThread::spawn_signalled("test-drop", move |signal| {
        signal.wait(LONG);
        flag.store(true, Ordering::SeqCst);
    })
    .unwrap();

    drop(thread);

    assert!(finished.load(Ordering::SeqCst));
}

#[test]
fn a_panicking_worker_is_survived() {
    let mut thread =
        StoppableThread::spawn("test-panic", || {}, || panic!("worker failed")).unwrap();

    thread.stop();
}
