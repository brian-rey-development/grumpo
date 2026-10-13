#![expect(
    unsafe_code,
    reason = "getuid has no safe wrapper in std and cannot fail"
)]

//! Installs the Grumpo service by running the installer bundled in this app as root, through
//! the system's administrator password prompt.
//!
//! The bundle sits where the person's own processes can write, so root never runs anything from
//! it directly. A fixed bootstrap first copies the installer and the daemon into a fresh folder
//! only root can touch, checks the copied daemon's signature, and runs the installer from the
//! copy, in an empty environment. With an ad hoc signature the check is integrity hygiene
//! and not authentication, see ADR 0011.

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use super::apple_script::OSASCRIPT;
use super::process::{run_bounded, RunError};
use crate::platform::{InstallOutcome, ServiceInstallError, ServiceInstaller};

const ENV: &str = "/usr/bin/env";
const SHELL: &str = "/bin/bash";
const CLEAN_ENVIRONMENT: [&str; 3] = ["-i", "PATH=/usr/bin:/bin:/usr/sbin:/sbin", "HOME=/var/root"];
/// Without these, bash reads an init file of root's when it believes a remote shell started it.
const SHELL_FLAGS: [&str; 3] = ["--noprofile", "--norc", "-c"];
const BOOTSTRAP_NAME: &str = "grumpo-install";
const CONTENTS_DIR: &str = "Contents";
const EXECUTABLE_DIR: &str = "MacOS";
const INSTALLER_DIR: &str = "Resources/installer";
const INSTALLER_SCRIPT: &str = "Resources/installer/scripts/install.sh";
const DAEMON_BINARY: &str = "MacOS/focusd";
/// Where the installer takes its lock, and the file in it that names the process holding it.
/// Root owns both, and anyone can read them.
const LOCK_PID_FILE: &str = "/var/run/focusd-install.lock/pid";
/// The password prompt can wait for the person, and the installer waits for the daemon to
/// answer before it ends.
const INSTALL_TIMEOUT: Duration = Duration::from_secs(600);
/// `userCanceledErr`, what the prompt reports when it is dismissed.
const USER_CANCELLED: i32 = -128;
const LOGGED_TAIL_CHARS: usize = 2000;

/// Runs as root with the installer folder, the daemon and the owner as `$1`, `$2` and `$3`.
/// Everything root writes lands in a folder `mktemp` creates under `/private/var/tmp` with mode
/// 700, and the installer runs from the copies. `cp -L` follows links, so a link inside the
/// bundle cannot point root at a file outside the copy. The folder is removed on every exit.
const BOOTSTRAP: &str = r#"set -eu
umask 022
work=$(/usr/bin/mktemp -d /private/var/tmp/grumpo-install.XXXXXX)
trap '/bin/rm -rf "$work"' EXIT
/bin/chmod 700 "$work"
/bin/cp -RL "$1" "$work/installer"
/bin/cp -L "$2" "$work/focusd"
/usr/bin/codesign --verify --strict "$work/focusd"
/bin/bash --noprofile --norc "$work/installer/scripts/install.sh" --binary "$work/focusd" --owner "$3"
"#;

/// Static on purpose, so no path or name can reach the script source. The values arrive as
/// arguments, and each one is shell quoted before the line runs as root. The prompt text is
/// what the password dialog shows in place of the generic wording that names osascript.
const SCRIPT: [&str; 8] = [
    "on run argv",
    "set quoted to {}",
    "repeat with n from 1 to count of argv",
    "set end of quoted to quoted form of item n of argv",
    "end repeat",
    "set AppleScript's text item delimiters to space",
    r#"do shell script (quoted as text) with prompt "Grumpo needs your password to install the service that blocks sites." with administrator privileges"#,
    "end run",
];

#[derive(Debug)]
pub(super) struct MacServiceInstaller;

impl ServiceInstaller for MacServiceInstaller {
    fn install(&self) -> Result<InstallOutcome, ServiceInstallError> {
        let executable = std::env::current_exe().map_err(|error| {
            tracing::debug!(%error, "current executable lookup failure detail");
            ServiceInstallError::Unavailable
        })?;
        let bundled =
            BundledInstaller::locate(&executable).ok_or(ServiceInstallError::Unavailable)?;
        if install_is_running() {
            return Err(ServiceInstallError::Busy);
        }
        run(&bundled, current_uid())
    }
}

#[derive(Debug, PartialEq, Eq)]
struct BundledInstaller {
    installer_dir: PathBuf,
    daemon: PathBuf,
}

impl BundledInstaller {
    /// Finds the installer next to `executable`, which must sit in `Contents/MacOS` of an app
    /// bundle that carries both the script and the daemon. Anything else, such as a build run
    /// from the target directory, has nothing to install.
    fn locate(executable: &Path) -> Option<Self> {
        let executable_dir = executable.parent()?;
        let contents = executable_dir.parent()?;
        let in_bundle =
            executable_dir.file_name()? == EXECUTABLE_DIR && contents.file_name()? == CONTENTS_DIR;
        let found = Self {
            installer_dir: contents.join(INSTALLER_DIR),
            daemon: contents.join(DAEMON_BINARY),
        };
        let has_script = contents.join(INSTALLER_SCRIPT).is_file();
        (in_bundle && has_script && found.daemon.is_file()).then_some(found)
    }

