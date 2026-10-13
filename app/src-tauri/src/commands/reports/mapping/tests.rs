use crate::time::test_support::fixed_day;
use focus_core::{AppRef, Domain};

use super::*;
use crate::storage::reports::DayTotal as StoredDayTotal;

const START: i64 = 1_760_000_000;

fn date() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 12).unwrap()
}

fn segment(start: i64, end: i64, domain: Option<&str>) -> Segment {
    Segment {
        start,
        end,
        app_id: "com.apple.Safari".into(),
        app_name: "Safari".into(),
        domain: domain.map(str::to_owned),
    }
}

fn usage(timeline: Vec<Segment>) -> DayUsage {
    let mut hourly = [0; 24];
    hourly[9] = 600;
    DayUsage {
        total_seconds: 600,
        hourly,
        timeline,
        top_apps: vec![UsageEntry {
            key: "com.apple.Safari".into(),
            label: "Safari".into(),
            seconds: 600,
        }],
        top_sites: vec![UsageEntry {
            key: "github.com".into(),
            label: "github.com".into(),
            seconds: 400,
        }],
    }
}

#[test]
fn timeline_times_are_seconds_since_local_midnight() {
    let day = fixed_day(date(), START);
    let timeline = vec![
        segment(START + 9 * 3600, START + 9 * 3600 + 1800, None),
        segment(
            START + 23 * 3600 + 3000,
            START + 24 * 3600,
            Some("github.com"),
        ),
    ];
    let report = day_report(date(), &day, usage(timeline), Vec::new());

    assert_eq!(report.timeline[0].start, 9 * 3600);
    assert_eq!(report.timeline[0].end, 9 * 3600 + 1800);
    assert_eq!(report.timeline[1].start, 23 * 3600 + 3000);
    assert_eq!(report.timeline[1].end, 86_400);
}

#[test]
fn a_segment_names_its_site_or_else_its_app() {
    let day = fixed_day(date(), START);
    let timeline = vec![
        segment(START, START + 60, None),
        segment(START + 60, START + 120, Some("github.com")),
    ];
    let report = day_report(date(), &day, usage(timeline), Vec::new());

    let app = &report.timeline[0];
    assert_eq!(
        (app.key.as_str(), app.label.as_str(), app.kind),
        ("com.apple.Safari", "Safari", UsageKind::App)
    );
    let site = &report.timeline[1];
    assert_eq!(
        (site.key.as_str(), site.label.as_str(), site.kind),
        ("github.com", "github.com", UsageKind::Site)
    );
}

#[test]
fn the_report_carries_totals_lists_and_windows() {
    let day = fixed_day(date(), START);
    let windows = vec![FocusWindow {
        start: 32_400,
        end: 43_200,
    }];
    let report = day_report(date(), &day, usage(Vec::new()), windows.clone());

    assert_eq!(report.date, "2026-10-12");
    assert_eq!(report.total_seconds, 600);
    assert_eq!(report.hourly.len(), 24);
    assert_eq!(report.hourly[9], 600);
    assert_eq!(report.focus_windows, windows);
    assert_eq!(report.top_apps[0].kind, UsageKind::App);
    assert_eq!(report.top_sites[0].kind, UsageKind::Site);
    assert_eq!(report.top_sites[0].seconds, 400);
}

#[test]
fn timelines_follow_the_wall_clock_across_a_spring_forward_day() {
    let hours: [i64; 24] = std::array::from_fn(|h| {
        let skipped_hour = i64::from(h > 2);
        START + (i64::try_from(h).unwrap() - skipped_hour) * 3600
    });
    let end = START + 23 * 3600;
    let day = LocalDay::new(date(), hours, end);

    assert_eq!(seconds_of_day(&day, START + 3600), 3600);
    assert_eq!(seconds_of_day(&day, START + 2 * 3600), 3 * 3600);
    assert_eq!(seconds_of_day(&day, end), 24 * 3600);
}

#[test]
fn instants_outside_the_day_are_clamped_to_its_edges() {
    let day = fixed_day(date(), START);
    assert_eq!(seconds_of_day(&day, START - 500), 0);
    assert_eq!(seconds_of_day(&day, START + 25 * 3600), 24 * 3600);
}

#[test]
fn week_reports_list_each_day() {
    let usage = WeekUsage {
        days: vec![
            StoredDayTotal {
                date: date(),
                total_seconds: 90,
            },
            StoredDayTotal {
                date: date().succ_opt().unwrap(),
                total_seconds: 30,
            },
        ],
        total_seconds: 120,
        top_apps: Vec::new(),
        top_sites: Vec::new(),
    };
    let report = week_report(usage);

    assert_eq!(report.total_seconds, 120);
    assert_eq!(
        report.days,
        [
            DayTotal {
                date: "2026-10-12".into(),
                total_seconds: 90
            },
            DayTotal {
                date: "2026-10-13".into(),
                total_seconds: 30
            },
        ]
    );
}

#[test]
fn limit_usage_converts_minutes_and_defaults_to_unused() {
    let site = LimitTarget::Site {
        domain: Domain::parse("reddit.com").unwrap(),
    };
    let app = LimitTarget::App {
        app: AppRef::new("com.valvesoftware.steam", "Steam").unwrap(),
    };
    let limits = vec![
        Limit {
            target: site.clone(),
            daily_minutes: 30,
        },
        Limit {
            target: app.clone(),
            daily_minutes: 90,
        },
    ];
    let rows = limit_usage(&limits, &[(site.clone(), 125)], &[]);

    assert_eq!(rows[0].target, site);
    assert_eq!((rows[0].used_seconds, rows[0].limit_seconds), (125, 1_800));
    assert_eq!(rows[1].target, app);
    assert_eq!((rows[1].used_seconds, rows[1].limit_seconds), (0, 5_400));
    assert!(!rows[0].reached && !rows[1].reached);
}

#[test]
fn a_limit_is_reached_when_the_time_is_used_up_or_the_daemon_says_so() {
    let site = LimitTarget::Site {
        domain: Domain::parse("reddit.com").unwrap(),
    };
    let app = LimitTarget::App {
        app: AppRef::new("com.valvesoftware.steam", "Steam").unwrap(),
    };
    let limit = |target: &LimitTarget| Limit {
        target: target.clone(),
        daily_minutes: 30,
    };
    let limits = [limit(&site), limit(&app)];

    let spent = limit_usage(&limits, &[(site.clone(), 1_800)], &[]);
    assert!(spent[0].reached);
    assert!(!spent[1].reached);

    let exhausted = limit_usage(&limits, &[(app.clone(), 60)], std::slice::from_ref(&app));
    assert!(!exhausted[0].reached);
    assert!(
        exhausted[1].reached,
        "time in private windows is not stored"
    );
}
