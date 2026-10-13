pub(crate) mod block_screen;
pub(crate) mod commands;
pub(crate) mod concurrency;
pub(crate) mod daemon_client;
pub(crate) mod dto;
pub(crate) mod enforcement;
pub(crate) mod error;
pub(crate) mod events;
pub(crate) mod icon_cache;
pub(crate) mod launch_at_login;
pub(crate) mod legacy_data;
pub(crate) mod notify;
pub(crate) mod platform;
pub(crate) mod setup;
pub(crate) mod state;
pub(crate) mod status;
pub(crate) mod storage;
pub(crate) mod telemetry;
pub(crate) mod time;
pub(crate) mod tracking;
pub(crate) mod tray;
pub(crate) mod window;

use std::process::ExitCode;

use tauri::{AppHandle, Manager, RunEvent};
#[cfg(target_os = "macos")]
use tauri_plugin_autostart::MacosLauncher;

use crate::state::Workers;

pub fn run() -> ExitCode {
    let _log_guard = telemetry::init();
    telemetry::install_panic_hook();

    match build() {
        Ok(app) => {
            let code = app.run_return(on_run_event);
            u8::try_from(code).map_or(ExitCode::FAILURE, ExitCode::from)
        }
        Err(error) => {
            tracing::error!(%error, "app run failed");
            ExitCode::FAILURE
        }
    }
}

fn build() -> tauri::Result<tauri::App> {
    tauri::Builder::default()
        .plugin(tauri_plugin_notification::init())
        .plugin(login_item_plugin())
        .setup(setup::setup)
        .on_window_event(window::hide_on_close)
        .invoke_handler(command_handler())
        .build(tauri::generate_context!())
}

fn command_handler() -> impl Fn(tauri::ipc::Invoke) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        commands::reports::get_day_report,
        commands::reports::get_week_report,
        commands::reports::get_limit_usage,
        commands::status::get_focus_status,
        commands::rules::save_rules,
        commands::unlock::request_unlock,
        commands::unlock::confirm_unlock,
        commands::unlock::cancel_unlock,
        commands::unlock::relock,
        commands::apps::list_known_apps,
        commands::apps::get_app_icon,
        commands::settings::get_settings,
        commands::settings::update_settings,
        commands::permissions::get_permissions,
        commands::permissions::request_permission,
        commands::permissions::open_permission_settings,
        commands::data::export_data,
        commands::data::delete_all_data,
        commands::clock::get_clock_style,
        commands::service::install_service,
        commands::onboarding::get_onboarding_state,
        commands::onboarding::complete_onboarding,
        commands::onboarding::reset_onboarding,
        commands::block_screen::get_block_screen,
        commands::block_screen::dismiss_block_screen,
    ]
}

fn login_item_plugin() -> tauri::plugin::TauriPlugin<tauri::Wry> {
    let builder = tauri_plugin_autostart::Builder::new()
        .arg(launch_at_login::AUTOSTART_ARG)
        .app_name(launch_at_login::LOGIN_ITEM_LABEL);
    #[cfg(target_os = "macos")]
    let builder = builder.macos_launcher(MacosLauncher::LaunchAgent);
    builder.build()
}

#[expect(
    clippy::needless_pass_by_value,
    reason = "the signature is fixed by `run_return`"
)]
fn on_run_event(app: &AppHandle, event: RunEvent) {
    match event {
        #[cfg(target_os = "macos")]
        RunEvent::Reopen { .. } => window::show_main(app),
        RunEvent::Exit => {
            if let Some(workers) = app.try_state::<Workers>() {
                workers.stop();
            }
        }
        _ => {}
    }
}
