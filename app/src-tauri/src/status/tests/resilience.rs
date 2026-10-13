use super::*;
use crate::time::test_support::FakeClock;

#[test]
fn one_blip_is_retried_and_never_reaches_the_listeners() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();

    hang_up(&daemon, 1);
    let status = hub.refresh();

    assert!(matches!(status, FocusStatus::Connected { .. }));
    assert_eq!(heard.load(Ordering::SeqCst), 1);
    assert_eq!(daemon.requests().len(), 3);
}

#[test]
fn a_single_failed_poll_keeps_the_connected_status() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    let connected = hub.refresh();

    hang_up(&daemon, 2);
    hub.refresh();

    assert_eq!(hub.current(), connected);
    assert_eq!(heard.load(Ordering::SeqCst), 1);
}

#[test]
fn two_failed_polls_in_a_row_make_the_service_unavailable() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    hub.refresh();

    hang_up(&daemon, 4);
    hub.refresh();
    hub.refresh();

    assert_eq!(
        reason(&hub.current()),
        DaemonUnavailableReason::NotResponding
    );
}

#[test]
fn a_success_between_two_failures_resets_the_count() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    hub.refresh();

    hang_up(&daemon, 2);
    hub.refresh();
    hub.refresh();
    hang_up(&daemon, 2);
    hub.refresh();

    assert!(matches!(hub.current(), FocusStatus::Connected { .. }));
}

#[test]
fn a_service_that_was_never_reached_is_unavailable_at_once() {
    let dir = tempfile::tempdir().unwrap();
    let hub = StatusHub::new(Arc::new(DaemonClient::new(dir.path().join("none.sock"))));

    let status = hub.refresh();

    assert_eq!(reason(&status), DaemonUnavailableReason::NotInstalled);
}

#[test]
fn the_snapshot_says_when_the_daemon_last_confirmed_the_status() {
    let daemon = FakeDaemon::start();
    let hub = StatusHub::with_clock(Arc::new(daemon.client()), Arc::new(FakeClock::at(1_234)));
    assert_eq!(hub.snapshot().1, None);

    hub.refresh();

    assert_eq!(hub.snapshot().1, Some(1_234));
}
