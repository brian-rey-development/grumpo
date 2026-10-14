use crate::storage::test_support::store;

#[test]
fn a_new_database_updates_automatically() {
    assert!(store().auto_update().unwrap());
}

#[test]
fn turning_updates_off_and_on_round_trips() {
    let store = store();

    store.set_auto_update(false).unwrap();
    assert!(!store.auto_update().unwrap());

    store.set_auto_update(true).unwrap();
    assert!(store.auto_update().unwrap());
}

#[test]
fn an_unrecognized_value_keeps_updates_on() {
    let store = store();
    store.set_setting("auto_update", "maybe").unwrap();
    assert!(store.auto_update().unwrap());
}
