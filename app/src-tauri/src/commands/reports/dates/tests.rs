use super::*;

fn date(y: i32, m: u32, d: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(y, m, d).unwrap()
}

#[test]
fn parses_well_formed_dates() {
    assert_eq!(parse_date("2026-10-12").unwrap(), date(2026, 10, 12));
    assert_eq!(parse_date("2024-02-29").unwrap(), date(2024, 2, 29));
    assert_eq!(parse_date("1970-01-01").unwrap(), date(1970, 1, 1));
}

#[test]
fn rejects_everything_else() {
    let cases = [
        "",
        "2026-1-5",
        "2026-10-1",
        "26-10-12",
        "+2026-10-12",
        "2026/10/12",
        "2026-10-12 ",
        " 2026-10-12",
        "2026-10-12T00:00",
        "2026-13-01",
        "2026-02-30",
        "2025-02-29",
        "1969-12-31",
        "today",
    ];
    for input in cases {
        assert!(parse_date(input).is_err(), "accepted {input:?}");
    }
}

#[test]
fn a_week_ends_on_the_given_day() {
    let week = week_ending(date(2026, 3, 2)).unwrap();
    assert_eq!(week[0], date(2026, 2, 24));
    assert_eq!(week[6], date(2026, 3, 2));
    assert!(week
        .windows(2)
        .all(|pair| pair[1] == pair[0].succ_opt().unwrap()));
}

#[test]
fn a_week_cannot_start_before_the_supported_dates() {
    assert!(week_ending(date(1970, 1, 3)).is_err());
    assert!(week_ending(date(1970, 1, 7)).is_ok());
}

#[test]
fn formats_with_zero_padding() {
    assert_eq!(format_date(date(2026, 1, 5)), "2026-01-05");
}
