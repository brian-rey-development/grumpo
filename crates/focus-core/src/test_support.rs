//! Builders for rules, instants and daemon status shared by the tests of every crate, enabled
//! outside this crate by the `test-support` feature.

#![expect(
    clippy::unwrap_used,
    clippy::missing_panics_doc,
    reason = "fixtures are written for tests, where a bad literal should panic"
)]

use std::thread;
use std::time::{Duration as StdDuration, Instant};

use chrono::{DateTime, Duration, NaiveDate, NaiveDateTime, Utc};

use crate::ipc::DaemonStatus;
use crate::{
    AppRef, Day, Domain, Exhausted, Limit, LimitTarget, Now, Rules, Schedule, TimeOfDay,
    UnlockState, ValidatedRules, Window,
};

pub fn local(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> NaiveDateTime {
    NaiveDate::from_ymd_opt(year, month, day)
        .and_then(|date| date.and_hms_opt(hour, minute, 0))
        .unwrap()
}

/// Builds a `Now` whose local clock equals UTC.
pub fn now_at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Now {
    now_from(local(year, month, day, hour, minute))
}

pub fn now_from(local: NaiveDateTime) -> Now {
    Now::new(DateTime::from_naive_utc_and_offset(local, Utc), local)
}

pub fn after(now: &Now, minutes: i64) -> Now {
    now_from(now.local() + Duration::minutes(minutes))
}

pub fn hm(hour: u16, minute: u16) -> u16 {
    hour * 60 + minute
}

pub fn site(host: &str) -> Domain {
    Domain::parse(host).unwrap()
}

pub fn app(bundle_id: &str) -> AppRef {
    named_app(bundle_id, bundle_id)
}

pub fn named_app(bundle_id: &str, name: &str) -> AppRef {
    AppRef::new(bundle_id, name).unwrap()
}

pub fn site_target(host: &str) -> LimitTarget {
    LimitTarget::Site { domain: site(host) }
}

pub fn app_target(bundle_id: &str) -> LimitTarget {
    LimitTarget::App {
        app: app(bundle_id),
    }
}

pub fn limit(target: LimitTarget, daily_minutes: u16) -> Limit {
    Limit {
        target,
        daily_minutes,
    }
}

/// A window over `days` between two minutes of the day, built without validation.
pub fn window(days: &[Day], start: u16, end: u16) -> Window {
    Window {
        days: days.iter().copied().collect(),
        start: TimeOfDay::new(start).unwrap(),
        end: TimeOfDay::new(end).unwrap(),
    }
}

pub fn schedule(windows: Vec<Window>) -> Schedule {
    Schedule { windows }
}

pub fn valid(rules: Rules) -> ValidatedRules {
    rules.validate().unwrap()
}

/// Two sites, one app and two limits around `schedule`.
pub fn rules_with(schedule: Schedule) -> ValidatedRules {
    valid(Rules {
        blocked_sites: [site("youtube.com"), site("reddit.com")].into(),
        blocked_apps: [app("com.valvesoftware.steam")].into(),
        schedule,
        limits: vec![
            limit(site_target("x.com"), 30),
            limit(app_target("com.discord"), 30),
        ],
    })
}

/// `rules_with` a Monday window from midnight to 19:00.
pub fn sample_rules() -> ValidatedRules {
    rules_with(schedule(vec![window(&[Day::Mon], 0, hm(19, 0))]))
}

/// Status of an idle daemon running `sample_rules` at `now`.
pub fn sample_status(now: &Now) -> DaemonStatus {
    DaemonStatus::build(
        &sample_rules(),
        &UnlockState::new(now),
        &Exhausted::new(now),
        now,
    )
}

/// A record in which every target in `targets` ran out of the allowance `rules` give it.
pub fn exhausted_by(rules: &ValidatedRules, targets: &[LimitTarget], now: &Now) -> Exhausted {
    targets.iter().fold(Exhausted::new(now), |record, target| {
        let allowance = rules
            .limits
            .iter()
            .find(|limit| &limit.target == target)
            .map_or(0, |limit| limit.daily_minutes);
        record
            .record(rules, target.clone(), allowance, now.local().date(), now)
            .unwrap()
    })
}

const EVENTUALLY_TIMEOUT: StdDuration = StdDuration::from_secs(5);
const EVENTUALLY_POLL: StdDuration = StdDuration::from_millis(5);

/// Polls `condition` until it holds, failing the test after a generous deadline.
pub fn eventually(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + EVENTUALLY_TIMEOUT;
    while !condition() {
        assert!(Instant::now() < deadline, "condition not met in time");
        thread::sleep(EVENTUALLY_POLL);
    }
}
