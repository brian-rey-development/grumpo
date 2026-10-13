use serde_json::json;

use focus_core::{AppRef, UnlockPhase};

use super::*;
use focus_core::test_support::now_at;

fn v1_document() -> Value {
    json!({
        "schemaVersion": 1,
        "rules": {
            "blockedSites": ["youtube.com"],
            "blockedApps": [],
            "schedule": { "windows": [] },
            "limits": []
        },
        "unlock": {
            "phase": { "kind": "locked" },
            "unlocksToday": 0,
            "day": "2026-10-12"
        },
        "exhausted": { "day": "2026-10-12", "targets": [] }
    })
}

fn document_at(version: u32) -> Value {
    let mut document = v1_document();
    document["schemaVersion"] = json!(version);
    document
}

fn unlocked_v2(extra: &Value) -> Value {
    let mut document = document_at(2);
    document["unlock"]["phase"] = json!({ "kind": "unlocked", "until": "2026-10-12T10:00:00Z" });
    if let Some(phase) = document["unlock"]["phase"].as_object_mut() {
        phase.extend(extra.as_object().cloned().unwrap_or_default());
    }
    document
}

fn cooling_v1(unlocks_today: u8) -> Value {
    let mut document = v1_document();
    document["unlock"] = json!({
        "phase": { "kind": "coolingDown", "readyAt": "2026-10-13T00:05:00Z" },
        "unlocksToday": unlocks_today,
        "day": "2026-10-12"
    });
    document
}

#[expect(
    clippy::unnecessary_wraps,
    reason = "must match the migration step signature"
)]
fn drop_legacy_note(mut document: Value) -> Result<Value, StateError> {
    if let Some(object) = document.as_object_mut() {
        object.remove("legacyNote");
    }
    Ok(document)
}

#[test]
fn migrations_cover_every_version_below_the_current_one() {
    assert_eq!(MIGRATIONS.len() + 1, SCHEMA_VERSION as usize);
}

#[test]
fn current_documents_load_unchanged() {
    let state = migrate(document_at(SCHEMA_VERSION)).unwrap();
    assert_eq!(state.rules.blocked_sites.len(), 1);
    let again = serde_json::to_value(PersistedState::from(&state)).unwrap();
    assert_eq!(again[VERSION_KEY], SCHEMA_VERSION);
    assert_eq!(migrate(again).unwrap(), state);
}

#[test]
fn fresh_state_serializes_to_a_loadable_document() {
    let fresh = Snapshot::fresh(&now_at(2026, 10, 12, 9, 0));
    let document = serde_json::to_value(PersistedState::from(&fresh)).unwrap();
    assert_eq!(migrate(document).unwrap(), fresh);
}

#[test]
fn a_step_upgrades_an_older_document_and_stamps_the_new_version() {
    let mut document = v1_document();
    document["legacyNote"] = json!("from the previous schema");
    let steps: &[Step] = &[drop_legacy_note];
    let upgraded = upgrade(document.clone(), steps).unwrap();
    assert_eq!(upgraded[VERSION_KEY], 2);
    assert!(upgraded.get("legacyNote").is_none());
    assert!(migrate_with(document.clone(), steps).is_ok());
    assert!(matches!(
        migrate_with(document, &[]),
        Err(StateError::Json(_))
    ));
}

#[test]
fn a_failing_step_aborts_the_migration() {
    fn fail(_: Value) -> Result<Value, StateError> {
        Err(StateError::MissingVersion)
    }
    let steps: &[Step] = &[fail];
    assert!(matches!(
        migrate_with(v1_document(), steps),
        Err(StateError::MissingVersion)
    ));
}

#[test]
fn a_v1_cooldown_gets_the_step_for_the_saved_daily_count() {
    for (used, expected) in [(0, 5), (1, 10), (2, 20), (3, 40), (9, 40)] {
        let state = migrate(cooling_v1(used)).unwrap();
        assert!(
            matches!(
                state.unlock.phase(),
                UnlockPhase::CoolingDown { minutes, .. } if minutes == expected
            ),
            "used {used}"
        );
    }
}

