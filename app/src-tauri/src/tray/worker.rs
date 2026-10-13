//! The one background thread behind the menu bar: it redraws the day's total at the trailing
//! edge of a burst of usage events, and redraws a countdown while one is on screen. Callers
//! only set a flag and wake it, so no caller ever waits for the database or the main thread.

use std::io;
use std::sync::{Arc, Condvar, Mutex, MutexGuard};
use std::time::{Duration, Instant};

use crate::concurrency::{Coalescer, CondvarExt, LockExt, StoppableThread};

const THREAD_NAME: &str = "focus-tray";

/// What the thread should do next.
#[derive(Debug, PartialEq, Eq)]
enum Step {
    Total,
    Line,
    Wait(Option<Duration>),
    Stop,
}

/// The deadlines, free of threads and clocks so the schedule can be tested directly.
#[derive(Debug)]
struct Timers {
    total: Coalescer,
    line_due: Option<Instant>,
    closed: bool,
}

impl Timers {
    fn new(total_interval: Duration) -> Self {
        Self {
            total: Coalescer::new(total_interval),
            line_due: None,
            closed: false,
        }
    }

    fn step(&mut self, now: Instant) -> Step {
        if self.closed {
            return Step::Stop;
        }
        if self.total.take_due(now) {
            return Step::Total;
        }
        if self.line_due.is_some_and(|due| due <= now) {
            self.line_due = None;
            return Step::Line;
        }
        let until_line = self.line_due.map(|due| due.saturating_duration_since(now));
        let wait = [self.total.wait_hint(now), until_line]
            .into_iter()
            .flatten()
            .min();
        Step::Wait(wait)
    }
}

#[derive(Debug)]
struct Shared {
    timers: Mutex<Timers>,
    wake: Condvar,
}

impl Shared {
    fn lock(&self) -> MutexGuard<'_, Timers> {
        self.timers.lock_or_recover()
    }

    fn close(&self) {
        self.lock().closed = true;
        self.wake.notify_all();
    }

    fn park<'a>(
        &'a self,
        timers: MutexGuard<'a, Timers>,
        wait: Option<Duration>,
    ) -> MutexGuard<'a, Timers> {
        match wait {
            None => self.wake.wait_or_recover(timers),
            Some(limit) => self.wake.wait_timeout_or_recover(timers, limit).0,
        }
    }
}

pub struct TrayWorker {
    shared: Arc<Shared>,
    _thread: StoppableThread,
}

impl std::fmt::Debug for TrayWorker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayWorker").finish_non_exhaustive()
    }
}

impl TrayWorker {
    /// `on_total` redraws the total. `on_line` redraws the status line and answers when it
    /// wants to be drawn again, if ever.
    pub fn start(
        total_interval: Duration,
        on_total: impl Fn() + Send + 'static,
        on_line: impl Fn() -> Option<Duration> + Send + 'static,
    ) -> io::Result<Self> {
        let shared = Arc::new(Shared {
            timers: Mutex::new(Timers::new(total_interval)),
            wake: Condvar::new(),
        });
        let stopper = Arc::clone(&shared);
        let runner = Arc::clone(&shared);
        let thread = StoppableThread::spawn(
            THREAD_NAME,
            move || stopper.close(),
            move || run(&runner, &on_total, &on_line),
        )?;
        Ok(Self {
            shared,
            _thread: thread,
        })
    }

    /// The total changed. The first change draws at once and later ones at most once per
    /// interval, with the last one always drawn.
    pub fn total_changed(&self) {
        self.shared.lock().total.mark();
        self.shared.wake.notify_one();
    }

    pub fn redraw_line_in(&self, after: Duration) {
        self.shared.lock().line_due = Some(Instant::now() + after);
        self.shared.wake.notify_one();
    }
}

fn run(shared: &Shared, on_total: &impl Fn(), on_line: &impl Fn() -> Option<Duration>) {
    let mut timers = shared.lock();
    loop {
        match timers.step(Instant::now()) {
            Step::Stop => return,
            Step::Total => {
                drop(timers);
                on_total();
                timers = shared.lock();
            }
            Step::Line => {
                drop(timers);
                let again = on_line();
                timers = shared.lock();
                if let Some(after) = again {
                    timers.line_due = Some(Instant::now() + after);
                }
            }
            Step::Wait(wait) => timers = shared.park(timers, wait),
        }
    }
}

#[cfg(test)]
mod tests;
