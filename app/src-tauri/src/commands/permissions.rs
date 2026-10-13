mod convert;

use tauri::State;

use super::blocking;
use crate::dto::{BrowserAccess, CommandError, PermissionKind, Permissions};
use crate::error::AppError;
use crate::platform::{AppLocator, Browser, Permission, PermissionProbe};
use crate::state::AppState;
use crate::tracking::health::ObserverHealth;
use convert::{browser_id, merge_with_health, permission_status};

#[tauri::command]
pub async fn get_permissions(state: State<'_, AppState>) -> Result<Permissions, CommandError> {
    let state = state.inner().clone();
    blocking(move || Ok(read(&state))).await
}

#[tauri::command]
pub async fn request_permission(
    state: State<'_, AppState>,
    permission: PermissionKind,
) -> Result<Permissions, CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        state.platform.permissions.request(permission.into());
        Ok(read(&state))
    })
    .await
}

#[tauri::command]
pub async fn open_permission_settings(
    state: State<'_, AppState>,
    permission: PermissionKind,
) -> Result<(), CommandError> {
    let state = state.inner().clone();
    blocking(move || {
        state
            .platform
            .permissions
            .open_settings(permission.into())
            .map_err(AppError::from)
    })
    .await
}

fn read(state: &AppState) -> Permissions {
    let probe = state.platform.permissions.as_ref();
    let health = state.health.snapshot();
    Permissions {
        accessibility: permission_status(probe.state(Permission::Accessibility)),
        browsers: Browser::ALL
            .into_iter()
            .map(|browser| browser_access(state.platform.locator.as_ref(), probe, &health, browser))
            .collect(),
    }
}

fn browser_access(
    locator: &dyn AppLocator,
    probe: &dyn PermissionProbe,
    health: &ObserverHealth,
    browser: Browser,
) -> BrowserAccess {
    let probed = probe.state(Permission::Automation(browser));
    BrowserAccess {
        browser: browser_id(browser),
        installed: locator.is_installed(browser.bundle_id()),
        permission: merge_with_health(probed, health.permission_denied().contains(&browser)),
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
