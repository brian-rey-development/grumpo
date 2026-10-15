//! Sites the daemon keeps blocked after losing its rules, which have no schedule or limit.

use focus_core::ipc::{EnforcementReason, UnlockPhaseView};

use super::*;
use crate::dto::BlockReason;
use crate::enforcement::test_support::daemon_status;

fn holding(hosts: &[&str]) -> DaemonStatus {
    DaemonStatus {
        held_sites: hosts.iter().map(|host| site(host)).collect(),
        ..daemon_status()
    }
    .with_held_enforced()
}

fn site_terms(host: &str, status: &DaemonStatus) -> Option<(Option<i64>, BlockReason, bool)> {
    let visit = browsing(Browser::Chrome, host);
    decide_now(Some(&visit), status)
        .into_iter()
        .find_map(|action| match action {
            Action::Notify(NoticeKind::SiteBlocked {
                until,
                reason,
                strict,
                ..
            }) => Some((until, reason, strict)),
            _ => None,
        })
}

#[test]
fn a_held_site_is_closed_and_announced_as_always_blocked() {
    let status = holding(&["youtube.com"]);
    let visit = browsing(Browser::Chrome, "www.youtube.com");

    let actions = decide_now(Some(&visit), &status);

    assert!(actions.contains(&Action::CloseTab {
        browser: Browser::Chrome,
        domain: site("www.youtube.com"),
    }));
    assert_eq!(
        site_terms("www.youtube.com", &status),
        Some((None, BlockReason::Always, true))
    );
}

#[test]
fn a_site_that_is_not_held_is_left_alone() {
    let status = holding(&["youtube.com"]);

    assert_eq!(site_terms("reddit.com", &status), None);
}

#[test]
fn a_held_site_keeps_its_own_terms_beside_a_spent_limit() {
    let mut status = holding(&["youtube.com"]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.enforcement.active_until = Some(MIDNIGHT);
    status.enforcement.blocked_sites.insert(site("reddit.com"));
    status.exhausted = vec![site_target("reddit.com")];

    assert_eq!(
        site_terms("youtube.com", &status),
        Some((None, BlockReason::Always, true))
    );
    assert_eq!(
        site_terms("reddit.com", &status),
        Some((Some(MIDNIGHT), BlockReason::Limit, true))
    );
}

#[test]
fn a_stale_status_still_blocks_a_held_site() {
    let visit = browsing(Browser::Chrome, "youtube.com");

    let actions = decide_stale(Some(&visit), &holding(&["youtube.com"]));

    assert!(actions.contains(&Action::CloseTab {
        browser: Browser::Chrome,
        domain: site("youtube.com"),
    }));
}

#[test]
fn a_running_unlock_leaves_held_sites_alone() {
    let mut status = DaemonStatus {
        held_sites: [site("youtube.com")].into(),
        ..daemon_status()
    };
    status.unlock.phase = UnlockPhaseView::Unlocked { until: MIDNIGHT };
    let status = status.with_held_enforced();

    assert!(!status.enforcement.active);
    assert_eq!(site_terms("youtube.com", &status), None);
}
