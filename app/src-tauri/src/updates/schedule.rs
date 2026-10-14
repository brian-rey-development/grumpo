use std::time::{Duration, SystemTime};

const CHECK_INTERVAL: Duration = Duration::from_hours(24);
/// The network is often still down right after login.
const RETRY_INTERVAL: Duration = Duration::from_hours(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Outcome {
    UpToDate,
    Installed,
    Failed,
}

pub(super) fn next_check(outcome: Outcome, now: SystemTime) -> SystemTime {
    match outcome {
        Outcome::Failed => now + RETRY_INTERVAL,
        Outcome::UpToDate | Outcome::Installed => now + CHECK_INTERVAL,
    }
}

/// Wall clock, so a slept Mac still checks. A next check beyond one interval means the clock
/// was set back, which counts as due.
pub(super) fn is_due(next_check: SystemTime, now: SystemTime) -> bool {
    next_check.duration_since(now).map_or(true, |remaining| {
        remaining.is_zero() || remaining > CHECK_INTERVAL
    })
}

#[cfg(test)]
mod tests;
