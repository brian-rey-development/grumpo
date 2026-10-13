//! Local calendar days as the store wants them, built so daylight-saving days come out
//! 23 or 25 hours long with the skipped and repeated hours placed where the wall clock has them.

use chrono::{Local, LocalResult, NaiveDate, NaiveDateTime, NaiveTime, TimeDelta, TimeZone, Utc};
use focus_core::Now;

use super::LocalDay;
use crate::time::units::SECONDS_PER_DAY;

/// A clock change skips at most a few hours, so a wall time inside the gap is moved forward
/// minute by minute to the first instant that exists.
const MAX_GAP_MINUTES: i64 = 180;

/// Converts between unix time and the local wall clock, behind a trait so tests can fix
/// the zone.
pub trait Calendar: Send + Sync {
    fn day_containing(&self, unix: i64) -> LocalDay;
    fn local_time(&self, unix: i64) -> Option<NaiveTime>;
    /// The instant as UTC and as the wall clock, which is what the rules engine reasons in.
    fn instant(&self, unix: i64) -> Option<Now>;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemCalendar;

impl Calendar for SystemCalendar {
    fn day_containing(&self, unix: i64) -> LocalDay {
        let date = Local
            .timestamp_opt(unix, 0)
            .single()
            .map_or_else(|| Local::now().date_naive(), |at| at.date_naive());
        local_day_for(date)
    }

    fn local_time(&self, unix: i64) -> Option<NaiveTime> {
        Local.timestamp_opt(unix, 0).single().map(|at| at.time())
    }

    fn instant(&self, unix: i64) -> Option<Now> {
        let at = Local.timestamp_opt(unix, 0).single()?;
        Some(Now::new(at.with_timezone(&Utc), at.naive_local()))
    }
}

pub fn local_day_for(date: NaiveDate) -> LocalDay {
    build_day(date, &|wall| {
        Local.from_local_datetime(&wall).map(|at| at.timestamp())
    })
}

pub fn today() -> LocalDay {
    local_day_for(Local::now().date_naive())
}

/// Builds the day from a function that maps a wall time to unix seconds. A repeated wall
/// time resolves to its first occurrence, so the repeated hour belongs to one local hour,
/// and a skipped one resolves to the first instant after the gap.
pub fn build_day(
    date: NaiveDate,
    resolve: &impl Fn(NaiveDateTime) -> LocalResult<i64>,
) -> LocalDay {
    let midnight = date.and_time(NaiveTime::MIN);
    let start = first_instant(midnight, resolve).unwrap_or_else(|| midnight.and_utc().timestamp());
    let hour_starts = std::array::from_fn(|hour| {
        let hours = i64::try_from(hour).unwrap_or_default();
        midnight
            .checked_add_signed(TimeDelta::hours(hours))
            .and_then(|wall| first_instant(wall, resolve))
            .unwrap_or(start)
    });
    let end = date
        .succ_opt()
        .and_then(|next| first_instant(next.and_time(NaiveTime::MIN), resolve))
        .unwrap_or(start + SECONDS_PER_DAY);
    LocalDay::new(date, hour_starts, end)
}

fn first_instant(
    wall: NaiveDateTime,
    resolve: &impl Fn(NaiveDateTime) -> LocalResult<i64>,
) -> Option<i64> {
    (0..=MAX_GAP_MINUTES).find_map(|minutes| {
        let probe = wall.checked_add_signed(TimeDelta::minutes(minutes))?;
        match resolve(probe) {
            LocalResult::Single(unix) => Some(unix),
            LocalResult::Ambiguous(first, second) => Some(first.min(second)),
            LocalResult::None => None,
        }
    })
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
