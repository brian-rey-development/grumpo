use std::fs;
use std::io::ErrorKind;

use chrono::{Local, NaiveDateTime};
use tauri::State;

use super::blocking;
use crate::dto::{CommandError, FocusStatus};
use crate::error::AppError;
use crate::state::AppState;
use crate::storage::StorageError;
use crate::time::{today, Clock, SystemClock};

const EXPORT_NAME_FORMAT: &str = "Grumpo export %Y-%m-%d %H%M%S.json";

/// Writes every session and setting to the Downloads folder and returns the file's path.
#[tauri::command]
pub async fn export_data(state: State<'_, AppState>) -> Result<String, CommandError> {
    let state = state.inner().clone();
    blocking(move || export(&state)).await
}

fn export(state: &AppState) -> Result<String, AppError> {
    let folder = &state.paths.exports;
    fs::create_dir_all(folder)
        .map_err(StorageError::io("create export folder"))
        .map_err(export_error)?;
    let path = folder.join(export_file_name(Local::now().naive_local()));
    state
        .store
        .export_json(&path, SystemClock.now_unix())
        .map_err(export_error)?;
    Ok(path.to_string_lossy().into_owned())
}

/// A folder that refuses the write is usually the Files and Folders permission, which the
/// person can grant, so it gets its own message instead of the generic storage failure.
fn export_error(error: StorageError) -> AppError {
    match &error {
        StorageError::Io { source, .. } if source.kind() == ErrorKind::PermissionDenied => {
            AppError::ExportDenied
        }
        _ => error.into(),
    }
}

/// Erases usage history. While a daily limit exists, today's usage stays, because that is
/// what the limit counts and deleting it would hand back the time already spent.
#[tauri::command]
pub async fn delete_all_data(state: State<'_, AppState>) -> Result<(), CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        let keep_today = keeps_today(&state.status.current());
        state.store.delete_all(today().start, keep_today)?;
        state.events.usage_rewritten();
        Ok(())
    })
    .await
}

pub fn export_file_name(local: NaiveDateTime) -> String {
    local.format(EXPORT_NAME_FORMAT).to_string()
}

/// Without the daemon the limits are unknown, so the safe answer is to keep today.
fn keeps_today(status: &FocusStatus) -> bool {
    status
        .daemon()
        .is_none_or(|daemon| !daemon.rules.limits.is_empty())
}

#[cfg(test)]
mod tests;
