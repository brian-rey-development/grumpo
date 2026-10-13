use super::{StorageError, Store};
use crate::time::units::SECONDS_PER_DAY;

pub const MIN_RETENTION_DAYS: u16 = 30;
pub const MAX_RETENTION_DAYS: u16 = 365;
pub const DEFAULT_RETENTION_DAYS: u16 = 90;
const SETTING_KEY: &str = "retention_days";

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
#[error("retention must be between {MIN_RETENTION_DAYS} and {MAX_RETENTION_DAYS} days, got {0}")]
pub struct RetentionOutOfRange(pub u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RetentionDays(u16);

impl RetentionDays {
    pub fn new(days: u16) -> Result<Self, RetentionOutOfRange> {
        if (MIN_RETENTION_DAYS..=MAX_RETENTION_DAYS).contains(&days) {
            Ok(Self(days))
        } else {
            Err(RetentionOutOfRange(days))
        }
    }

    pub fn get(self) -> u16 {
        self.0
    }

    fn seconds(self) -> i64 {
        i64::from(self.0) * SECONDS_PER_DAY
    }
}

impl Default for RetentionDays {
    fn default() -> Self {
        Self(DEFAULT_RETENTION_DAYS)
    }
}

impl TryFrom<u16> for RetentionDays {
    type Error = RetentionOutOfRange;

    fn try_from(days: u16) -> Result<Self, Self::Error> {
        Self::new(days)
    }
}

impl Store {
    /// A missing or unreadable value falls back to the default, because refusing to prune
    /// over a corrupt setting would let history grow without bound.
    pub fn retention_days(&self) -> Result<RetentionDays, StorageError> {
        let Some(raw) = self.setting(SETTING_KEY)? else {
            return Ok(RetentionDays::default());
        };
        let parsed = raw
            .parse::<u16>()
            .ok()
            .and_then(|d| RetentionDays::new(d).ok());
        Ok(parsed.unwrap_or_else(|| {
            tracing::warn!("stored retention setting is invalid, using the default");
            RetentionDays::default()
        }))
    }

    /// Saves the period and prunes at once, so shortening it deletes the older history now
    /// instead of at the next daily run. Returns how many sessions went away.
    pub fn set_retention_days(&self, days: RetentionDays, now: i64) -> Result<usize, StorageError> {
        self.set_setting(SETTING_KEY, &days.get().to_string())?;
        self.prune(now)
    }

    /// Deletes sessions that ended before `now` minus the retention period and returns how
    /// many rows went away.
    pub fn prune(&self, now: i64) -> Result<usize, StorageError> {
        let cutoff = now.saturating_sub(self.retention_days()?.seconds());
        self.with_conn(|conn| {
            Ok(conn.execute("DELETE FROM sessions WHERE ended_at < ?1", [cutoff])?)
        })
    }
}

#[cfg(test)]
mod tests;
