use chrono::{DateTime, NaiveDateTime, TimeDelta, Utc};

/// One instant seen as UTC and as the user's wall clock.
///
/// The core never reads a clock or a time zone, so callers supply both views.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Now {
    utc: DateTime<Utc>,
    local: NaiveDateTime,
}

impl Now {
    pub fn new(utc: DateTime<Utc>, local: NaiveDateTime) -> Self {
        Self { utc, local }
    }

    pub fn utc(&self) -> DateTime<Utc> {
        self.utc
    }

    pub fn local(&self) -> NaiveDateTime {
        self.local
    }

    /// Converts a later local wall-clock time to UTC using the offset in force right now.
    ///
    /// A daylight-saving change between now and `local` shifts the real instant by the
    /// size of that change, which the periodic reconciliation absorbs.
    pub fn local_to_utc(&self, local: NaiveDateTime) -> Option<DateTime<Utc>> {
        self.utc.checked_add_signed(local - self.local)
    }
}

/// Adds minutes to a timestamp, saturating at the far future instead of overflowing.
pub(crate) fn plus_minutes(at: DateTime<Utc>, minutes: u16) -> DateTime<Utc> {
    at.checked_add_signed(TimeDelta::minutes(i64::from(minutes)))
        .unwrap_or(DateTime::<Utc>::MAX_UTC)
}

#[cfg(test)]
mod tests;
