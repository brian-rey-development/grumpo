//! Opening the database when it may be damaged. A file that is not a database is moved aside
//! and replaced, because the app can run without its history but not without a database, and
//! the old file stays on disk in case it can still be read.

use std::ffi::OsString;
use std::fs;
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};

use super::{StorageError, Store};

/// The sidecar files SQLite keeps beside a database in WAL mode, which belong to the database
/// they sit next to and must move with it.
const SIDECAR_SUFFIXES: [&str; 3] = ["", "-wal", "-shm"];
const QUARANTINE_MARKER: &str = ".corrupt-";
const MAX_QUARANTINE_STAMPS: u32 = 100;

#[derive(Debug)]
pub enum Opened {
    Ready(Store),
    /// The old file could not be used, so it was moved to `kept_as` and a new database made.
    Recovered {
        store: Store,
        kept_as: PathBuf,
    },
}

/// Opens the database at `path`, starting over if it is damaged. `now_unix` names the moved
/// files, and a counter is added when that name is taken, so that a second recovery never
/// overwrites the first. Failures that are not damage
/// are returned as they are, including a schema from a newer version of the app.
pub fn open_or_recover(path: &Path, now_unix: i64) -> Result<Opened, StorageError> {
    match Store::open(path) {
        Ok(store) => Ok(Opened::Ready(store)),
        Err(error) if error.is_damaged() => {
            tracing::error!(%error, "the database is damaged, starting a new one");
            let kept_as = quarantine(path, now_unix)?;
            let store = Store::open(path)?;
            Ok(Opened::Recovered { store, kept_as })
        }
        Err(error) => Err(error),
    }
}

fn quarantine(path: &Path, now_unix: i64) -> Result<PathBuf, StorageError> {
    let stamp = free_stamp(path, now_unix)?;
    for suffix in SIDECAR_SUFFIXES {
        let source = with_suffix(path, suffix);
        let target = with_suffix(&source, &stamp);
        match fs::rename(&source, &target) {
            Ok(()) => tracing::info!(file = %target.display(), "moved a damaged database file"),
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(StorageError::io("move the damaged database aside")(error)),
        }
    }
    Ok(with_suffix(path, &stamp))
}

/// The first stamp for which no moved file exists yet, shared by the database and its sidecars
/// so they stay together.
fn free_stamp(path: &Path, now_unix: i64) -> Result<String, StorageError> {
    let base = format!("{QUARANTINE_MARKER}{now_unix}");
    (0..MAX_QUARANTINE_STAMPS)
        .map(|attempt| match attempt {
            0 => base.clone(),
            n => format!("{base}-{n}"),
        })
        .find(|stamp| {
            SIDECAR_SUFFIXES
                .iter()
                .all(|suffix| !with_suffix(&with_suffix(path, suffix), stamp).exists())
        })
        .ok_or_else(|| {
            let exhausted = io::Error::new(ErrorKind::AlreadyExists, "every moved name is taken");
            StorageError::io("find a free name for the damaged database")(exhausted)
        })
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = OsString::from(path.as_os_str());
    name.push(suffix);
    PathBuf::from(name)
}

#[cfg(test)]
mod tests;
