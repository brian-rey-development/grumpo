mod phase;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::time::{plus_minutes, Now};

pub use phase::UnlockPhase;

/// Cooldown before an unlock can be confirmed, indexed by unlocks already used today.
pub const COOLDOWN_STEPS_MINUTES: [u16; 4] = [5, 10, 20, 40];
pub const UNLOCK_DURATIONS_MINUTES: [u16; 3] = [15, 30, 60];
/// How long a finished cooldown stays confirmable before the request lapses.
pub const CONFIRM_WINDOW_MINUTES: u16 = 5;
pub const UNLOCK_SENTENCE: &str = "I am choosing to give up my focus time, and I accept that cost.";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum UnlockError {
    #[error("no unlock has been requested")]
    NotRequested,
    #[error("an unlock request is already in progress")]
    AlreadyPending,
    #[error("focus is already unlocked")]
    AlreadyUnlocked,
    #[error("the cooldown has not finished yet")]
    NotReady,
    #[error("the sentence does not match")]
    PhraseMismatch,
    #[error(
        "the unlock duration must be one of {:?} minutes",
        UNLOCK_DURATIONS_MINUTES
    )]
    DurationInvalid,
}

/// The unlock flow as an immutable value, where every transition returns the next state.
///
/// Callers apply `tick` before reading it, since the state only moves when time is supplied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UnlockState {
    phase: UnlockPhase,
    unlocks_today: u8,
    day: NaiveDate,
}

impl UnlockState {
    pub fn new(now: &Now) -> Self {
        Self {
            phase: UnlockPhase::Locked,
            unlocks_today: 0,
            day: now.local().date(),
        }
    }

    pub fn phase(&self) -> UnlockPhase {
        self.phase
    }

    /// Applies the passage of time: resets the daily count when the local date moves forward
    /// and moves expired phases forward. A date in the past changes nothing, so setting the
    /// clock back does not hand out a fresh count.
    #[must_use]
    pub fn tick(&self, now: &Now) -> Self {
        let today = now.local().date();
        let (unlocks_today, day) = if today > self.day {
            (0, today)
        } else {
            (self.unlocks_today, self.day)
        };
        Self {
            phase: self.phase.advance(now.utc()),
            unlocks_today,
            day,
        }
    }

    pub fn request(&self, now: &Now) -> Result<Self, UnlockError> {
        let current = self.tick(now);
        match current.phase {
            UnlockPhase::Locked => Ok(Self {
                phase: UnlockPhase::CoolingDown {
                    ready_at: plus_minutes(now.utc(), current.next_cooldown_minutes()),
                    minutes: current.next_cooldown_minutes(),
                },
                ..current
            }),
            UnlockPhase::CoolingDown { .. } | UnlockPhase::Ready { .. } => {
                Err(UnlockError::AlreadyPending)
            }
            UnlockPhase::Unlocked { .. } => Err(UnlockError::AlreadyUnlocked),
        }
    }

    /// Completes a ready request when the typed `phrase` matches the sentence, ignoring
    /// surrounding and repeated whitespace but not case.
    pub fn confirm(&self, phrase: &str, minutes: u16, now: &Now) -> Result<Self, UnlockError> {
        let current = self.tick(now);
        match current.phase {
            UnlockPhase::Ready { .. } => {}
            UnlockPhase::Locked => return Err(UnlockError::NotRequested),
            UnlockPhase::CoolingDown { .. } => return Err(UnlockError::NotReady),
            UnlockPhase::Unlocked { .. } => return Err(UnlockError::AlreadyUnlocked),
        }
        if collapse_whitespace(phrase) != UNLOCK_SENTENCE {
            return Err(UnlockError::PhraseMismatch);
        }
        if !UNLOCK_DURATIONS_MINUTES.contains(&minutes) {
            return Err(UnlockError::DurationInvalid);
        }
        Ok(Self {
            phase: UnlockPhase::Unlocked {
                until: plus_minutes(now.utc(), minutes),
                minutes,
            },
            unlocks_today: current.unlocks_today.saturating_add(1),
            ..current
        })
    }

    /// Abandons a pending request without counting it toward escalation.
    pub fn cancel(&self, now: &Now) -> Result<Self, UnlockError> {
        let current = self.tick(now);
        match current.phase {
            UnlockPhase::CoolingDown { .. } | UnlockPhase::Ready { .. } => Ok(Self {
                phase: UnlockPhase::Locked,
                ..current
            }),
            UnlockPhase::Locked => Err(UnlockError::NotRequested),
            UnlockPhase::Unlocked { .. } => Err(UnlockError::AlreadyUnlocked),
        }
    }

    #[must_use]
    pub fn relock(&self, now: &Now) -> Self {
        Self {
            phase: UnlockPhase::Locked,
            ..self.tick(now)
        }
    }

    pub fn is_unlocked(&self, now: &Now) -> bool {
        matches!(self.tick(now).phase, UnlockPhase::Unlocked { .. })
    }

    pub fn next_cooldown_minutes(&self) -> u16 {
        cooldown_minutes_after(self.unlocks_today)
    }
}

/// The cooldown a request gets when `unlocks_today` unlocks were already used, which stops
/// growing at the last step.
pub fn cooldown_minutes_after(unlocks_today: u8) -> u16 {
    let last_step = COOLDOWN_STEPS_MINUTES.len() - 1;
    COOLDOWN_STEPS_MINUTES[usize::from(unlocks_today).min(last_step)]
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests;
