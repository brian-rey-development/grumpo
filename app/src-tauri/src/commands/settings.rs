use tauri::State;

use super::blocking;
use crate::dto::{CommandError, Settings};
use crate::error::AppError;
use crate::launch_at_login::LaunchAtLogin;
use crate::state::AppState;
use crate::storage::retention::RetentionDays;
use crate::storage::Store;
use crate::time::{Clock, SystemClock};

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<Settings, CommandError> {
    let state = state.inner().clone();
    blocking(move || read(&state.store, state.launch.as_ref())).await
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    settings: Settings,
) -> Result<Settings, CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        apply(
            &state.store,
            state.launch.as_ref(),
            settings,
            SystemClock.now_unix(),
            || state.events.usage_rewritten(),
        )
    })
    .await
}

fn read(store: &Store, launch: &dyn LaunchAtLogin) -> Result<Settings, AppError> {
    let launch_at_login = launch.is_enabled().unwrap_or_else(|error| {
        tracing::warn!("could not read the launch at login state");
        tracing::debug!(%error, "launch at login state failure detail");
        false
    });
    Ok(Settings {
        retention_days: store.retention_days()?.get(),
        launch_at_login,
    })
}

/// Applies the retention first, which only touches the local database and prunes at once,
/// and then launch at login, which the system can refuse and which is only asked when the
/// wanted value differs from the current one. Pruning cannot be undone, so doing the step
/// that may fail last means a refusal never leaves history deleted behind an error that
/// claims nothing changed: `on_pruned` runs as soon as rows are gone, before that step.
fn apply(
    store: &Store,
    launch: &dyn LaunchAtLogin,
    wanted: Settings,
    now: i64,
    on_pruned: impl FnOnce(),
) -> Result<Settings, AppError> {
    let retention = RetentionDays::new(wanted.retention_days)
        .map_err(|error| AppError::invalid_input(error.to_string()))?;
    if store.set_retention_days(retention, now)? > 0 {
        on_pruned();
    }
    if launch.is_enabled()? != wanted.launch_at_login {
        launch.set_enabled(wanted.launch_at_login)?;
    }
    read(store, launch)
}

#[cfg(test)]
mod tests;
