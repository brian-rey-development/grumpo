use super::*;
use crate::time::test_support::{FakeHourCycle, FixedCalendar};

const EVENING: i64 = 19 * 3_600 + 5 * 60;

fn formatter(cycle: HourCycle) -> ClockFormatter {
    ClockFormatter::new(
        Arc::new(FixedCalendar::utc()),
        Arc::new(FakeHourCycle::new(cycle)),
    )
}

#[test]
fn a_twelve_hour_clock_drops_the_leading_zero_and_names_the_half_of_the_day() {
    let text = formatter(HourCycle::Twelve).clock(EVENING);
    assert_eq!(text.as_deref(), Some("7:05 PM"));
}

#[test]
fn a_twenty_four_hour_clock_pads_the_hour() {
    let text = formatter(HourCycle::TwentyFour).clock(5 * 3_600 + 60);
    assert_eq!(text.as_deref(), Some("05:01"));
}

#[test]
fn the_setting_is_read_again_for_every_time() {
    let cycle = Arc::new(FakeHourCycle::new(HourCycle::Twelve));
    let formatter = ClockFormatter::new(Arc::new(FixedCalendar::utc()), cycle.clone());
    assert_eq!(formatter.clock(EVENING).as_deref(), Some("7:05 PM"));

    cycle.set(HourCycle::TwentyFour);

    assert_eq!(formatter.clock(EVENING).as_deref(), Some("19:05"));
}

#[test]
fn durations_read_in_minutes_and_hours() {
    let cases = [
        (0, "0 min"),
        (59, "< 1 min"),
        (60, "1 min"),
        (45 * 60, "45 min"),
        (3_600, "1 h"),
        (3_600 + 5 * 60 + 59, "1 h 5 min"),
    ];
    for (seconds, expected) in cases {
        assert_eq!(compact_duration(seconds), expected, "{seconds} seconds");
    }
}

#[test]
fn minutes_round_up_and_are_never_zero() {
    assert_eq!(ceil_minutes(0), 1);
    assert_eq!(ceil_minutes(1), 1);
    assert_eq!(ceil_minutes(60), 1);
    assert_eq!(ceil_minutes(61), 2);
    assert_eq!(ceil_minutes(250), 5);
}

#[test]
fn a_sentence_pluralizes_minutes() {
    assert_eq!(minutes_phrase(1), "1 minute");
    assert_eq!(minutes_phrase(5), "5 minutes");
}
