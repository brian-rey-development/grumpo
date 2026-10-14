use std::path::Path;

use thiserror::Error;

use crate::instance_lock::{InstanceLock, LockError};
use crate::state::{FileStateStore, StateError};

#[derive(Debug, Error)]
pub enum StateDirError {
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    Lock(#[from] LockError),
}

/// Makes `directory` a private place for the state, then takes the instance lock on it.
///
/// Every command that writes the state starts here, in this order, so the directory exists
/// before the lock file is created in it and nothing else can write the state while the
/// returned lock is alive.
pub fn ensure_state_dir(directory: &Path) -> Result<(FileStateStore, InstanceLock), StateDirError> {
    let store = FileStateStore::open(directory)?;
    let lock = InstanceLock::acquire(directory)?;
    Ok((store, lock))
}

#[cfg(test)]
mod tests;
