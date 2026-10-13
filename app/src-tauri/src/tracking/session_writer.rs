use std::sync::Arc;

use super::session_builder::SessionEvent;
use crate::storage::sessions::{NewSession, SessionId};
use crate::storage::{StorageError, Store};

/// Longest a pending end time waits in memory before it is written, which bounds how much
/// usage a crash can lose and keeps the 2 s sampler from writing every tick.
pub const FLUSH_INTERVAL_SECONDS: i64 = 10;

#[derive(Debug)]
struct OpenRow {
    id: SessionId,
    /// What the row records. Kept so the row can be created again if usage is deleted while
    /// the session is still running, with `started_at` moved to the time of the new row.
    session: NewSession,
    written_end: i64,
    pending_end: i64,
}

#[derive(Debug, Clone, Copy)]
enum IfDeleted {
    /// The session keeps recording, so a new row replaces the deleted one.
    Reinsert,
    /// The session is ending anyway, so nothing is written for it.
    Forget,
}

/// Applies builder events to the store, batching end-time updates.
#[derive(Debug)]
pub struct SessionWriter {
    store: Arc<Store>,
    open: Option<OpenRow>,
    last_flush_at: i64,
}

impl SessionWriter {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            open: None,
            last_flush_at: i64::MIN,
        }
    }

    /// Returns true when the store changed, so the caller can announce fresh usage.
    pub fn apply(&mut self, event: SessionEvent, now: i64) -> bool {
        match event {
            SessionEvent::Opened(session) => self.open_row(session, now),
            SessionEvent::Extended { ended_at } => self.extend(ended_at, now),
            SessionEvent::Trimmed { ended_at } => self.trim(ended_at),
            SessionEvent::Closed => self.close(now),
        }
    }

    pub fn flush(&mut self, now: i64) -> bool {
        self.flush_with(now, IfDeleted::Reinsert)
    }

    fn flush_with(&mut self, now: i64, if_deleted: IfDeleted) -> bool {
        let Some(row) = self.open.as_mut().filter(|r| r.pending_end > r.written_end) else {
            return false;
        };
        match self.store.extend_session(row.id, row.pending_end) {
            Ok(()) => {
                row.written_end = row.pending_end;
                self.last_flush_at = now;
                true
            }
            Err(StorageError::SessionNotFound(_)) => self.recover(now, if_deleted),
            Err(error) => {
                tracing::error!(%error, "failed to extend a session, will retry");
                false
            }
        }
    }

    fn recover(&mut self, now: i64, if_deleted: IfDeleted) -> bool {
        tracing::warn!("open session vanished, usage was deleted while tracking");
        let Some(row) = self.open.take() else {
            return false;
        };
        match if_deleted {
            IfDeleted::Forget => false,
            IfDeleted::Reinsert => self.insert(
                NewSession {
                    started_at: now.max(row.pending_end),
                    ..row.session
                },
                now,
            ),
        }
    }

    fn open_row(&mut self, session: NewSession, now: i64) -> bool {
        let flushed = self.close(now);
        self.insert(session, now) || flushed
    }

    fn insert(&mut self, session: NewSession, now: i64) -> bool {
        match self.store.open_session(&session) {
            Ok(id) => {
                self.open = Some(OpenRow {
                    id,
                    written_end: session.started_at,
                    pending_end: session.started_at,
                    session,
                });
                self.last_flush_at = now;
                true
            }
            Err(error) => {
                tracing::error!(%error, "failed to open a session, usage for it is lost");
                false
            }
        }
    }

    fn extend(&mut self, ended_at: i64, now: i64) -> bool {
        let Some(row) = self.open.as_mut() else {
            return false;
        };
        row.pending_end = row.pending_end.max(ended_at);
        let due = now.saturating_sub(self.last_flush_at) >= FLUSH_INTERVAL_SECONDS;
        due && self.flush(now)
    }

    fn trim(&mut self, ended_at: i64) -> bool {
        let Some(row) = self.open.as_mut() else {
            return false;
        };
        match self.store.end_session(row.id, ended_at) {
            Ok(()) => {
                let end = ended_at.max(row.session.started_at);
                row.written_end = end;
                row.pending_end = end;
                true
            }
            Err(StorageError::SessionNotFound(_)) => {
                self.open = None;
                false
            }
            Err(error) => {
                tracing::error!(%error, "failed to trim a session after idle");
                false
            }
        }
    }

    fn close(&mut self, now: i64) -> bool {
        let flushed = self.flush_with(now, IfDeleted::Forget);
        self.open = None;
        flushed
    }
}

#[cfg(test)]
mod tests;
