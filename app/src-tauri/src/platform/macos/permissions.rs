//! Permission state and deep links into System Settings.

use std::process::Command;
use std::time::Duration;

use objc2_foundation::NSBundle;

use super::os_status::{
    OsStatus, ERR_NOT_PERMITTED, ERR_WOULD_REQUIRE_CONSENT, NO_ERR, PROC_NOT_FOUND,
};
use super::process::{run_bounded, RunError};
use super::{accessibility, apple_events, consent};
use crate::platform::{Browser, Permission, PermissionProbe, PermissionState, PlatformError};

const OPEN: &str = "/usr/bin/open";
const OPEN_TIMEOUT: Duration = Duration::from_secs(3);
const TCCUTIL: &str = "/usr/bin/tccutil";
const TCCUTIL_TIMEOUT: Duration = Duration::from_secs(3);
const ACCESSIBILITY_SERVICE: &str = "Accessibility";
const ACCESSIBILITY_PANE: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";
const AUTOMATION_PANE: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Automation";

#[derive(Debug)]
pub(super) struct MacPermissions;

impl PermissionProbe for MacPermissions {
    fn state(&self, permission: Permission) -> PermissionState {
        match permission {
            Permission::Accessibility => accessibility_state(),
            Permission::Automation(Browser::Firefox) => PermissionState::Unsupported,
            Permission::Automation(browser) => {
                map_automation_status(apple_events::automation_status(browser.bundle_id()))
            }
        }
    }

    fn request(&self, permission: Permission) -> PermissionState {
        match permission {
            Permission::Accessibility => request_accessibility(),
            Permission::Automation(Browser::Firefox) => PermissionState::Unsupported,
            Permission::Automation(browser) => {
                let status = apple_events::request_automation(browser.bundle_id());
                consent::forget(browser);
                map_automation_status(status)
            }
        }
    }

    fn open_settings(&self, permission: Permission) -> Result<(), PlatformError> {
        let output = run_bounded(Command::new(OPEN).arg(pane_url(permission)), OPEN_TIMEOUT)
            .map_err(|error| match error {
                RunError::Timeout => PlatformError::Timeout {
                    operation: "open settings",
                },
                RunError::Spawn(error) | RunError::Wait(error) => {
                    super::failed("open settings", error)
                }
            })?;
        if output.status.success() {
            return Ok(());
        }
        Err(super::failed(
            "open settings",
            format!("exit {}", output.status),
        ))
    }
}

fn request_accessibility() -> PermissionState {
    if !accessibility::is_trusted() {
        forget_stale_accessibility_grant();
    }
    accessibility_state_after(accessibility::request_trust())
}

/// A differently signed build keeps the old switch on in System Settings while macOS refuses
/// the new one, so the entry is removed to let the next prompt add a matching one.
fn forget_stale_accessibility_grant() {
    let Some(bundle_id) = NSBundle::mainBundle().bundleIdentifier() else {
        return;
    };
    let mut command = Command::new(TCCUTIL);
    command.args(accessibility_reset_args(&bundle_id.to_string()));
    match run_bounded(&mut command, TCCUTIL_TIMEOUT) {
        Ok(output) if output.status.success() => {}
        Ok(output) => tracing::warn!(status = %output.status, "tccutil reset failed"),
        Err(error) => tracing::warn!(?error, "tccutil reset did not run"),
    }
}

fn accessibility_reset_args(bundle_id: &str) -> [&str; 3] {
    ["reset", ACCESSIBILITY_SERVICE, bundle_id]
}

fn accessibility_state() -> PermissionState {
    accessibility_state_after(accessibility::is_trusted())
}

fn accessibility_state_after(trusted: bool) -> PermissionState {
    if trusted {
        PermissionState::Granted
    } else {
        PermissionState::Denied
    }
}

fn map_automation_status(status: OsStatus) -> PermissionState {
    match status {
        NO_ERR => PermissionState::Granted,
        ERR_NOT_PERMITTED => PermissionState::Denied,
        ERR_WOULD_REQUIRE_CONSENT => PermissionState::NotDetermined,
        PROC_NOT_FOUND => PermissionState::NotRunning,
        other => {
            tracing::debug!(status = other, "unexpected automation permission status");
            PermissionState::NotDetermined
        }
    }
}

fn pane_url(permission: Permission) -> &'static str {
    match permission {
        Permission::Accessibility => ACCESSIBILITY_PANE,
        Permission::Automation(_) => AUTOMATION_PANE,
    }
}

#[cfg(test)]
mod tests;
