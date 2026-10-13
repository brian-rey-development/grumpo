use tauri::{AppHandle, Emitter, Manager, WebviewUrl, WebviewWindow, WebviewWindowBuilder};

use super::fallback::queue_with_fallback;
#[cfg(target_os = "macos")]
use super::overlay::{self, OverlayError};
use super::{BlockSurface, Fallback};
use crate::dto::BlockScreen;
use crate::events::{BLOCK_SCREEN, BLOCK_SCREEN_HIDDEN};
use crate::window::{BLOCKER_WINDOW, MAIN_WINDOW};

const BLOCKER_PAGE: &str = "blocker.html";
const BLOCKER_TITLE: &str = "Grumpo";

/// Both methods only queue their work for the main thread, the same way the menu bar does,
/// since building or touching a window from a worker waits for that thread.
pub struct TauriBlockSurface {
    app: AppHandle,
}

impl std::fmt::Debug for TauriBlockSurface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TauriBlockSurface").finish_non_exhaustive()
    }
}

impl TauriBlockSurface {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }

    fn post(&self, job: impl FnOnce(&AppHandle) + Send + 'static) {
        let app = self.app.clone();
        if let Err(error) = self.app.run_on_main_thread(move || job(&app)) {
            tracing::debug!(%error, "could not queue a block screen update");
        }
    }
}

impl BlockSurface for TauriBlockSurface {
    fn show(&self, screen: BlockScreen, on_failure: Fallback) {
        let app = self.app.clone();
        queue_with_fallback(
            |job| self.app.run_on_main_thread(job),
            move || reveal(&app, &screen),
            on_failure,
        );
    }

    fn hide(&self) {
        self.post(|app| {
            if let Some(window) = app.get_webview_window(BLOCKER_WINDOW) {
                if let Err(error) = app.emit_to(BLOCKER_WINDOW, BLOCK_SCREEN_HIDDEN, ()) {
                    tracing::warn!(%error, "could not tell the block screen it is hiding");
                }
                if let Err(error) = window.hide() {
                    tracing::warn!(%error, "could not hide the block screen");
                }
            }
            hand_focus_back(app);
        });
    }
}

#[derive(Debug, thiserror::Error)]
enum RevealError {
    #[error(transparent)]
    Window(#[from] tauri::Error),
    #[cfg(target_os = "macos")]
    #[error(transparent)]
    Overlay(#[from] OverlayError),
}

/// The display is chosen again on every show, since the person may have moved to another one.
/// The event goes out before the window is ordered front, so the page already holds the new
/// block when the first frame becomes visible and never flashes the previous one.
fn reveal(app: &AppHandle, screen: &BlockScreen) -> Result<(), RevealError> {
    let window = match app.get_webview_window(BLOCKER_WINDOW) {
        Some(window) => window,
        None => build(app)?,
    };
    app.emit_to(BLOCKER_WINDOW, BLOCK_SCREEN, screen)?;
    cover_display(app, &window)?;
    window.show()?;
    window.set_focus()?;
    Ok(())
}

#[cfg(target_os = "macos")]
fn cover_display(app: &AppHandle, window: &WebviewWindow) -> Result<(), RevealError> {
    let main = app.get_webview_window(MAIN_WINDOW);
    Ok(overlay::cover_display(window, main.as_ref())?)
}

#[cfg(not(target_os = "macos"))]
fn cover_display(_app: &AppHandle, window: &WebviewWindow) -> Result<(), RevealError> {
    Ok(window.set_fullscreen(true)?)
}

fn build(app: &AppHandle) -> tauri::Result<WebviewWindow> {
    let builder =
        WebviewWindowBuilder::new(app, BLOCKER_WINDOW, WebviewUrl::App(BLOCKER_PAGE.into()))
            .title(BLOCKER_TITLE)
            .decorations(false)
            .shadow(false)
            .resizable(false)
            .minimizable(false)
            .maximizable(false)
            .always_on_top(true)
            .visible_on_all_workspaces(true)
            .skip_taskbar(true)
            .visible(false);
    #[cfg(target_os = "macos")]
    let builder = builder.background_color(overlay::surface_color());
    builder.build()
}

/// Hiding the app hands focus to whatever the person was in. It is left alone while the main
/// window is open, because hiding the app would take that window away as well.
#[cfg(target_os = "macos")]
fn hand_focus_back(app: &AppHandle) {
    let main_is_open = app
        .get_webview_window(MAIN_WINDOW)
        .and_then(|window| window.is_visible().ok())
        .unwrap_or(false);
    if main_is_open {
        return;
    }
    if let Err(error) = app.hide() {
        tracing::warn!(%error, "could not hand focus back");
    }
}

#[cfg(not(target_os = "macos"))]
fn hand_focus_back(_app: &AppHandle) {}
