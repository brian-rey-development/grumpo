use super::*;
use crate::dto::BlockReason;
use crate::platform::HourCycle;
use crate::time::test_support::{FakeHourCycle, FixedCalendar};
use crate::time::{Calendar, LocalDay};
use chrono::NaiveTime;
use focus_core::test_support::{named_app, site};
use focus_core::{AppRef, Domain, Now};
use std::sync::Arc;

pub const SECONDS_PER_HOUR: i64 = 3_600;

pub fn at(hour: i64, minute: i64) -> i64 {
    hour * SECONDS_PER_HOUR + minute * 60
}

pub fn formatter(cycle: HourCycle) -> ClockFormatter {
    ClockFormatter::new(
        Arc::new(FixedCalendar::utc()),
        Arc::new(FakeHourCycle::new(cycle)),
    )
}

pub fn plain() -> ClockFormatter {
    formatter(HourCycle::TwentyFour)
}

/// A calendar that cannot say what the local time is.
#[derive(Debug)]
pub struct Lost;

impl Calendar for Lost {
    fn day_containing(&self, unix: i64) -> LocalDay {
        FixedCalendar::utc().day_containing(unix)
    }

    fn local_time(&self, _unix: i64) -> Option<NaiveTime> {
        None
    }

    fn instant(&self, _unix: i64) -> Option<Now> {
        None
    }
}

pub fn steam() -> AppRef {
    named_app("com.valvesoftware.steam", "Steam")
}

pub fn youtube() -> Domain {
    site("youtube.com")
}

pub fn app_blocked(until: Option<i64>) -> NoticeKind {
    NoticeKind::AppBlocked {
        app: steam(),
        until,
        reason: BlockReason::Schedule,
        strict: true,
    }
}

pub fn site_blocked(until: Option<i64>) -> NoticeKind {
    NoticeKind::SiteBlocked {
        domain: youtube(),
        until,
        reason: BlockReason::Schedule,
        strict: true,
    }
}
