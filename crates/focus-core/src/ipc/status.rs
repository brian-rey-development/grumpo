use serde::{Deserialize, Serialize};

use super::{EnforcementView, UnlockView};
use crate::enforcement::{EnforcementPlan, Exhausted};
use crate::rules::{LimitTarget, Rules, ValidatedRules};
use crate::unlock::UnlockState;
use crate::Now;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DaemonStatus {
    pub rules: Rules,
    pub enforcement: EnforcementView,
    pub exhausted: Vec<LimitTarget>,
    pub unlock: UnlockView,
}

impl DaemonStatus {
    /// Snapshots the daemon for clients. The unlock state is advanced to `now` first and
    /// the enforcement plan is computed from that, so the parts always agree.
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
        }
    }
}

#[cfg(test)]
mod tests;
