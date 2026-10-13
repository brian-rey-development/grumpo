use rusqlite::Connection;

use super::StorageError;

const V1: &str = "
CREATE TABLE sessions (
    id INTEGER PRIMARY KEY,
    started_at INTEGER NOT NULL,
    ended_at INTEGER NOT NULL,
    app_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    domain TEXT,
    CHECK (ended_at >= started_at)
);
CREATE INDEX idx_sessions_started_at ON sessions (started_at);
CREATE INDEX idx_sessions_ended_at ON sessions (ended_at);
CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);
";

/// Append-only. Entry `n` upgrades the schema from version `n` to `n + 1`.
const MIGRATIONS: [&str; 1] = [V1];

pub fn latest_version() -> u32 {
    u32::try_from(MIGRATIONS.len()).unwrap_or(u32::MAX)
}

pub fn current_version(conn: &Connection) -> Result<u32, StorageError> {
    Ok(conn.pragma_query_value(None, "user_version", |row| row.get(0))?)
}

pub fn migrate(conn: &mut Connection) -> Result<(), StorageError> {
    let current = current_version(conn)?;
    let latest = latest_version();
    if current > latest {
        return Err(StorageError::SchemaTooNew {
            found: current,
            supported: latest,
        });
    }
    let pending = MIGRATIONS.iter().zip(1u32..).filter(|(_, v)| *v > current);
    for (sql, version) in pending {
        apply(conn, sql, version)?;
        tracing::info!(version, "database migrated");
    }
    Ok(())
}

fn apply(conn: &mut Connection, sql: &str, version: u32) -> Result<(), StorageError> {
    let tx = conn.transaction()?;
    tx.execute_batch(sql)?;
    tx.pragma_update(None, "user_version", version)?;
    tx.commit()?;
    Ok(())
}

#[cfg(test)]
mod tests;
