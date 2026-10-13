use std::fmt;

use chrono::{NaiveDateTime, Timelike};
use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

use super::units;

const MINUTES_PER_DAY: u16 = units::as_u16(units::MINUTES_PER_DAY);
const MINUTES_PER_HOUR: u16 = units::as_u16(units::MINUTES_PER_HOUR);
const HOURS_PER_DAY: u16 = units::as_u16(units::HOURS_PER_DAY);

/// Minutes since local midnight, from 0 to 1440 inclusive where 1440 is the end of the day.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "number"))]
pub struct TimeOfDay(u16);

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum TimeOfDayError {
    #[error("{0} minutes is past the end of the day ({max})", max = MINUTES_PER_DAY)]
    OutOfRange(u16),
    #[error("expected a time like 09:30, got \"{0}\"")]
    InvalidFormat(String),
}

impl TimeOfDay {
    pub const MIDNIGHT: Self = Self(0);
    pub const END_OF_DAY: Self = Self(MINUTES_PER_DAY);

    pub fn new(minutes: u16) -> Result<Self, TimeOfDayError> {
        if minutes > MINUTES_PER_DAY {
            return Err(TimeOfDayError::OutOfRange(minutes));
        }
        Ok(Self(minutes))
    }

    /// Builds the start of an hour from 0 to 24.
    ///
    /// # Panics
    ///
    /// Panics when `hour` is past 24, which fails the build when used in a `const`.
    pub const fn from_hour(hour: u16) -> Self {
        assert!(hour <= HOURS_PER_DAY, "hour is past the end of the day");
        Self(hour * MINUTES_PER_HOUR)
    }

    pub fn of(local: NaiveDateTime) -> Self {
        let minutes = local.hour() * u32::from(MINUTES_PER_HOUR) + local.minute();
        Self(u16::try_from(minutes).unwrap_or(MINUTES_PER_DAY))
    }

    /// Parses `HH:MM`, accepting `24:00` as the end of the day.
    pub fn parse(input: &str) -> Result<Self, TimeOfDayError> {
        let invalid = || TimeOfDayError::InvalidFormat(input.to_owned());
        let (hours, minutes) = input.trim().split_once(':').ok_or_else(invalid)?;
        let well_formed = hours.len() <= 2 && minutes.len() == 2;
        let hours: u16 = hours.parse().map_err(|_| invalid())?;
        let minutes: u16 = minutes.parse().map_err(|_| invalid())?;
        if !well_formed || minutes >= MINUTES_PER_HOUR {
            return Err(invalid());
        }
        Self::new(hours * MINUTES_PER_HOUR + minutes)
    }

    pub fn minutes(self) -> u16 {
        self.0
    }
}

impl Serialize for TimeOfDay {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u16(self.0)
    }
}

impl<'de> Deserialize<'de> for TimeOfDay {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::new(u16::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl fmt::Display for TimeOfDay {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:02}:{:02}",
            self.0 / MINUTES_PER_HOUR,
            self.0 % MINUTES_PER_HOUR
        )
    }
}

#[cfg(test)]
mod tests;
