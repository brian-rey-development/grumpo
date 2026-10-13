//! The property that matters most here: none of the entry points waits for the main thread,
//! so a main thread that never runs a queued job, as when it is stuck joining a worker at
//! exit, cannot hold any caller.

use std::sync::atomic::Ordering;
use std::thread;

use focus_core::test_support::eventually;

use super::test_support::{RecordingView, StalledMain};
use super::*;
use crate::dto::DaemonUnavailableReason;
use crate::platform::HourCycle;
use crate::storage::test_support::{record, store};
use crate::time::test_support::{FakeClock, FakeHourCycle, FixedCalendar};

fn down() -> FocusStatus {
    FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotInstalled,
        message: String::new(),
    }
}

fn tray(main: &Arc<StalledMain>, view: &Arc<RecordingView>, store: Arc<Store>) -> TrayHandle {
    TrayHandle::assemble(
        main.clone(),
        view.clone(),
        store,
        Arc::new(FakeClock::at(1_000)),
        ClockFormatter::new(
            Arc::new(FixedCalendar::utc()),
            Arc::new(FakeHourCycle::new(HourCycle::TwentyFour)),
        ),
        &down(),
    )
    .unwrap()
}

#[test]
fn updates_return_while_the_main_thread_never_runs_anything() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let handle = tray(&main, &view, Arc::new(store()));

    let caller = thread::spawn({
        let handle = handle.clone();
        move || {
            handle.on_status(&down());
            handle.on_usage_changed();
            handle.refresh_total();
        }
    });

    eventually(|| caller.is_finished());
    caller.join().unwrap();
    assert!(main.queued() > 0, "the updates were queued, not run");
    assert!(
        view.lines.lock().unwrap().is_empty(),
        "nothing touched a widget"
    );
}

#[test]
fn queued_updates_reach_the_view_only_from_the_thread_that_runs_them() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let store = Arc::new(store());
    record(
        &store,
        "com.a",
        "A",
        None,
        (crate::time::today().start, crate::time::today().start + 125),
    );
    let handle = tray(&main, &view, store);

    eventually(|| main.queued() >= 2);
    main.run_all();

    assert_eq!(
        view.lines.lock().unwrap().last().unwrap(),
        "Grumpo service not installed"
    );
    assert_eq!(view.relock.lock().unwrap().last(), Some(&false));
    assert_eq!(view.totals.lock().unwrap().last().unwrap(), "2 min");
    assert_eq!(view.touched_by_other_threads.load(Ordering::SeqCst), 0);
    drop(handle);
}

#[test]
fn dropping_the_tray_with_queued_work_does_not_wait_for_the_main_thread() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let handle = tray(&main, &view, Arc::new(store()));
    handle.on_usage_changed();

    let dropper = thread::spawn(move || drop(handle));

    eventually(|| dropper.is_finished());
    dropper.join().unwrap();
}

#[test]
fn a_cooldown_line_counts_the_minutes_left() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let handle = tray(&main, &view, Arc::new(store()));
    let mut status = crate::daemon_client::test_support::daemon_status();
    status.unlock.phase = focus_core::ipc::UnlockPhaseView::CoolingDown {
        ready_at: 1_000 + 240,
        minutes: 5,
    };

    handle.on_status(&FocusStatus::Connected {
        status: Box::new(status),
    });
    main.run_all();

    assert_eq!(
        view.lines.lock().unwrap().last().unwrap(),
        "Unlock in 4 min"
    );
}

#[test]
fn the_quit_item_warns_while_blocking_is_on() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let handle = tray(&main, &view, Arc::new(store()));
    let mut status = crate::daemon_client::test_support::daemon_status();
    status.enforcement.active = true;

    handle.on_status(&FocusStatus::Connected {
        status: Box::new(status),
    });
    main.run_all();

    assert_eq!(
        view.quit_labels.lock().unwrap().last().unwrap(),
        "Quit Grumpo (stops app and tab blocking)"
    );
}

#[test]
fn the_quit_item_is_plain_when_nothing_is_blocked() {
    let (main, view) = (
        Arc::new(StalledMain::default()),
        Arc::new(RecordingView::default()),
    );
    let handle = tray(&main, &view, Arc::new(store()));

    handle.on_status(&down());
    main.run_all();

    assert_eq!(
        view.quit_labels.lock().unwrap().last().unwrap(),
        "Quit Grumpo"
    );
}
