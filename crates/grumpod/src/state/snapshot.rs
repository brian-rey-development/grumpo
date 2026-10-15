use std::collections::BTreeSet;

use focus_core::ipc::DaemonStatus;
use focus_core::{Domain, EnforcementPlan, Exhausted, Now, UnlockState, ValidatedRules};

/// The daemon's working state, whose rules are validated so no code path can skip that.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub rules: ValidatedRules,
    pub unlock: UnlockState,
    pub exhausted: Exhausted,
    /// Sites read back from the hosts file when no saved state could be used. They stay blocked
    /// until the user saves rules again, because dropping them with the lost rules would end the
    /// very blocking those rules asked for. An unlock releases them like any rule, but only for
    /// its duration: they stay held and come back when it ends unless rules were saved meanwhile.
    pub held_sites: BTreeSet<Domain>,
}

impl Snapshot {
    pub fn fresh(now: &Now) -> Self {
        Self {
            rules: ValidatedRules::default(),
            unlock: UnlockState::new(now),
            exhausted: Exhausted::new(now),
            held_sites: BTreeSet::new(),
        }
    }

    /// The same state after time has passed: unlock phases expire and yesterday's limits clear.
    #[must_use]
    pub fn advanced(&self, now: &Now) -> Self {
        Self {
            rules: self.rules.clone(),
            unlock: self.unlock.tick(now),
            exhausted: self.exhausted.clone().roll_over(now),
            held_sites: self.held_sites.clone(),
        }
    }

    pub fn plan(&self, now: &Now) -> EnforcementPlan {
        EnforcementPlan::compute(&self.rules, &self.unlock, &self.exhausted, now)
    }

    pub fn blocked_sites(&self, now: &Now) -> BTreeSet<Domain> {
        let planned = match self.plan(now) {
            EnforcementPlan::Active { sites, .. } => sites,
            EnforcementPlan::Inactive => BTreeSet::new(),
        };
        if self.unlock.is_unlocked(now) {
            return planned;
        }
        planned.union(&self.held_sites).cloned().collect()
    }

    pub fn status(&self, now: &Now) -> DaemonStatus {
        DaemonStatus {
            held_sites: self.held_sites.clone(),
            ..DaemonStatus::build(&self.rules, &self.unlock, &self.exhausted, now)
        }
    }
}
