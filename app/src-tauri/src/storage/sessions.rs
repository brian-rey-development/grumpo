use focus_core::{AppRef, Domain};
use rusqlite::params;

use super::{StorageError, Store};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SessionId(i64);

impl SessionId {
    #[cfg(test)]
    pub fn get(self) -> i64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NewSession {
    pub app: AppRef,
    pub domain: Option<Domain>,
    pub started_at: i64,
}

const INSERT: &str = "INSERT INTO sessions (started_at, ended_at, app_id, app_name, domain)
                      VALUES (?1, ?1, ?2, ?3, ?4)";
const EXTEND: &str = "UPDATE sessions SET ended_at = MAX(ended_at, ?2) WHERE id = ?1";
const END: &str = "UPDATE sessions SET ended_at = MAX(started_at, ?2) WHERE id = ?1";

impl Store {
    /// Starts a zero-length session at `started_at`, to be grown with [`Store::extend_session`].
    pub fn open_session(&self, session: &NewSession) -> Result<SessionId, StorageError> {
        self.with_conn(|conn| {
            conn.prepare_cached(INSERT)?.execute(params![
                session.started_at,
                session.app.bundle_id(),
                session.app.name(),
                session.domain.as_ref().map(Domain::as_str),
            ])?;
            Ok(SessionId(conn.last_insert_rowid()))
        })
    }

    /// Moves the end forward. An earlier `ended_at` is ignored so a late write never shrinks
    /// a session.
    pub fn extend_session(&self, id: SessionId, ended_at: i64) -> Result<(), StorageError> {
        self.update_end(EXTEND, id, ended_at)
    }

    /// Sets the end exactly, even backwards, but never before the start. Used when the user
    /// went idle and the time since their last input must not be credited.
    pub fn end_session(&self, id: SessionId, ended_at: i64) -> Result<(), StorageError> {
        self.update_end(END, id, ended_at)
    }

    fn update_end(&self, sql: &str, id: SessionId, ended_at: i64) -> Result<(), StorageError> {
        let changed = self
            .with_conn(|conn| Ok(conn.prepare_cached(sql)?.execute(params![id.0, ended_at])?))?;
        if changed == 0 {
            return Err(StorageError::SessionNotFound(id.0));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests;
