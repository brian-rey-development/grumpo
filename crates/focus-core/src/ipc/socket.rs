use std::ffi::OsString;
use std::path::PathBuf;

pub const DEFAULT_SOCKET_PATH: &str = "/var/run/focusd.sock";
pub const SOCKET_ENV: &str = "FOCUSD_SOCKET";

/// Where the daemon listens, which `SOCKET_ENV` overrides when it is set and not empty.
///
/// Takes the lookup as a parameter so tests never touch the process environment.
pub fn socket_path(lookup: impl Fn(&str) -> Option<OsString>) -> PathBuf {
    lookup(SOCKET_ENV)
        .filter(|value| !value.is_empty())
        .map_or_else(|| PathBuf::from(DEFAULT_SOCKET_PATH), PathBuf::from)
}

#[cfg(test)]
mod tests;
