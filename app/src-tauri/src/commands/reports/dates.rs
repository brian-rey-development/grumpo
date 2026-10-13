use chrono::{Datelike, Days, NaiveDate};

use crate::error::AppError;
use crate::time::units::DAYS_PER_WEEK;

const DATE_FORMAT: &str = "%Y-%m-%d";
const MIN_YEAR: i32 = 1970;

/// Accepts exactly `YYYY-MM-DD` from 1970 on. chrono alone also takes `2026-1-5` and signed
/// years, so the text must survive a round trip to count as well formed.
pub fn parse_date(input: &str) -> Result<NaiveDate, AppError> {
    let invalid = || AppError::invalid_input("Pick a date in the form YYYY-MM-DD.");
    let date = NaiveDate::parse_from_str(input, DATE_FORMAT).map_err(|_| invalid())?;
    if date.year() < MIN_YEAR || format_date(date) != input {
        return Err(invalid());
    }
    Ok(date)
}

pub fn format_date(date: NaiveDate) -> String {
    date.format(DATE_FORMAT).to_string()
}

/// The seven days ending at `end`, oldest first.
pub fn week_ending(end: NaiveDate) -> Result<[NaiveDate; DAYS_PER_WEEK], AppError> {
    let mut days = [end; DAYS_PER_WEEK];
    for (back, slot) in days.iter_mut().rev().enumerate() {
        let offset = Days::new(u64::try_from(back).unwrap_or_default());
        *slot = end
            .checked_sub_days(offset)
            .filter(|day| day.year() >= MIN_YEAR)
            .ok_or_else(|| {
                AppError::invalid_input("That week starts before the supported dates.")
            })?;
    }
    Ok(days)
}

#[cfg(test)]
mod tests;
