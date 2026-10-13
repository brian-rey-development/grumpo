//! Window behaviour. Closing the main window hides it so tracking keeps running, and the
//! tray or the Dock brings it back. Closing the block screen dismisses it.

use tauri::{AppHandle, Manager, Window, WindowEvent};

use crate::state::AppState;

pub const MAIN_WINDOW: &str = "main";
pub const BLOCKER_WINDOW: &str = "blocker";

pub fn show_main(app: &AppHandle) {
    let Some(window) = app.get_webview_window(MAIN_WINDOW) else {
        tracing::warn!("main window is missing");
        return;
    };
    for result in [window.show(), window.unminimize(), window.set_focus()] {
        if let Err(error) = result {
            tracing::warn!(%error, "could not bring the main window forward");
        }
    }
}

pub fn hide_on_close(window: &Window, event: &WindowEvent) {
    let WindowEvent::CloseRequested { api, .. } = event else {
        return;
    };
    match window.label() {
        MAIN_WINDOW => {
            api.prevent_close();
            if let Err(error) = window.hide() {
                tracing::warn!(%error, "could not hide the main window");
            }
        }
        BLOCKER_WINDOW => {
            api.prevent_close();
            dismiss_block_screen(window.app_handle());
        }
        _ => {}
    }
}

fn dismiss_block_screen(app: &AppHandle) {
    if let Some(state) = app.try_state::<AppState>() {
        state.block_screens.dismiss(None);
    }
}
