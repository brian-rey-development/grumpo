use std::time::Duration;

use chrono::{NaiveTime, TimeDelta};
use focus_core::Now;

/// Slack past midnight so the tick lands on the new day even if the timer fires a hair early.
const MIDNIGHT_MARGIN: TimeDelta = TimeDelta::seconds(1);

/// How long to sleep before the next beat: the regular `interval`, cut short when local
/// midnight falls inside it, so day-bound state such as exhausted limits clears right away.
pub fn next_wait(now: &Now, interval: Duration) -> Duration {
    until_midnight(now)
        .and_then(|left| left.checked_add(&MIDNIGHT_MARGIN))
        .and_then(|left| left.to_std().ok())
        .map_or(interval, |left| left.min(interval))
}

fn until_midnight(now: &Now) -> Option<TimeDelta> {
    let local = now.local();
    let midnight = local.date().succ_opt()?.and_time(NaiveTime::MIN);
    Some(midnight - local)
}

#[cfg(test)]
mod tests;
