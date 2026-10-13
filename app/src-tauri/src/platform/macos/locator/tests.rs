use super::*;

#[test]
fn finder_is_installed_and_a_made_up_app_is_not() {
    assert!(MacLocator.is_installed("com.apple.finder"));
    assert!(!MacLocator.is_installed("invalid.example.not-an-app"));
}
