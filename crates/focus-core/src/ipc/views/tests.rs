use chrono::Duration;
use serde_json::json;

use super::*;
use crate::test_support::{now_at, site};
use crate::unlock::UnlockState;

#[test]
fn unlock_phase_views_use_unix_seconds_and_a_kind_tag() {
    let now = now_at(2026, 10, 12, 10, 0);
    let at = now.utc() + Duration::minutes(15);
    let seconds = at.timestamp();
    let cases = [
        (UnlockPhase::Locked, json!({"kind": "locked"})),
        (
            UnlockPhase::CoolingDown {
                ready_at: at,
                minutes: 20,
            },
            json!({"kind": "coolingDown", "readyAt": seconds, "minutes": 20}),
        ),
        (
            UnlockPhase::Ready {
                ready_at: at - Duration::minutes(5),
                expires_at: at,
            },
            json!({"kind": "ready", "readyAt": seconds - 300, "expiresAt": seconds}),
        ),
        (
            UnlockPhase::Unlocked {
                until: at,
                minutes: 15,
            },
            json!({"kind": "unlocked", "until": seconds}),
        ),
    ];
    for (phase, expected) in cases {
        assert_eq!(
            serde_json::to_value(UnlockPhaseView::from(phase)).unwrap(),
            expected
        );
    }
}

#[test]
fn an_inactive_plan_shows_reason_none_and_no_targets() {
    let view = EnforcementView::from(&EnforcementPlan::Inactive);
    assert_eq!(
        view,
        EnforcementView {
            active: false,
            reason: EnforcementReason::None,
            active_until: None,
            blocked_sites: BTreeSet::new(),
            blocked_apps: BTreeSet::new(),
        }
    );
}

#[test]
fn an_active_plan_maps_each_reason_and_the_end_time() {
    let now = now_at(2026, 10, 12, 10, 0);
    let cases = [
        (ActiveReason::Schedule, EnforcementReason::Schedule),
        (ActiveReason::Limit, EnforcementReason::Limit),
        (ActiveReason::Both, EnforcementReason::Both),
    ];
    for (reason, expected) in cases {
        let plan = EnforcementPlan::Active {
            reason,
            until: Some(now.utc()),
            sites: [site("x.com")].into(),
            apps: BTreeSet::new(),
        };
        let view = EnforcementView::from(&plan);
        assert!(view.active);
        assert_eq!(view.reason, expected);
        assert_eq!(view.active_until, Some(now.utc().timestamp()));
        assert_eq!(view.blocked_sites, [site("x.com")].into());
    }
}

#[test]
fn an_active_plan_without_an_end_has_a_null_end_time() {
    let plan = EnforcementPlan::Active {
        reason: ActiveReason::Schedule,
        until: None,
        sites: BTreeSet::new(),
        apps: BTreeSet::new(),
    };
    let value = serde_json::to_value(EnforcementView::from(&plan)).unwrap();
    assert_eq!(value["activeUntil"], serde_json::Value::Null);
    assert_eq!(value["reason"], json!("schedule"));
}

#[test]
fn the_unlock_view_carries_the_script_the_ui_needs() {
    let now = now_at(2026, 10, 12, 10, 0);
    let view = UnlockView::from(&UnlockState::new(&now));
    assert_eq!(view.phase, UnlockPhaseView::Locked);
    assert_eq!(view.next_cooldown_minutes, 5);
    assert_eq!(view.sentence, UNLOCK_SENTENCE);
    assert_eq!(view.durations, [15, 30, 60]);
}
