use crate::storage::test_support::store;

#[test]
fn a_new_database_has_not_completed_onboarding() {
    assert!(!store().onboarding_completed().unwrap());
}

#[test]
fn completing_and_resetting_round_trip() {
    let store = store();

    store.set_onboarding_completed(true).unwrap();
    assert!(store.onboarding_completed().unwrap());

    store.set_onboarding_completed(false).unwrap();
    assert!(!store.onboarding_completed().unwrap());
}

#[test]
fn an_unrecognized_value_reads_as_not_completed() {
    let store = store();
    store.set_setting("onboarding_completed", "yes").unwrap();
    assert!(!store.onboarding_completed().unwrap());
}
