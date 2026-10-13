use super::test_support::*;
use super::*;
use crate::dto::PermissionStatus;
use crate::platform::PermissionState;
use crate::tracking::health::{ObserverHealth, ProbeOutcome};

#[test]
fn installed_comes_from_the_locator_by_bundle_id() {
    let locator = FakeLocator {
        installed: vec!["com.apple.Safari"],
        ..FakeLocator::default()
    };
    let health = ObserverHealth::default();

    let safari = access(&locator, PermissionState::Granted, &health, Browser::Safari);
    let chrome = access(&locator, PermissionState::Granted, &health, Browser::Chrome);

    assert!(safari.installed);
    assert!(!chrome.installed);
    assert_eq!(
        *locator.asked.lock().unwrap(),
        ["com.apple.Safari", "com.google.Chrome"]
    );
}

#[test]
fn a_browser_that_is_not_running_is_reported_as_such() {
    let access = access(
        &FakeLocator::default(),
        PermissionState::NotRunning,
        &ObserverHealth::default(),
        Browser::Brave,
    );
    assert_eq!(access.permission, PermissionStatus::NotRunning);
}

#[test]
fn a_refusal_the_tracker_saw_refines_an_undetermined_probe_only() {
    let health = ObserverHealth::from_outcomes(&[(Browser::Edge, ProbeOutcome::PermissionDenied)]);
    let locator = FakeLocator::default();

    let undetermined = access(
        &locator,
        PermissionState::NotDetermined,
        &health,
        Browser::Edge,
    );
    let granted = access(&locator, PermissionState::Granted, &health, Browser::Edge);

    assert_eq!(undetermined.permission, PermissionStatus::Denied);
    assert_eq!(granted.permission, PermissionStatus::Granted);
}
