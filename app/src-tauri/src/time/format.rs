//! How times and durations read to the person. Clock times follow the system's 12 or 24 hour
//! setting, which is read through the platform port each time so a change applies at once.

use std::fmt;
use std::sync::Arc;

use chrono::NaiveTime;

use super::{units, Calendar};
use crate::platform::{HourCycle, HourCycleSource};

const TWENTY_FOUR_HOUR_FORMAT: &str = "%H:%M";
const TWELVE_HOUR_FORMAT: &str = "%-I:%M %p";
const UNDER_A_MINUTE: &str = "< 1 min";

/// Turns a unix time into the wall clock text of the person's zone and clock setting.
#[derive(Clone)]
pub struct ClockFormatter {
    calendar: Arc<dyn Calendar>,
    cycle: Arc<dyn HourCycleSource>,
}

impl fmt::Debug for ClockFormatter {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ClockFormatter").finish_non_exhaustive()
    }
}

impl ClockFormatter {
    pub fn new(calendar: Arc<dyn Calendar>, cycle: Arc<dyn HourCycleSource>) -> Self {
        Self { calendar, cycle }
    }

    /// The local time of day, or `None` when the instant has no single local reading.
    pub fn time_of(&self, unix: i64) -> Option<NaiveTime> {
        self.calendar.local_time(unix)
    }

    /// "7:05 PM" on a 12 hour clock and "19:05" on a 24 hour one.
    pub fn clock(&self, unix: i64) -> Option<String> {
        self.time_of(unix)
            .map(|time| clock_text(time, self.cycle.hour_cycle()))
    }
}

fn clock_text(time: NaiveTime, cycle: HourCycle) -> String {
    let format = match cycle {
        HourCycle::Twelve => TWELVE_HOUR_FORMAT,
        HourCycle::TwentyFour => TWENTY_FOUR_HOUR_FORMAT,
    };
    time.format(format).to_string()
}

/// The short form for tight places such as the menu bar, "45 min" or "1 h 5 min".
pub fn compact_duration(seconds: u32) -> String {
    let minutes = seconds / units::SECONDS_PER_MINUTE;
    let (hours, minutes) = (
        minutes / units::MINUTES_PER_HOUR,
        minutes % units::MINUTES_PER_HOUR,
    );
    match (hours, minutes) {
        (0, 0) if seconds > 0 => UNDER_A_MINUTE.to_owned(),
        (0, minutes) => format!("{minutes} min"),
        (hours, 0) => format!("{hours} h"),
        (hours, minutes) => format!("{hours} h {minutes} min"),
    }
}

/// Whole minutes in `seconds` rounded up and never below one, so a sentence never promises
/// a shorter time than there is.
pub fn ceil_minutes(seconds: u32) -> u32 {
    seconds.div_ceil(units::SECONDS_PER_MINUTE).max(1)
}

/// The spelled out form for sentences, "1 minute" or "5 minutes".
pub fn minutes_phrase(minutes: u32) -> String {
    match minutes {
        1 => "1 minute".to_owned(),
        minutes => format!("{minutes} minutes"),
    }
}

#[cfg(test)]
mod tests;
