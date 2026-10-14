//! Synchronous client for the `grumpod` socket: one request and one reply per connection.

mod error;

use std::ffi::OsString;
use std::io;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use focus_core::ipc::transport::{exchange, ExchangeError};
use focus_core::ipc::{socket_path, DaemonStatus, Request, Response, DEFAULT_SOCKET_PATH};

pub use error::DaemonError;

const EXCHANGE_BUDGET: Duration = Duration::from_secs(2);
/// Where the daemon listened before it was renamed from focusd to grumpod.
const FOCUSD_SOCKET_PATH: &str = "/var/run/focusd.sock";

#[derive(Debug, Clone)]
pub struct DaemonClient {
    socket: PathBuf,
    budget: Duration,
}

impl DaemonClient {
    pub fn new(socket: PathBuf) -> Self {
        Self {
            socket,
            budget: EXCHANGE_BUDGET,
        }
    }

    pub fn from_env() -> Self {
        Self::from_lookup(|name| std::env::var_os(name))
    }

    fn from_lookup(lookup: impl Fn(&str) -> Option<OsString>) -> Self {
        Self::new(socket_path(lookup))
    }

    /// Sends `request` and waits for the reply. Writing and reading share one deadline that
    /// starts once connected, so the call returns within about two seconds whatever the
    /// daemon does. Connecting to a Unix socket answers at once, so it needs no timeout.
    pub fn send(&self, request: &Request) -> Result<DaemonStatus, DaemonError> {
        let stream = self.connect()?;
        let response: Response =
            exchange(&stream, request, self.budget).map_err(|error| match error {
                ExchangeError::Send(source) => DaemonError::NotResponding(source),
                ExchangeError::Codec(codec) => codec.into(),
            })?;
        Ok(response.into_result()?)
    }

    fn connect(&self) -> Result<UnixStream, DaemonError> {
        UnixStream::connect(&self.socket).map_err(|error| match error.kind() {
            io::ErrorKind::NotFound if self.focusd_answers_instead() => DaemonError::Renamed,
            io::ErrorKind::NotFound => DaemonError::NotInstalled,
            io::ErrorKind::PermissionDenied => DaemonError::OtherUser,
            _ => DaemonError::NotResponding(error),
        })
    }

    fn focusd_answers_instead(&self) -> bool {
        self.socket == Path::new(DEFAULT_SOCKET_PATH) && Path::new(FOCUSD_SOCKET_PATH).exists()
    }
}

#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;
