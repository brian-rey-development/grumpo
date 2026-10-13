//! The hand-off from the sampler thread to the worker. Posting never blocks on anything but
//! a briefly held lock, so the sampler's cadence cannot be stretched by slow enforcement.
//!
//! Only the newest sample matters, because policy judges the present, so the box holds one
//! sample and a newer one replaces an older one that the worker had not reached yet. That
//! is the bounded queue with drop-oldest overflow, counted and logged once.

use std::sync::{Condvar, Mutex, MutexGuard};
use std::time::Duration;

use crate::concurrency::{CondvarExt, LockExt};
use crate::tracking::activity::Activity;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleMessage {
    pub activity: Option<Activity>,
    pub at: i64,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Batch {
    pub sample: Option<SampleMessage>,
    pub usage_written_at: Option<i64>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Wake {
    Batch(Batch),
    /// The wait timed out with nothing posted.
    Idle,
    Closed,
}

#[derive(Debug, Default)]
struct State {
    batch: Batch,
    closed: bool,
    dropped: u64,
}

#[derive(Debug, Default)]
pub struct Mailbox {
    state: Mutex<State>,
    ready: Condvar,
}

impl Mailbox {
    pub fn post_sample(&self, activity: Option<&Activity>, at: i64) {
        let message = SampleMessage {
            activity: activity.cloned(),
            at,
        };
        let mut state = self.lock();
        if state.batch.sample.replace(message).is_some() {
            state.dropped += 1;
            if state.dropped == 1 {
                tracing::warn!("enforcement is behind, dropping samples it has not reached");
            }
        }
        drop(state);
        self.ready.notify_one();
    }

    pub fn post_usage_written(&self, at: i64) {
        self.lock().batch.usage_written_at = Some(at);
        self.ready.notify_one();
    }

    pub fn close(&self) {
        self.lock().closed = true;
        self.ready.notify_all();
    }

    pub fn dropped(&self) -> u64 {
        self.lock().dropped
    }

    /// Blocks until something is posted, the box is closed or `timeout` passes.
    pub fn wait(&self, timeout: Option<Duration>) -> Wake {
        let mut state = self.lock();
        loop {
            if state.closed {
                return Wake::Closed;
            }
            if state.batch != Batch::default() {
                return Wake::Batch(std::mem::take(&mut state.batch));
            }
            state = match timeout {
                None => self.ready.wait_or_recover(state),
                Some(limit) => {
                    let (guard, result) = self.ready.wait_timeout_or_recover(state, limit);
                    if result.timed_out() && guard.batch == Batch::default() && !guard.closed {
                        return Wake::Idle;
                    }
                    guard
                }
            };
        }
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        self.state.lock_or_recover()
    }
}

#[cfg(test)]
mod tests;
