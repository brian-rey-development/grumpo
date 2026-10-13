use std::ffi::OsString;
use std::path::PathBuf;

use focus_core::ipc::socket_path;

const DEFAULT_STATE_DIR: &str = "/Library/Application Support/Focus";
pub const SYSTEM_HOSTS_FILE: &str = "/etc/hosts";

const STATE_DIR_VAR: &str = "FOCUSD_STATE_DIR";
const HOSTS_FILE_VAR: &str = "FOCUSD_HOSTS_FILE";
const LOG_FILE_VAR: &str = "FOCUSD_LOG_FILE";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Paths {
    pub state_dir: PathBuf,
    pub hosts_file: PathBuf,
    pub socket: PathBuf,
    /// Set only when launchd redirects stderr to a file that newsyslog rotates.
    pub log_file: Option<PathBuf>,
}

impl Paths {
    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var_os(name))
    }

    /// Takes the lookup as a parameter so tests never touch the process environment.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<OsString>) -> Self {
        let resolve = |name: &str, default: &str| {
            lookup(name)
                .filter(|value| !value.is_empty())
                .map_or_else(|| PathBuf::from(default), PathBuf::from)
        };
        Self {
            state_dir: resolve(STATE_DIR_VAR, DEFAULT_STATE_DIR),
            hosts_file: resolve(HOSTS_FILE_VAR, SYSTEM_HOSTS_FILE),
            socket: socket_path(&lookup),
            log_file: lookup(LOG_FILE_VAR)
                .filter(|value| !value.is_empty())
                .map(PathBuf::from),
        }
    }

    /// Only the real hosts file is worth flushing the resolver caches for.
    pub fn flushes_dns(&self) -> bool {
        self.hosts_file.as_os_str() == SYSTEM_HOSTS_FILE
    }
}

#[cfg(test)]
mod tests;
