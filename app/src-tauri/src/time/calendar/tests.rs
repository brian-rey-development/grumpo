use super::test_support::*;
use super::*;

#[test]
fn an_ordinary_day_has_24_even_hours() {
    let date = day(2026, 6, 15);
    let zone = Zone::new(-4, -4, i64::MAX);

    let built = zone.day(date);

    assert_eq!(built.start, utc(date, 4));
    assert_eq!(built.end - built.start, 24 * HOUR);
    let bounds = built.hour_bounds();
    assert!(bounds.windows(2).all(|pair| pair[1] - pair[0] == HOUR));
}

#[test]
fn spring_forward_is_23_hours_with_an_empty_local_hour_two() {
    let date = day(2026, 3, 8);
    let zone = Zone::new(-5, -4, utc(date, 7));

    let built = zone.day(date);

    let bounds = built.hour_bounds();
    assert_eq!(built.end - built.start, 23 * HOUR);
    assert_eq!(
        bounds[2],
        utc(date, 7),
        "02:00 does not exist, 03:00 EDT is 07:00 UTC"
    );
    assert_eq!(bounds[3], bounds[2], "so local hour 2 has no length");
    assert_eq!(bounds[3] - bounds[1], HOUR);
    assert_eq!(bounds[4] - bounds[3], HOUR);
}

#[test]
fn fall_back_is_25_hours_with_local_hour_one_covering_both_passes() {
    let date = day(2026, 11, 1);
    let zone = Zone::new(-4, -5, utc(date, 6));

    let built = zone.day(date);

    let bounds = built.hour_bounds();
    assert_eq!(built.end - built.start, 25 * HOUR);
    assert_eq!(
        bounds[1],
        utc(date, 5),
        "01:00 resolves to its first occurrence"
    );
    assert_eq!(bounds[2] - bounds[1], 2 * HOUR, "hour 1 spans the repeat");
    assert_eq!(bounds[3] - bounds[2], HOUR);
}

#[test]
fn a_gap_at_midnight_moves_the_start_of_the_day_to_the_first_real_instant() {
    let date = day(2026, 10, 18);
    let zone = Zone::new(-3, -2, utc(date, 3));

    let built = zone.day(date);

    let bounds = built.hour_bounds();
    assert_eq!(built.start, utc(date, 3), "01:00 -02 is 03:00 UTC");
    assert_eq!(bounds[0], bounds[1], "local hour 0 has no length");
    assert_eq!(built.end - built.start, 23 * HOUR);
}

#[test]
fn consecutive_days_tile_without_gaps_or_overlap_across_a_change() {
    let zone = Zone::new(-5, -4, utc(day(2026, 3, 8), 7));

    let saturday = zone.day(day(2026, 3, 7));
    let sunday = zone.day(day(2026, 3, 8));
    let monday = zone.day(day(2026, 3, 9));

    assert_eq!(saturday.end, sunday.start);
    assert_eq!(sunday.end, monday.start);
}

#[test]
fn the_system_day_is_a_plausible_calendar_day() {
    let built = today();

    let length = built.end - built.start;
    assert!((23 * HOUR..=25 * HOUR).contains(&length), "length {length}");
    assert_eq!(
        local_day_for(built.date).start,
        built.start,
        "the same date always builds the same day"
    );
}

#[test]
fn the_system_calendar_finds_the_day_around_an_instant() {
    let now = Local::now().timestamp();

    let found = SystemCalendar.day_containing(now);

    assert!(found.start <= now && now < found.end);
    assert!(SystemCalendar.local_time(now).is_some());
}
