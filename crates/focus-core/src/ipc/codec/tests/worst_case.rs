use super::*;
use crate::ipc::DaemonStatus;
use crate::test_support::{exhausted_by, local, now_at, valid};
use crate::{
    AppRef, Day, Domain, Exhausted, Limit, LimitTarget, Now, Rules, Schedule, TimeOfDay,
    UnlockState, ValidatedRules, Window, MAX_APPS, MAX_LIMITS, MAX_LIMIT_MINUTES, MAX_SITES,
    MAX_WINDOWS,
};

const LONGEST_LABEL: usize = 63;
const LAST_LABEL_LEN: usize = 53;
const MAX_FIELD_CHARS: usize = 255;
const WIDEST_CHAR: &str = "\u{1f600}";

fn longest_site(index: usize) -> Domain {
    let distinct = format!("{index:04}{}", "a".repeat(LONGEST_LABEL - 4));
    let filler = "b".repeat(LONGEST_LABEL);
    let host = format!(
        "{distinct}.{filler}.{filler}.{}.com",
        "c".repeat(LAST_LABEL_LEN)
    );
    Domain::parse(&host).unwrap()
}

fn longest_app(index: usize) -> AppRef {
    let bundle_id = format!("{index:03}{}", "a".repeat(MAX_FIELD_CHARS - 3));
    AppRef::new(bundle_id, WIDEST_CHAR.repeat(MAX_FIELD_CHARS)).unwrap()
}

fn all_week_window() -> Window {
    Window {
        days: Day::ALL.into(),
        start: TimeOfDay::MIDNIGHT,
        end: TimeOfDay::END_OF_DAY,
    }
}

fn worst_case_rules() -> ValidatedRules {
    let apps: Vec<AppRef> = (0..MAX_APPS + MAX_LIMITS).map(longest_app).collect();
    valid(Rules {
        blocked_sites: (0..MAX_SITES).map(longest_site).collect(),
        blocked_apps: apps[..MAX_APPS].iter().cloned().collect(),
        schedule: Schedule {
            windows: vec![all_week_window(); MAX_WINDOWS],
        },
        limits: apps[MAX_APPS..]
            .iter()
            .map(|app| Limit {
                target: LimitTarget::App { app: app.clone() },
                daily_minutes: MAX_LIMIT_MINUTES,
            })
            .collect(),
    })
}

fn all_exhausted(rules: &ValidatedRules, now: &Now) -> Exhausted {
    let targets: Vec<LimitTarget> = rules.limits.iter().map(|l| l.target.clone()).collect();
    exhausted_by(rules, &targets, now)
}

#[test]
fn the_worst_case_ruleset_is_valid_and_really_at_every_cap() {
    let rules = worst_case_rules();
    assert_eq!(rules.blocked_sites.len(), MAX_SITES);
    assert_eq!(rules.blocked_apps.len(), MAX_APPS);
    assert_eq!(rules.schedule.windows.len(), MAX_WINDOWS);
    assert_eq!(rules.limits.len(), MAX_LIMITS);
    assert!(rules
        .blocked_sites
        .iter()
        .all(|site| site.as_str().len() == 249));
}

#[test]
fn a_worst_case_update_request_fits_the_request_limit() {
    let request = Request::UpdateRules {
        rules: Rules::clone(&worst_case_rules()),
    };
    let size = encode(&request).unwrap().len();
    assert!(size <= MAX_REQUEST_BYTES, "{size} bytes");
}

#[test]
fn a_worst_case_limit_report_fits_the_request_limit() {
    let request = Request::ReportLimitReached {
        target: LimitTarget::App {
            app: longest_app(0),
        },
        daily_minutes: MAX_LIMIT_MINUTES,
        day: local(2026, 10, 12, 0, 0).date(),
    };
    let size = encode(&request).unwrap().len();
    assert!(size <= MAX_REQUEST_BYTES, "{size} bytes");
}

#[test]
fn a_worst_case_ok_response_fits_the_response_limit() {
    let rules = worst_case_rules();
    let now = now_at(2026, 10, 12, 10, 0);
    let status = DaemonStatus::build(
        &rules,
        &UnlockState::new(&now),
        &all_exhausted(&rules, &now),
        &now,
    );
    assert_eq!(status.exhausted.len(), MAX_LIMITS);
    assert_eq!(status.enforcement.blocked_apps.len(), MAX_APPS + MAX_LIMITS);
    let size = encode(&Response::Ok { status }).unwrap().len();
    assert!(size <= MAX_RESPONSE_BYTES, "{size} bytes");
}
