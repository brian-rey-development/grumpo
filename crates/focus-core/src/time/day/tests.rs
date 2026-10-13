use super::*;
use crate::test_support::local;

#[test]
fn converts_every_chrono_weekday() {
    let cases = [
        (Weekday::Mon, Day::Mon),
        (Weekday::Tue, Day::Tue),
        (Weekday::Wed, Day::Wed),
        (Weekday::Thu, Day::Thu),
        (Weekday::Fri, Day::Fri),
        (Weekday::Sat, Day::Sat),
        (Weekday::Sun, Day::Sun),
    ];
    for (weekday, expected) in cases {
        assert_eq!(Day::from(weekday), expected);
    }
}

#[test]
fn indexes_follow_the_all_ordering() {
    for (position, day) in Day::ALL.into_iter().enumerate() {
        assert_eq!(day.index(), position);
    }
}

#[test]
fn days_sort_monday_first() {
    let mut shuffled = [Day::Sun, Day::Wed, Day::Mon];
    shuffled.sort();
    assert_eq!(shuffled, [Day::Mon, Day::Wed, Day::Sun]);
}

#[test]
fn of_reads_the_weekday_of_a_local_time() {
    assert_eq!(Day::of(local(2026, 10, 12, 8, 0)), Day::Mon);
}

#[test]
fn serializes_as_lowercase_three_letter_names() {
    for day in Day::ALL {
        let json = serde_json::to_string(&day).unwrap();
        assert_eq!(json.len(), 5);
        assert_eq!(serde_json::from_str::<Day>(&json).unwrap(), day);
    }
    assert_eq!(serde_json::to_string(&Day::Mon).unwrap(), "\"mon\"");
    assert_eq!(serde_json::to_string(&Day::Sun).unwrap(), "\"sun\"");
}
