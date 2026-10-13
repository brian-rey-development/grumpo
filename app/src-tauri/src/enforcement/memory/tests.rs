use chrono::NaiveDate;
use focus_core::Limit;

use super::*;
use crate::dto::BlockReason;
use crate::enforcement::test_support::{app, app_target, site, T0};
use crate::platform::Browser;

fn day(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2027, 1, day).unwrap()
}

fn outcome(now: i64) -> Outcome {
    Outcome {
        now,
        succeeded: true,
    }
}

fn report(minutes: u16) -> Action {
    Action::ReportLimitReached {
        target: app_target("com.valvesoftware.steam"),
        daily_minutes: minutes,
    }
}

fn quit() -> Action {
    Action::QuitApp(app("com.valvesoftware.steam"))
}

fn blocked_notice(id: &str) -> Action {
    Action::Notify(NoticeKind::AppBlocked {
        app: app(id),
        until: None,
        reason: BlockReason::Schedule,
        strict: true,
    })
}

fn warning() -> Action {
    Action::Notify(NoticeKind::LimitWarning {
        target: app_target("com.valvesoftware.steam"),
        seconds_left: 300,
    })
}

fn limit(minutes: u16) -> Limit {
    focus_core::test_support::limit(app_target("com.valvesoftware.steam"), minutes)
}

#[test]
fn a_fresh_memory_allows_everything() {
    let memory = EnforcementMemory::new(day(15));

    for action in [quit(), blocked_notice("com.a"), warning()] {
        assert!(memory.allows(&action, T0), "{action:?}");
    }
}

#[test]
fn an_action_is_allowed_again_exactly_at_the_retry_interval() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&quit(), outcome(T0));

    assert!(!memory.allows(&quit(), T0 + ACTION_RETRY_SECONDS - 1));
    assert!(memory.allows(&quit(), T0 + ACTION_RETRY_SECONDS));
}

#[test]
fn a_notice_is_allowed_again_exactly_after_a_minute() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&blocked_notice("com.a"), outcome(T0));

    assert!(!memory.allows(&blocked_notice("com.a"), T0 + NOTICE_INTERVAL_SECONDS - 1));
    assert!(memory.allows(&blocked_notice("com.a"), T0 + NOTICE_INTERVAL_SECONDS));
    assert!(
        memory.allows(&blocked_notice("com.b"), T0),
        "other apps are independent"
    );
}

#[test]
fn sites_and_apps_are_remembered_separately() {
    let mut memory = EnforcementMemory::new(day(15));
    let close = Action::CloseTab {
        browser: Browser::Chrome,
        domain: site("youtube.com"),
    };
    memory.note(&close, outcome(T0));

    assert!(!memory.allows(&close, T0 + 1));
    assert!(memory.allows(&quit(), T0 + 1));
}

#[test]
fn a_clock_that_moves_backwards_does_not_lock_actions_out() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&quit(), outcome(T0));

    assert!(memory.allows(&quit(), T0 - 3_600));
}

#[test]
fn a_warning_is_allowed_until_it_is_given() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&warning(), outcome(T0));

    assert!(!memory.allows(&warning(), T0 + 86_000));
}

#[test]
fn rolling_to_the_same_day_keeps_the_memory() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&warning(), outcome(T0));

    assert!(!memory.roll_to(day(15)));
    assert!(!memory.allows(&warning(), T0));
}

#[test]
fn local_midnight_forgets_everything() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&warning(), outcome(T0));
    memory.note(&quit(), outcome(T0));
    memory.note(&blocked_notice("com.a"), outcome(T0));
    memory.note(&report(30), outcome(T0));

    assert!(memory.roll_to(day(16)));

    assert!(memory.allows(&warning(), T0 + 1));
    assert!(memory.allows(&quit(), T0 + 1));
    assert!(memory.allows(&blocked_notice("com.a"), T0 + 1));
    assert!(!memory.limit_reported(&limit(30)));
    assert_eq!(memory, EnforcementMemory::new(day(16)));
}

#[test]
fn a_date_that_moves_backwards_also_resets() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&warning(), outcome(T0));

    assert!(memory.roll_to(day(14)));
    assert!(memory.allows(&warning(), T0));
}

#[test]
fn only_a_successful_report_is_remembered_and_only_for_that_allowance() {
    let mut memory = EnforcementMemory::new(day(15));
    let report = report(30);
    let attempt = |succeeded| Outcome { now: T0, succeeded };

    memory.note(&report, attempt(false));
    assert!(!memory.limit_reported(&limit(30)));

    memory.note(&report, attempt(true));
    assert!(memory.limit_reported(&limit(30)));
    assert!(
        !memory.limit_reported(&limit(45)),
        "a raised limit is a new allowance"
    );
}

#[test]
fn a_report_for_a_new_allowance_replaces_the_old_one() {
    let mut memory = EnforcementMemory::new(day(15));

    memory.note(&report(30), outcome(T0));
    memory.note(&report(45), outcome(T0 + 60));

    assert!(memory.limit_reported(&limit(45)));
    assert!(!memory.limit_reported(&limit(30)));
}

#[test]
fn reports_are_never_rate_limited_by_time() {
    let mut memory = EnforcementMemory::new(day(15));
    memory.note(&report(30), outcome(T0));

    assert!(memory.allows(&report(30), T0));
}
