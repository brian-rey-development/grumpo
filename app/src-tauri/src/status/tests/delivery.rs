use std::sync::Mutex;

use super::*;

#[test]
fn a_slow_delivery_cannot_be_overtaken_by_a_newer_one() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let seen: Arc<Mutex<Vec<FocusStatus>>> = Arc::default();
    let first_call = Arc::new(AtomicUsize::new(0));
    hub.on_change(Box::new({
        let (seen, first_call) = (Arc::clone(&seen), Arc::clone(&first_call));
        move |status| {
            if first_call.fetch_add(1, Ordering::SeqCst) == 0 {
                thread::sleep(Duration::from_millis(200));
            }
            seen.lock().unwrap().push(status.clone());
        }
    }));

    let slow = thread::spawn({
        let hub = Arc::clone(&hub);
        move || hub.refresh()
    });
    eventually(|| first_call.load(Ordering::SeqCst) == 1);
    daemon.respond_with(changed_status());
    hub.refresh();
    slow.join().unwrap();

    let seen = seen.lock().unwrap();
    assert_eq!(seen.last(), Some(&hub.current()));
    assert_eq!(seen.len(), 2);
}

#[test]
fn cleared_listeners_hear_nothing_more() {
    let daemon = FakeDaemon::start();
    let hub = hub_for(&daemon);
    let heard = counting(&hub);
    hub.refresh();

    hub.clear_listeners();
    daemon.respond_with(changed_status());
    hub.refresh();

    assert_eq!(heard.load(Ordering::SeqCst), 1);
}
