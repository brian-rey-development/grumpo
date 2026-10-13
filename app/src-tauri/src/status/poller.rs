use std::io;
use std::sync::Arc;
use std::time::Duration;

use focus_core::ipc::UnlockPhaseView;

use super::StatusHub;
use crate::concurrency::{StopSignal, StoppableThread};
use crate::dto::FocusStatus;

const THREAD_NAME: &str = "focus-status-poller";
/// Added to a phase deadline so the daemon, whose clock may be a fraction of a second ahead of
/// the whole-second deadline it reported, has already moved on when asked.
const DEADLINE_SLACK: Duration = Duration::from_millis(100);
/// A deadline already in the past (the daemon has not moved on yet) must not become a busy loop.
const MIN_WAKE: Duration = Duration::from_millis(250);

/// Keeps the hub fresh in the background. Dropping the handle stops the thread, which can
/// take as long as a daemon call already in flight, at most a couple of seconds.
#[derive(Debug)]
pub struct PollerHandle {
    _thread: StoppableThread,
}

impl PollerHandle {
    pub(super) fn spawn(hub: Arc<StatusHub>, interval: Duration) -> io::Result<Self> {
        let thread = StoppableThread::spawn_signalled(THREAD_NAME, move |stop| {
            poll(&hub, interval, stop);
        })?;
        Ok(Self { _thread: thread })
    }
}

fn poll(hub: &StatusHub, interval: Duration, stop: &StopSignal) {
    loop {
        let status = hub.refresh();
        if stop.wait(next_wake(&status, hub.now(), interval)) {
            return;
        }
    }
}

/// How long to sleep before the next poll: the full `interval`, cut short when the unlock
/// state changes phase sooner, so a cooldown that ends or an unlock that expires shows up
/// within a second instead of waiting for the next regular poll.
pub(super) fn next_wake(status: &FocusStatus, now: i64, interval: Duration) -> Duration {
    let Some(deadline) = status
        .daemon()
        .and_then(|daemon| phase_deadline(daemon.unlock.phase))
    else {
        return interval;
    };
    let seconds = u64::try_from(deadline.saturating_sub(now)).unwrap_or(0);
    (Duration::from_secs(seconds) + DEADLINE_SLACK)
        .max(MIN_WAKE)
        .min(interval)
}

fn phase_deadline(phase: UnlockPhaseView) -> Option<i64> {
    match phase {
        UnlockPhaseView::Locked => None,
        UnlockPhaseView::CoolingDown { ready_at, .. } => Some(ready_at),
        UnlockPhaseView::Ready { expires_at, .. } => Some(expires_at),
        UnlockPhaseView::Unlocked { until } => Some(until),
    }
}

#[cfg(test)]
mod tests;
