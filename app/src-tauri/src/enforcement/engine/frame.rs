use focus_core::ipc::DaemonStatus;
use focus_core::{Limit, LimitTarget};

use crate::enforcement::policy::BlockEnds;

/// What one step knows about the world, read once so the policy and the sweep agree.
pub(super) struct Frame<'a> {
    pub status: Option<&'a DaemonStatus>,
    pub usage: Vec<(LimitTarget, u32)>,
    pub at: i64,
    pub predates_today: bool,
    pub ends: BlockEnds,
}

pub(super) fn limit_targets(status: Option<&DaemonStatus>) -> Vec<LimitTarget> {
    status.map_or_else(Vec::new, |status| {
        status
            .rules
            .limits
            .iter()
            .map(|Limit { target, .. }| target.clone())
            .collect()
    })
}
