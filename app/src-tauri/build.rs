use std::error::Error;

use tauri_build::{AppManifest, Attributes};

const COMMANDS: &[&str] = &[
    "get_day_report",
    "get_week_report",
    "get_limit_usage",
    "get_focus_status",
    "save_rules",
    "request_unlock",
    "confirm_unlock",
    "cancel_unlock",
    "relock",
    "list_known_apps",
    "get_app_icon",
    "get_settings",
    "update_settings",
    "get_permissions",
    "request_permission",
    "open_permission_settings",
    "export_data",
    "delete_all_data",
    "get_clock_style",
    "install_service",
    "get_onboarding_state",
    "complete_onboarding",
    "reset_onboarding",
    "get_block_screen",
    "dismiss_block_screen",
];

fn main() -> Result<(), Box<dyn Error>> {
    let manifest = AppManifest::new().commands(COMMANDS);
    tauri_build::try_build(Attributes::new().app_manifest(manifest))?;
    Ok(())
}
