use focus_core::ipc::Request;
use focus_core::UNLOCK_SENTENCE;

use crate::service::test_support::{monday, monday_until_seven, rules};

#[test]
fn startup_reconcile_applies_the_block_before_any_client_connects() {
    let daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.hosts.blocks("www.youtube.com"));
}

#[test]
fn reconcile_keeps_unrelated_lines_and_removes_the_legacy_block() {
    let daemon = {
        let mut daemon = monday(20, 0, rules(&["youtube.com"], monday_until_seven()));
        daemon.hosts.set_content(
            "127.0.0.1 localhost\n# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n",
        );
        daemon.reconcile();
        daemon
    };
    assert_eq!(daemon.hosts.content(), "127.0.0.1 localhost\n");
}

#[test]
fn reconcile_repairs_a_tampered_managed_block() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.hosts.set_content("127.0.0.1 localhost\n");

    daemon.reconcile();

    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.hosts.content().starts_with("127.0.0.1 localhost\n"));
}

#[test]
fn reconcile_is_quiet_when_nothing_changed() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    let (writes, saves) = (daemon.hosts.writes(), daemon.store.saves());

    daemon.reconcile();
    daemon.reconcile();

    assert_eq!(
        (daemon.hosts.writes(), daemon.store.saves()),
        (writes, saves)
    );
}

#[test]
fn an_expired_unlock_is_persisted_and_the_block_returns() {
    let mut daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.send(Request::RequestUnlock).unwrap();
    daemon.advance(5);
    daemon
        .send(Request::ConfirmUnlock {
            phrase: UNLOCK_SENTENCE.to_owned(),
            minutes: 15,
        })
        .unwrap();
    assert!(!daemon.hosts.has_managed_block());

    daemon.advance(15);
    daemon.reconcile();

    assert!(daemon.hosts.blocks("youtube.com"));
    let saved = daemon.store.saved().unwrap();
    assert_eq!(saved.unlock.phase(), focus_core::UnlockPhase::Locked);
}

#[test]
fn a_failed_save_during_reconcile_is_retried_on_the_next_tick() {
    let mut daemon = monday(23, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon.store.fail_saves(true);
    daemon.advance(61);

    daemon.reconcile();
    assert_eq!(
        daemon.store.saved().unwrap().exhausted.day().to_string(),
        "2026-10-12"
    );

    daemon.store.fail_saves(false);
    daemon.reconcile();
    assert_eq!(
        daemon.store.saved().unwrap().exhausted.day().to_string(),
        "2026-10-13"
    );
}
