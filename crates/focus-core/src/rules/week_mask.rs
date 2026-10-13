use chrono::NaiveDateTime;

use crate::time::units::{DAYS_PER_WEEK, MINUTES_PER_DAY};
use crate::time::{Day, TimeOfDay};

const MINUTES_PER_DAY_USIZE: usize = MINUTES_PER_DAY as usize;
pub(crate) const MINUTES_PER_WEEK: usize = DAYS_PER_WEEK * MINUTES_PER_DAY_USIZE;
const WORD_BITS: usize = u64::BITS as usize;
const WORDS: usize = MINUTES_PER_WEEK.div_ceil(WORD_BITS);

/// One bit per minute of the week, Monday 00:00 first, used to reason about schedule coverage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct WeekMask([u64; WORDS]);

impl Default for WeekMask {
    fn default() -> Self {
        Self([0; WORDS])
    }
}

impl WeekMask {
    /// Index of the minute containing `local`, ignoring seconds.
    pub(crate) fn position(local: NaiveDateTime) -> usize {
        Day::of(local).index() * MINUTES_PER_DAY_USIZE + usize::from(TimeOfDay::of(local).minutes())
    }

    /// Marks `start..end` minutes of `day`, ignoring empty and inverted ranges.
    pub(crate) fn fill(&mut self, day: Day, start: TimeOfDay, end: TimeOfDay) {
        let first = usize::from(start.minutes());
        let len = usize::from(end.minutes()).saturating_sub(first);
        self.fill_span(day.index() * MINUTES_PER_DAY_USIZE + first, len);
    }

    /// Marks `len` minutes from `start`, wrapping from Sunday night to Monday.
    pub(crate) fn fill_span(&mut self, start: usize, len: usize) {
        for position in start..start + len {
            self.set(position);
        }
    }

    pub(crate) fn is_set(&self, position: usize) -> bool {
        let position = position % MINUTES_PER_WEEK;
        self.0[position / WORD_BITS] & (1 << (position % WORD_BITS)) != 0
    }

    /// Number of minutes marked here that `other` does not mark.
    pub(crate) fn uncovered_by(&self, other: &Self) -> u32 {
        self.0
            .iter()
            .zip(&other.0)
            .map(|(mine, theirs)| (mine & !theirs).count_ones())
            .sum()
    }

    /// Length of the marked run starting at `start`, wrapping from Sunday night to Monday.
    ///
    /// Returns `Some(0)` when `start` is unmarked and `None` when every minute of the week
    /// is marked, because such a run has no end.
    pub(crate) fn run_from(&self, start: usize) -> Option<usize> {
        (0..MINUTES_PER_WEEK).find(|&offset| !self.is_set(start + offset))
    }

    fn set(&mut self, position: usize) {
        let position = position % MINUTES_PER_WEEK;
        self.0[position / WORD_BITS] |= 1 << (position % WORD_BITS);
    }
}

#[cfg(test)]
mod tests;
