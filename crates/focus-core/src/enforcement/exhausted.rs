use std::collections::BTreeMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::rules::{LimitTarget, ValidatedRules};
use crate::Now;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ExhaustedError {
    #[error("there is no limit for {0}")]
    LimitUnknown(LimitTarget),
    #[error("the report is for {reported}, but today is {today}")]
    WrongDay {
        reported: NaiveDate,
        today: NaiveDate,
    },
    #[error("the limit for {target} is {current} minutes, above the {reported} that ran out")]
    LimitRaised {
        target: LimitTarget,
        current: u16,
        reported: u16,
    },
}

/// Limit targets that used up their allowance on one local day, with the daily minutes
/// each had when it ran out.
///
/// A target stays blocked only while its current limit is not higher than that, so raising
/// a limit frees the target while keeping or lowering it does not. A record dated after
/// today, which a clock set backwards leaves behind, still blocks.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(from = "Record", into = "Record")]
pub struct Exhausted {
    day: NaiveDate,
    targets: BTreeMap<LimitTarget, u16>,
}

impl Exhausted {
    pub fn new(now: &Now) -> Self {
        Self {
            day: now.local().date(),
            targets: BTreeMap::new(),
        }
    }

    #[cfg(any(test, feature = "test-support"))]
    pub fn day(&self) -> NaiveDate {
        self.day
    }

    /// Moves the record to the current local day, dropping it when the date has moved
    /// forward. A date in the past keeps it, so setting the clock back frees nothing.
    #[must_use]
    pub fn roll_over(self, now: &Now) -> Self {
        if now.local().date() > self.day {
            return Self::new(now);
        }
        self
    }

    /// Notes that `target` ran out of `reported_minutes` on the reporter's local `day`, on a
    /// record rolled to today.
    ///
    /// A report is refused when it is for another day, so one that crosses midnight cannot
    /// block the new day, when the rules no longer have a limit for the target, or when that
    /// limit was raised past the reported allowance, since the usage that ran out no longer
    /// reaches it. Keeping the larger of two reports means a lowered limit cannot shrink what
    /// was already used.
    pub fn record(
        &self,
        rules: &ValidatedRules,
        target: LimitTarget,
        reported_minutes: u16,
        day: NaiveDate,
        now: &Now,
    ) -> Result<Self, ExhaustedError> {
        let today = now.local().date();
        if day != today {
            return Err(ExhaustedError::WrongDay {
                reported: day,
                today,
            });
        }
        let Some(limit) = rules.limits.iter().find(|limit| limit.target == target) else {
            return Err(ExhaustedError::LimitUnknown(target));
        };
        if limit.daily_minutes > reported_minutes {
            return Err(ExhaustedError::LimitRaised {
                target,
                current: limit.daily_minutes,
                reported: reported_minutes,
            });
        }
        let mut current = self.clone().roll_over(now);
        current
            .targets
            .entry(target)
            .and_modify(|recorded| *recorded = (*recorded).max(reported_minutes))
            .or_insert(reported_minutes);
        Ok(current)
    }

    /// Targets that are blocked now, in the order of `rules.limits`.
    pub fn effective<'a>(
        &'a self,
        rules: &'a ValidatedRules,
        now: &Now,
    ) -> impl Iterator<Item = &'a LimitTarget> {
        let counts = self.day >= now.local().date();
        rules
            .limits
            .iter()
            .filter(move |limit| {
                counts
                    && self
                        .targets
                        .get(&limit.target)
                        .is_some_and(|&recorded| limit.daily_minutes <= recorded)
            })
            .map(|limit| &limit.target)
    }
}

/// JSON object keys must be strings, so the map travels as a list of entries.
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Record {
    day: NaiveDate,
    targets: Vec<Entry>,
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Entry {
    target: LimitTarget,
    daily_minutes: u16,
}

impl From<Record> for Exhausted {
    fn from(record: Record) -> Self {
        Self {
            day: record.day,
            targets: record
                .targets
                .into_iter()
                .map(|entry| (entry.target, entry.daily_minutes))
                .collect(),
        }
    }
}

impl From<Exhausted> for Record {
    fn from(exhausted: Exhausted) -> Self {
        Self {
            day: exhausted.day,
            targets: exhausted
                .targets
                .into_iter()
                .map(|(target, daily_minutes)| Entry {
                    target,
                    daily_minutes,
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests;
