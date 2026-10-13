//! What the notice says about when a block ends, which differs per target when a schedule
//! and a spent limit are in force together.

use focus_core::ipc::EnforcementReason;

use super::*;

const DISCORD: &str = "com.discord";
const SLACK: &str = "com.tinyspeck.slackmacgap";
const LATER: i64 = MIDNIGHT + 3_600;

/// Steam is blocked by the schedule, Discord by a spent limit, and Slack by both.
fn both() -> DaemonStatus {
    let mut status = blocking(&[STEAM, DISCORD, SLACK], &[]);
    status.enforcement.reason = EnforcementReason::Both;
    status.enforcement.active_until = Some(LATER);
    status.rules.blocked_apps = [app(STEAM), app(SLACK)].into();
    status.exhausted = vec![app_target(DISCORD), app_target(SLACK)];
    status
}

fn until_for(bundle_id: &str, status: &DaemonStatus, ends: BlockEnds) -> Option<i64> {
    let memory = memory();
    let activity = in_app(bundle_id);
    let mut input = input_for(Some(&activity), Some(status), &[], &memory, T0);
    input.ends = ends;
    let notice = decide(&input).into_iter().find_map(|action| match action {
        Action::Notify(NoticeKind::AppBlocked { until, .. }) => Some(until),
        _ => None,
    });
    notice.expect("a notice")
}

#[test]
fn a_scheduled_target_ends_with_its_window_and_never_claims_midnight() {
    let ends = BlockEnds {
        schedule: Some(END),
        midnight: MIDNIGHT,
    };
    assert_eq!(until_for(STEAM, &both(), ends), Some(END));
}

#[test]
fn a_target_blocked_by_a_spent_limit_ends_at_midnight() {
    let ends = BlockEnds {
        schedule: Some(END),
        midnight: MIDNIGHT,
    };
    assert_eq!(until_for(DISCORD, &both(), ends), Some(MIDNIGHT));
}

#[test]
fn a_target_blocked_by_both_ends_when_the_whole_block_does() {
    let ends = BlockEnds {
        schedule: Some(END),
        midnight: MIDNIGHT,
    };
    assert_eq!(until_for(SLACK, &both(), ends), Some(LATER));
}

#[test]
fn a_schedule_that_never_ends_leaves_the_time_out() {
    let ends = BlockEnds {
        schedule: None,
        midnight: MIDNIGHT,
    };
    assert_eq!(until_for(STEAM, &both(), ends), None);
}

#[test]
fn with_one_reason_the_daemon_end_is_used_as_is() {
    let ends = BlockEnds {
        schedule: Some(1),
        midnight: 2,
    };
    assert_eq!(until_for(STEAM, &blocking(&[STEAM], &[]), ends), Some(END));

    let mut limit = blocking(&[DISCORD], &[]);
    limit.enforcement.reason = EnforcementReason::Limit;
    limit.exhausted = vec![app_target(DISCORD)];
    assert_eq!(until_for(DISCORD, &limit, ends), Some(END));
}

#[test]
fn a_site_under_both_reasons_gets_its_own_end() {
    let mut status = blocking(&[], &["youtube.com", "reddit.com"]);
    status.enforcement.reason = EnforcementReason::Both;
    status.rules.blocked_sites = [site("youtube.com")].into();
    status.exhausted = vec![site_target("reddit.com")];
    let ends = BlockEnds {
        schedule: Some(END),
        midnight: MIDNIGHT,
    };
    let memory = memory();
    let until = |host: &str| {
        let visit = browsing(Browser::Chrome, host);
        let mut input = input_for(Some(&visit), Some(&status), &[], &memory, T0);
        input.ends = ends;
        decide(&input).into_iter().find_map(|action| match action {
            Action::Notify(NoticeKind::SiteBlocked { until, .. }) => Some(until),
            _ => None,
        })
    };

    assert_eq!(until("m.youtube.com"), Some(Some(END)));
    assert_eq!(until("old.reddit.com"), Some(Some(MIDNIGHT)));
}

#[test]
fn a_block_that_rests_only_on_yesterdays_limit_is_not_enforced_before_the_daemon_confirms_it() {
    let mut status = blocking(&[DISCORD], &[]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.exhausted = vec![app_target(DISCORD)];

    assert_eq!(decide_stale(Some(&in_app(DISCORD)), &status), []);
    assert_eq!(sweep_stale(&[app(DISCORD)], &status), []);
    assert_eq!(decide_now(Some(&in_app(DISCORD)), &status).len(), 2);
}

#[test]
fn a_scheduled_block_is_enforced_even_when_the_status_is_old() {
    let status = both();

    assert_eq!(decide_stale(Some(&in_app(STEAM)), &status).len(), 2);
    assert_eq!(decide_stale(Some(&in_app(SLACK)), &status).len(), 2);
    assert_eq!(decide_stale(Some(&in_app(DISCORD)), &status), []);
}

#[test]
fn an_old_status_also_spares_a_tab_blocked_only_by_a_spent_limit() {
    let mut status = blocking(&[], &["reddit.com"]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.exhausted = vec![site_target("reddit.com")];
    let visit = browsing(Browser::Chrome, "reddit.com");

    assert_eq!(decide_stale(Some(&visit), &status), []);
    assert_eq!(decide_now(Some(&visit), &status).len(), 2);
}
