use std::collections::BTreeSet;

use chrono::NaiveDate;
use focus_core::ipc::{IpcError, Request};
use focus_core::{
    hosts, locked_loosenings, loosenings, LimitTarget, Loosening, Now, Rules, UnlockState,
};
use tracing::debug;

use crate::state::Snapshot;

pub fn kind(request: &Request) -> &'static str {
    match request {
        Request::Status => "status",
        Request::UpdateRules { .. } => "update_rules",
        Request::RequestUnlock => "request_unlock",
        Request::ConfirmUnlock { .. } => "confirm_unlock",
        Request::CancelUnlock => "cancel_unlock",
        Request::Relock => "relock",
        Request::ReportLimitReached { .. } => "report_limit_reached",
    }
}

/// The state a request leads to, and whether it stays in force when it cannot be saved.
///
/// Only a state that is at least as strict as the one before may be kept, since refusing it
/// because the disk is failing would let a full disk switch enforcement off.
#[derive(Debug)]
pub struct Applied {
    pub next: Snapshot,
    pub keep_on_save_failure: bool,
}

impl Applied {
    fn keep_on_save_failure(next: Snapshot) -> Self {
        Self {
            next,
            keep_on_save_failure: true,
        }
    }

    fn require_save(next: Snapshot) -> Self {
        Self {
            next,
            keep_on_save_failure: false,
        }
    }
}

/// The outcome of `request` from `base`, which must already be advanced to `now`.
pub fn apply(base: &Snapshot, request: Request, now: &Now) -> Result<Applied, IpcError> {
    match request {
        Request::Status => Ok(Applied::keep_on_save_failure(base.clone())),
        Request::UpdateRules { rules } => update_rules(base, rules, now),
        Request::RequestUnlock => Ok(Applied::require_save(with_unlock(
            base,
            base.unlock.request(now)?,
        ))),
        Request::ConfirmUnlock { phrase, minutes } => Ok(Applied::require_save(with_unlock(
            base,
            base.unlock.confirm(&phrase, minutes, now)?,
        ))),
        Request::CancelUnlock => Ok(Applied::keep_on_save_failure(with_unlock(
            base,
            base.unlock.cancel(now)?,
        ))),
        Request::Relock => Ok(Applied::keep_on_save_failure(with_unlock(
            base,
            base.unlock.relock(now),
        ))),
        Request::ReportLimitReached {
            target,
            daily_minutes,
            day,
        } => Ok(report_limit(base, target, daily_minutes, day, now)),
    }
}

fn update_rules(base: &Snapshot, rules: Rules, now: &Now) -> Result<Applied, IpcError> {
    let rules = rules.validate()?;
    let found = loosenings(&base.rules, &rules);
    let mut locked = locked_loosenings(&found, &base.plan(now), base.unlock.is_unlocked(now));
    let next = Snapshot {
        rules,
        held_sites: BTreeSet::new(),
        ..base.clone()
    };
    locked.extend(locked_releases(base, &next, now));
    if !locked.is_empty() {
        return Err(IpcError::loosening_locked(&locked));
    }
    let releases_held = !base.held_sites.is_empty();
    Ok(if found.is_empty() && !releases_held {
        Applied::keep_on_save_failure(next)
    } else {
        Applied::require_save(next)
    })
}

/// The held sites that `next` would stop blocking while no unlock is active.
///
/// Held sites come from the hosts file, so they are matched the way the hosts file blocks: a
/// new rule keeps a held site only when it lists that host or one of its aliases. A broader
/// domain does not, which is the same rule `loosenings` applies to saved sites, so a held site
/// cannot be dropped by a rule the hosts file would not honour. `next` has no held sites of its
/// own, so its blocked sites are exactly what the new rules plan for `now`.
fn locked_releases(base: &Snapshot, next: &Snapshot, now: &Now) -> Vec<Loosening> {
    if base.unlock.is_unlocked(now) {
        return Vec::new();
    }
    let kept = next.blocked_sites(now);
    base.held_sites
        .iter()
        .filter(|held| !kept.iter().any(|site| hosts::blocks(site, held)))
        .map(|domain| Loosening::SiteRemoved {
            domain: domain.clone(),
        })
        .collect()
}

/// A report that no longer matches the rules or the day is stale, since the user can edit
/// limits while the app is still counting and a report can arrive after midnight, so it is
/// dropped instead of failing the reporter.
fn report_limit(
    base: &Snapshot,
    target: LimitTarget,
    daily_minutes: u16,
    day: NaiveDate,
    now: &Now,
) -> Applied {
    match base
        .exhausted
        .record(&base.rules, target, daily_minutes, day, now)
    {
        Ok(exhausted) => Applied::keep_on_save_failure(Snapshot {
            exhausted,
            ..base.clone()
        }),
        Err(stale) => {
            debug!(reason = %stale, "ignoring a limit report that no longer matches the rules");
            Applied::keep_on_save_failure(base.clone())
        }
    }
}

fn with_unlock(base: &Snapshot, unlock: UnlockState) -> Snapshot {
    Snapshot {
        unlock,
        ..base.clone()
    }
}
