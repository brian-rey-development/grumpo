use focus_core::{hosts, LimitTarget};
use rusqlite::types::Value;
use rusqlite::{params, params_from_iter, Connection};

use super::piece::saturating_u32;
use super::{StorageError, Store};
use crate::time::LocalDay;

const APP_SECONDS: &str = "SELECT COALESCE(SUM(MIN(ended_at, ?2) - MAX(started_at, ?1)), 0)
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2 AND app_id = ?3";

const SITE_SECONDS: &str = "SELECT COALESCE(SUM(MIN(ended_at, ?2) - MAX(started_at, ?1)), 0)
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2 AND domain IN";

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
    let (sql, keys) = match target {
        LimitTarget::Site { domain } => {
            let hosts = hosts::hosts_of(domain);
            (site_seconds_sql(hosts.len()), hosts)
        }
        LimitTarget::App { app } => (APP_SECONDS.to_owned(), vec![app.bundle_id().to_owned()]),
    };
    let values = [Value::from(range.start), Value::from(range.end)]
        .into_iter()
        .chain(keys.into_iter().map(Value::from));
    let seconds: i64 = conn
        .prepare_cached(&sql)?
        .query_row(params_from_iter(values), |row| row.get(0))?;
    Ok(saturating_u32(seconds))
}

/// A site's time is the time on exactly the hosts its block covers, so a limit never counts a
/// subdomain it would leave open once spent.
fn site_seconds_sql(hosts: usize) -> String {
    let placeholders: Vec<String> = (0..hosts).map(|index| format!("?{}", index + 3)).collect();
    format!("{SITE_SECONDS} ({})", placeholders.join(", "))
}
