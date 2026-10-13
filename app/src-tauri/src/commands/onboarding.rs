use tauri::State;

use super::blocking;
use crate::dto::{CommandError, OnboardingState};
use crate::error::AppError;
use crate::state::AppState;
use crate::storage::Store;

#[tauri::command]
pub async fn get_onboarding_state(
    state: State<'_, AppState>,
) -> Result<OnboardingState, CommandError> {
    let state = state.inner().clone();
    blocking(move || read(&state.store)).await
}

#[tauri::command]
pub async fn complete_onboarding(
    state: State<'_, AppState>,
) -> Result<OnboardingState, CommandError> {
    let state = state.inner().clone();
    blocking(move || write(&state.store, true)).await
}

/// Backs "Show welcome again" in Settings, which reopens the flow without touching anything
/// else the person has set up.
#[tauri::command]
pub async fn reset_onboarding(state: State<'_, AppState>) -> Result<OnboardingState, CommandError> {
    let state = state.inner().clone();
    blocking(move || write(&state.store, false)).await
}

fn read(store: &Store) -> Result<OnboardingState, AppError> {
    Ok(OnboardingState {
        completed: store.onboarding_completed()?,
    })
}

fn write(store: &Store, completed: bool) -> Result<OnboardingState, AppError> {
    store.set_onboarding_completed(completed)?;
    read(store)
}

#[cfg(test)]
mod tests;
