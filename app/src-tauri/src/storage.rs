//! SQLite persistence for usage history. Only app id, app name, domain and timestamps are
//! stored, never URLs, paths, queries or titles.

mod apps;
mod erase;
mod error;
mod export;
mod migrations;
mod onboarding;
mod private_fs;
mod recovery;
pub mod reports;
pub mod retention;
pub mod sessions;
mod settings;
#[cfg(test)]
pub(crate) mod test_support;

use std::fmt;
use std::path::Path;
use std::sync::{Mutex, MutexGuard};
use std::time::Duration;

use rusqlite::Connection;

use crate::concurrency::LockExt;

pub use error::StorageError;
pub use recovery::{open_or_recover, Opened};

const BUSY_TIMEOUT: Duration = Duration::from_secs(2);
const INTEGRITY_OK: &str = "ok";

/// Owns the single connection. SQLite serializes writers anyway, so one mutex-guarded
/// connection is simpler than a pool and plenty for a 2 s sampler plus UI reads.
pub struct Store {
    conn: Mutex<Connection>,
}

impl fmt::Debug for Store {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Store").finish_non_exhaustive()
    }
}

impl Store {
    pub fn open(path: &Path) -> Result<Self, StorageError> {
        if let Some(dir) = path.parent().filter(|dir| !dir.as_os_str().is_empty()) {
            private_fs::create_dir_all(dir).map_err(StorageError::io("create data directory"))?;
        }
        private_fs::ensure_file(path).map_err(StorageError::io("create database file"))?;
        Self::configured(Connection::open(path)?, true)
    }

    #[cfg(test)]
    pub fn open_in_memory() -> Result<Self, StorageError> {
        Self::configured(Connection::open_in_memory()?, false)
    }

    fn configured(mut conn: Connection, persistent: bool) -> Result<Self, StorageError> {
        conn.busy_timeout(BUSY_TIMEOUT)?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "synchronous", "NORMAL")?;
        conn.pragma_update(None, "secure_delete", true)?;
        if persistent {
            enable_wal(&conn)?;
            check_integrity(&conn)?;
        }
        migrations::migrate(&mut conn)?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub(crate) fn with_conn<T>(
        &self,
        work: impl FnOnce(&mut Connection) -> Result<T, StorageError>,
    ) -> Result<T, StorageError> {
        work(&mut self.lock())
    }

    /// A panic while holding the lock cannot leave the database inconsistent, because an
    /// open transaction rolls back on drop, so the guard is recovered instead of bricking
    /// storage for the rest of the process.
    fn lock(&self) -> MutexGuard<'_, Connection> {
        self.conn.lock_or_recover()
    }
}

/// The quick check reads every page once, which costs a few milliseconds for the size of
/// history the app keeps, and finds damage that would otherwise surface as a failure on
/// every later query.
fn check_integrity(conn: &Connection) -> Result<(), StorageError> {
    let verdict: String = conn.pragma_query_value(None, "quick_check", |row| row.get(0))?;
    if verdict == INTEGRITY_OK {
        return Ok(());
    }
    Err(StorageError::Corrupt(verdict))
}

fn enable_wal(conn: &Connection) -> Result<(), StorageError> {
    let mode: String =
        conn.pragma_update_and_check(None, "journal_mode", "WAL", |row| row.get(0))?;
    if !mode.eq_ignore_ascii_case("wal") {
        tracing::warn!(mode, "database is not using WAL journaling");
    }
    Ok(())
}

#[cfg(test)]
mod tests;
