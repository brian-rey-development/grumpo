//! Makes sure the fallback of a show runs exactly once, whether the attempt fails on the main
//! thread or never gets queued at all.

use std::fmt::Display;
use std::sync::{Arc, Mutex};

use super::Fallback;
use crate::concurrency::LockExt;

type Slot = Arc<Mutex<Option<Fallback>>>;

/// Queues `attempt` through `queue`. The fallback runs if the attempt fails, and also if
/// `queue` refuses the work, since the closure it was handed is then dropped unrun.
pub fn queue_with_fallback<QueueError: Display, AttemptError: Display>(
    queue: impl FnOnce(Box<dyn FnOnce() + Send>) -> Result<(), QueueError>,
    attempt: impl FnOnce() -> Result<(), AttemptError> + Send + 'static,
    on_failure: Fallback,
) {
    let slot: Slot = Arc::new(Mutex::new(Some(on_failure)));
    let queued = Arc::clone(&slot);
    let job = move || {
        if let Err(error) = attempt() {
            tracing::warn!(%error, "could not show the block screen");
            run(&queued);
        }
    };
    if let Err(error) = queue(Box::new(job)) {
        tracing::warn!(%error, "could not queue the block screen");
        run(&slot);
    }
}

fn run(slot: &Slot) {
    let fallback = slot.lock_or_recover().take();
    if let Some(fallback) = fallback {
        fallback();
    }
}

#[cfg(test)]
mod tests;
