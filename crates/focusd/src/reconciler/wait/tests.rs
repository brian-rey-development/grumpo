use chrono::Timelike;
use focus_core::Now;

use super::*;
use focus_core::test_support::now_at;

const INTERVAL: Duration = Duration::from_secs(15);

fn at(hour: u32, minute: u32, second: u32, nano: u32) -> Now {
    let base = now_at(2026, 10, 12, hour, minute);
    let local = base
        .local()
        .with_second(second)
        .and_then(|local| local.with_nanosecond(nano))
        .unwrap();
    Now::new(local.and_utc(), local)
}

#[test]
fn keeps_the_regular_beat_far_from_midnight() {
    assert_eq!(next_wait(&at(9, 0, 0, 0), INTERVAL), INTERVAL);
    assert_eq!(next_wait(&at(23, 59, 0, 0), INTERVAL), INTERVAL);
    assert_eq!(next_wait(&at(0, 0, 0, 0), INTERVAL), INTERVAL);
}

#[test]
fn wakes_one_second_after_midnight_when_it_is_closer_than_the_beat() {
    assert_eq!(
        next_wait(&at(23, 59, 50, 0), INTERVAL),
        Duration::from_secs(11)
    );
    assert_eq!(
        next_wait(&at(23, 59, 59, 0), INTERVAL),
        Duration::from_secs(2)
    );
}

#[test]
fn counts_fractions_of_a_second() {
    assert_eq!(
        next_wait(&at(23, 59, 59, 500_000_000), INTERVAL),
        Duration::from_millis(1_500)
    );
}

#[test]
fn the_beat_is_cut_only_once_midnight_plus_the_margin_comes_sooner() {
    assert_eq!(
        next_wait(&at(23, 59, 45, 0), INTERVAL),
        Duration::from_secs(15)
    );
    assert_eq!(
        next_wait(&at(23, 59, 46, 0), INTERVAL),
        Duration::from_secs(15)
    );
    assert_eq!(
        next_wait(&at(23, 59, 47, 0), INTERVAL),
        Duration::from_secs(14)
    );
}

#[test]
fn never_exceeds_a_shorter_interval() {
    let tight = Duration::from_millis(5);
    assert_eq!(next_wait(&at(23, 59, 59, 0), tight), tight);
}
