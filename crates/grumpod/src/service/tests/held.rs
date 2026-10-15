use focus_core::hosts::render_bytes;
use focus_core::ipc::ErrorCode;

use crate::clock::test_support::FakeClock;
use crate::service::test_support::{
    always_on, monday, monday_until_seven, rules, sites, Harness, MemoryHosts, MemoryStore,
};
use crate::service::{Daemon, Service};
use crate::state::{FileStateStore, StateError};

const LOCALHOST: &str = "127.0.0.1 localhost\n";

fn hosts_blocking(blocked: &[&str]) -> MemoryHosts {
    let hosts = MemoryHosts::default();
    let bytes = render_bytes(LOCALHOST.as_bytes(), &sites(blocked));
    hosts.set_content_bytes(&bytes);
    hosts
}

fn clock() -> FakeClock {
    FakeClock::at(2026, 10, 12, 20, 0)
}

/// Starts a daemon on `hosts` and `store`, then reconciles like the run command does.
fn start(clock: FakeClock, hosts: MemoryHosts, store: MemoryStore) -> Harness {
    let mut service = Service::new(clock.clone(), hosts.clone(), store.clone()).unwrap();
    let _flush = service.reconcile();
    Harness {
        service,
        clock,
        hosts,
        store,
    }
}

/// A daemon that finds a managed block for `blocked` and no saved state at all.
fn lost_state(blocked: &[&str]) -> Harness {
    start(clock(), hosts_blocking(blocked), MemoryStore::default())
}

fn restart(previous: Harness) -> Harness {
    start(previous.clock, previous.hosts, previous.store)
}

#[test]
fn lost_state_keeps_the_block_it_finds_through_reconcile() {
    let mut daemon = lost_state(&["youtube.com", "reddit.com"]);
    let before = daemon.hosts.content();

    daemon.reconcile();

    assert!(daemon.hosts.blocks("www.youtube.com"));
    assert!(daemon.hosts.blocks("old.reddit.com"));
    assert_eq!(daemon.hosts.content(), before);
    assert_eq!(daemon.hosts.writes(), 0);
}

#[test]
fn held_sites_are_saved_and_still_blocked_after_a_restart() {
    let daemon = lost_state(&["youtube.com", "m.tiktok.com"]);
    let saved = daemon.store.saved().unwrap();
    assert_eq!(saved.held_sites, sites(&["youtube.com", "m.tiktok.com"]));
    assert!(saved.rules.blocked_sites.is_empty());

    let mut restarted = restart(daemon);

    let held = restarted.status().held_sites;
    assert_eq!(held, sites(&["youtube.com", "m.tiktok.com"]));
    assert!(restarted.hosts.blocks("youtube.com"));
    assert!(restarted.hosts.blocks("m.tiktok.com"));
}

#[test]
fn an_unlock_releases_the_held_sites_but_keeps_them_held() {
    let mut daemon = lost_state(&["youtube.com"]);

    daemon.unlock(15);
    daemon.reconcile();

    assert!(!daemon.hosts.blocks("youtube.com"));
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
    assert_eq!(
        daemon.store.saved().unwrap().held_sites,
        sites(&["youtube.com"])
    );
}

#[test]
fn held_sites_come_back_when_the_unlock_ends() {
    let mut daemon = lost_state(&["youtube.com"]);
    daemon.unlock(15);
    daemon.reconcile();
    assert!(!daemon.hosts.blocks("youtube.com"));

    daemon.advance(16);
    daemon.reconcile();

    assert!(daemon.hosts.blocks("youtube.com"));
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
}

#[test]
fn held_sites_released_by_an_unlock_survive_a_restart_and_return_after_it() {
    let mut daemon = lost_state(&["youtube.com"]);
    daemon.unlock(15);
    daemon.reconcile();

    let mut restarted = restart(daemon);
    assert!(!restarted.hosts.blocks("youtube.com"));
    assert_eq!(restarted.status().held_sites, sites(&["youtube.com"]));

    restarted.advance(16);
    restarted.reconcile();
    assert!(restarted.hosts.blocks("youtube.com"));
}

#[test]
fn status_reports_the_held_sites() {
    let mut daemon = lost_state(&["youtube.com"]);
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
}

#[test]
fn saving_rules_that_block_the_held_sites_takes_them_over() {
    let mut daemon = lost_state(&["youtube.com"]);

    let status = daemon
        .update(rules(&["youtube.com", "tiktok.com"], always_on()))
        .unwrap();

    assert!(status.held_sites.is_empty());
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.hosts.blocks("tiktok.com"));
    assert!(daemon.store.saved().unwrap().held_sites.is_empty());
    assert!(restart(daemon).status().held_sites.is_empty());
}

#[test]
fn empty_rules_cannot_release_held_sites_while_locked() {
    let mut daemon = lost_state(&["youtube.com"]);
    let saves = daemon.store.saves();

    let error = daemon.update(rules(&[], always_on())).unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(error.message.contains("youtube.com"), "{}", error.message);
    assert_eq!(daemon.store.saves(), saves);
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
    assert!(daemon.hosts.blocks("youtube.com"));
}