#[test]
fn a_migrated_cooldown_is_saved_at_the_current_version_and_reloads_identically() {
    let state = migrate(cooling_v1(2)).unwrap();
    let saved = serde_json::to_value(PersistedState::from(&state)).unwrap();
    assert_eq!(saved[VERSION_KEY], SCHEMA_VERSION);
    assert_eq!(saved["unlock"]["phase"]["minutes"], 20);
    assert_eq!(migrate(saved).unwrap(), state);
}

#[test]
fn other_v1_phases_are_left_alone() {
    let mut document = v1_document();
    document["unlock"]["phase"] = json!({ "kind": "ready", "readyAt": "2026-10-12T09:55:00Z", "expiresAt": "2026-10-12T10:00:00Z" });
    let upgraded = upgrade(document.clone(), MIGRATIONS).unwrap();
    assert_eq!(upgraded["unlock"]["phase"], document["unlock"]["phase"]);
    assert!(migrate(document).is_ok());
}

#[test]
fn a_running_v2_unlock_gets_the_longest_menu_length() {
    let state = migrate(unlocked_v2(&json!({}))).unwrap();
    assert!(matches!(
        state.unlock.phase(),
        UnlockPhase::Unlocked { minutes: 60, .. }
    ));
}

#[test]
fn an_unlock_that_already_has_its_length_keeps_it() {
    let state = migrate(unlocked_v2(&json!({ "minutes": 15 }))).unwrap();
    assert!(matches!(
        state.unlock.phase(),
        UnlockPhase::Unlocked { minutes: 15, .. }
    ));
}

#[test]
fn a_current_unlock_survives_a_save_and_a_load() {
    let state = migrate(unlocked_v2(&json!({ "minutes": 30 }))).unwrap();
    let saved = serde_json::to_value(PersistedState::from(&state)).unwrap();
    assert_eq!(saved["unlock"]["phase"]["minutes"], 30);
    assert_eq!(migrate(saved).unwrap(), state);
}

#[test]
fn protected_apps_are_dropped_instead_of_discarding_the_whole_state() {
    let app = |id: &str| json!({ "bundleId": id, "name": id });
    let mut document = v1_document();
    document["rules"]["blockedApps"] = json!([app("com.apple.Finder"), app("com.steam")]);
    document["rules"]["limits"] = json!([
        { "target": { "kind": "app", "app": app("com.apple.dock") }, "dailyMinutes": 30 },
        { "target": { "kind": "app", "app": app("com.discord") }, "dailyMinutes": 30 },
        { "target": { "kind": "site", "domain": "x.com" }, "dailyMinutes": 30 }
    ]);

    let state = migrate(document).unwrap();

    let apps: Vec<&str> = state
        .rules
        .blocked_apps
        .iter()
        .map(AppRef::bundle_id)
        .collect();
    assert_eq!(apps, ["com.steam"]);
    assert_eq!(state.rules.limits.len(), 2);
}

#[test]
fn migration_tolerates_documents_missing_the_fields_it_touches() {
    let upgraded = v1_to_v2(json!({ "schemaVersion": 1 })).unwrap();
    assert_eq!(upgraded, json!({ "schemaVersion": 1 }));
    assert!(v1_to_v2(json!("not an object")).is_ok());
    assert!(v1_to_v2(json!({ "unlock": 5, "rules": [] })).is_ok());
}

#[test]
fn rejects_missing_zero_and_future_versions() {
    let mut missing = v1_document();
    missing.as_object_mut().unwrap().remove("schemaVersion");
    assert!(matches!(migrate(missing), Err(StateError::MissingVersion)));
    for version in [
        json!(0),
        json!(SCHEMA_VERSION + 1),
        json!(-1),
        json!("1"),
        json!(u64::MAX),
    ] {
        let mut document = v1_document();
        document["schemaVersion"] = version;
        assert!(migrate(document).is_err());
    }
}

#[test]
fn rejects_rules_that_fail_validation() {
    let mut document = v1_document();
    document["rules"]["limits"] = json!([
        { "target": { "kind": "site", "domain": "x.com" }, "dailyMinutes": 0 }
    ]);
    assert!(matches!(
        migrate(document),
        Err(StateError::InvalidRules(_))
    ));
}

#[test]
fn rejects_unknown_fields() {
    let mut document = v1_document();
    document["extra"] = json!(true);
    assert!(matches!(migrate(document), Err(StateError::Json(_))));
}
