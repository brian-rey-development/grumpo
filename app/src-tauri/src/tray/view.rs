//! What the menu bar item shows, and how a change reaches it.
//!
//! Tauri's tray and menu setters wait for the main thread when they are called from another
//! one. Calling them from a worker while the main thread is blocked joining that worker, as
//! it is when the app exits, would hang the exit forever. So nothing outside the main thread
//! touches a widget: callers go through [`MainThread::post`], which only queues the change
//! and returns at once, and the [`TrayView`] is only ever called from the queued closure.

use tauri::menu::MenuItem;
use tauri::tray::TrayIcon;
use tauri::{AppHandle, Wry};

pub trait TrayView: Send + Sync {
    fn set_status_line(&self, line: &str);
    fn set_relock_enabled(&self, enabled: bool);
    fn set_quit_label(&self, label: &str);
    fn set_total(&self, title: &str);
}

/// Runs a closure on the main thread without waiting for it.
pub trait MainThread: Send + Sync {
    fn post(&self, job: Box<dyn FnOnce() + Send>);
}

pub struct TauriMainThread(pub AppHandle);

impl std::fmt::Debug for TauriMainThread {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriMainThread").finish_non_exhaustive()
    }
}

impl MainThread for TauriMainThread {
    fn post(&self, job: Box<dyn FnOnce() + Send>) {
        if let Err(error) = self.0.run_on_main_thread(job) {
            tracing::debug!(%error, "could not queue a menu bar update");
        }
    }
}

pub struct TauriView {
    pub tray: TrayIcon,
    pub status_item: MenuItem<Wry>,
    pub relock_item: MenuItem<Wry>,
    pub quit_item: MenuItem<Wry>,
}

impl std::fmt::Debug for TauriView {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriView").finish_non_exhaustive()
    }
}

impl TrayView for TauriView {
    fn set_status_line(&self, line: &str) {
        log_failure("set status line", self.status_item.set_text(line));
    }

    fn set_relock_enabled(&self, enabled: bool) {
        log_failure("set relock state", self.relock_item.set_enabled(enabled));
    }

    fn set_quit_label(&self, label: &str) {
        log_failure("set quit label", self.quit_item.set_text(label));
    }

    fn set_total(&self, title: &str) {
        log_failure("set title", self.tray.set_title(Some(title)));
    }
}

fn log_failure(action: &str, result: tauri::Result<()>) {
    if let Err(error) = result {
        tracing::debug!(%error, action, "tray update failed");
    }
}
