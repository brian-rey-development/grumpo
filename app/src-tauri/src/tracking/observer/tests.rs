use std::sync::atomic::Ordering;
use std::sync::Arc;

use super::test_support::*;
use super::*;
use crate::platform::PlatformErrorKind;
use crate::tracking::test_support::{FakeProbe, CHROME, FOREGROUND_PID};
use focus_core::test_support::app;

#[test]
fn no_foreground_app_means_no_activity() {
    let probe = Arc::new(FakeProbe::new());
    assert_eq!(observer(&probe).sample(T0), None);
}

#[test]
fn a_foreground_error_means_no_activity() {
    let probe = Arc::new(FakeProbe::new());
    probe.fail_foreground(PlatformErrorKind::Failed);
    assert_eq!(observer(&probe).sample(T0), None);
}

#[test]
fn a_regular_app_is_engaged_without_a_domain_or_tab_read() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some("com.apple.dt.Xcode"));
    let activity = observer(&probe).sample(T0).unwrap();
    assert_eq!(activity.app, app("com.apple.dt.Xcode"));
    assert_eq!(
        (activity.domain, activity.browser, activity.engaged),
        (None, None, true)
    );
    assert!(!activity.private);
    assert_eq!(tab_reads(&probe), 0);
}

#[test]
fn a_browser_reports_the_domain_of_its_tab() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some(CHROME));
    probe.set_host(Browser::Chrome, Some("https://www.YouTube.com/watch?v=1"));
    let activity = observer(&probe).sample(T0).unwrap();
    assert_eq!(activity.browser, Some(Browser::Chrome));
    assert_eq!(domain_of(&activity), Some("youtube.com"));
}

#[test]
fn a_browser_without_a_tab_has_no_domain() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some("com.apple.Safari"));
    probe.set_host(Browser::Safari, None);
    assert_eq!(observer(&probe).sample(T0).unwrap().domain, None);
}

#[test]
fn a_private_window_is_flagged_but_keeps_its_domain_for_enforcement() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some(CHROME));
    probe.set_tab(Browser::Chrome, Some("youtube.com"), true);
    let activity = observer(&probe).sample(T0).unwrap();
    assert!(activity.private);
    assert_eq!(domain_of(&activity), Some("youtube.com"));
}

#[test]
fn idle_below_the_threshold_is_engaged_and_skips_the_media_check() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some("com.a"));
    probe.set_idle(IDLE_THRESHOLD.as_secs() - 1);
    assert!(observer(&probe).sample(T0).unwrap().engaged);
    assert_eq!(probe.media_reads.load(Ordering::SeqCst), 0);
}

#[test]
fn idle_at_the_threshold_disengages_unless_the_frontmost_app_plays_media() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some("com.a"));
    probe.set_idle(IDLE_THRESHOLD.as_secs());
    let idle = observer(&probe).sample(T0).unwrap();
    assert!(!idle.engaged);
    assert_eq!(idle.idle, IDLE_THRESHOLD);
    assert!(idle.went_idle());

    probe.set_media(true);
    let watching = observer(&probe).sample(T0).unwrap();
    assert!(watching.engaged && watching.credited_by_media());
    assert_eq!(probe.media_pid.load(Ordering::SeqCst), FOREGROUND_PID);
}

#[test]
fn a_locked_screen_is_disengaged_even_with_recent_input_and_media() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some(CHROME));
    probe.set_locked(true);
    probe.set_media(true);
    probe.set_idle(IDLE_THRESHOLD.as_secs() + 5);
    let activity = observer(&probe).sample(T0).unwrap();
    assert!(!activity.engaged);

    probe.set_idle(1);
    assert!(!observer(&probe).sample(T0).unwrap().engaged);
}

#[test]
fn probe_errors_degrade_without_dropping_the_activity() {
    let probe = Arc::new(FakeProbe::new());
    probe.set_foreground(Some(CHROME));
    probe.fail_idle(PlatformErrorKind::Timeout);
    probe.fail_locked(PlatformErrorKind::Failed);
    probe.fail_host(Browser::Chrome, PlatformErrorKind::Failed);
    let activity = observer(&probe).sample(T0).unwrap();
    assert!(activity.engaged);
    assert_eq!(activity.domain, None);

    probe.set_idle(500);
    probe.fail_media(PlatformErrorKind::Failed);
    assert!(!observer(&probe).sample(T0).unwrap().engaged);
}
