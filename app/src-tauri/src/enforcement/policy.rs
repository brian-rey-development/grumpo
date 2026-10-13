//! The enforcement rules as a pure function. It sees the world as plain values and answers
//! with actions, so every rule is testable without a clock, a daemon or an operating system.

mod blocking;

use focus_core::ipc::DaemonStatus;
use focus_core::{AppRef, Limit, LimitTarget};

use super::action::{Action, NoticeKind};
use super::memory::EnforcementMemory;
use crate::time::units::SECONDS_PER_MINUTE;
use crate::tracking::activity::Activity;

pub use blocking::BlockEnds;

const WARNING_LEAD_SECONDS: u32 = 5 * SECONDS_PER_MINUTE;

#[derive(Debug, Clone, Copy)]
pub struct PolicyInput<'a> {
    pub activity: Option<&'a Activity>,
    /// `None` while the daemon is unavailable. Without it nothing is known about the rules,
    /// so nothing is enforced and nothing is reported. Usage keeps accumulating in the store
    /// meanwhile, which is what lets a limit passed in the dark be reported once the daemon is back.
    pub status: Option<&'a DaemonStatus>,
    /// Seconds used today for each limit target, including time not yet written to the store.
    pub usage: &'a [(LimitTarget, u32)],
    pub memory: &'a EnforcementMemory,
    pub now: i64,
    /// True when the daemon last confirmed `status` before the local day began. The daemon
    /// frees the targets whose limit ran out yesterday at midnight, so until the next
    /// confirmation a block that rests only on such a limit may already be over.
    pub status_predates_today: bool,
    pub ends: BlockEnds,
}

pub fn decide(input: &PolicyInput<'_>) -> Vec<Action> {
    let Some(status) = input.status else {
        return Vec::new();
    };
    let mut actions = blocking::foreground(input, status);
    actions.extend(limits(input, status));
    actions
}

/// What to do about blocked apps that are running behind the front window. Tabs are only
/// ever handled in the front window, since the browsers can only be asked about that one.
pub fn sweep(input: &PolicyInput<'_>, running: &[AppRef]) -> Vec<Action> {
    let Some(status) = input.status else {
        return Vec::new();
    };
    blocking::background(input, status, running)
}

fn limits(input: &PolicyInput<'_>, status: &DaemonStatus) -> Vec<Action> {
    status
        .rules
        .limits
        .iter()
        .flat_map(|limit| limit_actions(input, status, limit))
        .collect()
}

fn limit_actions(input: &PolicyInput<'_>, status: &DaemonStatus, limit: &Limit) -> Vec<Action> {
    let Some(used) = used_seconds(input.usage, &limit.target) else {
        return Vec::new();
    };
    let allowance = u32::from(limit.daily_minutes) * SECONDS_PER_MINUTE;
    match allowance.checked_sub(used).filter(|left| *left > 0) {
        None => report(input, status, limit).into_iter().collect(),
        Some(left) if left <= WARNING_LEAD_SECONDS => permitted(
            input,
            [Action::Notify(NoticeKind::LimitWarning {
                target: limit.target.clone(),
                seconds_left: left,
            })],
        ),
        Some(_) => Vec::new(),
    }
}

fn report(input: &PolicyInput<'_>, status: &DaemonStatus, limit: &Limit) -> Option<Action> {
    let settled = status.exhausted.contains(&limit.target) || input.memory.limit_reported(limit);
    (!settled).then(|| Action::ReportLimitReached {
        target: limit.target.clone(),
        daily_minutes: limit.daily_minutes,
    })
}

fn used_seconds(usage: &[(LimitTarget, u32)], target: &LimitTarget) -> Option<u32> {
    usage
        .iter()
        .find(|(candidate, _)| candidate == target)
        .map(|(_, seconds)| *seconds)
}

fn permitted<const N: usize>(input: &PolicyInput<'_>, candidates: [Action; N]) -> Vec<Action> {
    candidates
        .into_iter()
        .filter(|action| input.memory.allows(action, input.now))
        .collect()
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
