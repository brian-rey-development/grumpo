use std::fs::{self, DirBuilder, Permissions};
use std::io;
use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
use std::path::{Path, PathBuf};

use focus_core::Now;
use tracing::{debug, error, info, warn};

use super::migrate::migrate;
use super::persisted::PersistedState;
use super::{Snapshot, StateError, StateStore};
use crate::atomic_write::write_atomic;

const DIR_MODE: u32 = 0o700;
const FILE_MODE: u32 = 0o600;
const STATE_FILE: &str = "state.json";
const MAX_QUARANTINE_NAMES: u32 = 100;

#[derive(Debug)]
pub struct FileStateStore {
    file: PathBuf,
}

impl FileStateStore {
    /// Prepares `directory` as a private place for the state file.
    pub fn open(directory: &Path) -> Result<Self, StateError> {
        let io_error = |action, source| StateError::Io {
            action,
            path: directory.to_path_buf(),
            source,
        };
        DirBuilder::new()
            .recursive(true)
            .mode(DIR_MODE)
            .create(directory)
            .map_err(|source| io_error("create the state directory", source))?;
        fs::set_permissions(directory, Permissions::from_mode(DIR_MODE))
            .map_err(|source| io_error("restrict the state directory", source))?;
        Ok(Self {
            file: directory.join(STATE_FILE),
        })
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
        let Some(target) = self.free_corrupt_name(now.utc().timestamp()) else {
            error!("no free name to set the unusable state file aside, it will be overwritten");
            return;
        };
        match fs::rename(&self.file, &target) {
            Ok(()) => warn!(moved_to = %target.display(), "unusable state file set aside"),
            Err(error) => error!(%error, "could not set the unusable state file aside"),
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

impl StateStore for FileStateStore {
    fn load(&self, now: &Now) -> Result<Snapshot, StateError> {
        match self.read() {
            Ok(Some(state)) => {
                info!(
                    sites = state.rules.blocked_sites.len(),
                    apps = state.rules.blocked_apps.len(),
                    windows = state.rules.schedule.windows.len(),
                    limits = state.rules.limits.len(),
                    "state loaded"
                );
                Ok(state)
            }
            Ok(None) => {
                info!("no saved state, starting from defaults");
                Ok(Snapshot::fresh(now))
            }
            Err(error) if error.is_unusable_file() => {
                warn!(reason = %error.summary(), "saved state is unusable, starting from defaults");
                debug!(%error, "saved state failure in full");
                self.set_aside(now);
                Ok(Snapshot::fresh(now))
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
