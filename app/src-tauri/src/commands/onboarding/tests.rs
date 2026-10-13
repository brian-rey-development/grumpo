use super::*;
use crate::storage::test_support::store;

#[test]
fn a_first_launch_has_not_completed_onboarding() {
    assert_eq!(
        read(&store()).unwrap(),
        OnboardingState { completed: false }
    );
}

#[test]
fn completing_is_remembered_and_idempotent() {
    let store = store();

    assert!(write(&store, true).unwrap().completed);
    assert!(write(&store, true).unwrap().completed);
    assert!(read(&store).unwrap().completed);
}

#[test]
fn resetting_shows_the_welcome_flow_again() {
    let store = store();
    write(&store, true).unwrap();

    assert!(!write(&store, false).unwrap().completed);
    assert!(!read(&store).unwrap().completed);
}

#[test]
fn the_state_is_sent_with_a_completed_field() {
    let json = serde_json::to_string(&OnboardingState { completed: true }).unwrap();
    assert_eq!(json, r#"{"completed":true}"#);
}
