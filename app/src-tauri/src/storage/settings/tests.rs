use crate::storage::test_support::store;

#[test]
fn missing_settings_read_as_none() {
    assert_eq!(store().setting("nothing").unwrap(), None);
}

#[test]
fn setting_twice_overwrites() {
    let store = store();
    store.set_setting("k", "one").unwrap();
    store.set_setting("k", "two").unwrap();
    assert_eq!(store.setting("k").unwrap().as_deref(), Some("two"));
    assert_eq!(store.all_settings().unwrap().len(), 1);
}
