use super::*;

#[test]
fn logs_once_per_run_of_identical_errors() {
    let mut log = ErrorLog::default();
    let failure = Some(PlatformErrorKind::Timeout);
    assert_eq!(
        log.record(Source::Idle, failure),
        Transition::Started(PlatformErrorKind::Timeout)
    );
    assert_eq!(log.record(Source::Idle, failure), Transition::Unchanged);
    assert_eq!(log.record(Source::Idle, failure), Transition::Unchanged);
}

#[test]
fn a_different_kind_is_a_new_transition() {
    let mut log = ErrorLog::default();
    log.record(Source::Media, Some(PlatformErrorKind::Timeout));
    assert_eq!(
        log.record(Source::Media, Some(PlatformErrorKind::Failed)),
        Transition::Started(PlatformErrorKind::Failed)
    );
}

#[test]
fn recovery_is_reported_once_and_success_without_failure_is_silent() {
    let mut log = ErrorLog::default();
    assert_eq!(log.record(Source::Idle, None), Transition::Unchanged);
    log.record(Source::Idle, Some(PlatformErrorKind::Failed));
    assert_eq!(log.record(Source::Idle, None), Transition::Recovered);
    assert_eq!(log.record(Source::Idle, None), Transition::Unchanged);
    assert_eq!(
        log.record(Source::Idle, Some(PlatformErrorKind::Failed)),
        Transition::Started(PlatformErrorKind::Failed)
    );
}

#[test]
fn sources_are_tracked_independently() {
    let mut log = ErrorLog::default();
    log.record(
        Source::BrowserTab(Browser::Chrome),
        Some(PlatformErrorKind::Failed),
    );
    assert_eq!(
        log.record(
            Source::BrowserTab(Browser::Safari),
            Some(PlatformErrorKind::Failed)
        ),
        Transition::Started(PlatformErrorKind::Failed)
    );
    assert_eq!(
        log.record(
            Source::BrowserTab(Browser::Chrome),
            Some(PlatformErrorKind::Failed)
        ),
        Transition::Unchanged
    );
}
