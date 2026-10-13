use focus_core::ipc::Request;
use focus_core::Rules;
use serde::Deserialize;
use serde_json::Value;
use tauri::State;

use super::blocking;
use crate::dto::{CommandError, FocusStatus};
use crate::error::{diagnose_unreadable_rules, AppError};
use crate::state::AppState;

/// Takes the rules as raw JSON and reads them here instead of in Tauri's argument handling,
/// because a rule the core rejects while reading it (a domain such as `co.uk`) has to come
/// back as a `rulesInvalid` error that names the fault, not as an argument failure the UI
/// cannot tell apart from a bug.
#[tauri::command]
pub async fn save_rules(
    state: State<'_, AppState>,
    rules: Value,
) -> Result<FocusStatus, CommandError> {
    let status = std::sync::Arc::clone(&state.status);
    blocking(move || {
        let validated = parse_rules(&rules)?.validate()?;
        let request = Request::UpdateRules {
            rules: validated.into_inner(),
        };
        Ok(status.send(&request)?)
    })
    .await
}

fn parse_rules(raw: &Value) -> Result<Rules, AppError> {
    Rules::deserialize(raw).map_err(|error| AppError::RulesInvalid {
        fault: diagnose_unreadable_rules(raw),
        detail: error.to_string(),
    })
}

#[cfg(test)]
mod tests;
