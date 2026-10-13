use tauri::State;

use super::blocking;
use crate::dto::{CommandError, FocusStatus};
use crate::state::AppState;

/// Always asks the daemon. The call is a cheap local socket exchange, and a cached answer
/// could show a phase that ended seconds ago, such as a cooldown that has finished.
#[tauri::command]
pub async fn get_focus_status(state: State<'_, AppState>) -> Result<FocusStatus, CommandError> {
    let status = std::sync::Arc::clone(&state.status);
    blocking(move || Ok(status.refresh())).await
}
