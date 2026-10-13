use std::collections::BTreeMap;
use std::fmt;

use super::{LimitTarget, ValidatedRules};
use crate::{hosts, AppRef, Domain};

/// A change that makes the rules easier to break, so it needs an unlock while enforcing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Loosening {
    SiteRemoved {
        domain: Domain,
    },
    AppRemoved {
        app: AppRef,
    },
    LimitRemoved {
        target: LimitTarget,
    },
    LimitRaised {
        target: LimitTarget,
        from: u16,
        to: u16,
    },
    ScheduleReduced {
        minutes_uncovered: u32,
    },
}

impl Loosening {
    /// Whether this raises or removes a daily limit.
    pub fn is_limit_change(&self) -> bool {
        matches!(self, Self::LimitRemoved { .. } | Self::LimitRaised { .. })
    }
}

impl fmt::Display for Loosening {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SiteRemoved { domain } => write!(f, "unblocks {domain}"),
            Self::AppRemoved { app } => write!(f, "unblocks {app}"),
            Self::LimitRemoved { target } => write!(f, "removes the limit for {target}"),
            Self::LimitRaised { target, from, to } => {
                write!(
                    f,
                    "raises the limit for {target} from {from} to {to} minutes"
                )
            }
            Self::ScheduleReduced { minutes_uncovered } => {
                write!(
                    f,
                    "frees {minutes_uncovered} minutes of scheduled blocking each week"
                )
            }
        }
    }
}

/// Lists every way `new` is more permissive than `old`.
///
/// Adding rules or lowering limits is never a loosening. Sites are compared by what the
/// hosts file would block, which has no wildcards, so a broader domain in `new` does not
/// keep an old subdomain blocked.
pub fn loosenings(old: &ValidatedRules, new: &ValidatedRules) -> Vec<Loosening> {
    let mut found = removed_sites(old, new);
    found.extend(removed_apps(old, new));
    found.extend(changed_limits(old, new));
    found.extend(reduced_schedule(old, new));
    found
}

// Only the domain's own hostname is compared, because the aliases of an alias such as
// `m.youtube.com` are not real hosts, which keeps `m.youtube.com` -> `youtube.com` harmless.
fn removed_sites(old: &ValidatedRules, new: &ValidatedRules) -> Vec<Loosening> {
    let still_blocked = hosts::blocked_hosts(&new.blocked_sites);
    old.blocked_sites
        .iter()
        .filter(|site| !still_blocked.contains(site.as_str()))
        .map(|domain| Loosening::SiteRemoved {
            domain: domain.clone(),
        })
        .collect()
}

fn removed_apps(old: &ValidatedRules, new: &ValidatedRules) -> Vec<Loosening> {
    old.blocked_apps
        .difference(&new.blocked_apps)
        .map(|app| Loosening::AppRemoved { app: app.clone() })
        .collect()
}

fn changed_limits(old: &ValidatedRules, new: &ValidatedRules) -> Vec<Loosening> {
    let new_minutes: BTreeMap<&LimitTarget, u16> = new
        .limits
        .iter()
        .map(|limit| (&limit.target, limit.daily_minutes))
        .collect();
    old.limits
        .iter()
        .filter_map(|limit| match new_minutes.get(&limit.target) {
            None => Some(Loosening::LimitRemoved {
                target: limit.target.clone(),
            }),
            Some(&to) if to > limit.daily_minutes => Some(Loosening::LimitRaised {
                target: limit.target.clone(),
                from: limit.daily_minutes,
                to,
            }),
            Some(_) => None,
        })
        .collect()
}

fn reduced_schedule(old: &ValidatedRules, new: &ValidatedRules) -> Option<Loosening> {
    let minutes_uncovered = old.schedule.mask().uncovered_by(&new.schedule.mask());
    (minutes_uncovered > 0).then_some(Loosening::ScheduleReduced { minutes_uncovered })
}

#[cfg(test)]
mod tests;
