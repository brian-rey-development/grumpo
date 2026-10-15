//! Blocked apps and tabs: which ones to act on, and what to tell the person about it.

use focus_core::ipc::{DaemonStatus, EnforcementReason, UnlockPhaseView};
use focus_core::{hosts, is_protected, AppRef, Domain, LimitTarget};

use super::{permitted, PolicyInput};
use crate::dto::BlockReason;
use crate::enforcement::action::{Action, NoticeKind};
use crate::tracking::activity::Activity;

/// Where the blocks that need a separate answer end, in unix seconds. Both are computed by
/// the caller from its calendar, because the policy knows no time zone.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockEnds {
    /// When the schedule window in force now ends, or `None` if it never does.
    pub schedule: Option<i64>,
    /// The coming local midnight, when a block caused by a spent limit lifts.
    pub midnight: i64,
}

/// Why a particular app or site is blocked right now.
#[derive(Debug, Clone, Copy)]
struct Cause {
    scheduled: bool,
    exhausted: bool,
    /// The daemon keeps this target blocked because it lost the rules that asked for it, so
    /// the block has no schedule or limit to end it and lasts until rules are saved again.
    held: bool,
}

impl Cause {
    fn limit_only(self) -> bool {
        self.exhausted && !self.scheduled
    }
}

/// How the block on one target is described to the person.
#[derive(Debug, Clone, Copy)]
struct Terms {
    until: Option<i64>,
    reason: BlockReason,
    strict: bool,
}

pub(super) fn foreground(input: &PolicyInput<'_>, status: &DaemonStatus) -> Vec<Action> {
    let Some(activity) = input.activity.filter(|_| status.enforcement.active) else {
        return Vec::new();
    };
    if status.enforcement.blocked_apps.contains(&activity.app) {
        return app_actions(input, status, &activity.app);
    }
    tab_actions(input, status, activity).unwrap_or_default()
}

pub(super) fn background(
    input: &PolicyInput<'_>,
    status: &DaemonStatus,
    running: &[AppRef],
) -> Vec<Action> {
    if !status.enforcement.active {
        return Vec::new();
    }
    running
        .iter()
        .filter(|app| status.enforcement.blocked_apps.contains(*app))
        .flat_map(|app| app_actions(input, status, app))
        .collect()
}

/// Quit and announce a blocked app, unless it is one Grumpo must never close or the block
/// rests on a limit that the daemon has probably already lifted at midnight.
fn app_actions(input: &PolicyInput<'_>, status: &DaemonStatus, app: &AppRef) -> Vec<Action> {
    let cause = app_cause(status, app);
    if is_protected(app.bundle_id()) || stale(input, cause) {
        return Vec::new();
    }
    let terms = terms(input, status, cause);
    permitted(
        input,
        [
            Action::QuitApp(app.clone()),
            Action::Notify(NoticeKind::AppBlocked {
                app: app.clone(),
                until: terms.until,
                reason: terms.reason,
                strict: terms.strict,
            }),
        ],
    )
}

fn tab_actions(
    input: &PolicyInput<'_>,
    status: &DaemonStatus,
    activity: &Activity,
) -> Option<Vec<Action>> {
    let browser = activity.browser?;
    let domain = activity.domain.as_ref()?;
    let rule = covering_rule(status, domain)?;
    let cause = site_cause(status, domain);
    if stale(input, cause) {
        return Some(Vec::new());
    }
    let terms = terms(input, status, cause);
    let notice = Action::Notify(NoticeKind::SiteBlocked {
        domain: rule.clone(),
        until: terms.until,
        reason: terms.reason,
        strict: terms.strict,
    });
    if !browser.can_close_tabs() {
        return Some(permitted(input, [notice]));
    }
    let close = Action::CloseTab {
        browser,
        domain: domain.clone(),
    };
    Some(permitted(input, [close, notice]))
}

fn covering_rule<'a>(status: &'a DaemonStatus, visited: &Domain) -> Option<&'a Domain> {
    status
        .enforcement
        .blocked_sites
        .iter()
        .find(|rule| hosts::blocks(rule, visited))
}

fn stale(input: &PolicyInput<'_>, cause: Cause) -> bool {
    input.status_predates_today && cause.limit_only()
}

fn schedule_in_force(status: &DaemonStatus) -> bool {
    matches!(
        status.enforcement.reason,
        EnforcementReason::Schedule | EnforcementReason::Both
    )
}

fn app_cause(status: &DaemonStatus, app: &AppRef) -> Cause {
    Cause {
        scheduled: schedule_in_force(status) && status.rules.blocked_apps.contains(app),
        exhausted: status
            .exhausted
            .iter()
            .any(|target| matches!(target, LimitTarget::App { app: spent } if spent == app)),
        held: false,
    }
}

fn site_cause(status: &DaemonStatus, visited: &Domain) -> Cause {
    Cause {
        scheduled: schedule_in_force(status)
            && status
                .rules
                .blocked_sites
                .iter()
                .any(|rule| hosts::blocks(rule, visited)),
        exhausted: status
            .exhausted
            .iter()
            .any(|target| matches!(target, LimitTarget::Site { domain } if hosts::blocks(domain, visited))),
        held: status
            .held_sites
            .iter()
            .any(|held| hosts::blocks(held, visited)),
    }
}

fn terms(input: &PolicyInput<'_>, status: &DaemonStatus, cause: Cause) -> Terms {
    if cause.held {
        return Terms {
            until: None,
            reason: BlockReason::Always,
            strict: is_strict(status),
        };
    }
    let until = notice_end(input, status, cause);
    Terms {
        until,
        reason: block_reason(status, cause, until),
        strict: is_strict(status),
    }
}

/// A spent limit is the reason only when no schedule also holds the target. A scheduled
/// block with no end is `Always`. A target the status lists under neither cause, which a
/// consistent daemon never sends, follows the plan's own reason.
fn block_reason(status: &DaemonStatus, cause: Cause, until: Option<i64>) -> BlockReason {
    let limited = if cause.scheduled || cause.exhausted {
        cause.limit_only()
    } else {
        status.enforcement.reason == EnforcementReason::Limit
    };
    match (limited, until) {
        (true, _) => BlockReason::Limit,
        (false, Some(_)) => BlockReason::Schedule,
        (false, None) => BlockReason::Always,
    }
}

/// Loosening is locked whenever blocking is on and no unlock is running, which is the state
/// the daemon rejects rule changes in. The unlock phase can only be `Unlocked` while the
/// plan is inactive, so this holds for every block that gets this far, but the rule is
/// stated here rather than assumed.
fn is_strict(status: &DaemonStatus) -> bool {
    status.enforcement.active && !matches!(status.unlock.phase, UnlockPhaseView::Unlocked { .. })
}

/// When the block on this target ends. The daemon reports one end for the whole plan, which
/// is the right answer when a single reason is in force. With both a schedule and a spent
/// limit active, each target has its own end: a scheduled one lasts as long as its window
/// and a limit one until midnight, so a sentence about a target must not borrow the other's.
fn notice_end(input: &PolicyInput<'_>, status: &DaemonStatus, cause: Cause) -> Option<i64> {
    if status.enforcement.reason != EnforcementReason::Both {
        return status.enforcement.active_until;
    }
    match (cause.exhausted, cause.scheduled) {
        (true, true) => status.enforcement.active_until,
        (true, false) => Some(input.ends.midnight),
        (false, _) => input.ends.schedule,
    }
}
