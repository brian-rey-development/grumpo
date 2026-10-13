mod protected;
mod schedule;
mod strictness;
mod validate;
mod week_mask;

use std::collections::BTreeSet;
use std::fmt;

use serde::{Deserialize, Serialize};

use crate::{AppRef, Domain};

pub use protected::is_protected;
pub use schedule::{Coverage, Schedule, Window};
pub use strictness::{loosenings, Loosening};
pub use validate::{
    RuleCollection, RulesError, ValidatedRules, MAX_APPS, MAX_LIMITS, MAX_LIMIT_MINUTES, MAX_SITES,
    MAX_WINDOWS, MIN_LIMIT_MINUTES,
};

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Rules {
    pub blocked_sites: BTreeSet<Domain>,
    pub blocked_apps: BTreeSet<AppRef>,
    pub schedule: Schedule,
    pub limits: Vec<Limit>,
}

/// A daily allowance for one site or app, after which the target is blocked until local midnight.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Limit {
    pub target: LimitTarget,
    pub daily_minutes: u16,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum LimitTarget {
    Site { domain: Domain },
    App { app: AppRef },
}

impl fmt::Display for LimitTarget {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Site { domain } => domain.fmt(f),
            Self::App { app } => app.fmt(f),
        }
    }
}

#[cfg(test)]
mod tests;
