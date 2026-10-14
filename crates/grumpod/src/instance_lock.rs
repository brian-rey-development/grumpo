use std::fs::{File, OpenOptions, TryLockError};
use std::io;
use std::os::unix::fs::OpenOptionsExt;
use std::path::{Path, PathBuf};

use thiserror::Error;

const LOCK_FILE: &str = "grumpod.lock";
const LOCK_MODE: u32 = 0o600;

#[derive(Debug, Error)]
pub enum LockError {
    #[error("grumpod is already running, it holds the lock {}", .0.display())]
    Held(PathBuf),
    #[error("could not take the instance lock {}: {source}", path.display())]
    Io { path: PathBuf, source: io::Error },
}

/// Proof that this process is the only one working on the state directory.
///
/// The daemon holds it for its whole life, and the commands that edit state or the hosts
/// file hold it while they run, so none of them can race the daemon or each other. The kernel
/// drops the lock when the process dies, so a crash never leaves a stale one behind.
#[derive(Debug)]
pub struct InstanceLock {
    _file: File,
}

impl InstanceLock {
    pub fn acquire(directory: &Path) -> Result<Self, LockError> {
        let path = directory.join(LOCK_FILE);
        let io_error = |source| LockError::Io {
            path: path.clone(),
            source,
        };
        let file = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .mode(LOCK_MODE)
            .open(&path)
            .map_err(io_error)?;
        match file.try_lock() {
            Ok(()) => Ok(Self { _file: file }),
            Err(TryLockError::WouldBlock) => Err(LockError::Held(path.clone())),
            Err(TryLockError::Error(source)) => Err(io_error(source)),
        }
    }

    /// Like [`acquire`](Self::acquire), but `None` when the directory does not exist, since
    /// then no daemon has ever run there and there is nothing to race.
    pub fn acquire_if_initialized(directory: &Path) -> Result<Option<Self>, LockError> {
        match Self::acquire(directory) {
            Err(LockError::Io { source, .. }) if source.kind() == io::ErrorKind::NotFound => {
                Ok(None)
            }
            other => other.map(Some),
        }
    }
}

#[cfg(test)]
mod tests;
