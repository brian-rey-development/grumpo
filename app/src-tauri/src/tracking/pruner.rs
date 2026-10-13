use std::sync::Arc;

use crate::storage::Store;
use crate::time::units;

pub const PRUNE_INTERVAL_SECONDS: i64 = units::SECONDS_PER_DAY;
pub const PRUNE_RETRY_SECONDS: i64 = units::SECONDS_PER_HOUR;

/// Runs retention on the first call and then once a day.
#[derive(Debug)]
pub struct Pruner {
    store: Arc<Store>,
    next_at: i64,
}

impl Pruner {
    pub fn new(store: Arc<Store>) -> Self {
        Self {
            store,
            next_at: i64::MIN,
        }
    }

    pub fn run_if_due(&mut self, now: i64) {
        if now < self.next_at {
            return;
        }
        self.next_at = match self.store.prune(now) {
            Ok(deleted) => {
                tracing::info!(deleted, "retention prune finished");
                now.saturating_add(PRUNE_INTERVAL_SECONDS)
            }
            Err(error) => {
                tracing::error!(%error, "retention prune failed, will retry");
                now.saturating_add(PRUNE_RETRY_SECONDS)
            }
        };
    }
}

#[cfg(test)]
mod tests;
