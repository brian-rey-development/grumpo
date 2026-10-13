use focus_core::ipc::Request;
use tauri::State;

use super::blocking;
use crate::dto::{CommandError, FocusStatus};
use crate::state::AppState;

#[tauri::command]
pub async fn request_unlock(state: State<'_, AppState>) -> Result<FocusStatus, CommandError> {
    send(&state, Request::RequestUnlock).await
}

#[tauri::command]
pub async fn confirm_unlock(
    state: State<'_, AppState>,
    phrase: String,
    minutes: u16,
) -> Result<FocusStatus, CommandError> {
    send(&state, Request::ConfirmUnlock { phrase, minutes }).await
}

#[tauri::command]
pub async fn cancel_unlock(state: State<'_, AppState>) -> Result<FocusStatus, CommandError> {
    send(&state, Request::CancelUnlock).await
}

#[tauri::command]
pub async fn relock(state: State<'_, AppState>) -> Result<FocusStatus, CommandError> {
    send(&state, Request::Relock).await
}

async fn send(state: &State<'_, AppState>, request: Request) -> Result<FocusStatus, CommandError> {
    let status = std::sync::Arc::clone(&state.status);
    blocking(move || Ok(status.send(&request)?)).await
}
