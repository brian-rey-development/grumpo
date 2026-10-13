use std::sync::Arc;

use super::test_support::{domain_of, observer, refresh, tab_reads};
use super::*;
use crate::platform::PlatformErrorKind;
use crate::tracking::tab_memory::MAX_CARRIED_FAILURES;
use crate::tracking::test_support::{FakeProbe, CHROME};

const T0: i64 = 1_000;

#[test]
fn a_disengaged_sample_skips_the_tab_read_and_reuses_the_last_domain() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("youtube.com"));
    observer.sample(T0);
    assert_eq!(tab_reads(&probe), 1);

    probe.set_idle(IDLE_THRESHOLD.as_secs() + 60);
    probe.set_host(Browser::Chrome, Some("docs.rs"));
    let idle = observer.sample(T0 + 60).unwrap();
    assert!(!idle.engaged);
    assert_eq!(domain_of(&idle), Some("youtube.com"));
    assert_eq!(tab_reads(&probe), 1);

    probe.set_idle(0);
    let back = observer.sample(T0 + 61).unwrap();
    assert_eq!(domain_of(&back), Some("docs.rs"));
    assert_eq!(tab_reads(&probe), 2);
}

#[test]
fn the_same_browser_is_read_at_most_once_per_refresh_interval() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("youtube.com"));
    for offset in 0..refresh() {
        observer.sample(T0 + offset);
    }
    assert_eq!(tab_reads(&probe), 1);

    probe.set_host(Browser::Chrome, Some("docs.rs"));
    let carried = observer.sample(T0 + refresh() - 1).unwrap();
    assert_eq!(domain_of(&carried), Some("youtube.com"));
    let refreshed = observer.sample(T0 + refresh()).unwrap();
    assert_eq!(domain_of(&refreshed), Some("docs.rs"));
    assert_eq!(tab_reads(&probe), 2);
}

#[test]
fn switching_to_another_browser_reads_immediately() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("youtube.com"));
    observer.sample(T0);

    probe.set_foreground(Some("com.apple.Safari"));
    probe.set_host(Browser::Safari, Some("docs.rs"));
    let activity = observer.sample(T0 + 1).unwrap();
    assert_eq!(domain_of(&activity), Some("docs.rs"));
    assert_eq!(tab_reads(&probe), 2);
}

#[test]
fn failed_tab_reads_carry_the_previous_domain_a_few_times() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("youtube.com"));
    observer.sample(T0);

    probe.fail_host(Browser::Chrome, PlatformErrorKind::Timeout);
    for failure in 1..=i64::from(MAX_CARRIED_FAILURES) {
        let activity = observer.sample(T0 + failure * refresh()).unwrap();
        assert_eq!(
            domain_of(&activity),
            Some("youtube.com"),
            "failure {failure}"
        );
    }
    let dropped = observer
        .sample(T0 + (i64::from(MAX_CARRIED_FAILURES) + 1) * refresh())
        .unwrap();
    assert_eq!(dropped.domain, None);
}

#[test]
fn a_successful_empty_read_is_not_carried() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("youtube.com"));
    observer.sample(T0);
    probe.set_host(Browser::Chrome, None);
    assert_eq!(observer.sample(T0 + refresh()).unwrap().domain, None);
}

#[test]
fn health_tracks_each_browsers_last_outcome() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    let health = observer.health();
    assert_eq!(health.snapshot().outcome(Browser::Chrome), None);

    probe.set_foreground(Some(CHROME));
    probe.fail_host(Browser::Chrome, PlatformErrorKind::PermissionDenied);
    observer.sample(T0);
    assert_eq!(health.snapshot().permission_denied(), [Browser::Chrome]);

    probe.set_host(Browser::Chrome, Some("example.com"));
    observer.sample(T0 + refresh());
    assert_eq!(
        health.snapshot().outcome(Browser::Chrome),
        Some(ProbeOutcome::Ok)
    );
    assert_eq!(health.snapshot().permission_denied().len(), 0);
}

#[test]
fn a_browser_that_still_needs_consent_is_reported_as_such() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some(CHROME));
    probe.fail_host(Browser::Chrome, PlatformErrorKind::NeedsConsent);
    observer.sample(T0);
    let snapshot = observer.health().snapshot();
    assert_eq!(
        snapshot.outcome(Browser::Chrome),
        Some(ProbeOutcome::NeedsConsent)
    );
    assert_eq!(snapshot.needs_consent(), [Browser::Chrome]);
}

#[test]
fn unsupported_and_failed_tab_reads_are_recorded_distinctly() {
    let probe = Arc::new(FakeProbe::new());
    let mut observer = observer(&probe);
    probe.set_foreground(Some("org.mozilla.firefox"));
    probe.fail_host(Browser::Firefox, PlatformErrorKind::Unsupported);
    observer.sample(T0);
    assert_eq!(
        observer.health().snapshot().outcome(Browser::Firefox),
        Some(ProbeOutcome::Unsupported)
    );
    probe.fail_host(Browser::Firefox, PlatformErrorKind::Timeout);
    observer.sample(T0 + refresh());
    assert_eq!(
        observer.health().snapshot().outcome(Browser::Firefox),
        Some(ProbeOutcome::Failed)
    );
}
