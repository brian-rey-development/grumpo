use std::collections::BTreeSet;

use chrono::{NaiveDateTime, NaiveTime, TimeDelta};
use serde::{Deserialize, Serialize};

use super::week_mask::WeekMask;
use crate::time::{Day, TimeOfDay};

/// A recurring weekly span of blocking, from `start` inclusive to `end` exclusive.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Window {
    pub days: BTreeSet<Day>,
    pub start: TimeOfDay,
    pub end: TimeOfDay,
}

/// How long the schedule keeps blocking from a given moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Coverage {
    Inactive,
    /// Blocking continues, merging adjacent and overlapping windows, until this local time.
    Until(NaiveDateTime),
    /// Every minute of the week is covered, so blocking never ends.
    Always,
}

impl Coverage {
    fn of(mask: &WeekMask, local: NaiveDateTime) -> Self {
        match mask.run_from(WeekMask::position(local)) {
            None => Self::Always,
            Some(0) => Self::Inactive,
            Some(minutes) => minutes_after(local, minutes).map_or(Self::Always, Self::Until),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct Schedule {
    pub windows: Vec<Window>,
}

impl Schedule {
    pub fn active_at(&self, local: NaiveDateTime) -> bool {
        self.active_until(local) != Coverage::Inactive
    }

    pub fn active_until(&self, local: NaiveDateTime) -> Coverage {
        Coverage::of(&self.mask(), local)
    }

    /// Coverage when the rest of the local day is blocked regardless of the windows, which
    /// is how an exhausted limit behaves, so a window that continues past midnight extends it.
    pub(crate) fn active_until_through_day(&self, local: NaiveDateTime) -> Coverage {
        let mut mask = self.mask();
        let left_today = TimeOfDay::END_OF_DAY.minutes() - TimeOfDay::of(local).minutes();
        mask.fill_span(WeekMask::position(local), usize::from(left_today));
        Coverage::of(&mask, local)
    }

    pub(crate) fn mask(&self) -> WeekMask {
        let mut mask = WeekMask::default();
        for window in &self.windows {
            for &day in &window.days {
                mask.fill(day, window.start, window.end);
            }
        }
        mask
    }
}

fn minutes_after(local: NaiveDateTime, minutes: usize) -> Option<NaiveDateTime> {
    let elapsed = usize::from(TimeOfDay::of(local).minutes()) + minutes;
    let delta = TimeDelta::try_minutes(i64::try_from(elapsed).ok()?)?;
    local
        .date()
        .and_time(NaiveTime::MIN)
        .checked_add_signed(delta)
}

#[cfg(test)]
mod tests;
