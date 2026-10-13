use super::*;
use crate::platform::{PermissionState, PlatformError};
use crate::tracking::health::ObserverHealth;
use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct FakeLocator {
    pub installed: Vec<&'static str>,
    pub asked: Mutex<Vec<String>>,
}

impl AppLocator for FakeLocator {
    fn is_installed(&self, bundle_id: &str) -> bool {
        self.asked.lock().unwrap().push(bundle_id.to_owned());
        self.installed.contains(&bundle_id)
    }
}

pub struct FakeProbe(PermissionState);

impl PermissionProbe for FakeProbe {
    fn state(&self, _permission: Permission) -> PermissionState {
        self.0
    }

    fn request(&self, _permission: Permission) -> PermissionState {
        self.0
    }

    fn open_settings(&self, _permission: Permission) -> Result<(), PlatformError> {
        Ok(())
    }
}

pub fn access(
    locator: &FakeLocator,
    probed: PermissionState,
    health: &ObserverHealth,
    browser: Browser,
) -> BrowserAccess {
    browser_access(locator, &FakeProbe(probed), health, browser)
}
