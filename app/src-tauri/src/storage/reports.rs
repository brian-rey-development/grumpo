//! Read side of the store. Queries load the sessions that overlap a range, and pure
//! functions clip, bucket, merge and rank them, so the aggregation logic is testable without
//! a database.

mod hourly;
mod piece;
mod timeline;
mod top;
mod types;
mod usage;

use rusqlite::Connection;

use super::{StorageError, Store};
use crate::time::units::DAYS_PER_WEEK;
use crate::time::LocalDay;
use piece::{clip_all, total_seconds, Piece};

pub use types::*;

const OVERLAPPING_SESSIONS: &str = "SELECT started_at, ended_at, app_id, app_name, domain
     FROM sessions WHERE ended_at > ?1 AND started_at < ?2 ORDER BY started_at, id";

impl Store {
    pub fn day_usage(&self, day: LocalDay) -> Result<DayUsage, StorageError> {
        let rows = self.with_conn(|conn| load_overlapping(conn, day.start, day.end))?;
        let pieces = clip_all(&rows, day.start, day.end);
        Ok(DayUsage {
            total_seconds: total_seconds(&pieces),
            hourly: hourly::hourly_buckets(&pieces, &day),
            timeline: timeline::build_timeline(&pieces, day.start, day.end),
            top_apps: top::top_apps(&pieces),
            top_sites: top::top_sites(&pieces),
        })
    }

    pub fn week_usage(&self, days: &[LocalDay; DAYS_PER_WEEK]) -> Result<WeekUsage, StorageError> {
        let first = days.iter().map(|day| day.start).min().unwrap_or_default();
        let last = days.iter().map(|day| day.end).max().unwrap_or_default();
        let rows = self.with_conn(|conn| load_overlapping(conn, first, last))?;
        let per_day: Vec<Vec<Piece>> = days
            .iter()
            .map(|day| clip_all(&rows, day.start, day.end))
            .collect();
        let all: Vec<Piece> = per_day.iter().flatten().cloned().collect();
        Ok(WeekUsage {
            days: days
                .iter()
                .zip(&per_day)
                .map(|(day, pieces)| DayTotal {
                    date: day.date,
                    total_seconds: total_seconds(pieces),
                })
                .collect(),
            total_seconds: total_seconds(&all),
            top_apps: top::top_apps(&all),
            top_sites: top::top_sites(&all),
        })
    }
}

fn load_overlapping(conn: &Connection, start: i64, end: i64) -> Result<Vec<Piece>, StorageError> {
    let mut stmt = conn.prepare_cached(OVERLAPPING_SESSIONS)?;
    let rows = stmt.query_map([start, end], |row| {
        Ok(Piece {
            start: row.get(0)?,
            end: row.get(1)?,
            app_id: row.get(2)?,
            app_name: row.get(3)?,
            domain: row.get(4)?,
        })
    })?;
    Ok(rows.collect::<Result<_, _>>()?)
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
