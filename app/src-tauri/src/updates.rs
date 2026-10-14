//! Installs signed releases in the background and restarts into them once no window is open.

mod relaunch;
mod schedule;

use std::fs::{self, File};
use std::io;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};

use tauri::{AppHandle, Manager};
use tauri_plugin_updater::UpdaterExt;

use crate::concurrency::{StopSignal, StoppableThread};
use crate::launch_at_login;
use crate::notify::{Notice, Notifier};
use crate::storage::Store;
use schedule::Outcome;

const THREAD_NAME: &str = "updates";
const FIRST_CHECK_DELAY: Duration = Duration::from_mins(2);
const POLL_INTERVAL: Duration = Duration::from_mins(1);
const WRITE_PROBE: &str = ".grumpo-update-probe";

pub struct UpdateDeps {
    pub app: AppHandle,
    pub store: Arc<Store>,
    pub notifier: Arc<dyn Notifier>,
}

/// Skipped where the app cannot replace itself, or the updater asks for an admin password.
pub fn start(deps: UpdateDeps) -> io::Result<Option<StoppableThread>> {
    if cfg!(debug_assertions) || !can_replace_itself() {
        return Ok(None);
    }
    StoppableThread::spawn_signalled(THREAD_NAME, move |signal| run(&deps, signal)).map(Some)
}

fn can_replace_itself() -> bool {
    let Ok(executable) = std::env::current_exe() else {
        return false;
    };
    if launch_at_login::is_transient_location(&executable) {
        return false;
    }
    relaunch::bundle_of(&executable)
        .and_then(|bundle| bundle.parent().map(is_writable))
        .unwrap_or(false)
}

fn is_writable(dir: &Path) -> bool {
    let probe = dir.join(WRITE_PROBE);
    let writable = File::create(&probe).is_ok();
    let _ = fs::remove_file(&probe);
    writable
}

fn run(deps: &UpdateDeps, signal: &StopSignal) {
    if signal.wait(FIRST_CHECK_DELAY) || !install_when_available(deps, signal) {
        return;
    }
    if has_visible_window(&deps.app) {
        deps.notifier.notify(installed_notice());
    }
    if wait_until_hidden(&deps.app, signal) {
        relaunch::restart_hidden(&deps.app);
    }
}

/// Returns false when told to stop before anything was installed.
fn install_when_available(deps: &UpdateDeps, signal: &StopSignal) -> bool {
    let mut next_check = SystemTime::now();
    loop {
        let now = SystemTime::now();
        if schedule::is_due(next_check, now) && enabled(&deps.store) {
            match check_and_install(&deps.app) {
                Outcome::Installed => return true,
                outcome => next_check = schedule::next_check(outcome, now),
            }
        }
        if signal.wait(POLL_INTERVAL) {
            return false;
        }
    }
}

fn wait_until_hidden(app: &AppHandle, signal: &StopSignal) -> bool {
    while has_visible_window(app) {
        if signal.wait(POLL_INTERVAL) {
            return false;
        }
    }
    true
}

fn enabled(store: &Store) -> bool {
    store.auto_update().unwrap_or_else(|error| {
        tracing::warn!(%error, "could not read the update setting, checking anyway");
        true
    })
}

fn check_and_install(app: &AppHandle) -> Outcome {
    match tauri::async_runtime::block_on(install_available(app)) {
        Ok(None) => Outcome::UpToDate,
        Ok(Some(version)) => {
            tracing::info!(%version, "installed an update");
            Outcome::Installed
        }
        Err(error) => {
            tracing::warn!(%error, "update check failed");
            Outcome::Failed
        }
    }
}

async fn install_available(app: &AppHandle) -> tauri_plugin_updater::Result<Option<String>> {
    let Some(update) = app.updater()?.check().await? else {
        return Ok(None);
    };
    update.download_and_install(|_, _| {}, || {}).await?;
    Ok(Some(update.version))
}

fn installed_notice() -> Notice {
    Notice {
        title: "A new version of Grumpo is installed".to_owned(),
        body: "Grumpo restarts into it when you close its windows.".to_owned(),
    }
}

fn has_visible_window(app: &AppHandle) -> bool {
    app.webview_windows()
        .values()
        .any(|window| window.is_visible().unwrap_or(true))
}
