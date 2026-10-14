use std::collections::BTreeSet;

use focus_core::ipc::DaemonStatus;
use focus_core::{Domain, EnforcementPlan, Exhausted, Now, UnlockState, ValidatedRules};

/// The daemon's working state, whose rules are validated so no code path can skip that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub rules: ValidatedRules,
    pub unlock: UnlockState,
    pub exhausted: Exhausted,
}

impl Snapshot {
    pub fn fresh(now: &Now) -> Self {
        Self {
            rules: ValidatedRules::default(),
            unlock: UnlockState::new(now),
            exhausted: Exhausted::new(now),
        }
    }

    /// The same state after time has passed: unlock phases expire and yesterday's limits clear.
    #[must_use]
    pub fn advanced(&self, now: &Now) -> Self {
        Self {
            rules: self.rules.clone(),
            unlock: self.unlock.tick(now),
            exhausted: self.exhausted.clone().roll_over(now),
        }
    }

    pub fn plan(&self, now: &Now) -> EnforcementPlan {
        EnforcementPlan::compute(&self.rules, &self.unlock, &self.exhausted, now)
    }

    pub fn blocked_sites(&self, now: &Now) -> BTreeSet<Domain> {
        match self.plan(now) {
            EnforcementPlan::Active { sites, .. } => sites,
            EnforcementPlan::Inactive => BTreeSet::new(),
        }
    }

    pub fn status(&self, now: &Now) -> DaemonStatus {
        DaemonStatus::build(&self.rules, &self.unlock, &self.exhausted, now)
    }
}
