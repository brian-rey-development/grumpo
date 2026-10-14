use focus_core::ipc::{EnforcementReason, Request};
use focus_core::test_support::site_target;
use focus_core::LimitTarget;

use crate::service::test_support::{
    monday, monday_until_seven, rules, with_limit, Harness, MemoryHosts,
};
use crate::service::{Daemon, Service};

fn after_hours_with_limit() -> Harness {
    let limited = with_limit(rules(&["youtube.com"], monday_until_seven()), "x.com", 30);
    monday(20, 0, limited)
}

#[test]
fn reaching_a_limit_blocks_the_target_and_reports_it() {
    let mut daemon = after_hours_with_limit();
    assert!(!daemon.hosts.has_managed_block());

    let status = daemon.report_limit("x.com", 30).unwrap();

    assert_eq!(status.exhausted, vec![site_target("x.com")]);
    assert!(status.enforcement.active);
    assert_eq!(status.enforcement.reason, EnforcementReason::Limit);
    assert!(daemon.hosts.blocks("x.com"));
    assert!(!daemon.hosts.blocks("youtube.com"));
    assert_eq!(
        daemon.store.saved().unwrap().exhausted.day().to_string(),
        "2026-10-12"
    );
}

#[test]
fn reporting_a_target_without_a_limit_is_ignored() {
    let mut daemon = after_hours_with_limit();
    let saves = daemon.store.saves();

    let status = daemon.report_limit("reddit.com", 30).unwrap();

    assert_eq!(status.exhausted, Vec::<LimitTarget>::new());
    assert_eq!(daemon.store.saves(), saves);
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn a_report_older_than_a_raised_limit_is_ignored() {
    let mut daemon = after_hours_with_limit();
    daemon.unlock(15);
    daemon
        .update(with_limit(
            rules(&["youtube.com"], monday_until_seven()),
            "x.com",
            60,
        ))
        .unwrap();
    let saves = daemon.store.saves();

    let status = daemon.report_limit("x.com", 30).unwrap();

    assert_eq!(status.exhausted, Vec::<LimitTarget>::new());
    assert_eq!(daemon.store.saves(), saves);
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn a_report_that_crosses_midnight_does_not_block_the_new_day() {
    let mut daemon = after_hours_with_limit();
    daemon.advance(4 * 60);
    let yesterday = daemon.today().pred_opt().unwrap();
    let saves = daemon.store.saves();

    let status = daemon.report_limit_on("x.com", 30, yesterday).unwrap();

    assert_eq!(status.exhausted, Vec::<LimitTarget>::new());
    assert_eq!(daemon.store.saves(), saves);
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn a_report_for_a_day_that_has_not_started_is_ignored() {
    let mut daemon = after_hours_with_limit();
    let tomorrow = daemon.today().succ_opt().unwrap();

    let status = daemon.report_limit_on("x.com", 30, tomorrow).unwrap();

    assert_eq!(status.exhausted, Vec::<LimitTarget>::new());
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn a_clock_set_back_does_not_free_an_exhausted_target() {
    let mut daemon = after_hours_with_limit();
    daemon.report_limit("x.com", 30).unwrap();
    assert!(daemon.hosts.blocks("x.com"));

    daemon.advance(-2 * 24 * 60);
    daemon.reconcile();

    assert!(daemon.hosts.blocks("x.com"));
    assert_eq!(daemon.status().exhausted, vec![site_target("x.com")]);
}

#[test]
fn a_report_above_a_lowered_limit_still_blocks() {
    let mut daemon = after_hours_with_limit();
    daemon
        .update(with_limit(
            rules(&["youtube.com"], monday_until_seven()),
            "x.com",
            20,
        ))
        .unwrap();

    let status = daemon.report_limit("x.com", 30).unwrap();

    assert_eq!(status.exhausted, vec![site_target("x.com")]);
    assert!(daemon.hosts.blocks("x.com"));
}

#[test]
fn the_block_ends_at_local_midnight() {
    let mut daemon = after_hours_with_limit();
    daemon.report_limit("x.com", 30).unwrap();
    assert!(daemon.hosts.blocks("x.com"));

    daemon.advance(3 * 60 + 59);
    daemon.reconcile();
    assert!(daemon.hosts.blocks("x.com"));

    daemon.advance(1);
    daemon.reconcile();
    assert!(!daemon.hosts.has_managed_block());
    assert_eq!(daemon.status().exhausted, Vec::<LimitTarget>::new());
    let saved = daemon.store.saved().unwrap();
    assert_eq!(saved.exhausted.day().to_string(), "2026-10-13");
}

#[test]
fn an_exhausted_target_stays_blocked_after_a_restart() {
    let mut daemon = after_hours_with_limit();
    daemon.report_limit("x.com", 30).unwrap();

    let fresh_hosts = MemoryHosts::default();
    let mut restarted = Service::new(
        daemon.clock.clone(),
        fresh_hosts.clone(),
        daemon.store.clone(),
    )
    .unwrap();
    let _flush = restarted.reconcile();
    assert!(fresh_hosts.blocks("x.com"));
    let response = restarted.handle(Request::Status);

    assert_eq!(
        response.response.into_result().unwrap().exhausted,
        vec![site_target("x.com")]
    );
}

#[test]
fn a_reported_limit_is_enforced_even_when_it_cannot_be_saved() {
    let mut daemon = after_hours_with_limit();
    let saves = daemon.store.saves();
    daemon.store.fail_saves(true);

    let status = daemon.report_limit("x.com", 30).unwrap();

    assert_eq!(status.exhausted, vec![site_target("x.com")]);
    assert!(daemon.hosts.blocks("x.com"));
    assert_eq!(daemon.store.saves(), saves);
}
