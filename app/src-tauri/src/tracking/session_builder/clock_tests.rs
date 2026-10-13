use super::test_support::activity;
use super::*;
use focus_core::test_support::app;

fn opened(bundle_id: &str, at: i64) -> SessionEvent {
    SessionEvent::Opened(NewSession {
        app: app(bundle_id),
        domain: None,
        started_at: at,
    })
}

fn extended(ended_at: i64) -> SessionEvent {
    SessionEvent::Extended { ended_at }
}

#[test]
fn a_clock_step_back_never_shrinks_or_overlaps_the_open_session() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    assert_eq!(builder.feed(Some(&a), 90).len(), 0);
    assert_eq!(builder.feed(Some(&a), 101), [extended(101)]);
}

#[test]
fn a_switch_right_after_a_clock_step_back_starts_at_the_last_seen_time() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    let events = builder.feed(Some(&activity("com.b", None, true)), 90);
    assert_eq!(events, [SessionEvent::Closed, opened("com.b", 100)]);
}

#[test]
fn a_new_session_never_starts_before_the_previous_one_ended() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    builder.feed(Some(&a), 104);
    builder.feed(None, 106);
    let events = builder.feed(Some(&a), 95);
    assert_eq!(events, [opened("com.a", 104)]);
}

#[test]
fn a_gap_after_a_clock_step_back_still_reopens_at_the_clamped_time() {
    let mut builder = SessionBuilder::new();
    let a = activity("com.a", None, true);
    builder.feed(Some(&a), 100);
    let events = builder.feed(Some(&a), 100 + MAX_SAMPLE_GAP + 50);
    assert_eq!(
        events,
        [
            SessionEvent::Closed,
            opened("com.a", 100 + MAX_SAMPLE_GAP + 50)
        ]
    );
}
