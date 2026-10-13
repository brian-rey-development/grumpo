use std::io;
use std::os::unix::net::UnixStream;
use std::path::{Path, PathBuf};
use std::time::Duration;

use focus_core::ipc::transport::{exchange, ExchangeError};
use focus_core::ipc::{IpcCodecError, IpcError, Request, Response};
use thiserror::Error;

const PING_BUDGET: Duration = Duration::from_secs(2);

#[derive(Debug, Error)]
pub enum PingError {
    #[error("could not connect to {}: {source}", path.display())]
    Connect { path: PathBuf, source: io::Error },
    #[error("could not send the request: {0}")]
    Send(io::Error),
    #[error("the daemon did not speak the protocol: {0}")]
    Protocol(IpcCodecError),
    #[error("the daemon answered with an error ({:?}): {}", .0.code, .0.message)]
    Rejected(IpcError),
}

/// Proves the daemon behind `socket` is alive and speaks the protocol, by asking for its status.
pub fn ping(socket: &Path) -> Result<(), PingError> {
    let stream = UnixStream::connect(socket).map_err(|source| PingError::Connect {
        path: socket.to_path_buf(),
        source,
    })?;
    let response: Response =
        exchange(&stream, &Request::Status, PING_BUDGET).map_err(|error| match error {
            ExchangeError::Send(source) => PingError::Send(source),
            ExchangeError::Codec(codec) => PingError::Protocol(codec),
        })?;
    response
        .into_result()
        .map(|_status| ())
        .map_err(PingError::Rejected)
}

#[cfg(test)]
mod tests;
