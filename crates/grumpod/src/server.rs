mod connection;
#[cfg(test)]
mod tests;

use std::fs::{self, Permissions};
use std::io;
use std::os::unix::fs::{chown, FileTypeExt, PermissionsExt};
use std::os::unix::net::UnixListener;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use thiserror::Error;
use tracing::{info, warn};

use crate::service::Daemon;
use crate::shared::Shared;
use crate::sys;

const SOCKET_MODE: u32 = 0o600;
const ACCEPT_RETRY_DELAY: Duration = Duration::from_millis(100);

#[derive(Debug, Error)]
pub enum ServerError {
    #[error("could not {action} {}: {source}", path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    #[error("{} exists and is not a socket, refusing to replace it", .0.display())]
    NotASocket(PathBuf),
}

#[derive(Debug)]
pub struct Server {
    listener: UnixListener,
    owner_uid: u32,
}

impl Server {
    /// Listens on `path`, replacing a stale socket left by a previous run, and restricts it
    /// to `owner_uid`.
    pub fn bind(path: &Path, owner_uid: u32) -> Result<Self, ServerError> {
        let io_error = |action, source| ServerError::Io {
            action,
            path: path.to_path_buf(),
            source,
        };
        remove_stale_socket(path)?;
        let listener = UnixListener::bind(path).map_err(|source| io_error("bind", source))?;
        fs::set_permissions(path, Permissions::from_mode(SOCKET_MODE))
            .map_err(|source| io_error("restrict", source))?;
        hand_over(path, owner_uid).map_err(|source| io_error("hand over", source))?;
        info!(socket = %path.display(), owner_uid, "listening");
        Ok(Self {
            listener,
            owner_uid,
        })
    }

    /// Serves clients one at a time, forever.
    ///
    /// Requests are tiny and rare (the app polls every few seconds), and a connection gets a
    /// fixed time budget to send its request and another to take the response, so a stuck
    /// client delays others by a few seconds at most. In exchange there is no thread
    /// bookkeeping, and the service lock is taken by one party at a time anyway. The lock is
    /// held while a request is applied, which includes the state and hosts file writes, but
    /// never during socket I/O or the resolver flush.
    pub fn serve<D: Daemon>(&self, service: &Shared<D>) -> ! {
        loop {
            match self.listener.accept() {
                Ok((stream, _)) => connection::serve(&stream, self.owner_uid, service),
                Err(error) => {
                    warn!(%error, "could not accept a connection");
                    thread::sleep(ACCEPT_RETRY_DELAY);
                }
            }
        }
    }
}

/// Gives the socket to the owner when running as root. Only root can give a file away, so
/// any other user keeps the socket, which is what tests and development runs want.
fn hand_over(path: &Path, owner_uid: u32) -> io::Result<()> {
    if sys::effective_uid() != 0 {
        return Ok(());
    }
    chown(path, Some(owner_uid), None)
}

fn remove_stale_socket(path: &Path) -> Result<(), ServerError> {
    let io_error = |source| ServerError::Io {
        action: "inspect",
        path: path.to_path_buf(),
        source,
    };
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_socket() => {
            fs::remove_file(path).map_err(|source| ServerError::Io {
                action: "remove the stale socket",
                path: path.to_path_buf(),
                source,
            })
        }
        Ok(_) => Err(ServerError::NotASocket(path.to_path_buf())),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(io_error(source)),
    }
}
