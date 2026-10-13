//! User notifications behind a trait, so enforcement never depends on Tauri.

use tauri_plugin_notification::NotificationExt;

#[cfg(test)]
pub(crate) mod test_support;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Notice {
    pub title: String,
    pub body: String,
}

/// Delivery is best effort: an implementation logs a failure itself and never panics.
pub trait Notifier: Send + Sync {
    fn notify(&self, notice: Notice);
}

#[derive(Debug, Clone)]
pub struct TauriNotifier {
    app: tauri::AppHandle,
}

impl TauriNotifier {
    pub fn new(app: tauri::AppHandle) -> Self {
        Self { app }
    }
}

impl Notifier for TauriNotifier {
    fn notify(&self, notice: Notice) {
        let shown = self
            .app
            .notification()
            .builder()
            .title(notice.title)
            .body(notice.body)
            .show();
        if let Err(error) = shown {
            tracing::warn!(%error, "could not show a notification");
        }
    }
}
