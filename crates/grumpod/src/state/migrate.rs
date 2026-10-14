use focus_core::{cooldown_minutes_after, is_protected, UNLOCK_DURATIONS_MINUTES};
use serde_json::{json, Value};

use super::persisted::PersistedState;
use super::{Snapshot, StateError};

/// Rewrites a document of one version into the next. `MIGRATIONS[0]` upgrades v1 to v2,
/// so adding v4 means appending one function here, and the current version follows.
type Step = fn(Value) -> Result<Value, StateError>;

const MIGRATIONS: &[Step] = &[v1_to_v2, v2_to_v3];

const VERSION_KEY: &str = "schemaVersion";

/// The version this daemon writes, which is the one after the last migration.
#[expect(
    clippy::cast_possible_truncation,
    reason = "a handful of migrations, which the assertion on the line above bounds"
)]
pub const SCHEMA_VERSION: u32 = {
    assert!(MIGRATIONS.len() < u32::MAX as usize);
    MIGRATIONS.len() as u32 + 1
};

/// Gives a pending cooldown the length it was requested with and drops protected apps, which
/// v2 rules refuse and would otherwise make the whole saved state unusable.
///
/// v1 never stored the length. The step for the saved `unlocksToday` is the length the request
/// got, because that count only changes once the unlock is confirmed or the day rolls over,
/// and a rollover in between is the case this field exists to survive.
#[expect(
    clippy::unnecessary_wraps,
    reason = "must match the migration step signature"
)]
fn v1_to_v2(mut document: Value) -> Result<Value, StateError> {
    fill_cooldown_minutes(&mut document);
    drop_protected_apps(&mut document);
    Ok(document)
}

fn fill_cooldown_minutes(document: &mut Value) {
    let used = document
        .pointer("/unlock/unlocksToday")
        .and_then(Value::as_u64)
        .and_then(|count| u8::try_from(count).ok())
        .unwrap_or(u8::MAX);
    let cooling = document
        .pointer_mut("/unlock/phase")
        .and_then(Value::as_object_mut)
        .filter(|phase| phase.get("kind") == Some(&json!("coolingDown")));
    if let Some(phase) = cooling {
        phase
            .entry("minutes")
            .or_insert_with(|| json!(cooldown_minutes_after(used)));
    }
}

/// Gives a running unlock the length it was granted with, which v2 never stored. The longest
/// menu entry is the most it can have been, so an unlock in force across the upgrade keeps its
/// time and only a clock set back past that is caught.
#[expect(
    clippy::unnecessary_wraps,
    reason = "must match the migration step signature"
)]
fn v2_to_v3(mut document: Value) -> Result<Value, StateError> {
    let longest = UNLOCK_DURATIONS_MINUTES
        .iter()
        .max()
        .copied()
        .unwrap_or_default();
    let unlocked = document
        .pointer_mut("/unlock/phase")
        .and_then(Value::as_object_mut)
        .filter(|phase| phase.get("kind") == Some(&json!("unlocked")));
    if let Some(phase) = unlocked {
        phase.entry("minutes").or_insert_with(|| json!(longest));
    }
    Ok(document)
}

fn drop_protected_apps(document: &mut Value) {
    let blocked = document.pointer_mut("/rules/blockedApps");
    if let Some(apps) = blocked.and_then(Value::as_array_mut) {
        apps.retain(|app| !is_protected_app(app));
    }
    let limits = document.pointer_mut("/rules/limits");
    if let Some(limits) = limits.and_then(Value::as_array_mut) {
        limits.retain(|limit| !is_protected_app(&limit["target"]["app"]));
    }
}

fn is_protected_app(app: &Value) -> bool {
    app["bundleId"].as_str().is_some_and(is_protected)
}

/// Upgrades a stored document to the current schema and validates its rules, which is the
/// only place a loaded state is checked.
pub fn migrate(document: Value) -> Result<Snapshot, StateError> {
    migrate_with(document, MIGRATIONS)
}

fn migrate_with(document: Value, steps: &[Step]) -> Result<Snapshot, StateError> {
    let state: PersistedState = serde_json::from_value(upgrade(document, steps)?)?;
    Snapshot::try_from(state)
}

fn upgrade(mut document: Value, steps: &[Step]) -> Result<Value, StateError> {
    let latest = u32::try_from(steps.len()).map_or(u32::MAX, |count| count.saturating_add(1));
    let version = read_version(&document, latest)?;
    for (step, from) in steps.iter().zip(1_u32..) {
        if from >= version {
            document = step(document)?;
            write_version(&mut document, from + 1);
        }
    }
    Ok(document)
}

fn read_version(document: &Value, latest: u32) -> Result<u32, StateError> {
    let found = document
        .get(VERSION_KEY)
        .and_then(Value::as_u64)
        .ok_or(StateError::MissingVersion)?;
    match u32::try_from(found) {
        Ok(version) if (1..=latest).contains(&version) => Ok(version),
        _ => Err(StateError::UnsupportedVersion {
            found,
            supported: latest,
        }),
    }
}

fn write_version(document: &mut Value, version: u32) {
    if let Some(object) = document.as_object_mut() {
        object.insert(VERSION_KEY.to_owned(), Value::from(version));
    }
}

#[cfg(test)]
mod tests;
