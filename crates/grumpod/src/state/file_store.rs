use std::fs::{self, DirBuilder, File, OpenOptions, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use focus_core::ipc::peer::effective_uid;
use focus_core::Now;
use tracing::{debug, error, info, warn};

use super::migrate::migrate;
use super::persisted::PersistedState;
use super::{Snapshot, StateError, StateStore};
use crate::atomic_write::{remove_stale_temps, write_atomic};

const DIR_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;
const STATE_FILE: &str = "state.json";
const MAX_QUARANTINE_NAMES: u32 = 100;
/// Where a damaged state file is copied when it cannot be moved to a name of its own, so one
/// copy to examine survives even when the next save replaces the damaged file.
const LAST_CORRUPT_SLOT: &str = "state.json.corrupt-last";

#[derive(Debug)]
pub struct FileStateStore {
    file: PathBuf,
}

impl FileStateStore {
    /// Prepares `directory` as a private place for the state file, owned by the user the daemon
    /// runs as.
    ///
    /// A symlink in place of the directory or the state file is refused, and so is a directory
    /// that someone else owns, because either would let another user steer or read the rules.
    /// Nothing is chowned, since the right answer to a foreign owner is for a person to look.
    pub fn open(directory: &Path) -> Result<Self, StateError> {
        Self::open_as(directory, effective_uid())
    }

    fn open_as(directory: &Path, expected_uid: u32) -> Result<Self, StateError> {
        let io_error = |action, source| StateError::Io {
            action,
            path: directory.to_path_buf(),
            source,
        };
        refuse_symlink(directory)?;
        DirBuilder::new()
            .recursive(true)
            .mode(DIR_MODE)
            .create(directory)
            .map_err(|source| io_error("create the state directory", source))?;
        let handle = open_directory(directory)
            .map_err(|source| io_error("open the state directory", source))?;
        verify_owner(&handle, directory, expected_uid)?;
        handle
            .set_permissions(Permissions::from_mode(DIR_MODE))
            .map_err(|source| io_error("restrict the state directory", source))?;
        let file = directory.join(STATE_FILE);
        refuse_symlink(&file)?;
        Ok(Self { file })
    }

    /// Removes the temporary files an interrupted save left behind. The caller must hold the
    /// instance lock, since a save in progress would look the same.
    pub fn remove_stale_temps(&self) {
        remove_stale_temps(&self.file);
    }

    pub fn exists(&self) -> bool {
        self.file.exists()
    }

    fn read(&self) -> Result<Option<Snapshot>, StateError> {
        let bytes = match fs::read(&self.file) {
            Ok(bytes) => bytes,
            Err(source) if source.kind() == io::ErrorKind::NotFound => return Ok(None),
            Err(source) => return Err(self.io_error("read the state file", source)),
        };
        migrate(serde_json::from_slice(&bytes)?).map(Some)
    }

    fn io_error(&self, action: &'static str, source: io::Error) -> StateError {
        StateError::Io {
            action,
            path: self.file.clone(),
            source,
        }
    }

    fn set_aside(&self, now: &Now) {
        self.set_aside_with(now, |from, to| fs::rename(from, to));
    }

    fn set_aside_with(&self, now: &Now, rename: impl FnOnce(&Path, &Path) -> io::Result<()>) {
        let Some(target) = self.free_corrupt_name(now.utc().timestamp()) else {
            error!("no free name to set the unusable state file aside, keeping a single copy");
            self.copy_to_last_slot();
            return;
        };
        match rename(&self.file, &target) {
            Ok(()) => warn!(moved_to = %target.display(), "unusable state file set aside"),
            Err(error) => {
                error!(%error, "could not set the unusable state file aside, keeping a single copy");
                self.copy_to_last_slot();
            }
        }
    }

    fn copy_to_last_slot(&self) {
        let slot = self.file.with_file_name(LAST_CORRUPT_SLOT);
        let copied =
            fs::read(&self.file).and_then(|bytes| write_atomic(&slot, &bytes, FILE_MODE, None));
        match copied {
            Ok(()) => warn!(copied_to = %slot.display(), "unusable state file copied"),
            Err(error) => error!(%error, "could not keep a copy of the unusable state file"),
        }
    }

    fn free_corrupt_name(&self, unix: i64) -> Option<PathBuf> {
        let base = format!("{STATE_FILE}.corrupt-{unix}");
        (0..MAX_QUARANTINE_NAMES)
            .map(|attempt| match attempt {
                0 => base.clone(),
                n => format!("{base}-{n}"),
            })
            .map(|name| self.file.with_file_name(name))
            .find(|candidate| !candidate.exists())
    }
}

fn refuse_symlink(path: &Path) -> Result<(), StateError> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_symlink() => {
            Err(StateError::Symlink(path.to_path_buf()))
        }
        Ok(_) => Ok(()),
        Err(source) if source.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(source) => Err(StateError::Io {
            action: "inspect",
            path: path.to_path_buf(),
            source,
        }),
    }
}

/// Opened without following a final symlink, so the owner check and the permission change
/// both act on the directory that was inspected even if the path is swapped meanwhile.
fn open_directory(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_DIRECTORY)
        .open(path)
}

fn verify_owner(handle: &File, path: &Path, expected: u32) -> Result<(), StateError> {
    let found = handle
        .metadata()
        .map_err(|source| StateError::Io {
            action: "inspect the state directory",
            path: path.to_path_buf(),
            source,
        })?
        .uid();
    if found == expected {
        return Ok(());
    }
    Err(StateError::ForeignOwner {
        path: path.to_path_buf(),
        found,
        expected,
    })
}

impl StateStore for FileStateStore {
    fn load(&self, now: &Now) -> Result<Option<Snapshot>, StateError> {
        match self.read() {
            Ok(Some(state)) => {
                info!(
                    sites = state.rules.blocked_sites.len(),
                    apps = state.rules.blocked_apps.len(),
                    windows = state.rules.schedule.windows.len(),
                    limits = state.rules.limits.len(),
                    "state loaded"
                );
                Ok(Some(state))
            }
            Ok(None) => {
                info!("no saved state");
                Ok(None)
            }
            Err(error) if error.is_unusable_file() => {
                warn!(reason = %error.summary(), "saved state is unusable, setting it aside");
                debug!(%error, "saved state failure in full");
                self.set_aside(now);
                Ok(None)
            }
            Err(error) => {
                error!(reason = %error.summary(), "saved state cannot be used, leaving it in place");
                Err(error)
            }
        }
    }

    fn save(&self, state: &Snapshot) -> Result<(), StateError> {
        let bytes = serde_json::to_vec_pretty(&PersistedState::from(state))?;
        write_atomic(&self.file, &bytes, FILE_MODE, None)
            .map_err(|source| self.io_error("write the state file", source))
    }
}

#[cfg(test)]
mod tests;
