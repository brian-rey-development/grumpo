use std::sync::Arc;
use std::thread;

use super::*;

fn poisoned() -> Arc<Mutex<u32>> {
    let mutex = Arc::new(Mutex::new(7));
    let held = Arc::clone(&mutex);
    let result = thread::spawn(move || {
        let _guard = held.lock().unwrap();
        panic!("poison the lock");
    })
    .join();
    assert!(result.is_err());
    mutex
}

#[test]
fn a_poisoned_mutex_still_hands_out_its_value() {
    let mutex = poisoned();

    assert!(mutex.lock().is_err());
    assert_eq!(*mutex.lock_or_recover(), 7);
}

#[test]
fn a_timed_wait_on_a_poisoned_mutex_recovers() {
    let mutex = poisoned();
    let ready = Condvar::new();

    let (guard, result) =
        ready.wait_timeout_or_recover(mutex.lock_or_recover(), Duration::from_millis(1));

    assert!(result.timed_out());
    assert_eq!(*guard, 7);
}
