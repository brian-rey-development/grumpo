use std::time::{Duration, SystemTime};

use super::{is_due, next_check, Outcome, CHECK_INTERVAL, RETRY_INTERVAL};

const MINUTE: Duration = Duration::from_mins(1);

fn start() -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_hours(500_000)
}

#[test]
fn a_check_that_found_nothing_waits_a_day() {
    assert_eq!(
        next_check(Outcome::UpToDate, start()),
        start() + CHECK_INTERVAL
    );
}

#[test]
fn a_failed_check_tries_again_within_the_hour() {
    assert_eq!(
        next_check(Outcome::Failed, start()),
        start() + RETRY_INTERVAL
    );
}

#[test]
fn a_check_is_due_once_its_time_has_come() {
    let next = start() + RETRY_INTERVAL;
    assert!(!is_due(next, next - MINUTE));
    assert!(is_due(next, next));
    assert!(is_due(next, next + MINUTE));
}

#[test]
fn a_clock_set_back_makes_the_check_due() {
    let next = start() + CHECK_INTERVAL;
    assert!(is_due(next, start() - CHECK_INTERVAL));
}