#[test]
fn rules_that_block_other_sites_cannot_release_held_sites_while_locked() {
    let mut daemon = lost_state(&["youtube.com"]);

    let error = daemon
        .update(rules(&["tiktok.com"], always_on()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(daemon.hosts.blocks("youtube.com"));
}

#[test]
fn rules_that_only_block_outside_the_current_time_cannot_release_held_sites() {
    let mut daemon = lost_state(&["youtube.com"]);

    let error = daemon
        .update(rules(&["youtube.com"], monday_until_seven()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
}

#[test]
fn releasing_only_some_held_sites_is_rejected_and_names_the_rest() {
    let mut daemon = lost_state(&["youtube.com", "reddit.com"]);

    let error = daemon
        .update(rules(&["youtube.com"], always_on()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert!(error.message.contains("reddit.com"), "{}", error.message);
    assert!(!error.message.contains("youtube.com"), "{}", error.message);
    assert_eq!(
        daemon.status().held_sites,
        sites(&["youtube.com", "reddit.com"])
    );
}

#[test]
fn a_broader_rule_does_not_keep_a_held_subdomain() {
    let mut daemon = lost_state(&["studio.youtube.com"]);

    let error = daemon
        .update(rules(&["youtube.com"], always_on()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::LooseningLocked);
}

#[test]
fn a_rule_for_the_registrable_domain_keeps_its_held_alias() {
    let mut daemon = lost_state(&["www.youtube.com"]);

    let status = daemon.update(rules(&["youtube.com"], always_on())).unwrap();

    assert!(status.held_sites.is_empty());
    assert!(daemon.hosts.blocks("www.youtube.com"));
}

#[test]
fn an_unlock_lets_saved_rules_release_the_held_sites() {
    let mut daemon = lost_state(&["youtube.com"]);
    daemon.unlock(15);

    let status = daemon.update(rules(&["tiktok.com"], always_on())).unwrap();

    assert!(status.held_sites.is_empty());
    assert!(daemon.store.saved().unwrap().held_sites.is_empty());
    daemon.reconcile();
    assert!(!daemon.hosts.blocks("youtube.com"));
}

#[test]
fn releasing_held_sites_while_unlocked_without_blocking_anything_clears_the_block() {
    let mut daemon = lost_state(&["youtube.com"]);
    daemon.unlock(15);

    daemon
        .update(rules(&["tiktok.com"], monday_until_seven()))
        .unwrap();
    daemon.reconcile();

    assert!(!daemon.hosts.has_managed_block());
    assert!(daemon.store.saved().unwrap().held_sites.is_empty());
}

#[test]
fn releasing_held_sites_is_rejected_when_it_cannot_be_saved() {
    let mut daemon = lost_state(&["youtube.com"]);
    daemon.store.fail_saves(true);

    let error = daemon
        .update(rules(&["youtube.com"], always_on()))
        .unwrap_err();

    assert_eq!(error.code, ErrorCode::Internal);
    assert_eq!(daemon.status().held_sites, sites(&["youtube.com"]));
    assert!(daemon.hosts.blocks("youtube.com"));
}

#[test]
fn a_failed_save_of_held_sites_keeps_the_block_and_is_retried() {
    let store = MemoryStore::default();
    store.fail_saves(true);
    let mut daemon = start(clock(), hosts_blocking(&["youtube.com"]), store);
    assert!(daemon.hosts.blocks("youtube.com"));
    assert!(daemon.store.saved().is_none());

    daemon.store.fail_saves(false);
    daemon.reconcile();

    let saved = daemon.store.saved().unwrap();
    assert_eq!(saved.held_sites, sites(&["youtube.com"]));
}

#[test]
fn a_fresh_install_without_a_block_stays_clean() {
    let mut daemon = lost_state(&[]);

    daemon.reconcile();

    assert!(daemon.status().held_sites.is_empty());
    assert_eq!(daemon.store.saves(), 0);
    assert_eq!(daemon.hosts.writes(), 0);
    assert!(!daemon.hosts.has_managed_block());
}

#[test]
fn a_missing_hosts_file_starts_with_nothing_held() {
    let hosts = MemoryHosts::default();
    hosts.set_missing(true);

    let mut daemon = start(clock(), hosts, MemoryStore::default());

    assert!(daemon.status().held_sites.is_empty());
    assert_eq!(daemon.store.saves(), 0);
}

#[test]
fn an_unreadable_hosts_file_stops_the_start_instead_of_risking_the_block() {
    let hosts = hosts_blocking(&["youtube.com"]);
    hosts.fail_reads(true);

    let error = Service::new(clock(), hosts.clone(), MemoryStore::default()).unwrap_err();

    assert!(matches!(error, StateError::HostsUnreadable(_)), "{error}");
    assert_eq!(hosts.writes(), 0);
}

#[test]
fn saved_rules_win_over_whatever_the_hosts_block_lists() {
    let daemon = monday(10, 0, rules(&["youtube.com"], monday_until_seven()));
    daemon
        .hosts
        .set_content_bytes(&render_bytes(b"", &sites(&["reddit.com"])));

    let mut restarted = restart(daemon);

    assert!(restarted.status().held_sites.is_empty());
    assert!(restarted.hosts.blocks("youtube.com"));
    assert!(!restarted.hosts.blocks("reddit.com"));
}

#[test]
fn a_damaged_state_file_keeps_the_block_through_start_reconcile_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    std::fs::write(dir.path().join("state.json"), "{ not json").unwrap();
    let hosts = hosts_blocking(&["youtube.com"]);

    let store = FileStateStore::open(dir.path()).unwrap();
    let mut service = Service::new(clock(), hosts.clone(), store).unwrap();
    let _flush = service.reconcile();
    assert!(hosts.blocks("youtube.com"));

    let store = FileStateStore::open(dir.path()).unwrap();
    let mut again = Service::new(clock(), hosts.clone(), store).unwrap();
    let _flush = again.reconcile();
    assert!(hosts.blocks("youtube.com"));
    let saved = std::fs::read_to_string(dir.path().join("state.json")).unwrap();
    assert!(saved.contains("heldSites"), "{saved}");
}
