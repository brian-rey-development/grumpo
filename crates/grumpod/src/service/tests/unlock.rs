use focus_core::ipc::{ErrorCode, Request, UnlockPhaseView};
use focus_core::UNLOCK_SENTENCE;

use crate::service::test_support::{monday, monday_until_seven, rules, Harness};
use crate::service::{Daemon, Service};

const COOLDOWN_MINUTES: i64 = 5;

fn blocked_day() -> Harness {
    monday(
        10,
        0,
        rules(&["youtube.com", "reddit.com"], monday_until_seven()),
    )
}

fn confirm(minutes: u16) -> Request {
    Request::ConfirmUnlock {
        phrase: UNLOCK_SENTENCE.to_owned(),
        minutes,
    }
}

#[test]
fn the_full_flow_lifts_the_block_allows_loosening_and_relocks_on_expiry() {
    let mut daemon = blocked_day();

    let status = daemon.send(Request::RequestUnlock).unwrap();
    assert!(matches!(
        status.unlock.phase,
        UnlockPhaseView::CoolingDown { .. }
    ));
    assert!(daemon.hosts.blocks("youtube.com"));

    daemon.advance(COOLDOWN_MINUTES - 1);
    let early = daemon.send(confirm(15)).unwrap_err();
    assert_eq!(early.code, ErrorCode::UnlockNotReady);

    daemon.advance(1);
    let status = daemon.send(confirm(15)).unwrap();
    assert!(matches!(
        status.unlock.phase,
        UnlockPhaseView::Unlocked { .. }
    ));
    assert!(!status.enforcement.active);
    assert!(!daemon.hosts.has_managed_block());
    assert_eq!(status.unlock.next_cooldown_minutes, 10);

    let loosened = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap();
    assert_eq!(loosened.rules.blocked_sites.len(), 1);

    daemon.advance(16);
    daemon.reconcile();
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(!daemon.hosts.blocks("reddit.com"));
    assert!(matches!(
        daemon.status().unlock.phase,
        UnlockPhaseView::Locked
    ));
}

#[test]
fn relock_restores_the_block_immediately() {
    let mut daemon = blocked_day();
    daemon.unlock(30);
    assert!(!daemon.hosts.has_managed_block());

    let status = daemon.send(Request::Relock).unwrap();

    assert!(status.enforcement.active);
    assert!(daemon.hosts.blocks("reddit.com"));
}

#[test]
fn cancel_abandons_a_pending_request() {
    let mut daemon = blocked_day();
    daemon.send(Request::RequestUnlock).unwrap();

    let status = daemon.send(Request::CancelUnlock).unwrap();

    assert!(matches!(status.unlock.phase, UnlockPhaseView::Locked));
    assert_eq!(
        daemon.send(Request::CancelUnlock).unwrap_err().code,
        ErrorCode::UnlockNotRequested
    );
}

#[test]
fn unlock_errors_map_to_their_codes() {
    let mut daemon = blocked_day();
    assert_eq!(
        daemon.send(confirm(15)).unwrap_err().code,
        ErrorCode::UnlockNotRequested
    );

    daemon.send(Request::RequestUnlock).unwrap();
    assert_eq!(
        daemon.send(Request::RequestUnlock).unwrap_err().code,
        ErrorCode::UnlockAlreadyPending
    );

    daemon.advance(COOLDOWN_MINUTES);
    let wrong = Request::ConfirmUnlock {
        phrase: "I would like to stop".to_owned(),
        minutes: 15,
    };
    assert_eq!(
        daemon.send(wrong).unwrap_err().code,
        ErrorCode::PhraseMismatch
    );
    assert_eq!(
        daemon.send(confirm(7)).unwrap_err().code,
        ErrorCode::DurationInvalid
    );

    daemon.send(confirm(15)).unwrap();
    assert_eq!(
        daemon.send(Request::RequestUnlock).unwrap_err().code,
        ErrorCode::UnlockAlreadyActive
    );
}

#[test]
fn a_ready_request_lapses_if_it_is_not_confirmed_in_time() {
    let mut daemon = blocked_day();
    daemon.send(Request::RequestUnlock).unwrap();
    daemon.advance(COOLDOWN_MINUTES + 6);

    assert_eq!(
        daemon.send(confirm(15)).unwrap_err().code,
        ErrorCode::UnlockNotRequested
    );
}

#[test]
fn unlock_progress_survives_a_restart() {
    let mut daemon = blocked_day();
    daemon.send(Request::RequestUnlock).unwrap();

    let mut restarted = Service::new(
        daemon.clock.clone(),
        daemon.hosts.clone(),
        daemon.store.clone(),
    )
    .unwrap();
    let status = restarted
        .handle(Request::Status)
        .response
        .into_result()
        .unwrap();

    assert!(matches!(
        status.unlock.phase,
        UnlockPhaseView::CoolingDown { .. }
    ));
}

#[test]
fn relock_and_cancel_still_work_when_the_disk_fails() {
    let mut daemon = blocked_day();
    daemon.unlock(30);
    daemon.store.fail_saves(true);

    let status = daemon.send(Request::Relock).unwrap();

    assert!(status.enforcement.active);
    assert!(daemon.hosts.blocks("reddit.com"));

    daemon.store.fail_saves(false);
    daemon.send(Request::RequestUnlock).unwrap();
    daemon.store.fail_saves(true);
    let status = daemon.send(Request::CancelUnlock).unwrap();
    assert!(matches!(status.unlock.phase, UnlockPhaseView::Locked));
}

#[test]
fn starting_or_confirming_an_unlock_is_rejected_when_it_cannot_be_saved() {
    let mut daemon = blocked_day();
    daemon.store.fail_saves(true);
    let error = daemon.send(Request::RequestUnlock).unwrap_err();
    assert_eq!(error.code, ErrorCode::Internal);
    assert!(matches!(
        daemon.status().unlock.phase,
        UnlockPhaseView::Locked
    ));

    daemon.store.fail_saves(false);
    daemon.send(Request::RequestUnlock).unwrap();
    daemon.advance(COOLDOWN_MINUTES);
    daemon.store.fail_saves(true);
    let error = daemon.send(confirm(15)).unwrap_err();
    assert_eq!(error.code, ErrorCode::Internal);
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.status().enforcement.active);
}

#[test]
fn a_clock_set_back_during_an_unlock_brings_the_block_back() {
    let mut daemon = blocked_day();
    daemon.unlock(15);
    assert!(!daemon.hosts.has_managed_block());

    daemon.advance(-30);
    daemon.reconcile();

    assert!(daemon.hosts.blocks("youtube.com"));
    let status = daemon.status();
    assert!(matches!(status.unlock.phase, UnlockPhaseView::Locked));
    assert!(status.enforcement.active);
}

#[test]
fn a_clock_set_back_during_the_cooldown_restarts_it() {
    let mut daemon = blocked_day();
    daemon.send(Request::RequestUnlock).unwrap();

    daemon.advance(-60);
    daemon.reconcile();
    daemon.advance(COOLDOWN_MINUTES - 1);
    let early = daemon.send(confirm(15)).unwrap_err();
    assert_eq!(early.code, ErrorCode::UnlockNotReady);

    daemon.advance(1);
    assert!(daemon.send(confirm(15)).is_ok());
}
