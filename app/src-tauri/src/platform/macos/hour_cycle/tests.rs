use super::*;

#[test]
fn the_one_to_twelve_and_zero_to_eleven_fields_are_a_twelve_hour_clock() {
    assert_eq!(cycle_of("h a"), HourCycle::Twelve);
    assert_eq!(cycle_of("a h"), HourCycle::Twelve);
    assert_eq!(cycle_of("Bh"), HourCycle::Twelve);
    assert_eq!(cycle_of("K"), HourCycle::Twelve);
}

#[test]
fn the_zero_to_twenty_three_and_one_to_twenty_four_fields_are_a_twenty_four_hour_clock() {
    assert_eq!(cycle_of("HH"), HourCycle::TwentyFour);
    assert_eq!(cycle_of("H"), HourCycle::TwentyFour);
    assert_eq!(cycle_of("k"), HourCycle::TwentyFour);
}

#[test]
fn letters_inside_quoted_literals_are_not_hour_fields() {
    assert_eq!(cycle_of("'h'H"), HourCycle::TwentyFour);
    assert_eq!(cycle_of("'H 'h"), HourCycle::Twelve);
    assert_eq!(cycle_of("'o''clock' h"), HourCycle::Twelve);
    assert_eq!(cycle_of("'h'"), HourCycle::TwentyFour);
}

#[test]
fn a_pattern_with_no_hour_field_defaults_to_twenty_four() {
    assert_eq!(cycle_of(""), HourCycle::TwentyFour);
}

#[test]
fn the_system_answers_with_one_of_the_two() {
    let cycle = MacHourCycle.hour_cycle();
    assert!(matches!(cycle, HourCycle::Twelve | HourCycle::TwentyFour));
}
