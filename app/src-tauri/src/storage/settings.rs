use std::collections::BTreeMap;

use rusqlite::{params, OptionalExtension};

use super::{StorageError, Store};

impl Store {
    pub fn setting(&self, key: &str) -> Result<Option<String>, StorageError> {
        self.with_conn(|conn| {
            Ok(conn
                .query_row("SELECT value FROM settings WHERE key = ?1", [key], |row| {
                    row.get(0)
                })
                .optional()?)
        })
    }

    pub fn set_setting(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.with_conn(|conn| {
            conn.execute(
                "INSERT INTO settings (key, value) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET value = excluded.value",
                params![key, value],
            )?;
            Ok(())
        })
    }

    pub fn all_settings(&self) -> Result<BTreeMap<String, String>, StorageError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare("SELECT key, value FROM settings")?;
            let rows = stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))?;
            Ok(rows.collect::<Result<_, _>>()?)
        })
    }
}

#[cfg(test)]
mod tests;
