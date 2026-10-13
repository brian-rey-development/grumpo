use chrono::{DateTime, TimeDelta, Utc};
use serde::{Deserialize, Serialize};

use super::CONFIRM_WINDOW_MINUTES;
use crate::time::plus_minutes;

const CLOCK_SLACK_SECONDS: i64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
pub enum UnlockPhase {
    Locked,
    CoolingDown {
        ready_at: DateTime<Utc>,
        minutes: u16,
    },
    Ready {
        ready_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    },
    Unlocked {
        until: DateTime<Utc>,
        minutes: u16,
    },
}

impl UnlockPhase {
    /// Moves the phase to `at`, which also undoes a clock set backwards: a phase can never have
    /// more time left than it was granted, so one that does is restarted or dropped. A small
    /// step back, like an NTP correction, stays within the slack and changes nothing.
    pub(super) fn advance(self, at: DateTime<Utc>) -> Self {
        match self {
            Self::CoolingDown { ready_at, .. } if at >= ready_at => Self::Ready {
                ready_at,
                expires_at: plus_minutes(ready_at, CONFIRM_WINDOW_MINUTES),
            }
            .advance(at),
            Self::CoolingDown { ready_at, minutes } if exceeds(ready_at, at, minutes) => {
                Self::CoolingDown {
                    ready_at: plus_minutes(at, minutes),
                    minutes,
                }
            }
            Self::Ready { expires_at, .. }
                if at >= expires_at || exceeds(expires_at, at, CONFIRM_WINDOW_MINUTES) =>
            {
                Self::Locked
            }
            Self::Unlocked { until, minutes } if at >= until || exceeds(until, at, minutes) => {
                Self::Locked
            }
            unchanged => unchanged,
        }
    }
}

fn exceeds(end: DateTime<Utc>, at: DateTime<Utc>, allowed_minutes: u16) -> bool {
    end - at
        > TimeDelta::minutes(i64::from(allowed_minutes)) + TimeDelta::seconds(CLOCK_SLACK_SECONDS)
}
