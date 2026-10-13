//! The one fake clock, calendar and clock setting the tests of every module share.

use std::sync::atomic::{AtomicBool, AtomicI64, Ordering};

use chrono::{FixedOffset, NaiveDate, NaiveTime, TimeZone, Utc};
use focus_core::Now;

use super::{build_day, units, Calendar, Clock, LocalDay, HOURS_PER_DAY};
use crate::platform::{HourCycle, HourCycleSource};

#[derive(Debug, Default)]
pub struct FakeClock(AtomicI64);

impl FakeClock {
    pub fn at(unix: i64) -> Self {
        Self(AtomicI64::new(unix))
    }

    pub fn advance(&self, seconds: i64) {
        self.0.fetch_add(seconds, Ordering::SeqCst);
    }

    pub fn set(&self, unix: i64) {
        self.0.store(unix, Ordering::SeqCst);
    }
}

impl Clock for FakeClock {
    fn now_unix(&self) -> i64 {
        self.0.load(Ordering::SeqCst)
    }
}

/// A calendar in a fixed UTC offset, so tests do not depend on the machine's zone.
#[derive(Debug, Clone, Copy)]
pub struct FixedCalendar(pub FixedOffset);

impl FixedCalendar {
    pub fn utc() -> Self {
        Self(FixedOffset::east_opt(0).unwrap())
    }
}

impl Calendar for FixedCalendar {
    fn day_containing(&self, unix: i64) -> LocalDay {
        let date = self.0.timestamp_opt(unix, 0).unwrap().date_naive();
        build_day(date, &|wall| {
            self.0.from_local_datetime(&wall).map(|at| at.timestamp())
        })
    }

    fn local_time(&self, unix: i64) -> Option<NaiveTime> {
        self.0.timestamp_opt(unix, 0).single().map(|at| at.time())
    }

    fn instant(&self, unix: i64) -> Option<Now> {
        let at = self.0.timestamp_opt(unix, 0).single()?;
        Some(Now::new(at.with_timezone(&Utc), at.naive_local()))
    }
}

#[derive(Debug)]
pub struct FakeHourCycle(AtomicBool);

impl FakeHourCycle {
    pub fn new(cycle: HourCycle) -> Self {
        Self(AtomicBool::new(cycle == HourCycle::Twelve))
    }

    pub fn set(&self, cycle: HourCycle) {
        self.0.store(cycle == HourCycle::Twelve, Ordering::SeqCst);
    }
}

impl HourCycleSource for FakeHourCycle {
    fn hour_cycle(&self) -> HourCycle {
        if self.0.load(Ordering::SeqCst) {
            HourCycle::Twelve
        } else {
            HourCycle::TwentyFour
        }
    }
}

/// A day of 24 equal hours starting at `start`.
pub fn fixed_day(date: NaiveDate, start: i64) -> LocalDay {
    let hour_starts = std::array::from_fn(|hour| {
        start + i64::try_from(hour).unwrap_or(0) * units::SECONDS_PER_HOUR
    });
    let length = i64::try_from(HOURS_PER_DAY).unwrap_or(0) * units::SECONDS_PER_HOUR;
    LocalDay::new(date, hour_starts, start + length)
}
