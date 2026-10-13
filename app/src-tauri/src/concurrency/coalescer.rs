use std::time::{Duration, Instant};

/// Lets the first event through at once and then at most one per interval, remembering one
/// that arrived too early so the last change is always announced.
#[derive(Debug)]
pub struct Coalescer {
    interval: Duration,
    last_emit: Option<Instant>,
    pending: bool,
}

impl Coalescer {
    pub fn new(interval: Duration) -> Self {
        Self {
            interval,
            last_emit: None,
            pending: false,
        }
    }

    pub fn mark(&mut self) {
        self.pending = true;
    }

    /// True once when a marked event may be emitted now.
    pub fn take_due(&mut self, now: Instant) -> bool {
        if !self.pending || !self.remaining(now).is_zero() {
            return false;
        }
        self.pending = false;
        self.last_emit = Some(now);
        true
    }

    /// How long until a pending event may go out, or `None` when nothing is pending.
    pub fn wait_hint(&self, now: Instant) -> Option<Duration> {
        self.pending.then(|| self.remaining(now))
    }

    fn remaining(&self, now: Instant) -> Duration {
        self.last_emit.map_or(Duration::ZERO, |last| {
            (last + self.interval).saturating_duration_since(now)
        })
    }
}

#[cfg(test)]
mod tests;
