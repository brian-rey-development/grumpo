use super::*;

#[test]
fn each_system_cycle_maps_to_its_style() {
    assert_eq!(style_of(HourCycle::Twelve), ClockStyle::H12);
    assert_eq!(style_of(HourCycle::TwentyFour), ClockStyle::H23);
}

#[test]
fn the_styles_are_sent_as_h12_and_h23() {
    assert_eq!(serde_json::to_string(&ClockStyle::H12).unwrap(), "\"h12\"");
    assert_eq!(serde_json::to_string(&ClockStyle::H23).unwrap(), "\"h23\"");
}
