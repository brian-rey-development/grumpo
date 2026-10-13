//! Time for the app: the clock, the local calendar, the local day that reports and limits
//! count in, and the text that shows times to the person.

mod calendar;
mod clock;
mod format;
mod local_day;
#[cfg(test)]
pub(crate) mod test_support;

pub use calendar::{local_day_for, today, Calendar, SystemCalendar};
pub use clock::{Clock, SystemClock};
pub use focus_core::time::units;
pub use format::{ceil_minutes, compact_duration, minutes_phrase, ClockFormatter};
pub use local_day::{LocalDay, HOURS_PER_DAY};

#[cfg(test)]
pub(crate) use calendar::build_day;
