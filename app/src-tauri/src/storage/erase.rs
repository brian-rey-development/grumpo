use rusqlite::Transaction;

use super::{StorageError, Store};

const DELETE_ALL_SESSIONS: &str = "DELETE FROM sessions";
const DELETE_BEFORE: &str = "DELETE FROM sessions WHERE ended_at <= ?1";
const CLIP_TO: &str = "UPDATE sessions SET started_at = ?1 WHERE started_at < ?1";

impl Store {
    /// Removes usage history, then compacts the file and truncates the WAL so deleted rows
    /// do not linger on disk. Settings, which hold no usage, are kept.
    ///
    /// Everything before `today_start` always goes. Today's sessions go too, unless
    /// `keep_today` is true, which the caller sets while daily limits exist: today's usage is
    /// what those limits count, so erasing it would hand back the time already spent.
    /// A session that crosses `today_start` is cut so only its part after it remains.
    pub fn delete_all(&self, today_start: i64, keep_today: bool) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            let tx = conn.transaction()?;
            if keep_today {
                erase_before(&tx, today_start)?;
            } else {
                tx.execute(DELETE_ALL_SESSIONS, [])?;
            }
            tx.commit()?;
            conn.execute_batch("VACUUM;")?;
            conn.pragma_update_and_check(None, "wal_checkpoint", "TRUNCATE", |_| Ok(()))?;
            Ok(())
        })
    }
}

fn erase_before(tx: &Transaction<'_>, today_start: i64) -> Result<(), StorageError> {
    tx.execute(DELETE_BEFORE, [today_start])?;
    tx.execute(CLIP_TO, [today_start])?;
    Ok(())
}

#[cfg(test)]
mod tests;
