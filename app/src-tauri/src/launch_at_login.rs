//! Launch at login, behind a seam so the settings logic is testable without the OS.

use std::io;
use std::path::{Path, PathBuf};

use tauri::AppHandle;
use tauri_plugin_autostart::ManagerExt;

use crate::storage::Store;

mod error;
#[cfg(test)]
pub(crate) mod test_support;

pub use error::LaunchAtLoginError;

/// The name the login item goes by in System Settings and in `~/Library/LaunchAgents`.
pub const LOGIN_ITEM_LABEL: &str = "io.github.brian-rey-development.grumpo";
/// The labels earlier releases registered, whose property lists would keep launching an app
/// that is no longer there and show it under its old name in System Settings.
const LEGACY_LOGIN_ITEM_LABELS: [&str; 2] = ["io.github.brian-rey-development.focus", "Focus"];
/// "Focus" is a common name, so a property list under a legacy label is only ours when it
/// launches one of these bundles.
const OWN_APP_BUNDLES: [&str; 2] = ["Focus.app", "Grumpo.app"];
const AGENTS_FROM_HOME: &str = "Library/LaunchAgents";
/// Remembers that the first-run default was applied, so a later opt-out sticks.
const INITIALIZED_KEY: &str = "launch_at_login_initialized";
const INITIALIZED_VALUE: &str = "1";
/// The argument the login item passes, so the app can tell a start at login from a start by
/// hand and stay out of the way in the first case.
pub const AUTOSTART_ARG: &str = "--autostarted";
/// Where macOS mounts disk images and where it copies an app it has quarantined before the
/// first launch. The app in either place disappears, so a login item pointing there breaks.
const VOLUMES_PREFIX: &str = "/Volumes/";
const TRANSLOCATION_MARKER: &str = "AppTranslocation";

pub trait LaunchAtLogin: Send + Sync {
    fn is_enabled(&self) -> Result<bool, LaunchAtLoginError>;
    fn set_enabled(&self, enabled: bool) -> Result<(), LaunchAtLoginError>;
}

pub struct Autostart {
    app: AppHandle,
}

impl std::fmt::Debug for Autostart {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Autostart").finish_non_exhaustive()
    }
}

impl Autostart {
    pub fn new(app: AppHandle) -> Self {
        Self { app }
    }
}

impl LaunchAtLogin for Autostart {
    fn is_enabled(&self) -> Result<bool, LaunchAtLoginError> {
        Ok(self.app.autolaunch().is_enabled()?)
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), LaunchAtLoginError> {
        let manager = self.app.autolaunch();
        let result = if enabled {
            manager.enable()
        } else {
            manager.disable()
        };
        Ok(result?)
    }
}

/// Whether this run was started by the login item.
pub fn was_autostarted(mut args: impl Iterator<Item = String>) -> bool {
    args.any(|arg| arg == AUTOSTART_ARG)
}

/// True when the app runs from a place that will not be there next login.
pub fn is_transient_location(executable: &Path) -> bool {
    executable.starts_with(VOLUMES_PREFIX)
        || executable
            .components()
            .any(|part| part.as_os_str() == TRANSLOCATION_MARKER)
}

/// Where the system keeps a user's login items, under `home`.
pub fn agents_dir(home: &Path) -> PathBuf {
    home.join(AGENTS_FROM_HOME)
}

/// Keeps the login item in step with this start. The first run turns it on and is marked done
/// only after the system accepted the change, so a refusal is retried on the next start. Every
/// later run rewrites an existing item, because the plist holds the path of the app that
/// created it and an app that was moved or updated would otherwise leave it pointing nowhere.
/// An item left by an earlier name of the app is replaced by one under the current label.
/// Nothing is written from a disk image or a translocated copy, and a first run there stays
/// pending until the app runs from where it will live.
pub fn sync_on_start(
    store: &Store,
    launch: &dyn LaunchAtLogin,
    executable: &Path,
    agents_dir: &Path,
) -> Result<(), LaunchAtLoginError> {
    if is_transient_location(executable) {
        tracing::info!("not touching launch at login, the app is not installed yet");
        return Ok(());
    }
    if replace_legacy_items(launch, agents_dir)? {
        store.set_setting(INITIALIZED_KEY, INITIALIZED_VALUE)?;
        return Ok(());
    }
    if store.setting(INITIALIZED_KEY)?.is_none() {
        launch.set_enabled(true)?;
        store.set_setting(INITIALIZED_KEY, INITIALIZED_VALUE)?;
        return Ok(());
    }
    if launch.is_enabled()? {
        launch.set_enabled(true)?;
    }
    Ok(())
}

/// Turns the new item on before deleting the old ones, so a crash between the two steps leaves
/// both and the next start finishes the job, where the other order could leave neither.
fn replace_legacy_items(
    launch: &dyn LaunchAtLogin,
    agents_dir: &Path,
) -> Result<bool, LaunchAtLoginError> {
    let found: Vec<PathBuf> = LEGACY_LOGIN_ITEM_LABELS
        .iter()
        .map(|label| agents_dir.join(format!("{label}.plist")))
        .filter(|plist| plist.exists() && launches_own_app(plist))
        .collect();
    if found.is_empty() {
        return Ok(false);
    }
    launch.set_enabled(true)?;
    for plist in &found {
        remove_if_present(plist)?;
    }
    tracing::info!("replaced the login item of the earlier app identity");
    Ok(true)
}

fn launches_own_app(plist: &Path) -> bool {
    let ours = program_of(plist).is_some_and(|program| {
        program.components().any(|part| {
            OWN_APP_BUNDLES
                .iter()
                .any(|bundle| part.as_os_str() == *bundle)
        })
    });
    if !ours {
        tracing::info!("left a login item under an earlier label alone, it is not this app's");
        tracing::debug!(plist = %plist.display(), "login item kept");
    }
    ours
}

fn program_of(plist: &Path) -> Option<PathBuf> {
    let value = plist::Value::from_file(plist).ok()?;
    let dictionary = value.as_dictionary()?;
    let first_argument = dictionary
        .get("ProgramArguments")
        .and_then(plist::Value::as_array)
        .and_then(|arguments| arguments.first())
        .and_then(plist::Value::as_string);
    let program = first_argument.or_else(|| dictionary.get("Program")?.as_string());
    program.map(PathBuf::from)
}

fn remove_if_present(path: &Path) -> Result<(), LaunchAtLoginError> {
    match std::fs::remove_file(path) {
        Err(error) if error.kind() != io::ErrorKind::NotFound => {
            Err(LaunchAtLoginError::LegacyItem(error))
        }
        _ => Ok(()),
    }
}

#[cfg(test)]
mod tests;
