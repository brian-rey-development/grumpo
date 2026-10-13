use super::*;

#[test]
fn the_constants_agree_with_each_other() {
    assert_eq!(
        i64::from(SECONDS_PER_MINUTE * MINUTES_PER_HOUR),
        SECONDS_PER_HOUR
    );
    assert_eq!(SECONDS_PER_HOUR * i64::from(HOURS_PER_DAY), SECONDS_PER_DAY);
    assert_eq!(MINUTES_PER_DAY, 1440);
}

#[test]
fn as_u16_keeps_values_that_fit() {
    assert_eq!(as_u16(MINUTES_PER_DAY), 1440);
    assert_eq!(as_u16(u32::from(u16::MAX)), u16::MAX);
}

#[test]
#[should_panic(expected = "does not fit")]
fn as_u16_refuses_values_that_do_not_fit() {
    let _ = as_u16(u32::from(u16::MAX) + 1);
}
