use super::test_support::{activity, idle_for, opened, private, threshold};
use super::*;
use crate::tracking::test_support::CHROME;

/// Samples every two seconds from `from` to `to`, all engaged with the last input at
/// `last_input`, as a person reading without touching anything would produce.
fn read_without_input(builder: &mut SessionBuilder, last_input: i64, from: i64, to: i64) {
    for at in (from..=to).step_by(2) {
        let idle = u64::try_from(at - last_input).unwrap();
        builder.feed(Some(&idle_for(activity("com.a", None, true), idle)), at);
    }
}

fn gave_up_at(at: i64, last_input: i64) -> Activity {
    let idle = u64::try_from(at - last_input).unwrap();
    idle_for(activity("com.a", None, false), idle)
}

#[test]
fn going_idle_trims_the_session_back_to_the_last_input() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    read_without_input(&mut builder, 100, 102, 218);
    let events = builder.feed(Some(&gave_up_at(220, 100)), 220);
    assert_eq!(
        events,
        [
            SessionEvent::Trimmed { ended_at: 100 },
            SessionEvent::Closed
        ]
    );
    assert!(!builder.is_open());
}

#[test]
fn the_trim_keeps_everything_up_to_the_last_input() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    read_without_input(&mut builder, 100, 102, 150);
    builder.feed(Some(&activity("com.a", None, true)), 150);
    read_without_input(&mut builder, 150, 152, 268);
    let events = builder.feed(Some(&gave_up_at(270, 150)), 270);
    assert_eq!(
        events,
        [
            SessionEvent::Trimmed { ended_at: 150 },
            SessionEvent::Closed
        ]
    );
}

#[test]
fn media_keeps_the_time_it_covered_when_the_idle_session_ends() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    read_without_input(&mut builder, 100, 102, 218);
    read_without_input(&mut builder, 100, 220, 230);
    let events = builder.feed(Some(&gave_up_at(232, 100)), 232);
    assert_eq!(
        events,
        [
            SessionEvent::Trimmed { ended_at: 230 },
            SessionEvent::Closed
        ]
    );
}

#[test]
fn the_trim_never_reaches_before_the_session_started() {
    let mut builder = SessionBuilder::new();
    let watching = idle_for(activity("com.a", None, true), threshold() + 500);
    builder.feed(Some(&watching), 1_000);
    assert_eq!(
        builder.feed(Some(&gave_up_at(1_002, 380)), 1_002),
        [
            SessionEvent::Trimmed { ended_at: 1_000 },
            SessionEvent::Closed
        ]
    );
}

#[test]
fn a_session_can_reopen_right_after_an_idle_trim() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    builder.feed(Some(&gave_up_at(104, -16)), 104);
    let back = builder.feed(Some(&activity("com.a", None, true)), 106);
    assert_eq!(back, [opened("com.a", None, 106)]);
}

#[test]
fn a_locked_screen_closes_at_the_last_seen_time_without_a_trim() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity("com.a", None, true)), 100);
    builder.feed(Some(&activity("com.a", None, true)), 102);
    let locked = idle_for(activity("com.a", None, false), 1);
    assert_eq!(builder.feed(Some(&locked), 104), [SessionEvent::Closed]);
}

#[test]
fn private_activity_is_never_stored_and_closes_the_open_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&activity(CHROME, Some("a.com"), true)), 100);
    let hidden = private(activity(CHROME, Some("a.com"), true));
    assert_eq!(builder.feed(Some(&hidden), 102), [SessionEvent::Closed]);
    assert_eq!(builder.feed(Some(&hidden), 104).len(), 0);
    assert!(!builder.is_open());
}

#[test]
fn leaving_a_private_window_starts_a_fresh_session() {
    let mut builder = SessionBuilder::new();
    builder.feed(Some(&private(activity(CHROME, Some("a.com"), true))), 100);
    let events = builder.feed(Some(&activity(CHROME, Some("a.com"), true)), 102);
    assert_eq!(events, [opened(CHROME, Some("a.com"), 102)]);
}
