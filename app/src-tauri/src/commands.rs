//! Tauri commands. Each one validates its input, calls a service and maps the result, and
//! blocking work runs off the UI thread.

pub mod apps;
pub mod block_screen;
pub mod clock;
pub mod data;
pub mod onboarding;
pub mod permissions;
pub mod reports;
pub mod rules;
pub mod service;
pub mod settings;
pub mod status;
pub mod unlock;

use crate::dto::CommandError;
use crate::error::AppError;

/// Runs blocking work (SQLite, the daemon socket, `AppKit`) on the blocking pool and maps the
/// failure once.
pub(crate) async fn blocking<T, F>(work: F) -> Result<T, CommandError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    match tauri::async_runtime::spawn_blocking(work).await {
        Ok(result) => result.map_err(CommandError::from),
        Err(error) => Err(AppError::Internal(error.to_string()).into()),
    }
}

#[cfg(test)]
mod tests;
