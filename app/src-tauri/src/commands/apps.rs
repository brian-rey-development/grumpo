mod icon;
mod known;

use focus_core::AppRef;
use tauri::State;

use super::blocking;
use crate::dto::{CommandError, KnownApp};
use crate::platform::Platform;
use crate::state::AppState;
use crate::time::units::SECONDS_PER_DAY;
use crate::time::{Clock, SystemClock};

const HISTORY_DAYS: i64 = 30;

#[tauri::command]
pub async fn list_known_apps(state: State<'_, AppState>) -> Result<Vec<KnownApp>, CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        let since = SystemClock.now_unix() - HISTORY_DAYS * SECONDS_PER_DAY;
        let history = state.store.apps_since(since)?;
        let apps = known::merge(running_apps(&state.platform), history);
        Ok(known::without_unblockable(apps, &state.own_bundle_id))
    })
    .await
}

#[tauri::command]
pub async fn get_app_icon(
    state: State<'_, AppState>,
    bundle_id: String,
) -> Result<Option<String>, CommandError> {
    let state = state.inner().clone();
    blocking(move || icon::app_icon(&state, &bundle_id)).await
}

/// A failed read of the running apps must not hide the apps found in history, so it is
/// logged and treated as none running.
fn running_apps(platform: &Platform) -> Vec<AppRef> {
    platform.control.running_apps().unwrap_or_else(|error| {
        tracing::warn!(kind = %error.kind(), "could not list running apps");
        tracing::debug!(%error, "running apps listing failure detail");
        Vec::new()
    })
}
