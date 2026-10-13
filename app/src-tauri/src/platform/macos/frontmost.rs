//! Frontmost application via `NSWorkspace`.
//!
//! None of these bindings require a `MainThreadMarker`: `NSWorkspace` and `NSRunningApplication`
//! are plain thread-safe classes, but their time-varying state is refreshed by the main run
//! loop, which Tauri keeps spinning, so reading from the sampler thread sees current values.

use focus_core::AppRef;
use objc2::rc::autoreleasepool;
use objc2_app_kit::{NSRunningApplication, NSWorkspace};

use crate::platform::ForegroundApp;

pub(super) fn foreground_app() -> Option<ForegroundApp> {
    autoreleasepool(|_| {
        let app = NSWorkspace::sharedWorkspace().frontmostApplication()?;
        Some(ForegroundApp {
            app: app_ref_of(&app)?,
            pid: app.processIdentifier(),
        })
    })
}

/// Skips apps without a usable bundle id, which the rules cannot refer to.
pub(super) fn app_ref_of(app: &NSRunningApplication) -> Option<AppRef> {
    let bundle_id = app.bundleIdentifier()?.to_string();
    let name = app
        .localizedName()
        .map(|n| n.to_string())
        .unwrap_or_default();
    match AppRef::new(bundle_id, name) {
        Ok(app_ref) => Some(app_ref),
        Err(error) => {
            tracing::debug!(%error, "ignoring app with an unusable identity");
            None
        }
    }
}
