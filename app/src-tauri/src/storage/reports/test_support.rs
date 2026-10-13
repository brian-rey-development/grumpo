use crate::time::test_support::fixed_day;
use crate::time::LocalDay;
use chrono::NaiveDate;

pub const HOUR: i64 = 3_600;

pub const DAY_START: i64 = 1_000_000;

pub fn date(day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 3, day).unwrap()
}

pub fn standard_day() -> LocalDay {
    fixed_day(date(1), DAY_START)
}
