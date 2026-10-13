use std::sync::atomic::{AtomicU32, Ordering};
use std::thread;

use focus_core::test_support::eventually;

use super::*;

const SECOND: Duration = Duration::from_secs(1);

#[test]
fn the_first_change_is_due_at_once_and_later_ones_wait_out_the_interval() {
    let mut timers = Timers::new(30 * SECOND);
    let start = Instant::now();
    assert_eq!(timers.step(start), Step::Wait(None));

    timers.total.mark();
    assert_eq!(timers.step(start), Step::Total);

    timers.total.mark();
    timers.total.mark();
    assert_eq!(
        timers.step(start + 10 * SECOND),
        Step::Wait(Some(20 * SECOND))
    );
    assert_eq!(timers.step(start + 30 * SECOND), Step::Total);
    assert_eq!(timers.step(start + 30 * SECOND), Step::Wait(None));
}

#[test]
fn the_nearest_deadline_decides_how_long_to_sleep() {
    let mut timers = Timers::new(30 * SECOND);
    let start = Instant::now();
    timers.total.mark();
    assert_eq!(timers.step(start), Step::Total);
    timers.total.mark();
    timers.line_due = Some(start + 5 * SECOND);

    assert_eq!(timers.step(start), Step::Wait(Some(5 * SECOND)));
    assert_eq!(timers.step(start + 5 * SECOND), Step::Line);
    assert_eq!(
        timers.step(start + 5 * SECOND),
        Step::Wait(Some(25 * SECOND))
    );
}

#[test]
fn a_closed_schedule_stops() {
    let mut timers = Timers::new(SECOND);
    timers.total.mark();
    timers.closed = true;
    assert_eq!(timers.step(Instant::now()), Step::Stop);
}

#[test]
fn a_burst_of_changes_ends_with_a_draw_after_the_interval() {
    let draws = Arc::new(AtomicU32::new(0));
    let worker = TrayWorker::start(
        Duration::from_millis(80),
        {
            let draws = Arc::clone(&draws);
            move || {
                draws.fetch_add(1, Ordering::SeqCst);
            }
        },
        || None,
    )
    .unwrap();

    worker.total_changed();
    eventually(|| draws.load(Ordering::SeqCst) == 1);
    for _ in 0..5 {
        worker.total_changed();
    }
    eventually(|| draws.load(Ordering::SeqCst) == 2);
    thread::sleep(Duration::from_millis(200));

    assert_eq!(
        draws.load(Ordering::SeqCst),
        2,
        "one trailing draw, not five"
    );
}

#[test]
fn a_line_that_asks_to_be_redrawn_is_redrawn_until_it_stops_asking() {
    let draws = Arc::new(AtomicU32::new(0));
    let worker = TrayWorker::start(SECOND, || {}, {
        let draws = Arc::clone(&draws);
        move || {
            let seen = draws.fetch_add(1, Ordering::SeqCst) + 1;
            (seen < 3).then_some(Duration::from_millis(10))
        }
    })
    .unwrap();

    worker.redraw_line_in(Duration::from_millis(10));
    eventually(|| draws.load(Ordering::SeqCst) == 3);
    thread::sleep(Duration::from_millis(100));

    assert_eq!(draws.load(Ordering::SeqCst), 3);
}

#[test]
fn dropping_the_worker_stops_the_thread_even_with_work_pending() {
    let worker = TrayWorker::start(Duration::from_secs(3_600), || {}, || None).unwrap();
    worker.total_changed();
    worker.total_changed();
    worker.redraw_line_in(Duration::from_secs(3_600));

    drop(worker);
}
