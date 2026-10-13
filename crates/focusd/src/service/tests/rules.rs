use focus_core::ipc::{ErrorCode, Request, Response};
use focus_core::test_support::{site_target, window};
use focus_core::{Day, Limit, LimitTarget, Schedule};

use crate::dns::PendingFlush;

use crate::service::test_support::{always_on, monday, monday_until_seven, rules, with_limit};

#[test]
fn tightening_is_accepted_while_blocking_and_reaches_the_hosts_file() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    assert!(daemon.hosts.blocks("youtube.com"));

    let status = daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();

    assert_eq!(status.rules.blocked_sites.len(), 2);
    assert!(status.enforcement.active);
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.hosts.blocks("reddit.com"));
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 2);
}

#[test]
fn tightening_the_schedule_and_adding_limits_is_accepted_while_blocking() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    let tighter = with_limit(rules(&["youtube.com"], always_on()), "x.com", 30);
    assert!(daemon.update(tighter).is_ok());
}

#[test]
fn loosening_is_rejected_while_blocking_and_changes_nothing() {
    let mut daemon = monday(
        10,
        0,
        rules(&["youtube.com", "reddit.com"], monday_until_seven()),
    );
    let saves = daemon.store.saves();
    let writes = daemon.hosts.writes();

    let error = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(error.message.contains("reddit.com"), "{}", error.message);
    assert_eq!(daemon.store.saves(), saves);
    assert_eq!(daemon.hosts.writes(), writes);
    assert!(daemon.hosts.blocks("reddit.com"));
    assert_eq!(daemon.status().rules.blocked_sites.len(), 2);
}

#[test]
fn shortening_the_schedule_or_raising_a_limit_is_a_loosening() {
    let original = with_limit(rules(&["youtube.com"], monday_until_seven()), "x.com", 30);
    let mut daemon = monday(10, 0, original);

    let shorter = Schedule {
        windows: vec![window(&[Day::Mon], 0, 12 * 60)],
    };
    let shortened = with_limit(rules(&["youtube.com"], shorter), "x.com", 30);
    assert_eq!(
        daemon.update(shortened).unwrap_err().code,
        ErrorCode::LooseningLocked
    );

    let mut raised = with_limit(rules(&["youtube.com"], monday_until_seven()), "x.com", 30);
    raised.limits = vec![Limit {
        target: site_target("x.com"),
        daily_minutes: 90,
    }];
    assert_eq!(
        daemon.update(raised).unwrap_err().code,
        ErrorCode::LooseningLocked
    );
}

#[test]
fn loosening_is_accepted_when_nothing_is_being_enforced() {
    let mut daemon = monday(
        20,
        0,
        rules(&["youtube.com", "reddit.com"], monday_until_seven()),
    );
    assert!(!daemon.hosts.has_managed_block());

    let status = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap();

    assert_eq!(status.rules.blocked_sites.len(), 1);
    assert!(!status.enforcement.active);
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn rules_that_fail_validation_are_rejected_without_side_effects() {
    let mut daemon = monday(20, 0, rules(&["youtube.com"], monday_until_seven()));
    let saves = daemon.store.saves();
    let invalid = with_limit(rules(&["youtube.com"], monday_until_seven()), "x.com", 0);

    let error = daemon.update(invalid).unwrap_err();

    assert_eq!(error.code, ErrorCode::RulesInvalid);
    assert_eq!(daemon.store.saves(), saves);
}

#[test]
fn status_reports_the_current_plan() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    let status = daemon.status();
    assert!(status.enforcement.active);
    assert_eq!(status.enforcement.blocked_sites.len(), 1);
    assert_eq!(status.exhausted, Vec::<LimitTarget>::new());
}

#[test]
fn a_failed_save_rejects_a_loosening_and_keeps_memory_equal_to_disk() {
    let mut daemon = monday(
        20,
        0,
        rules(&["youtube.com", "reddit.com"], monday_until_seven()),
    );
    daemon.store.fail_saves(true);

    let error = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::Internal);
    assert!(!error.message.contains("disk full"));
    assert_eq!(daemon.status().rules.blocked_sites.len(), 2);
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 2);
    daemon.store.fail_saves(false);
    assert!(daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .is_ok());
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 1);
}

#[test]
fn a_failed_save_does_not_stop_a_tightening_from_taking_effect() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.store.fail_saves(true);

    let status = daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();

    assert_eq!(status.rules.blocked_sites.len(), 2);
    assert!(daemon.hosts.blocks("reddit.com"));
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 1);
}

#[test]
fn an_unsaved_tightening_is_saved_by_the_next_tick_once_the_disk_recovers() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.store.fail_saves(true);
    daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();

    daemon.reconcile();
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 1);

    daemon.store.fail_saves(false);
    daemon.reconcile();
    assert_eq!(daemon.store.saved().unwrap().rules.blocked_sites.len(), 2);
    let saves = daemon.store.saves();
    daemon.reconcile();
    assert_eq!(daemon.store.saves(), saves);
}

#[test]
fn status_never_fails_because_a_save_is_pending() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.store.fail_saves(true);
    daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();
    daemon.advance(1);

    let status = daemon.send(Request::Status).unwrap();

    assert_eq!(status.rules.blocked_sites.len(), 2);
}

