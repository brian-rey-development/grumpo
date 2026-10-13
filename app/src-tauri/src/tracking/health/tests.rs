use super::*;

#[test]
fn records_the_latest_outcome_per_browser() {
    let handle = HealthHandle::default();
    assert_eq!(handle.snapshot().outcome(Browser::Chrome), None);

    handle.record(Browser::Chrome, ProbeOutcome::PermissionDenied);
    handle.record(Browser::Safari, ProbeOutcome::Ok);
    handle.record(Browser::Brave, ProbeOutcome::PermissionDenied);
    assert_eq!(
        handle.snapshot().permission_denied(),
        [Browser::Chrome, Browser::Brave]
    );

    handle.record(Browser::Chrome, ProbeOutcome::Ok);
    let snapshot = handle.snapshot();
    assert_eq!(snapshot.outcome(Browser::Chrome), Some(ProbeOutcome::Ok));
    assert_eq!(snapshot.permission_denied(), [Browser::Brave]);
}

#[test]
fn consent_and_denial_are_listed_separately() {
    let handle = HealthHandle::default();
    handle.record(Browser::Chrome, ProbeOutcome::NeedsConsent);
    handle.record(Browser::Safari, ProbeOutcome::PermissionDenied);
    let snapshot = handle.snapshot();
    assert_eq!(snapshot.needs_consent(), [Browser::Chrome]);
    assert_eq!(snapshot.permission_denied(), [Browser::Safari]);
}

#[test]
fn clones_share_state() {
    let handle = HealthHandle::default();
    let clone = handle.clone();
    clone.record(Browser::Edge, ProbeOutcome::Failed);
    assert_eq!(
        handle.snapshot().outcome(Browser::Edge),
        Some(ProbeOutcome::Failed)
    );
}