    fn arguments(&self, owner_uid: u32) -> Vec<OsString> {
        let mut arguments: Vec<OsString> = vec![ENV.into()];
        arguments.extend(CLEAN_ENVIRONMENT.map(OsString::from));
        arguments.push(SHELL.into());
        arguments.extend(SHELL_FLAGS.map(OsString::from));
        arguments.extend([
            BOOTSTRAP.into(),
            BOOTSTRAP_NAME.into(),
            self.installer_dir.clone().into_os_string(),
            self.daemon.clone().into_os_string(),
            owner_uid.to_string().into(),
        ]);
        arguments
    }
}

fn current_uid() -> u32 {
    // SAFETY: getuid takes no arguments, touches no memory and always succeeds.
    unsafe { libc::getuid() }
}

fn run(bundled: &BundledInstaller, owner_uid: u32) -> Result<InstallOutcome, ServiceInstallError> {
    let mut command = osascript(&SCRIPT, &bundled.arguments(owner_uid));
    let output = run_bounded(&mut command, INSTALL_TIMEOUT)
        .map_err(|error| failed_to_run(error, install_is_running()))?;
    outcome_of(
        output.status.success(),
        &String::from_utf8_lossy(&output.stderr),
    )
}

fn osascript(script: &[&str], arguments: &[OsString]) -> Command {
    let mut command = Command::new(OSASCRIPT);
    for line in script {
        command.arg("-e").arg(line);
    }
    command.args(arguments);
    command
}

/// Stopping the wait for the prompt does not stop the installer it started as root. If that
/// one still holds its lock, saying the install failed would invite a second run on top of it.
fn failed_to_run(error: RunError, installer_still_running: bool) -> ServiceInstallError {
    let detail = match error {
        RunError::Timeout if installer_still_running => return ServiceInstallError::Busy,
        RunError::Timeout => "the installer did not finish in time".to_owned(),
        RunError::Spawn(error) | RunError::Wait(error) => error.to_string(),
    };
    ServiceInstallError::Failed(detail)
}

/// Whether the installer holds its lock under a process that is still alive. A process of
/// another user answers `EPERM` to the probe, which still means it exists.
fn install_is_running() -> bool {
    std::fs::read_to_string(LOCK_PID_FILE)
        .ok()
        .and_then(|text| text.trim().parse::<libc::pid_t>().ok())
        .is_some_and(|pid| pid > 1 && process_exists(pid))
}

fn process_exists(pid: libc::pid_t) -> bool {
    // SAFETY: signal 0 only checks that the process exists and sends nothing.
    let alive = unsafe { libc::kill(pid, 0) } == 0;
    alive || std::io::Error::last_os_error().raw_os_error() == Some(libc::EPERM)
}

/// The installer writes its progress and its errors to the output, which carries paths and
/// the account, so it never reaches the error the UI receives. The log keeps only a fixed
/// sentence above debug, and the tail at debug with home folders blanked.
fn outcome_of(succeeded: bool, stderr: &str) -> Result<InstallOutcome, ServiceInstallError> {
    if succeeded {
        return Ok(InstallOutcome::Installed);
    }
    let code = trailing_error_number(stderr);
    if code == Some(USER_CANCELLED) {
        return Ok(InstallOutcome::Cancelled);
    }
    tracing::warn!("the service installer failed");
    tracing::debug!(tail = %redact_homes(tail(stderr)), "service installer output");
    Err(ServiceInstallError::Failed(match code {
        Some(code) => format!("the installer ended with status {code}"),
        None => "the installer ended without a status".to_owned(),
    }))
}

/// osascript ends its diagnostic with the status in parentheses, as in
/// `... execution error: User canceled. (-128)`, and a failing shell command reports its exit
/// status the same way. Only the last group counts, because the installer's own output comes
/// before it and may hold parentheses too.
fn trailing_error_number(stderr: &str) -> Option<i32> {
    let inner = stderr.trim_end().strip_suffix(')')?;
    inner.rsplit_once('(')?.1.parse().ok()
}

/// Replaces the account name in every `/Users/<name>` with `~`, since the installer's output
/// names the home folder of the person who ran it.
fn redact_homes(text: &str) -> String {
    const USERS: &str = "/Users/";
    let mut redacted = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(at) = rest.find(USERS) {
        let (before, after) = rest.split_at(at + USERS.len());
        redacted.push_str(before);
        redacted.push('~');
        let name_end = after
            .find(|c: char| c == '/' || c.is_whitespace() || matches!(c, '"' | '\''))
            .unwrap_or(after.len());
        rest = &after[name_end..];
    }
    redacted.push_str(rest);
    redacted
}

fn tail(text: &str) -> &str {
    let trimmed = text.trim_end();
    let start = trimmed
        .char_indices()
        .rev()
        .nth(LOGGED_TAIL_CHARS)
        .map_or(0, |(index, _)| index);
    &trimmed[start..]
}

#[cfg(test)]
mod tests;