#[test]
fn a_loosening_after_an_unsaved_tightening_is_still_rejected_while_the_disk_fails() {
    let mut daemon = monday(20, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.store.fail_saves(true);
    daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();

    let error = daemon
        .update(rules(&["reddit.com"], monday_until_seven()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::Internal);
    assert_eq!(daemon.status().rules.blocked_sites.len(), 2);
}

#[test]
fn hosts_failures_do_not_reject_an_accepted_change() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.hosts.fail_writes(true);

    let status = daemon
        .update(rules(&["youtube.com", "reddit.com"], monday_until_seven()))
        .unwrap();

    assert_eq!(status.rules.blocked_sites.len(), 2);
    assert!(!daemon.hosts.blocks("reddit.com"));
    daemon.hosts.fail_writes(false);
    daemon.reconcile();
    assert!(daemon.hosts.blocks("reddit.com"));
}

#[test]
fn an_unreadable_hosts_file_is_left_alone() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.hosts.fail_reads(true);
    let before = daemon.hosts.content();
    assert!(daemon.send(Request::Status).is_ok());
    assert_eq!(daemon.hosts.content(), before);
}

#[test]
fn a_hosts_file_that_is_not_utf8_still_gets_the_block_and_keeps_its_bytes() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    let foreign = b"127.0.0.1 localhost\n# caf\xE9\n";
    daemon.hosts.set_content_bytes(foreign);

    daemon.reconcile();

    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.hosts.content_bytes().starts_with(foreign));
}

#[test]
fn a_flush_is_requested_only_when_the_hosts_file_changed() {
    let mut daemon = monday(20, 0, rules(&["youtube.com"], monday_until_seven()));
    assert_eq!(daemon.reconcile_flush(), PendingFlush::NONE);

    let tightening = rules(&["youtube.com", "reddit.com"], always_on());
    let handled = daemon.handle(Request::UpdateRules { rules: tightening });
    assert_eq!(handled.flush, PendingFlush::NEEDED);

    assert_eq!(daemon.handle(Request::Status).flush, PendingFlush::NONE);
    assert_eq!(daemon.reconcile_flush(), PendingFlush::NONE);

    daemon.hosts.set_content("127.0.0.1 localhost\n");
    assert_eq!(daemon.reconcile_flush(), PendingFlush::NEEDED);
}

#[test]
fn a_rejected_request_requests_no_flush() {
    let mut daemon = monday(
        10,
        0,
        rules(&["youtube.com", "reddit.com"], monday_until_seven()),
    );
    let handled = daemon.handle(Request::UpdateRules {
        rules: rules(&["youtube.com"], monday_until_seven()),
    });
    assert!(matches!(handled.response, Response::Err { .. }));
    assert_eq!(handled.flush, PendingFlush::NONE);
}

fn after_hours_with_limit() -> crate::service::test_support::Harness {
    let original = with_limit(rules(&["youtube.com"], monday_until_seven()), "x.com", 30);
    monday(20, 0, original)
}

fn limit_rules(minutes: u16) -> focus_core::Rules {
    with_limit(
        rules(&["youtube.com"], monday_until_seven()),
        "x.com",
        minutes,
    )
}

#[test]
fn raising_a_limit_is_locked_even_outside_every_window() {
    let mut daemon = after_hours_with_limit();
    assert!(!daemon.status().enforcement.active);
    let saves = daemon.store.saves();

    let error = daemon.update(limit_rules(90)).unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(error.message.contains("x.com"), "{}", error.message);
    assert!(!error.message.contains(':'), "{}", error.message);
    assert_eq!(daemon.store.saves(), saves);
}

#[test]
fn removing_a_limit_is_locked_even_outside_every_window() {
    let mut daemon = after_hours_with_limit();

    let error = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert_eq!(daemon.status().rules.limits.len(), 1);
}

#[test]
fn adding_or_lowering_a_limit_stays_free_outside_every_window() {
    let mut daemon = after_hours_with_limit();

    assert!(daemon.update(limit_rules(20)).is_ok());

    let mut two = limit_rules(20);
    two.limits.push(Limit {
        target: site_target("reddit.com"),
        daily_minutes: 45,
    });
    assert!(daemon.update(two).is_ok());
}

#[test]
fn other_loosenings_stay_free_outside_every_window_while_limits_are_locked() {
    let mut daemon = after_hours_with_limit();

    let mixed = with_limit(rules(&[], monday_until_seven()), "x.com", 90);
    let error = daemon.update(mixed).unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(!error.message.contains("youtube.com"), "{}", error.message);
    assert_eq!(daemon.status().rules.blocked_sites.len(), 1);
    assert!(daemon
        .update(with_limit(rules(&[], monday_until_seven()), "x.com", 30))
        .is_ok());
}

#[test]
fn an_unlock_requested_for_a_locked_limit_change_lets_it_through() {
    let mut daemon = after_hours_with_limit();
    assert_eq!(
        daemon.update(limit_rules(90)).unwrap_err().code,
        ErrorCode::LooseningLocked
    );

    daemon.unlock(15);
    let status = daemon.update(limit_rules(90)).unwrap();

    assert_eq!(status.rules.limits[0].daily_minutes, 90);
    daemon.advance(16);
    daemon.reconcile();
    assert_eq!(daemon.status().rules.limits[0].daily_minutes, 90);
    assert_eq!(
        daemon.update(limit_rules(120)).unwrap_err().code,
        ErrorCode::LooseningLocked
    );
}

#[test]
fn an_unlock_can_be_requested_and_cancelled_outside_every_window() {
    let mut daemon = after_hours_with_limit();
    assert!(daemon.send(Request::RequestUnlock).is_ok());
    assert!(daemon.send(Request::CancelUnlock).is_ok());
    assert!(daemon.send(Request::RequestUnlock).is_ok());
}
