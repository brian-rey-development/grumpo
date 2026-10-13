use super::*;

#[test]
fn the_hub_starts_unavailable_until_the_first_refresh() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);

    assert!(matches!(hub.current(), FocusStatus::Unavailable { .. }));
    assert_eq!(daemon.requests().len(), 0);
}

#[test]
fn refresh_stores_the_status_and_notifies_only_when_it_changes() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);

    let first = hub.refresh();
    hub.refresh();

    assert!(matches!(first, FocusStatus::Connected { .. }));
    assert_eq!(hub.current(), first);
    assert_eq!(heard.load(Ordering::SeqCst), 1);
    assert_eq!(daemon.requests(), [Request::Status, Request::Status]);
}

#[test]
fn a_changed_daemon_status_notifies_again() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();

    daemon.respond_with(changed_status());
    hub.refresh();

    assert_eq!(heard.load(Ordering::SeqCst), 2);
}

#[test]
fn a_missing_daemon_becomes_not_installed_with_a_message() {
    let dir = tempfile::tempdir().unwrap();
    let hub = StatusHub::new(Arc::new(DaemonClient::new(dir.path().join("none.sock"))));
    let heard = counting(&hub);

    let status = hub.refresh();
    hub.refresh();

    assert_eq!(reason(&status), DaemonUnavailableReason::NotInstalled);
    assert!(matches!(&status, FocusStatus::Unavailable { message, .. } if !message.is_empty()));
    assert_eq!(heard.load(Ordering::SeqCst), 1);
}

#[test]
fn a_version_mismatch_is_reported_as_such() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::Raw(
        b"{\"protocolVersion\":7,\"body\":{}}\n".to_vec(),
    ));

    let status = hub_for(&daemon).refresh();

    assert_eq!(reason(&status), DaemonUnavailableReason::AppOutdated);
}

#[test]
fn an_older_service_is_reported_as_needing_an_update() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::Raw(
        b"{\"protocolVersion\":1,\"body\":{}}\n".to_vec(),
    ));

    let status = hub_for(&daemon).refresh();

    assert_eq!(reason(&status), DaemonUnavailableReason::ServiceOutdated);
}

#[test]
fn losing_the_daemon_after_connecting_notifies_and_reconnecting_notifies_again() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();

    hang_up(&daemon, 4);
    hub.refresh();
    let lost = hub.refresh();
    let back = hub.refresh();

    assert_eq!(reason(&lost), DaemonUnavailableReason::NotResponding);
    assert!(matches!(back, FocusStatus::Connected { .. }));
    assert_eq!(heard.load(Ordering::SeqCst), 3);
}

#[test]
fn a_panicking_listener_does_not_stop_the_others() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    hub.on_change(Box::new(|_| panic!("boom")));
    let heard = counting(&hub);

    hub.refresh();

    assert_eq!(heard.load(Ordering::SeqCst), 1);
}
