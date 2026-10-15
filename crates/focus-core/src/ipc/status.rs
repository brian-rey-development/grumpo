use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use super::{EnforcementReason, EnforcementView, UnlockPhaseView, UnlockView};
use crate::enforcement::{EnforcementPlan, Exhausted};
use crate::rules::{LimitTarget, Rules, ValidatedRules};
use crate::unlock::UnlockState;
use crate::{Domain, Now};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DaemonStatus {
    pub rules: Rules,
    pub enforcement: EnforcementView,
    pub exhausted: Vec<LimitTarget>,
    pub unlock: UnlockView,
    /// Sites the daemon keeps blocking because it lost its saved rules and read them back from
    /// the hosts file. They stay until rules are saved again. Absent from an older daemon,
    /// which never held any.
    #[serde(default)]
    pub held_sites: BTreeSet<Domain>,
}

impl DaemonStatus {
    /// Snapshots the daemon for clients. The unlock state is advanced to `now` first and
    /// the enforcement plan is computed from that, so the parts always agree. The daemon fills
    /// `held_sites` itself, since nothing here knows them.
    pub fn build(
        rules: &ValidatedRules,
        unlock: &UnlockState,
        exhausted: &Exhausted,
        now: &Now,
    ) -> Self {
        let unlock = unlock.tick(now);
        let plan = EnforcementPlan::compute(rules, &unlock, exhausted, now);
        Self {
            rules: Rules::clone(rules),
            enforcement: (&plan).into(),
            exhausted: exhausted.effective(rules, now).cloned().collect(),
            unlock: (&unlock).into(),
            held_sites: BTreeSet::new(),
        }
    }

    /// The same status with the held sites counted as blocked, which is how a client that
    /// enforces on its own has to read it, since the daemon keeps them blocked in the hosts file
    /// whatever its plan says. An unlock is the exception, because the daemon releases the held
    /// sites for its duration like any rule, so a status in that phase is returned as it is.
    ///
    /// Held sites join `blocked_sites` so every check against that set agrees with the daemon.
    /// When nothing else is being enforced the status becomes active with no end, because the
    /// held block lasts until rules are saved. It is reported as a schedule block, the one
    /// reason that reads as "on until further notice", since a held site has no limit behind it.
    /// A status that is already active keeps its own reason and end, which describe the plan.
    #[must_use]
    pub fn with_held_enforced(mut self) -> Self {
        let unlocked = matches!(self.unlock.phase, UnlockPhaseView::Unlocked { .. });
        if self.held_sites.is_empty() || unlocked {
            return self;
        }
        self.enforcement
            .blocked_sites
            .extend(self.held_sites.iter().cloned());
        if !self.enforcement.active {
            self.enforcement.active = true;
            self.enforcement.reason = EnforcementReason::Schedule;
            self.enforcement.active_until = None;
        }
        self
    }
}

#[cfg(test)]
mod tests;
