use std::sync::atomic::{AtomicBool, Ordering};

use tauri::State;

use super::blocking;
use crate::dto::{CommandError, ServiceInstallOutcome};
use crate::error::AppError;
use crate::platform::{InstallOutcome, ServiceInstallError, ServiceInstaller};
use crate::state::AppState;
use crate::status::StatusHub;

/// Installs or updates the Grumpo service, which makes macOS ask for the Mac password. The
/// person dismissing that prompt is an outcome, not an error. A second call while one is
/// running fails as busy, because two installers taking each other's backups apart would
/// leave nothing to roll back to.
#[tauri::command]
pub async fn install_service(
    state: State<'_, AppState>,
) -> Result<ServiceInstallOutcome, CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        install(
            state.platform.installer.as_ref(),
            &state.status,
            &state.installing,
        )
    })
    .await
}

/// Asks the daemon right after a successful install, so the UI sees it connected without
/// waiting for the next poll.
fn install(
    installer: &dyn ServiceInstaller,
    status: &StatusHub,
    installing: &AtomicBool,
) -> Result<ServiceInstallOutcome, AppError> {
    let _claim = Claim::take(installing).ok_or(ServiceInstallError::Busy)?;
    let outcome = installer.install()?;
    if outcome == InstallOutcome::Installed {
        status.refresh();
    }
    Ok(outcome_dto(outcome))
}

/// Holds the in-flight flag until dropped, also when the installer panics.
struct Claim<'a>(&'a AtomicBool);

impl<'a> Claim<'a> {
    fn take(flag: &'a AtomicBool) -> Option<Self> {
        flag.compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
            .is_ok()
            .then(|| Self(flag))
    }
}

impl Drop for Claim<'_> {
    fn drop(&mut self) {
        self.0.store(false, Ordering::Release);
    }
}

fn outcome_dto(outcome: InstallOutcome) -> ServiceInstallOutcome {
    match outcome {
        InstallOutcome::Installed => ServiceInstallOutcome::Installed,
        InstallOutcome::Cancelled => ServiceInstallOutcome::Cancelled,
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
