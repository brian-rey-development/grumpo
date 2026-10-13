//! The menu behind the menu bar item and what choosing its entries does.

use std::sync::Arc;

use focus_core::ipc::Request;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::TrayIconBuilder;
use tauri::{AppHandle, Manager};

use super::view::TauriView;
use crate::state::AppState;
use crate::window;

const TRAY_ID: &str = "focus";
const TOOLTIP: &str = "Grumpo";
const STATUS_ID: &str = "status";
const OPEN_ID: &str = "open";
const RELOCK_ID: &str = "relock";
const QUIT_ID: &str = "quit";

pub(super) fn build_view(app: &AppHandle) -> tauri::Result<TauriView> {
    let status_item = MenuItem::with_id(app, STATUS_ID, "", false, None::<&str>)?;
    let open_item = MenuItem::with_id(app, OPEN_ID, "Open Grumpo", true, None::<&str>)?;
    let relock_item = MenuItem::with_id(app, RELOCK_ID, "Relock now", false, None::<&str>)?;
    let quit_item = MenuItem::with_id(app, QUIT_ID, "Quit Grumpo", true, None::<&str>)?;
    let separator = PredefinedMenuItem::separator(app)?;
    let menu = Menu::with_items(
        app,
        &[
            &status_item,
            &open_item,
            &relock_item,
            &separator,
            &quit_item,
        ],
    )?;
    let tray = TrayIconBuilder::with_id(TRAY_ID)
        .icon(tauri::include_image!("icons/tray-template.png"))
        .icon_as_template(true)
        .tooltip(TOOLTIP)
        .menu(&menu)
        .show_menu_on_left_click(true)
        .on_menu_event(|app, event| on_menu_event(app, event.id().as_ref()))
        .build(app)?;
    Ok(TauriView {
        tray,
        status_item,
        relock_item,
        quit_item,
    })
}

fn on_menu_event(app: &AppHandle, id: &str) {
    match id {
        OPEN_ID => window::show_main(app),
        RELOCK_ID => relock(app),
        QUIT_ID => app.exit(0),
        _ => {}
    }
}

fn relock(app: &AppHandle) {
    let Some(state) = app.try_state::<AppState>() else {
        return;
    };
    let status = Arc::clone(&state.status);
    tauri::async_runtime::spawn_blocking(move || {
        if let Err(error) = status.send(&Request::Relock) {
            tracing::warn!(%error, "relock from the tray failed");
        }
    });
}
