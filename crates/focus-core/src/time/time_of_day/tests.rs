use super::*;
use crate::test_support::local;

#[test]
fn new_accepts_the_inclusive_range() {
    assert_eq!(TimeOfDay::new(0), Ok(TimeOfDay::MIDNIGHT));
    assert_eq!(TimeOfDay::new(1440), Ok(TimeOfDay::END_OF_DAY));
    assert_eq!(TimeOfDay::new(1441), Err(TimeOfDayError::OutOfRange(1441)));
}

#[test]
fn parse_table() {
    let cases = [
        ("00:00", Some(0)),
        ("09:30", Some(570)),
        ("9:30", Some(570)),
        (" 19:00 ", Some(1140)),
        ("23:59", Some(1439)),
        ("24:00", Some(1440)),
        ("24:01", None),
        ("25:00", None),
        ("10:60", None),
        ("10:5", None),
        ("10", None),
        ("", None),
        ("ab:cd", None),
        ("-1:30", None),
        ("100:00", None),
        ("10:30:00", None),
    ];
    for (input, expected) in cases {
        let parsed = TimeOfDay::parse(input).ok().map(TimeOfDay::minutes);
        assert_eq!(parsed, expected, "input {input:?}");
    }
}

#[test]
fn parse_reports_the_offending_input() {
    assert_eq!(
        TimeOfDay::parse("nope"),
        Err(TimeOfDayError::InvalidFormat("nope".to_owned()))
    );
}

#[test]
fn display_round_trips_through_parse() {
    for minutes in [0, 1, 59, 60, 570, 1139, 1140, 1439, 1440] {
        let time = TimeOfDay::new(minutes).unwrap();
        assert_eq!(TimeOfDay::parse(&time.to_string()), Ok(time));
    }
    assert_eq!(TimeOfDay::from_hour(7).to_string(), "07:00");
    assert_eq!(TimeOfDay::END_OF_DAY.to_string(), "24:00");
}

#[test]
fn from_hour_accepts_zero_through_twenty_four() {
    assert_eq!(TimeOfDay::from_hour(0), TimeOfDay::MIDNIGHT);
    assert_eq!(TimeOfDay::from_hour(19).minutes(), 1140);
    assert_eq!(TimeOfDay::from_hour(24), TimeOfDay::END_OF_DAY);
}

#[test]
#[should_panic(expected = "past the end of the day")]
fn from_hour_refuses_to_clamp_past_twenty_four() {
    let _ = TimeOfDay::from_hour(25);
}

#[test]
fn of_ignores_seconds() {
    let with_seconds = local(2026, 10, 12, 13, 45).with_second(59).unwrap();
    assert_eq!(TimeOfDay::of(with_seconds).minutes(), 13 * 60 + 45);
}

#[test]
fn serializes_as_a_plain_number() {
    let time = TimeOfDay::new(570).unwrap();
    assert_eq!(serde_json::to_string(&time).unwrap(), "570");
    assert_eq!(serde_json::from_str::<TimeOfDay>("570").unwrap(), time);
}

#[test]
fn deserialization_rejects_out_of_range_values() {
    assert!(serde_json::from_str::<TimeOfDay>("1441").is_err());
    assert!(serde_json::from_str::<TimeOfDay>("-1").is_err());
}

#[test]
fn orders_by_minutes() {
    assert!(TimeOfDay::MIDNIGHT < TimeOfDay::from_hour(1));
    assert!(TimeOfDay::from_hour(23) < TimeOfDay::END_OF_DAY);
}
