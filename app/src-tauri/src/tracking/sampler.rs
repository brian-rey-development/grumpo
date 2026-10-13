use std::io;
use std::sync::Arc;
use std::time::Instant;

use super::activity::Activity;
use super::observer::Observer;
use super::pruner::Pruner;
use super::session_builder::SessionBuilder;
use super::session_writer::SessionWriter;
pub use super::timing::SAMPLE_INTERVAL;
use crate::concurrency::{StopSignal, StoppableThread};
use crate::storage::Store;
use crate::time::Clock;

const THREAD_NAME: &str = "focus-sampler";

/// Receives every sample so enforcement and UI events attach without touching the sampler.
///
/// Calls run on the sampler thread between two ticks, so an implementation must not block:
/// a slow sink stretches the gap between samples and can split sessions. Hand the work to
/// another thread and return.
pub trait SampleSink: Send + Sync {
    fn on_sample(&self, activity: Option<&Activity>, at: i64);
    /// Called after a tick that changed stored usage. Extends are batched, so stored usage
    /// can trail the live session by up to the flush interval.
    fn on_usage_written(&self);
}

pub struct Sampler {
    observer: Observer,
    builder: SessionBuilder,
    writer: SessionWriter,
    pruner: Pruner,
    clock: Arc<dyn Clock>,
    sink: Arc<dyn SampleSink>,
}

impl std::fmt::Debug for Sampler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Sampler").finish_non_exhaustive()
    }
}

impl Sampler {
    pub fn new(
        store: Arc<Store>,
        observer: Observer,
        clock: Arc<dyn Clock>,
        sink: Arc<dyn SampleSink>,
    ) -> Self {
        Self {
            observer,
            builder: SessionBuilder::new(),
            writer: SessionWriter::new(Arc::clone(&store)),
            pruner: Pruner::new(store),
            clock,
            sink,
        }
    }

    /// One observe, build, store, notify cycle.
    pub fn tick(&mut self) {
        let now = self.clock.now_unix();
        self.pruner.run_if_due(now);
        let activity = self.observer.sample(now);
        let mut wrote = false;
        for event in self.builder.feed(activity.as_ref(), now) {
            wrote |= self.writer.apply(event, now);
        }
        if wrote {
            self.sink.on_usage_written();
        }
        self.sink.on_sample(activity.as_ref(), now);
    }

    /// Writes any batched end time. Called on shutdown, and safe to call at any time.
    pub fn finish(&mut self) {
        if self.writer.flush(self.clock.now_unix()) {
            self.sink.on_usage_written();
        }
    }

    pub fn start(self) -> io::Result<SamplerHandle> {
        let thread = StoppableThread::spawn_signalled(THREAD_NAME, move |stop| self.run(stop))?;
        Ok(SamplerHandle(thread))
    }

    fn run(mut self, stop: &StopSignal) {
        let mut deadline = Instant::now();
        loop {
            self.tick();
            deadline = next_deadline(deadline, Instant::now());
            if stop.wait(deadline.saturating_duration_since(Instant::now())) {
                break;
            }
        }
        self.finish();
    }
}

/// The tick after `previous` is due one interval later, so the cadence does not drift by
/// the time a tick takes. Ticks that ran past their slot are skipped rather than replayed,
/// which would hammer the probes right after a slow read.
fn next_deadline(previous: Instant, now: Instant) -> Instant {
    let mut next = previous + SAMPLE_INTERVAL;
    while next <= now {
        next += SAMPLE_INTERVAL;
    }
    next
}

/// Stops the sampler thread. Dropping the handle stops it too, after flushing.
#[derive(Debug)]
pub struct SamplerHandle(StoppableThread);

impl SamplerHandle {
    pub fn stop(mut self) {
        self.0.stop();
    }
}

#[cfg(test)]
mod session_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
