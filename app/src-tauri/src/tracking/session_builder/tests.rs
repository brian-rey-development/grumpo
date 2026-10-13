use super::test_support::*;
use super::*;
use crate::tracking::test_support::CHROME;

#[test]
fn nothing_happens_without_an_engaged_activity() {
    let mut builder = SessionBuilder::new();
    assert_eq!(builder.feed(None, 0).len(), 0);
    assert_eq!(
        builder.feed(Some(&activity("com.a", None, false)), 2).len(),
        0
    );
    assert!(!builder.is_open());
}

#[test]
fn the_first_engaged_sample_opens_a_session() {
    let mut builder = SessionBuilder::new();
    let events = builder.feed(Some(&activity("com.a", None, true)), 100);
    assert_eq!(events, [opened("com.a", None, 100)]);
    assert!(builder.is_open());
}

#[test]
fn the_same_target_extends_the_session() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    assert_eq!(builder.feed(Some(&a), 102), [extended(102)]);
    assert_eq!(builder.feed(Some(&a), 104), [extended(104)]);
}

#[test]
fn a_repeated_timestamp_emits_nothing() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    assert_eq!(builder.feed(Some(&a), 100).len(), 0);
    assert_eq!(builder.feed(Some(&a), 102), [extended(102)]);
}

#[test]
fn a_gap_of_exactly_the_maximum_still_continues() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    assert_eq!(
        builder.feed(Some(&a), 100 + MAX_SAMPLE_GAP),
        [extended(106)]
    );
}

#[test]
fn changing_app_extends_to_the_switch_closes_and_opens() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    let events = builder.feed(Some(&activity("com.b", None, true)), 102);
    assert_eq!(
        events,
        [
            extended(102),
            SessionEvent::Closed,
            opened("com.b", None, 102)
        ]
    );
}

#[test]
fn changing_domain_within_a_browser_starts_a_new_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity(CHROME, Some("youtube.com"), true)), 100);
    let events = builder.feed(Some(&activity(CHROME, Some("docs.rs"), true)), 102);
    assert_eq!(
        events,
        [
            extended(102),
            SessionEvent::Closed,
            opened(CHROME, Some("docs.rs"), 102)
        ]
    );
}

#[test]
fn losing_the_domain_is_a_change() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity(CHROME, Some("youtube.com"), true)), 100);
    let events = builder.feed(Some(&activity(CHROME, None, true)), 102);
    assert_eq!(
        events,
        [
            extended(102),
            SessionEvent::Closed,
            opened(CHROME, None, 102)
        ]
    );
}

#[test]
fn a_renamed_app_with_the_same_bundle_id_is_the_same_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    let mut renamed = activity("com.a", None, true);
    renamed.app = focus_core::AppRef::new("com.a", "Brand New Name").unwrap();
    assert_eq!(builder.feed(Some(&renamed), 102), [extended(102)]);
}

#[test]
fn disengaging_closes_without_extending() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    builder.feed(Some(&activity("com.a", None, true)), 102);
    let events = builder.feed(Some(&activity("com.a", None, false)), 104);
    assert_eq!(events, [SessionEvent::Closed]);
    assert!(!builder.is_open());
}

#[test]
fn no_foreground_app_closes_the_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    assert_eq!(builder.feed(None, 102), [SessionEvent::Closed]);
    assert_eq!(builder.feed(None, 104).len(), 0);
}

#[test]
fn a_gap_closes_the_old_session_and_reopens_at_the_current_time() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    builder.feed(Some(&a), 102);
    let events = builder.feed(Some(&a), 102 + MAX_SAMPLE_GAP + 1);
    assert_eq!(events, [SessionEvent::Closed, opened("com.a", None, 109)]);
}

#[test]
fn a_gap_followed_by_disengagement_only_closes() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    let events = builder.feed(Some(&activity("com.a", None, false)), 5_000);
    assert_eq!(events, [SessionEvent::Closed]);
}

#[test]
fn a_gap_with_a_different_app_never_extends_the_old_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    let events = builder.feed(Some(&activity("com.b", None, true)), 5_000);
    assert_eq!(events, [SessionEvent::Closed, opened("com.b", None, 5_000)]);
}

#[test]
fn the_same_app_can_reopen_after_disengaging() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    builder.feed(Some(&activity("com.a", None, false)), 102);
    let events = builder.feed(Some(&activity("com.a", None, true)), 104);
    assert_eq!(events, [opened("com.a", None, 104)]);
}
