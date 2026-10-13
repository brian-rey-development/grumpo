use super::super::hourly::hourly_buckets;
use super::super::piece::Piece;
use super::super::timeline::{build_timeline, MIN_SLOT_USAGE_SECONDS, SLOT_SECONDS};
use super::super::top::{top_apps, top_sites, TOP_LIST_LIMIT};
use super::{date, DAY_START};
use crate::time::test_support::fixed_day;

const SLOT: i64 = SLOT_SECONDS;
const NO_END: i64 = i64::MAX;

fn piece(app: &str, domain: Option<&str>, start: i64, end: i64) -> Piece {
    Piece {
        start,
        end,
        app_id: app.into(),
        app_name: app.to_uppercase(),
        domain: domain.map(Into::into),
    }
}

fn timeline(pieces: &[Piece]) -> Vec<(i64, i64, String)> {
    build_timeline(pieces, 0, NO_END)
        .into_iter()
        .map(|s| (s.start, s.end, s.app_id))
        .collect()
}

#[test]
fn consecutive_slots_of_one_target_merge_into_a_segment() {
    let pieces = [piece("a", None, 0, 100), piece("a", None, 160, 400)];
    assert_eq!(timeline(&pieces), [(0, 7 * SLOT, "a".to_owned())]);
}

#[test]
fn an_empty_slot_splits_a_target_into_two_segments() {
    let pieces = [piece("a", None, 0, 100), piece("a", None, 400, 500)];
    assert_eq!(
        timeline(&pieces),
        [
            (0, 2 * SLOT, "a".to_owned()),
            (6 * SLOT, 9 * SLOT, "a".to_owned())
        ]
    );
}

#[test]
fn a_different_app_or_domain_never_merges() {
    let pieces = [
        piece("a", Some("x.com"), 0, SLOT),
        piece("a", Some("y.com"), SLOT, 2 * SLOT),
        piece("a", None, 2 * SLOT, 3 * SLOT),
        piece("b", None, 3 * SLOT, 4 * SLOT),
    ];
    assert_eq!(build_timeline(&pieces, 0, NO_END).len(), 4);
}

#[test]
fn a_slot_with_less_than_the_minimum_usage_stays_empty() {
    let pieces = [
        piece("a", None, 0, MIN_SLOT_USAGE_SECONDS - 1),
        piece("b", None, 10 * SLOT, 10 * SLOT + MIN_SLOT_USAGE_SECONDS),
    ];
    assert_eq!(timeline(&pieces), [(10 * SLOT, 11 * SLOT, "b".to_owned())]);
}

#[test]
fn separate_short_pieces_in_one_slot_add_up() {
    let pieces = [piece("a", None, 0, 20), piece("a", None, 30, 50)];
    assert_eq!(timeline(&pieces), [(0, SLOT, "a".to_owned())]);
}

#[test]
fn the_dominant_target_of_a_slot_wins() {
    let pieces = [piece("a", None, 0, 20), piece("b", None, 20, 60)];
    assert_eq!(timeline(&pieces), [(0, SLOT, "b".to_owned())]);
}

#[test]
fn a_tie_goes_to_the_target_that_started_first() {
    let pieces = [piece("late", None, 30, 60), piece("early", None, 0, 30)];
    assert_eq!(timeline(&pieces), [(0, SLOT, "early".to_owned())]);
}

#[test]
fn a_piece_spanning_slots_is_cut_at_the_grid() {
    let pieces = [piece("a", None, 30, 150)];
    assert_eq!(timeline(&pieces), [(0, 3 * SLOT, "a".to_owned())]);
}

#[test]
fn the_last_segment_ends_at_the_end_of_the_day() {
    let segments = build_timeline(&[piece("a", None, 0, 100)], 0, 100);
    assert_eq!((segments[0].start, segments[0].end), (0, 100));
}

#[test]
fn slots_are_anchored_at_the_start_of_the_day() {
    let segments = build_timeline(&[piece("a", None, 1_030, 1_090)], 1_000, NO_END);
    assert_eq!(
        (segments[0].start, segments[0].end),
        (1_000, 1_000 + 2 * SLOT)
    );
}

/// An hour of switching between an editor and a browser every 15 to 25 seconds, which the
/// old merge-then-drop approach rendered as an empty ribbon.
#[test]
fn rapid_alternation_for_an_hour_still_paints_every_minute() {
    const HOUR: i64 = 3_600;
    let stints = [15, 25, 20, 17, 23, 19];
    let mut pieces = Vec::new();
    let mut cursor = 0;
    for (index, stint) in stints.iter().cycle().enumerate() {
        if cursor >= HOUR {
            break;
        }
        let end = (cursor + stint).min(HOUR);
        let (app, domain) = if index % 2 == 0 {
            ("com.editor", None)
        } else {
            ("com.browser", Some("docs.rs"))
        };
        pieces.push(piece(app, domain, cursor, end));
        cursor = end;
    }

    let segments = build_timeline(&pieces, 0, HOUR);

    assert_ne!(segments.len(), 0);
    let painted: i64 = segments.iter().map(|s| s.end - s.start).sum();
    assert_eq!(painted, HOUR);
    assert!(segments.windows(2).all(|pair| pair[0].end <= pair[1].start));
}

#[test]
fn hourly_buckets_split_a_piece_across_hours() {
    let day = fixed_day(date(1), 0);
    let buckets = hourly_buckets(&[piece("a", None, 3_000, 7_300)], &day);
    assert_eq!(buckets[..3], [600, 3_600, 100]);
    let _ = DAY_START;
}

#[test]
fn top_lists_sort_by_seconds_then_label_and_use_the_latest_name() {
    let mut renamed = piece("b", None, 500, 600);
    renamed.app_name = "Renamed".into();
    let pieces = [
        piece("b", None, 0, 100),
        piece("a", None, 100, 200),
        piece("c", None, 200, 400),
        renamed,
    ];
    let apps = top_apps(&pieces);
    let order: Vec<_> = apps
        .iter()
        .map(|e| (e.key.as_str(), e.label.as_str(), e.seconds))
        .collect();
    assert_eq!(
        order,
        [("c", "C", 200), ("b", "Renamed", 200), ("a", "A", 100)]
    );
}

#[test]
fn top_sites_ignore_pieces_without_a_domain() {
    let pieces = [
        piece("a", None, 0, 500),
        piece("a", Some("x.com"), 500, 600),
    ];
    let sites = top_sites(&pieces);
    assert_eq!(sites.len(), 1);
    assert_eq!(
        (sites[0].key.as_str(), sites[0].label.as_str()),
        ("x.com", "x.com")
    );
}

#[test]
fn top_lists_are_capped() {
    let count = i64::try_from(TOP_LIST_LIMIT).unwrap() + 5;
    let pieces: Vec<_> = (0..count)
        .map(|i| piece(&format!("app{i:02}"), None, i * 100, i * 100 + 50))
        .collect();
    assert_eq!(top_apps(&pieces).len(), TOP_LIST_LIMIT);
}
