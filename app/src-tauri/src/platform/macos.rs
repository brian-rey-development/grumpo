//! macOS adapters for the platform ports.

mod accessibility;
mod app_nap;
mod apple_events;
mod apple_script;
mod ax;
mod browsers;
mod consent;
mod control;
mod dialogs;
mod firefox;
mod firefox_search;
mod frontmost;
mod hour_cycle;
mod icons;
mod idle;
mod locator;
mod media;
mod os_status;
mod permissions;
mod power_assertions;
mod process;
mod process_tree;
mod screen_lock;
mod service_installer;
#[cfg(test)]
mod smoke;
mod tab_scripts;

use std::sync::Arc;
use std::time::Duration;

use super::{
    ActivityProbe, Browser, ForegroundApp, Platform, PlatformError, TabReading, TrackingActivity,
};

#[derive(Debug, Default)]
struct MacProbe {
    firefox: firefox::FirefoxReader,
}

impl ActivityProbe for MacProbe {
    fn foreground_app(&self) -> Result<Option<ForegroundApp>, PlatformError> {
        Ok(frontmost::foreground_app())
    }

    fn idle_time(&self) -> Result<Duration, PlatformError> {
        idle::idle_time()
    }

    fn screen_locked(&self) -> Result<bool, PlatformError> {
        screen_lock::screen_locked()
    }

    fn media_playing(&self, frontmost_pid: i32) -> Result<bool, PlatformError> {
        media::media_playing(frontmost_pid)
    }

    fn active_tab(&self, browser: Browser, pid: i32) -> Result<TabReading, PlatformError> {
        match browser {
            Browser::Firefox => self.firefox.active_tab(pid),
            _ => browsers::active_tab(browser),
        }
    }
}

fn failed(operation: &'static str, detail: impl std::fmt::Display) -> PlatformError {
    PlatformError::Failed {
        operation,
        detail: detail.to_string(),
    }
}

pub(super) fn begin_tracking_activity() -> TrackingActivity {
    TrackingActivity::holding(app_nap::begin())
}

pub(super) fn platform() -> Platform {
    Platform {
        probe: Arc::new(MacProbe::default()),
        control: Arc::new(control::MacControl),
        icons: Arc::new(icons::MacIcons),
        locator: Arc::new(locator::MacLocator),
        permissions: Arc::new(permissions::MacPermissions),
        hour_cycle: Arc::new(hour_cycle::MacHourCycle),
        dialogs: Arc::new(dialogs::MacDialogs),
        installer: Arc::new(service_installer::MacServiceInstaller),
    }
}
