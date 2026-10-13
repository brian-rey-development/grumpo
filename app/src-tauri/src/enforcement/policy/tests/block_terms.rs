//! What a block screen is told about a block, which depends on why the target is blocked.

use focus_core::ipc::{EnforcementReason, UnlockPhaseView};

use super::*;
use crate::dto::BlockReason;

const DISCORD: &str = "com.discord";
const SLACK: &str = "com.tinyspeck.slackmacgap";

fn app_terms(bundle_id: &str, status: &DaemonStatus) -> (BlockReason, bool) {
    let activity = in_app(bundle_id);
    let notice = decide_now(Some(&activity), status)
        .into_iter()
        .find_map(|action| match action {
            Action::Notify(NoticeKind::AppBlocked { reason, strict, .. }) => Some((reason, strict)),
            _ => None,
        });
    notice.expect("a block notice")
}

fn limit_status() -> DaemonStatus {
    let mut status = blocking(&[DISCORD], &[]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.exhausted = vec![app_target(DISCORD)];
    status
}

fn both_status() -> DaemonStatus {
    let mut status = blocking(&[STEAM, DISCORD, SLACK], &[]);
    status.enforcement.reason = EnforcementReason::Both;
    status.rules.blocked_apps = [app(STEAM), app(SLACK)].into();
    status.exhausted = vec![app_target(DISCORD), app_target(SLACK)];
    status
}

#[test]
fn a_scheduled_block_with_an_end_is_a_schedule_block() {
    let mut status = blocking(&[STEAM], &[]);
    status.rules.blocked_apps = [app(STEAM)].into();

    assert_eq!(app_terms(STEAM, &status), (BlockReason::Schedule, true));
}

#[test]
fn a_scheduled_block_that_never_ends_is_an_always_block() {
    let mut status = blocking(&[STEAM], &[]);
    status.rules.blocked_apps = [app(STEAM)].into();
    status.enforcement.active_until = None;

    assert_eq!(app_terms(STEAM, &status).0, BlockReason::Always);
}

#[test]
fn a_spent_limit_is_a_limit_block() {
    assert_eq!(app_terms(DISCORD, &limit_status()).0, BlockReason::Limit);
}

#[test]
fn under_both_reasons_each_target_keeps_its_own() {
    let status = both_status();

    assert_eq!(app_terms(STEAM, &status).0, BlockReason::Schedule);
    assert_eq!(app_terms(DISCORD, &status).0, BlockReason::Limit);
}

#[test]
fn a_target_held_by_both_a_schedule_and_a_limit_reads_as_a_schedule_block() {
    assert_eq!(app_terms(SLACK, &both_status()).0, BlockReason::Schedule);
}

#[test]
fn a_target_listed_under_no_cause_follows_the_reason_of_the_plan() {
    assert_eq!(
        app_terms(STEAM, &blocking(&[STEAM], &[])).0,
        BlockReason::Schedule
    );

    let mut status = blocking(&[STEAM], &[]);
    status.enforcement.reason = EnforcementReason::Limit;
    assert_eq!(app_terms(STEAM, &status).0, BlockReason::Limit);
}

#[test]
fn a_site_block_carries_the_same_terms() {
    let mut status = blocking(&[], &["reddit.com"]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.exhausted = vec![site_target("reddit.com")];
    let visit = browsing(Browser::Chrome, "old.reddit.com");

    let terms = decide_now(Some(&visit), &status)
        .into_iter()
        .find_map(|action| match action {
            Action::Notify(NoticeKind::SiteBlocked { reason, strict, .. }) => {
                Some((reason, strict))
            }
            _ => None,
        });

    assert_eq!(terms, Some((BlockReason::Limit, true)));
}

#[test]
fn a_block_is_strict_while_no_unlock_is_running() {
    let mut status = blocking(&[STEAM], &[]);
    for phase in [
        UnlockPhaseView::Locked,
        UnlockPhaseView::CoolingDown {
            ready_at: END,
            minutes: 5,
        },
        UnlockPhaseView::Ready {
            ready_at: T0,
            expires_at: END,
        },
    ] {
        status.unlock.phase = phase;
        assert!(app_terms(STEAM, &status).1, "{phase:?}");
    }
}

#[test]
fn a_block_is_not_strict_while_an_unlock_is_running() {
    let mut status = blocking(&[STEAM], &[]);
    status.unlock.phase = UnlockPhaseView::Unlocked { until: END };

    assert!(!app_terms(STEAM, &status).1);
}
