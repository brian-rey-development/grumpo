use std::thread;

use super::*;

#[test]
fn gives_mutable_access_across_clones() {
    let shared = Shared::new(1);
    let other = shared.clone();
    other.with(|value| *value += 1);
    assert_eq!(shared.with(|value| *value), 2);
}

#[test]
fn recovers_after_a_panic_while_locked() {
    let shared = Shared::new(String::from("kept"));
    let clone = shared.clone();
    let result = thread::spawn(move || clone.with(|_| panic!("boom"))).join();
    assert!(result.is_err());
    assert_eq!(shared.with(|value| value.clone()), "kept");
    assert_eq!(shared.with(|value| value.clone()), "kept");
}
