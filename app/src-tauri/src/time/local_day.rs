use chrono::NaiveDate;

use super::units;

pub const HOURS_PER_DAY: usize = units::HOURS_PER_DAY as usize;

/// One local calendar day as a half-open `[start, end)` range of unix seconds, plus the
/// instant each local clock hour begins.
///
/// The caller builds it from local times, so a daylight-saving day spans 23 or 25 hours.
/// On a spring-forward day the skipped hour starts at the same instant as the next one and
/// is a zero-length bucket. On a fall-back day the repeated hour is folded into its own
/// local hour, which then spans two real hours. Hourly buckets follow the clock on the wall.
/// An inverted or empty range simply yields empty reports.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LocalDay {
    pub date: NaiveDate,
    pub start: i64,
    pub end: i64,
    hour_starts: [i64; HOURS_PER_DAY],
}

impl LocalDay {
    /// `hour_starts[h]` is when local hour `h` begins. Values are clamped into the day and
    /// made non-decreasing, so a careless caller cannot produce overlapping buckets.
    pub fn new(date: NaiveDate, hour_starts: [i64; HOURS_PER_DAY], end: i64) -> Self {
        let start = hour_starts[0];
        let mut previous = start;
        let hour_starts = hour_starts.map(|instant| {
            previous = instant.clamp(previous, end.max(previous));
            previous
        });
        Self {
            date,
            start,
            end,
            hour_starts,
        }
    }

    /// The 24 hour starts followed by the end of the day, so bucket `h` is
    /// `bounds[h]..bounds[h + 1]`.
    pub fn hour_bounds(&self) -> [i64; HOURS_PER_DAY + 1] {
        let mut bounds = [self.end; HOURS_PER_DAY + 1];
        bounds[..HOURS_PER_DAY].copy_from_slice(&self.hour_starts);
        bounds
    }
}
