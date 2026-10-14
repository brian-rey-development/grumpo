use std::os::unix::process::CommandExt;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use tauri::AppHandle;

use crate::launch_at_login::AUTOSTART_ARG;

const SHELL: &str = "/bin/sh";
const APP_BUNDLE_EXTENSION: &str = "app";
const EXECUTABLE_DEPTH_IN_BUNDLE: usize = 3;
/// About 30 seconds at 0.2 seconds a round.
const EXIT_WAIT_ROUNDS: &str = "150";
/// Waits for this process to exit so two copies never track at once, then reopens it hidden.
const RELAUNCH_SCRIPT: &str = r#"for _ in $(seq "$4"); do kill -0 "$1" 2>/dev/null || exec /usr/bin/open -g "$2" --args "$3"; sleep 0.2; done"#;

pub(super) fn restart_hidden(app: &AppHandle) {
    let Some(bundle) = std::env::current_exe().ok().and_then(|exe| bundle_of(&exe)) else {
        tracing::warn!("no app bundle to relaunch, the update applies at the next launch");
        return;
    };
    match relaunch_command(&bundle).spawn() {
        Ok(_) => app.exit(0),
        Err(error) => tracing::warn!(%error, "could not schedule the relaunch"),
    }
}

/// The shell needs its own process group, because launchd kills a login item's whole group on exit.
fn relaunch_command(bundle: &Path) -> Command {
    let mut command = Command::new(SHELL);
    command
        .args(["-c", RELAUNCH_SCRIPT, SHELL])
        .arg(std::process::id().to_string())
        .arg(bundle)
        .args([AUTOSTART_ARG, EXIT_WAIT_ROUNDS])
        .process_group(0)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    command
}

pub(super) fn bundle_of(executable: &Path) -> Option<PathBuf> {
    let bundle = executable.ancestors().nth(EXECUTABLE_DEPTH_IN_BUNDLE)?;
    let is_bundle = bundle
        .extension()
        .is_some_and(|ext| ext == APP_BUNDLE_EXTENSION);
    is_bundle.then(|| bundle.to_path_buf())
}

#[cfg(test)]
mod tests;
