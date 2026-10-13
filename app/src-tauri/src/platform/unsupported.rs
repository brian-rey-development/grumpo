use std::sync::Arc;
use std::time::Duration;

use focus_core::{AppRef, Domain};

use super::{
    ActivityProbe, AppControl, AppLocator, Browser, Dialogs, ForegroundApp, HourCycle,
    HourCycleSource, IconProvider, InstallOutcome, Permission, PermissionProbe, PermissionState,
    Platform, PlatformError, ServiceInstallError, ServiceInstaller, TabReading,
};

#[derive(Debug)]
struct Unsupported;

impl ActivityProbe for Unsupported {
    fn foreground_app(&self) -> Result<Option<ForegroundApp>, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn idle_time(&self) -> Result<Duration, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn screen_locked(&self) -> Result<bool, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn media_playing(&self, _frontmost_pid: i32) -> Result<bool, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn active_tab(&self, _browser: Browser, _pid: i32) -> Result<TabReading, PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

impl AppControl for Unsupported {
    fn running_apps(&self) -> Result<Vec<AppRef>, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn is_running(&self, _bundle_id: &str) -> Result<bool, PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn quit(&self, _app: &AppRef) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }

    fn close_active_tab_if(
        &self,
        _browser: Browser,
        _domain: &Domain,
    ) -> Result<bool, PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

impl IconProvider for Unsupported {
    fn app_icon_png(
        &self,
        _bundle_id: &str,
        _size_px: u32,
    ) -> Result<Option<Vec<u8>>, PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

impl AppLocator for Unsupported {
    fn is_installed(&self, _bundle_id: &str) -> bool {
        false
    }
}

impl PermissionProbe for Unsupported {
    fn state(&self, _permission: Permission) -> PermissionState {
        PermissionState::Unsupported
    }

    fn request(&self, _permission: Permission) -> PermissionState {
        PermissionState::Unsupported
    }

    fn open_settings(&self, _permission: Permission) -> Result<(), PlatformError> {
        Err(PlatformError::Unsupported)
    }
}

impl HourCycleSource for Unsupported {
    fn hour_cycle(&self) -> HourCycle {
        HourCycle::TwentyFour
    }
}

impl Dialogs for Unsupported {
    fn alert(&self, title: &str, message: &str) {
        tracing::error!(title, message, "no dialog on this system");
    }
}

impl ServiceInstaller for Unsupported {
    fn install(&self) -> Result<InstallOutcome, ServiceInstallError> {
        Err(ServiceInstallError::Unavailable)
    }
}

pub(super) fn platform() -> Platform {
    let unsupported = Arc::new(Unsupported);
    Platform {
        probe: unsupported.clone(),
        control: unsupported.clone(),
        icons: unsupported.clone(),
        locator: unsupported.clone(),
        permissions: unsupported.clone(),
        hour_cycle: unsupported.clone(),
        dialogs: unsupported.clone(),
        installer: unsupported,
    }
}
