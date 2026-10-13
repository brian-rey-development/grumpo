//! The enforcement loop. The sampler hands every sample to [`EngineSink`], a worker thread
//! combines it with today's usage and the daemon's status, and the pure policy in `policy`
//! decides what to do about it.

mod action;
mod engine;
mod executor;
mod mailbox;
mod memory;
mod notice;
mod policy;
mod sink;
mod usage;

#[cfg(test)]
mod test_support;

use std::sync::Arc;

use crate::block_screen::BlockScreenHub;
use crate::notify::Notifier;
use crate::platform::{AppControl, HourCycleSource};
use crate::status::StatusHub;
use crate::storage::Store;
use crate::time::Clock;

pub use sink::{EngineHandle, EngineSink};

/// Told when stored usage changed, so the UI can refetch. Calls come from the worker
/// thread, at most once per second.
pub trait UsageEvents: Send + Sync {
    fn usage_changed(&self);
}

pub struct EnforcementDeps {
    pub control: Arc<dyn AppControl>,
    pub store: Arc<Store>,
    pub status: Arc<StatusHub>,
    pub notifier: Arc<dyn Notifier>,
    pub block_screens: Arc<BlockScreenHub>,
    pub events: Arc<dyn UsageEvents>,
    pub clock: Arc<dyn Clock>,
    pub hour_cycle: Arc<dyn HourCycleSource>,
}
