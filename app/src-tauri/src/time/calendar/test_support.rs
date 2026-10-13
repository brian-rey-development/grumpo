use super::*;
use chrono::{FixedOffset, LocalResult, NaiveDate, NaiveDateTime, TimeZone};

pub const HOUR: i64 = 3_600;

/// A zone that switches from one fixed offset to another at one instant, which is all a
/// daylight-saving change is, so the transition rules are visible in the test.
pub struct Zone {
    before: FixedOffset,
    after: FixedOffset,
    switch_at: i64,
}

impl Zone {
    pub fn new(before_hours: i32, after_hours: i32, switch_at: i64) -> Self {
        Self {
            before: FixedOffset::east_opt(before_hours * 3_600).unwrap(),
            after: FixedOffset::east_opt(after_hours * 3_600).unwrap(),
            switch_at,
        }
    }

    fn resolve(&self, wall: NaiveDateTime) -> LocalResult<i64> {
        let before = self.before.from_local_datetime(&wall).unwrap().timestamp();
        let after = self.after.from_local_datetime(&wall).unwrap().timestamp();
        let valid_before = before < self.switch_at;
        let valid_after = after >= self.switch_at;
        match (valid_before, valid_after) {
            (true, true) => LocalResult::Ambiguous(before, after),
            (true, false) => LocalResult::Single(before),
            (false, true) => LocalResult::Single(after),
            (false, false) => LocalResult::None,
        }
    }

    pub fn day(&self, date: NaiveDate) -> LocalDay {
        build_day(date, &|wall| self.resolve(wall))
    }
}

pub fn utc(date: NaiveDate, hour: i64) -> i64 {
    date.and_time(NaiveTime::MIN).and_utc().timestamp() + hour * HOUR
}

pub fn day(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}
