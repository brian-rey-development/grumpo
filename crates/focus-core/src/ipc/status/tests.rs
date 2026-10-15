use chrono::Duration;
use serde_json::json;

use super::*;
use crate::ipc::{EnforcementReason, UnlockPhaseView};
use crate::test_support::{exhausted_by, now_at, sample_rules, site, site_target, valid};
use crate::unlock::{UnlockPhase, UNLOCK_SENTENCE};

fn unlock_from(json: serde_json::Value) -> UnlockState {
    serde_json::from_value(json).unwrap()
}

#[test]
fn status_reflects_an_enforcing_daemon() {
    let now = now_at(2026, 10, 12, 10, 0);
    let exhausted = exhausted_by(&sample_rules(), &[site_target("x.com")], &now);
    let rules = sample_rules();
    let status = DaemonStatus::build(&rules, &UnlockState::new(&now), &exhausted, &now);

    assert_eq!(status.rules, Rules::clone(&rules));
    assert!(status.enforcement.active);
    assert_eq!(status.enforcement.reason, EnforcementReason::Both);
    assert_eq!(
        status.enforcement.active_until,
        Some(
            crate::test_support::local(2026, 10, 13, 0, 0)
                .and_utc()
                .timestamp()
        )
    );
    assert!(status.enforcement.blocked_sites.contains(&site("x.com")));
    assert_eq!(status.exhausted, [site_target("x.com")]);
    assert_eq!(status.unlock.phase, UnlockPhaseView::Locked);
    assert_eq!(status.unlock.next_cooldown_minutes, 5);
    assert_eq!(status.unlock.sentence, UNLOCK_SENTENCE);
    assert_eq!(status.unlock.durations, [15, 30, 60]);
}

#[test]
fn status_of_an_idle_daemon_has_no_end_time() {
    let now = now_at(2026, 10, 14, 20, 0);
    let status = DaemonStatus::build(
        &sample_rules(),
        &UnlockState::new(&now),
        &Exhausted::new(&now),
        &now,
    );
    assert!(!status.enforcement.active);
    assert_eq!(status.enforcement.reason, EnforcementReason::None);
    assert_eq!(status.enforcement.active_until, None);
    assert_eq!(status.exhausted.len(), 0);
}

#[test]
fn status_advances_the_unlock_state_to_the_present() {
    let now = now_at(2026, 10, 12, 10, 0);
    let ready_at = now.utc() - Duration::minutes(1);
    let stale = unlock_from(json!({
        "phase": {"kind": "coolingDown", "readyAt": ready_at, "minutes": 20},
        "unlocksToday": 2,
        "day": "2026-10-12",
    }));
    let status = DaemonStatus::build(&sample_rules(), &stale, &Exhausted::new(&now), &now);
    assert_eq!(
        status.unlock.phase,
        UnlockPhaseView::Ready {
            ready_at: ready_at.timestamp(),
            expires_at: (now.utc() + Duration::minutes(4)).timestamp()
        }
    );
    assert_eq!(status.unlock.next_cooldown_minutes, 20);
}

#[test]
fn an_unlock_that_ran_out_shows_as_locked_and_enforcing_in_the_same_snapshot() {
    let now = now_at(2026, 10, 12, 10, 0);
    let stale = unlock_from(json!({
        "phase": {"kind": "unlocked", "until": now.utc() - Duration::minutes(1), "minutes": 15},
        "unlocksToday": 1,
        "day": "2026-10-12",
    }));
    let status = DaemonStatus::build(&sample_rules(), &stale, &Exhausted::new(&now), &now);
    assert_eq!(status.unlock.phase, UnlockPhaseView::Locked);
    assert!(status.enforcement.active);
}

#[test]
fn a_running_unlock_shows_as_unlocked_and_not_enforcing() {
    let now = now_at(2026, 10, 12, 10, 0);
    let until = now.utc() + Duration::minutes(10);
    let running = unlock_from(json!({
        "phase": {"kind": "unlocked", "until": until, "minutes": 15},
        "unlocksToday": 1,
        "day": "2026-10-12",
    }));
    assert!(matches!(running.phase(), UnlockPhase::Unlocked { .. }));
    let status = DaemonStatus::build(&sample_rules(), &running, &Exhausted::new(&now), &now);
    assert_eq!(
        status.unlock.phase,
        UnlockPhaseView::Unlocked {
            until: until.timestamp()
        }
    );
    assert!(!status.enforcement.active);
}

#[test]
fn status_hides_exhausted_targets_from_earlier_days() {
    let now = now_at(2026, 10, 12, 10, 0);
    let yesterday = now_at(2026, 10, 11, 10, 0);
    let stale = exhausted_by(&sample_rules(), &[site_target("x.com")], &yesterday);
    let status = DaemonStatus::build(&sample_rules(), &UnlockState::new(&now), &stale, &now);
    assert_eq!(status.exhausted.len(), 0);
}

