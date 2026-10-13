//! Operating-system ports. Everything OS-specific sits behind these traits so the
//! tracker and enforcer stay portable and testable with fakes.

use std::sync::Arc;

mod browser;
mod error;
#[cfg(target_os = "macos")]
mod macos;
mod permission;
mod ports;
mod service_install;
#[cfg_attr(
    target_os = "macos",
    expect(
        dead_code,
        reason = "the stub adapters back the builds for other systems"
    )
)]
mod unsupported;

pub use browser::Browser;
pub use error::{PlatformError, PlatformErrorKind};
pub use permission::{Permission, PermissionState};
pub use ports::{
    ActivityProbe, AppControl, AppLocator, Dialogs, ForegroundApp, HourCycle, HourCycleSource,
    IconProvider, PermissionProbe, ServiceInstaller, TabReading,
};
pub use service_install::{InstallOutcome, ServiceInstallError};

#[derive(Clone)]
pub struct Platform {
    pub probe: Arc<dyn ActivityProbe>,
    pub control: Arc<dyn AppControl>,
    pub icons: Arc<dyn IconProvider>,
    pub locator: Arc<dyn AppLocator>,
    pub permissions: Arc<dyn PermissionProbe>,
    pub hour_cycle: Arc<dyn HourCycleSource>,
    pub dialogs: Arc<dyn Dialogs>,
    pub installer: Arc<dyn ServiceInstaller>,
}

impl std::fmt::Debug for Platform {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Platform").finish_non_exhaustive()
    }
}

/// Proof that the system has been asked not to throttle this process for background work.
/// The request ends when the value is dropped, so hold it for the app's lifetime.
#[derive(Debug)]
pub struct TrackingActivity {
    _held: Option<Box<dyn std::any::Any + Send + Sync>>,
}

impl TrackingActivity {
    #[cfg_attr(
        not(target_os = "macos"),
        expect(dead_code, reason = "only the macOS adapter holds an activity")
    )]
    fn holding(token: impl std::any::Any + Send + Sync) -> Self {
        Self {
            _held: Some(Box::new(token)),
        }
    }
}

/// Keeps the sampler and the status poller on schedule while the window is hidden. A no-op
/// where the system has no such throttling.
#[cfg(target_os = "macos")]
pub fn begin_tracking_activity() -> TrackingActivity {
    macos::begin_tracking_activity()
}

#[cfg(not(target_os = "macos"))]
pub fn begin_tracking_activity() -> TrackingActivity {
    TrackingActivity { _held: None }
}

#[cfg(target_os = "macos")]
pub fn current() -> Platform {
    macos::platform()
}

#[cfg(not(target_os = "macos"))]
pub fn current() -> Platform {
    unsupported::platform()
}
