use focus_core::{Domain, LimitTarget};
use rusqlite::{params, Connection};

use super::piece::saturating_u32;
use super::{StorageError, Store};
use crate::time::LocalDay;

const APP_SECONDS: &str = "SELECT COALESCE(SUM(MIN(ended_at, ?2) - MAX(started_at, ?1)), 0)
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2 AND app_id = ?3";

/// Matches the domain itself or any subdomain of it, the same rule as `Domain::covers`.
const SITE_SECONDS: &str = "SELECT COALESCE(SUM(MIN(ended_at, ?2) - MAX(started_at, ?1)), 0)
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2 AND domain IS NOT NULL
     AND (domain = ?3 OR substr(domain, -(length(?3) + 1)) = '.' || ?3)";

/// The same clipping as the day report, as one `SUM`, for callers that need only the total.
const TOTAL_SECONDS: &str = "SELECT COALESCE(SUM(MIN(ended_at, ?2) - MAX(started_at, ?1)), 0)
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2";

impl Store {
    /// All tracked seconds inside `day`, without loading a single session. The menu bar asks
    /// for this often, and the full report would read and bucket every row each time.
    pub fn total_seconds(&self, day: LocalDay) -> Result<u32, StorageError> {
        self.with_conn(|conn| {
            let seconds: i64 = conn
                .prepare_cached(TOTAL_SECONDS)?
                .query_row(params![day.start, day.end], |row| row.get(0))?;
            Ok(saturating_u32(seconds))
        })
    }

    /// Seconds spent on each target inside `range`, in the order of `targets`.
    pub fn usage_seconds(
        &self,
        range: LocalDay,
        targets: &[LimitTarget],
    ) -> Result<Vec<(LimitTarget, u32)>, StorageError> {
        self.with_conn(|conn| {
            targets
                .iter()
                .map(|target| Ok((target.clone(), seconds_for(conn, range, target)?)))
                .collect()
        })
    }
}

fn seconds_for(
    conn: &Connection,
    range: LocalDay,
    target: &LimitTarget,
) -> Result<u32, StorageError> {
    let (sql, key) = match target {
        LimitTarget::Site { domain } => (SITE_SECONDS, Domain::as_str(domain)),
        LimitTarget::App { app } => (APP_SECONDS, app.bundle_id()),
    };
    let seconds: i64 = conn
        .prepare_cached(sql)?
        .query_row(params![range.start, range.end, key], |row| row.get(0))?;
    Ok(saturating_u32(seconds))
}
