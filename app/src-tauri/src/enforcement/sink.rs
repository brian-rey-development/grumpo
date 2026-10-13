use std::io;
use std::sync::Arc;
use std::time::{Duration, Instant};

use super::engine::Engine;
use super::mailbox::{Batch, Mailbox, Wake};
use super::{EnforcementDeps, UsageEvents};
use crate::concurrency::{Coalescer, StoppableThread};
use crate::time::{Calendar, Clock, SystemCalendar};
use crate::tracking::activity::Activity;
use crate::tracking::sampler::SampleSink;

const THREAD_NAME: &str = "focus-enforcement";
const USAGE_EVENT_INTERVAL: Duration = Duration::from_secs(1);

/// The sampler's door into enforcement. Both methods only post to a mailbox, so the
/// sampler never waits on osascript, SQLite or the daemon.
pub struct EngineSink {
    mailbox: Arc<Mailbox>,
    clock: Arc<dyn Clock>,
}

impl std::fmt::Debug for EngineSink {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("EngineSink").finish_non_exhaustive()
    }
}

/// Stops the worker when dropped, after the step it is in the middle of.
#[derive(Debug)]
pub struct EngineHandle {
    _thread: StoppableThread,
}

impl EngineSink {
    pub fn start(deps: EnforcementDeps) -> io::Result<(Arc<Self>, EngineHandle)> {
        Self::start_in(deps, Arc::new(SystemCalendar))
    }

    pub(super) fn start_in(
        deps: EnforcementDeps,
        calendar: Arc<dyn Calendar>,
    ) -> io::Result<(Arc<Self>, EngineHandle)> {
        let mailbox = Arc::new(Mailbox::default());
        let sink = Arc::new(Self {
            mailbox: Arc::clone(&mailbox),
            clock: Arc::clone(&deps.clock),
        });
        let events = Arc::clone(&deps.events);
        let engine = Engine::new(deps, calendar);
        let stopper = Arc::clone(&mailbox);
        let thread = StoppableThread::spawn(
            THREAD_NAME,
            move || stopper.close(),
            move || run(engine, &mailbox, events.as_ref()),
        )?;
        Ok((sink, EngineHandle { _thread: thread }))
    }
}

impl SampleSink for EngineSink {
    fn on_sample(&self, activity: Option<&Activity>, at: i64) {
        self.mailbox.post_sample(activity, at);
    }

    fn on_usage_written(&self) {
        self.mailbox.post_usage_written(self.clock.now_unix());
    }
}

fn run(mut engine: Engine, mailbox: &Mailbox, events: &dyn UsageEvents) {
    let mut announcements = Coalescer::new(USAGE_EVENT_INTERVAL);
    loop {
        match mailbox.wait(announcements.wait_hint(Instant::now())) {
            Wake::Closed => break,
            Wake::Idle => {}
            Wake::Batch(batch) => handle(&mut engine, &mut announcements, events, batch),
        }
        announce_if_due(&mut announcements, events);
    }
    tracing::debug!(dropped = mailbox.dropped(), "enforcement worker stopped");
}

fn handle(
    engine: &mut Engine,
    announcements: &mut Coalescer,
    events: &dyn UsageEvents,
    batch: Batch,
) {
    if let Some(at) = batch.usage_written_at {
        engine.usage_written(at);
        announcements.mark();
        announce_if_due(announcements, events);
    }
    if let Some(sample) = batch.sample {
        engine.sample(sample.activity.as_ref(), sample.at);
    }
}

fn announce_if_due(announcements: &mut Coalescer, events: &dyn UsageEvents) {
    if announcements.take_due(Instant::now()) {
        events.usage_changed();
    }
}

#[cfg(test)]
mod tests;
