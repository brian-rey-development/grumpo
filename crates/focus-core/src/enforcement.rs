mod exhausted;

use std::collections::BTreeSet;

use chrono::{DateTime, Utc};

use crate::rules::{Coverage, LimitTarget, Loosening, ValidatedRules};
use crate::unlock::UnlockState;
use crate::{AppRef, Domain, Now};

pub use exhausted::{Exhausted, ExhaustedError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveReason {
    Schedule,
    Limit,
    Both,
}

/// What must be blocked right now, as decided purely from rules, unlock state and usage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnforcementPlan {
    Inactive,
    Active {
        reason: ActiveReason,
        /// When blocking ends, or `None` if it never does.
        until: Option<DateTime<Utc>>,
        sites: BTreeSet<Domain>,
        apps: BTreeSet<AppRef>,
    },
}

impl EnforcementPlan {
    pub fn compute(
        rules: &ValidatedRules,
        unlock: &UnlockState,
        exhausted: &Exhausted,
        now: &Now,
    ) -> Self {
        if unlock.is_unlocked(now) {
            return Self::Inactive;
        }
        let schedule = rules.schedule.active_until(now.local());
        let spent: BTreeSet<LimitTarget> = exhausted.effective(rules, now).cloned().collect();
        let Some(reason) = reason_for(schedule != Coverage::Inactive, !spent.is_empty()) else {
            return Self::Inactive;
        };
        let (mut sites, mut apps) = scheduled_targets(rules, schedule);
        for target in spent.iter().cloned() {
            match target {
                LimitTarget::Site { domain } => sites.insert(domain),
                LimitTarget::App { app } => apps.insert(app),
            };
        }
        Self::Active {
            reason,
            until: end_of_blocking(rules, schedule, !spent.is_empty(), now),
            sites,
            apps,
        }
    }
}

/// The changes among `found` that cannot be applied now.
///
/// Raising or removing a limit needs an active unlock whatever the schedule says, because a
/// limit is spent per day and the schedule has no bearing on it. Every other loosening is
/// locked only while blocking is active.
pub fn locked_loosenings(
    found: &[Loosening],
    plan: &EnforcementPlan,
    unlocked: bool,
) -> Vec<Loosening> {
    let enforcing = matches!(plan, EnforcementPlan::Active { .. });
    found
        .iter()
        .filter(|loosening| {
            if loosening.is_limit_change() {
                !unlocked
            } else {
                enforcing
            }
        })
        .cloned()
        .collect()
}

fn reason_for(scheduled: bool, limited: bool) -> Option<ActiveReason> {
    match (scheduled, limited) {
        (true, true) => Some(ActiveReason::Both),
        (true, false) => Some(ActiveReason::Schedule),
        (false, true) => Some(ActiveReason::Limit),
        (false, false) => None,
    }
}

fn scheduled_targets(
    rules: &ValidatedRules,
    schedule: Coverage,
) -> (BTreeSet<Domain>, BTreeSet<AppRef>) {
    if schedule == Coverage::Inactive {
        return (BTreeSet::new(), BTreeSet::new());
    }
    (rules.blocked_sites.clone(), rules.blocked_apps.clone())
}

/// An exhausted limit blocks until local midnight, and a window that starts before
/// midnight and goes on after it extends that.
fn end_of_blocking(
    rules: &ValidatedRules,
    schedule: Coverage,
    limited: bool,
    now: &Now,
) -> Option<DateTime<Utc>> {
    let coverage = if limited {
        rules.schedule.active_until_through_day(now.local())
    } else {
        schedule
    };
    match coverage {
        Coverage::Until(end) => now.local_to_utc(end),
        Coverage::Always | Coverage::Inactive => None,
    }
}

#[cfg(test)]
mod tests;
