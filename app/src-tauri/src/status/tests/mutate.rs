use super::*;

#[test]
fn a_successful_mutation_updates_the_status_and_always_notifies() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();

    let sent = hub.send(&Request::Relock).unwrap();

    assert_eq!(sent, hub.current());
    assert_eq!(heard.load(Ordering::SeqCst), 2);
    assert_eq!(daemon.requests(), [Request::Status, Request::Relock]);
}

#[test]
fn a_rejected_mutation_keeps_the_status_and_refreshes_it() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let before = hub.refresh();
    let error = IpcError::new(ErrorCode::UnlockNotReady, "wait");
    daemon.push(Reply::Respond(Box::new(Response::Err { error })));
    daemon.respond_with(changed_status());

    let failure = hub.send(&Request::RequestUnlock).unwrap_err();

    assert!(matches!(failure, DaemonError::Rejected(_)));
    assert_ne!(
        hub.current(),
        before,
        "the refresh picked up the new status"
    );
    assert_eq!(
        daemon.requests(),
        [Request::Status, Request::RequestUnlock, Request::Status]
    );
}

#[test]
fn a_mutation_that_keeps_failing_marks_the_daemon_unavailable_on_the_second_miss() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();
    hang_up(&daemon, 4);

    let first = hub.send(&Request::Relock).unwrap_err();
    assert!(matches!(first, DaemonError::NotResponding(_)));
    assert!(matches!(hub.current(), FocusStatus::Connected { .. }));

    hub.send(&Request::Relock).unwrap_err();

    assert_eq!(
        reason(&hub.current()),
        DaemonUnavailableReason::NotResponding
    );
    assert_eq!(heard.load(Ordering::SeqCst), 2);
}

#[test]
fn a_mutation_that_cannot_be_repeated_is_sent_only_once() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    daemon.push(Reply::HangUp);

    hub.send(&Request::RequestUnlock).unwrap_err();

    assert_eq!(daemon.requests(), [Request::RequestUnlock]);
}

#[test]
fn a_repeatable_mutation_is_tried_again_after_a_blip() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    daemon.push(Reply::HangUp);

    hub.send(&Request::Relock).unwrap();

    assert_eq!(daemon.requests(), [Request::Relock, Request::Relock]);
}
