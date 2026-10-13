use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::enforcement::{ActiveReason, EnforcementPlan};
use crate::unlock::{UnlockPhase, UnlockState, UNLOCK_DURATIONS_MINUTES, UNLOCK_SENTENCE};
use crate::{AppRef, Domain};

/// Seconds since the Unix epoch, the only timestamp format that crosses the IPC boundary.
pub type UnixSeconds = i64;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum EnforcementReason {
    None,
    Schedule,
    Limit,
    Both,
}

impl From<ActiveReason> for EnforcementReason {
    fn from(reason: ActiveReason) -> Self {
        match reason {
            ActiveReason::Schedule => Self::Schedule,
            ActiveReason::Limit => Self::Limit,
            ActiveReason::Both => Self::Both,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct EnforcementView {
    pub active: bool,
    pub reason: EnforcementReason,
    pub active_until: Option<UnixSeconds>,
    pub blocked_sites: BTreeSet<Domain>,
    pub blocked_apps: BTreeSet<AppRef>,
}

impl From<&EnforcementPlan> for EnforcementView {
    fn from(plan: &EnforcementPlan) -> Self {
        match plan {
            EnforcementPlan::Inactive => Self {
                active: false,
                reason: EnforcementReason::None,
                active_until: None,
                blocked_sites: BTreeSet::new(),
                blocked_apps: BTreeSet::new(),
            },
            EnforcementPlan::Active {
                reason,
                until,
                sites,
                apps,
            } => Self {
                active: true,
                reason: (*reason).into(),
                active_until: until.map(|at| at.timestamp()),
                blocked_sites: sites.clone(),
                blocked_apps: apps.clone(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum UnlockPhaseView {
    Locked,
    CoolingDown {
        ready_at: UnixSeconds,
        minutes: u16,
    },
    Ready {
        ready_at: UnixSeconds,
        expires_at: UnixSeconds,
    },
    Unlocked {
        until: UnixSeconds,
    },
}

impl From<UnlockPhase> for UnlockPhaseView {
    fn from(phase: UnlockPhase) -> Self {
        match phase {
            UnlockPhase::Locked => Self::Locked,
            UnlockPhase::CoolingDown { ready_at, minutes } => Self::CoolingDown {
                ready_at: ready_at.timestamp(),
                minutes,
            },
            UnlockPhase::Ready {
                ready_at,
                expires_at,
            } => Self::Ready {
                ready_at: ready_at.timestamp(),
                expires_at: expires_at.timestamp(),
            },
            UnlockPhase::Unlocked { until, .. } => Self::Unlocked {
                until: until.timestamp(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct UnlockView {
    pub phase: UnlockPhaseView,
    pub next_cooldown_minutes: u16,
    pub sentence: String,
    pub durations: Vec<u16>,
}

impl From<&UnlockState> for UnlockView {
    fn from(state: &UnlockState) -> Self {
        Self {
            phase: state.phase().into(),
            next_cooldown_minutes: state.next_cooldown_minutes(),
            sentence: UNLOCK_SENTENCE.to_owned(),
            durations: UNLOCK_DURATIONS_MINUTES.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests;
