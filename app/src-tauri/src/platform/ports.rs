use std::time::Duration;

use focus_core::{AppRef, Domain};

use super::{
    Browser, InstallOutcome, Permission, PermissionState, PlatformError, ServiceInstallError,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ForegroundApp {
    pub app: AppRef,
    pub pid: i32,
}

/// What the front window of a browser shows. `private` is true for an incognito window,
/// which the tracker enforces on but never stores.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TabReading {
    pub domain: Option<Domain>,
    pub private: bool,
}

/// Raw observations; the tracker turns them into `Activity`.
pub trait ActivityProbe: Send + Sync {
    fn foreground_app(&self) -> Result<Option<ForegroundApp>, PlatformError>;
    fn idle_time(&self) -> Result<Duration, PlatformError>;
    fn screen_locked(&self) -> Result<bool, PlatformError>;
    /// True when the frontmost app, or a helper process working for it, holds a
    /// display-sleep assertion, which video players and calls do while playing. Media in
    /// other apps does not count as the user's attention.
    fn media_playing(&self, frontmost_pid: i32) -> Result<bool, PlatformError>;
    /// The host and privacy of the front window's active tab. The address is reduced to a
    /// [`Domain`] inside the platform layer, so no URL ever leaves it. `pid` is the
    /// frontmost process, which Firefox needs because it is read through Accessibility.
    fn active_tab(&self, browser: Browser, pid: i32) -> Result<TabReading, PlatformError>;
}

pub trait AppControl: Send + Sync {
    fn running_apps(&self) -> Result<Vec<AppRef>, PlatformError>;
    /// Whether any process with `bundle_id` runs, menu bar apps and background agents included,
    /// which `running_apps` leaves out.
    fn is_running(&self, bundle_id: &str) -> Result<bool, PlatformError>;
    fn quit(&self, app: &AppRef) -> Result<(), PlatformError>;
    /// Closes the active tab only if it still shows `domain`, so a tab switch
    /// between the check and the action never closes the wrong tab.
    fn close_active_tab_if(&self, browser: Browser, domain: &Domain)
        -> Result<bool, PlatformError>;
}

pub trait IconProvider: Send + Sync {
    fn app_icon_png(&self, bundle_id: &str, size_px: u32)
        -> Result<Option<Vec<u8>>, PlatformError>;
}

/// Answers whether an app is installed, which is a lookup and must never render an icon.
pub trait AppLocator: Send + Sync {
    fn is_installed(&self, bundle_id: &str) -> bool;
}

pub trait PermissionProbe: Send + Sync {
    /// Reads the current state without ever prompting.
    fn state(&self, permission: Permission) -> PermissionState;
    /// Asks the system to prompt the user, then reports the resulting state. Blocks until
    /// the user answers or a generous timeout passes, so call it off the UI thread.
    fn request(&self, permission: Permission) -> PermissionState;
    fn open_settings(&self, permission: Permission) -> Result<(), PlatformError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HourCycle {
    Twelve,
    TwentyFour,
}

/// The system's 12 or 24 hour clock setting, asked each time so a change applies at once.
pub trait HourCycleSource: Send + Sync {
    fn hour_cycle(&self) -> HourCycle;
}

/// A native alert for the failures that leave the app unable to start, shown before the
/// window exists.
pub trait Dialogs: Send + Sync {
    /// Shows `message` under `title` and returns once the person dismisses it. Call it from
    /// the main thread.
    fn alert(&self, title: &str, message: &str);
}

/// Installs or updates the Grumpo service, which runs as root and so needs the person's
/// administrator password.
pub trait ServiceInstaller: Send + Sync {
    /// Blocks until the person answers the system prompt and the installer finishes, so call
    /// it off the UI thread.
    fn install(&self) -> Result<InstallOutcome, ServiceInstallError>;
}
