use super::*;

#[test]
fn polling_refreshes_repeatedly_and_notifies_once_for_an_unchanged_status() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);

    let poller = hub.start_polling(POLL);
    eventually(|| daemon.requests().len() >= 3);
    daemon.respond_with(changed_status());
    eventually(|| heard.load(Ordering::SeqCst) == 2);
    drop(poller);

    assert!(daemon.requests().iter().all(|r| *r == Request::Status));
}

#[test]
fn dropping_the_poller_stops_the_requests() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let poller = hub.start_polling(POLL);
    eventually(|| !daemon.requests().is_empty());

    drop(poller);
    let after_stop = daemon.requests().len();
    thread::sleep(POLL * 6);

    assert_eq!(daemon.requests().len(), after_stop);
}
