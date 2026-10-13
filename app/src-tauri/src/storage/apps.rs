use focus_core::AppRef;

use super::{StorageError, Store};

/// One row per app, named as it was at its latest session. SQLite reads the bare
/// `app_name` from the row that holds the `MAX(started_at)`.
const RECENT_APPS: &str = "SELECT app_id, app_name, MAX(started_at) FROM sessions
     WHERE ended_at >= ?1 GROUP BY app_id";

impl Store {
    /// Apps with any usage that ended at or after `since`, in no particular order. A stored
    /// id that is no longer a valid bundle id is skipped instead of failing the list.
    pub fn apps_since(&self, since: i64) -> Result<Vec<AppRef>, StorageError> {
        self.with_conn(|conn| {
            let mut stmt = conn.prepare_cached(RECENT_APPS)?;
            let rows = stmt.query_map([since], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })?;
            let mut apps = Vec::new();
            for row in rows {
                let (bundle_id, name) = row?;
                match AppRef::new(bundle_id, name) {
                    Ok(app) => apps.push(app),
                    Err(error) => tracing::debug!(%error, "skipping a stored app that is invalid"),
                }
            }
            Ok(apps)
        })
    }
}

#[cfg(test)]
mod tests;
