use chrono::{Datelike, NaiveDateTime, Weekday};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum Day {
    Mon,
    Tue,
    Wed,
    Thu,
    Fri,
    Sat,
    Sun,
}

impl Day {
    pub const ALL: [Day; 7] = [
        Day::Mon,
        Day::Tue,
        Day::Wed,
        Day::Thu,
        Day::Fri,
        Day::Sat,
        Day::Sun,
    ];

    /// Position in the week, Monday first.
    pub(crate) fn index(self) -> usize {
        match self {
            Day::Mon => 0,
            Day::Tue => 1,
            Day::Wed => 2,
            Day::Thu => 3,
            Day::Fri => 4,
            Day::Sat => 5,
            Day::Sun => 6,
        }
    }

    pub fn of(local: NaiveDateTime) -> Self {
        local.weekday().into()
    }
}

impl From<Weekday> for Day {
    fn from(weekday: Weekday) -> Self {
        match weekday {
            Weekday::Mon => Day::Mon,
            Weekday::Tue => Day::Tue,
            Weekday::Wed => Day::Wed,
            Weekday::Thu => Day::Thu,
            Weekday::Fri => Day::Fri,
            Weekday::Sat => Day::Sat,
            Weekday::Sun => Day::Sun,
        }
    }
}

#[cfg(test)]
mod tests;
