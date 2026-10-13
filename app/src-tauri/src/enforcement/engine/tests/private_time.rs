use super::*;
use crate::time::test_support::FixedCalendar;
use crate::time::Calendar;

const SAMPLE_GAP: i64 = 2;

fn private_visit(host: &str) -> crate::tracking::activity::Activity {
    let mut visit = browsing(Browser::Chrome, host);
    visit.private = true;
    visit
}

fn reddit_limited(minutes: u16) -> focus_core::ipc::DaemonStatus {
    with_limit(daemon_status(), site_target("reddit.com"), minutes)
}

#[test]
fn private_browsing_counts_toward_a_limit_without_a_single_stored_session() {
    let rig = Rig::new(reddit_limited(1));
    let mut engine = rig.engine();
    let visit = private_visit("old.reddit.com");

    for step in 0..30 {
        engine.sample(Some(&visit), T0 + step * SAMPLE_GAP);
    }
    assert_eq!(reports(&rig), 0, "58 s are not a minute yet");
    engine.sample(Some(&visit), T0 + 30 * SAMPLE_GAP);

    assert_eq!(reports(&rig), 1);
    let day = FixedCalendar::utc().day_containing(T0);
    assert_eq!(rig.store.total_seconds(day).unwrap(), 0);
}

#[test]
fn the_report_names_the_allowance_in_force() {
    let rig = Rig::new(reddit_limited(1));
    let mut engine = rig.engine();
    let visit = private_visit("reddit.com");

    for step in 0..=30 {
        engine.sample(Some(&visit), T0 + step * SAMPLE_GAP);
    }

    let reported = rig
        .daemon
        .requests()
        .into_iter()
        .find_map(|request| match request {
            Request::ReportLimitReached {
                target,
                daily_minutes,
                ..
            } => Some((target, daily_minutes)),
            _ => None,
        });
    assert_eq!(reported, Some((site_target("reddit.com"), 1)));
}

#[test]
fn a_stalled_worker_cannot_credit_more_than_one_sample_gap() {
    let rig = Rig::new(reddit_limited(1));
    let mut engine = rig.engine();
    let visit = private_visit("reddit.com");

    engine.sample(Some(&visit), T0);
    engine.sample(Some(&visit), T0 + 3_600);

    assert_eq!(reports(&rig), 0);
}

#[test]
fn midnight_forgets_private_time() {
    let rig = Rig::new(reddit_limited(1));
    let mut engine = rig.engine();
    let visit = private_visit("reddit.com");
    for step in 0..29 {
        engine.sample(Some(&visit), T0 + step * SAMPLE_GAP);
    }

    for step in 0..2 {
        engine.sample(Some(&visit), T0 + DAY + step * SAMPLE_GAP);
    }

    assert_eq!(reports(&rig), 0);
}
