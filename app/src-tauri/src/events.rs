//! Backend events for the webview, plus the tray refreshes that ride along with them.

use tauri::{AppHandle, Emitter};

use crate::dto::FocusStatus;
use crate::enforcement::UsageEvents;
use crate::tray::TrayHandle;

pub const USAGE_CHANGED: &str = "usage-changed";
pub const STATUS_CHANGED: &str = "status-changed";
pub const BLOCK_SCREEN: &str = "block-screen";
/// Sent to the block screen window just before it hides, so the page can drop what it shows.
pub const BLOCK_SCREEN_HIDDEN: &str = "block-screen-hidden";

pub struct AppEvents {
    app: AppHandle,
    tray: TrayHandle,
}

impl std::fmt::Debug for AppEvents {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppEvents").finish_non_exhaustive()
    }
}

impl AppEvents {
    pub fn new(app: AppHandle, tray: TrayHandle) -> Self {
        Self { app, tray }
    }

    pub fn status_changed(&self, status: &FocusStatus) {
        self.tray.on_status(status);
        if let Err(error) = self.app.emit(STATUS_CHANGED, status) {
            tracing::warn!(%error, "could not emit the status event");
        }
    }

    /// For history rewritten in one go, such as a delete or a shorter retention, where the
    /// menu bar total must follow at once instead of waiting for the coalescing interval. Blocks on the
    /// database, so call it from a blocking task.
    pub fn usage_rewritten(&self) {
        self.emit_usage_changed();
        self.tray.refresh_total();
    }

    fn emit_usage_changed(&self) {
        if let Err(error) = self.app.emit(USAGE_CHANGED, ()) {
            tracing::warn!(%error, "could not emit the usage event");
        }
    }
}

impl UsageEvents for AppEvents {
    fn usage_changed(&self) {
        self.emit_usage_changed();
        self.tray.on_usage_changed();
    }
}