#[test]
fn status_lists_only_targets_whose_limit_is_still_exhausted() {
    let now = now_at(2026, 10, 12, 10, 0);
    let spent = exhausted_by(&sample_rules(), &[site_target("x.com")], &now);
    let raised = valid(Rules {
        limits: vec![crate::test_support::limit(site_target("x.com"), 45)],
        ..Rules::clone(&sample_rules())
    });
    let status = DaemonStatus::build(&raised, &UnlockState::new(&now), &spent, &now);
    assert_eq!(status.exhausted.len(), 0);
}

#[test]
fn status_serializes_with_camel_case_keys_and_no_version() {
    let now = now_at(2026, 10, 12, 10, 0);
    let status = crate::test_support::sample_status(&now);
    let value = serde_json::to_value(&status).unwrap();
    for key in ["rules", "enforcement", "exhausted", "unlock", "heldSites"] {
        assert!(value.get(key).is_some(), "missing {key}");
    }
    assert!(value.get("protocolVersion").is_none());
    assert!(value["enforcement"].get("activeUntil").is_some());
    assert!(value["enforcement"].get("blockedSites").is_some());
    assert!(value["unlock"].get("nextCooldownMinutes").is_some());
    assert_eq!(
        serde_json::from_value::<DaemonStatus>(value).unwrap(),
        status
    );
}

#[test]
fn a_status_from_a_daemon_without_held_sites_decodes_with_none() {
    let now = now_at(2026, 10, 12, 10, 0);
    let mut value = serde_json::to_value(crate::test_support::sample_status(&now)).unwrap();
    value.as_object_mut().unwrap().remove("heldSites");
    let status: DaemonStatus = serde_json::from_value(value).unwrap();
    assert!(status.held_sites.is_empty());
}

fn idle_status() -> DaemonStatus {
    let now = now_at(2026, 10, 14, 20, 0);
    DaemonStatus::build(
        &sample_rules(),
        &UnlockState::new(&now),
        &Exhausted::new(&now),
        &now,
    )
}

#[test]
fn held_sites_make_an_idle_status_active_without_an_end() {
    let status = DaemonStatus {
        held_sites: [site("youtube.com")].into(),
        ..idle_status()
    }
    .with_held_enforced();

    assert!(status.enforcement.active);
    assert_eq!(status.enforcement.reason, EnforcementReason::Schedule);
    assert_eq!(status.enforcement.active_until, None);
    assert_eq!(
        status.enforcement.blocked_sites,
        [site("youtube.com")].into()
    );
    assert_eq!(status.held_sites, [site("youtube.com")].into());
}

#[test]
fn held_sites_join_the_sites_of_an_active_plan_and_keep_its_reason_and_end() {
    let now = now_at(2026, 10, 12, 10, 0);
    let plan = DaemonStatus::build(
        &sample_rules(),
        &UnlockState::new(&now),
        &Exhausted::new(&now),
        &now,
    );
    let held = DaemonStatus {
        held_sites: [site("youtube.com")].into(),
        ..plan.clone()
    }
    .with_held_enforced();

    assert!(held
        .enforcement
        .blocked_sites
        .contains(&site("youtube.com")));
    assert!(plan
        .enforcement
        .blocked_sites
        .iter()
        .all(|planned| held.enforcement.blocked_sites.contains(planned)));
    assert_eq!(held.enforcement.reason, plan.enforcement.reason);
    assert_eq!(held.enforcement.active_until, plan.enforcement.active_until);
}

#[test]
fn a_status_without_held_sites_is_left_as_it_is() {
    let status = idle_status();

    assert_eq!(status.clone().with_held_enforced(), status);
}

#[test]
fn held_sites_are_not_counted_while_an_unlock_runs() {
    let now = now_at(2026, 10, 12, 10, 0);
    let running = unlock_from(json!({
        "phase": {"kind": "unlocked", "until": now.utc() + Duration::minutes(10), "minutes": 15},
        "unlocksToday": 1,
        "day": "2026-10-12",
    }));
    let status = DaemonStatus {
        held_sites: [site("youtube.com")].into(),
        ..DaemonStatus::build(&sample_rules(), &running, &Exhausted::new(&now), &now)
    };

    let enforced = status.clone().with_held_enforced();

    assert_eq!(enforced, status);
    assert!(!enforced.enforcement.active);
    assert!(enforced.enforcement.blocked_sites.is_empty());
    assert_eq!(enforced.held_sites, [site("youtube.com")].into());
}
