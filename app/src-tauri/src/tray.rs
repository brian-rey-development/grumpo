//! The menu bar item, which shows the state of focus and today's total.
//!
//! Every change is queued to the main thread and never waited for, and everything slow, the
//! database read and the countdown, happens on the tray worker. That is what keeps the app
//! able to exit: no thread the exit has to join ever waits on the main thread.

mod menu;
#[cfg(test)]
mod test_support;
mod text;
mod view;
mod worker;

use std::sync::{Arc, Mutex};
use std::time::Duration;

use tauri::AppHandle;

use crate::concurrency::LockExt;
use crate::dto::FocusStatus;
use crate::storage::Store;
use crate::time::{compact_duration, today, Clock, ClockFormatter, SystemClock};
use view::{MainThread, TauriMainThread, TrayView};
use worker::TrayWorker;

const TOTAL_REFRESH_INTERVAL: Duration = Duration::from_secs(30);

/// Everything the redraws need, without the worker, so the worker's callbacks can hold it
/// without holding the worker itself.
struct Core {
    main: Arc<dyn MainThread>,
    view: Arc<dyn TrayView>,
    store: Arc<Store>,
    clock: Arc<dyn Clock>,
    clock_text: ClockFormatter,
    status: Mutex<FocusStatus>,
}

impl Core {
    /// Queues the status line, the relock state and the quit label, and says when the line
    /// changes by itself.
    fn draw_status(&self) -> Option<Duration> {
        let status = self.status.lock_or_recover().clone();
        let line = text::status_line(&status, self.clock.now_unix(), &self.clock_text);
        let quit = text::quit_label(&status);
        let unlocked = text::is_unlocked(&status);
        let view = Arc::clone(&self.view);
        self.main.post(Box::new(move || {
            view.set_status_line(&line);
            view.set_relock_enabled(unlocked);
            view.set_quit_label(quit);
        }));
        text::redraw_after(&status)
    }

    fn draw_total(&self) {
        match self.store.total_seconds(today()) {
            Ok(seconds) => {
                let title = compact_duration(seconds);
                let view = Arc::clone(&self.view);
                self.main.post(Box::new(move || view.set_total(&title)));
            }
            Err(error) => tracing::warn!(%error, "could not total today's usage for the tray"),
        }
    }
}

fn start_worker(core: &Arc<Core>) -> std::io::Result<TrayWorker> {
    let for_total = Arc::clone(core);
    let for_line = Arc::clone(core);
    TrayWorker::start(
        TOTAL_REFRESH_INTERVAL,
        move || for_total.draw_total(),
        move || for_line.draw_status(),
    )
}

#[derive(Clone)]
pub struct TrayHandle {
    core: Arc<Core>,
    worker: Arc<TrayWorker>,
}

impl std::fmt::Debug for TrayHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TrayHandle").finish_non_exhaustive()
    }
}

impl TrayHandle {
    pub fn build(
        app: &AppHandle,
        store: Arc<Store>,
        status: &FocusStatus,
        clock_text: ClockFormatter,
    ) -> tauri::Result<Self> {
        let view = Arc::new(menu::build_view(app)?);
        let main = Arc::new(TauriMainThread(app.clone()));
        let clock = Arc::new(SystemClock);
        Ok(Self::assemble(
            main, view, store, clock, clock_text, status,
        )?)
    }

    fn assemble(
        main: Arc<dyn MainThread>,
        view: Arc<dyn TrayView>,
        store: Arc<Store>,
        clock: Arc<dyn Clock>,
        clock_text: ClockFormatter,
        status: &FocusStatus,
    ) -> std::io::Result<Self> {
        let core = Arc::new(Core {
            main,
            view,
            store,
            clock,
            clock_text,
            status: Mutex::new(status.clone()),
        });
        let worker = start_worker(&core)?;
        let handle = Self {
            core,
            worker: Arc::new(worker),
        };
        handle.on_status(status);
        handle.on_usage_changed();
        Ok(handle)
    }

    /// Never blocks: it records the status, queues the redraw and wakes the worker if the
    /// line needs to count down.
    pub fn on_status(&self, status: &FocusStatus) {
        *self.core.status.lock_or_recover() = status.clone();
        if let Some(after) = self.core.draw_status() {
            self.worker.redraw_line_in(after);
        }
    }

    /// Never blocks, since usage events arrive on the enforcement thread. The worker reads
    /// the total, at most once per interval and always after the last change.
    pub fn on_usage_changed(&self) {
        self.worker.total_changed();
    }

    /// Reads the total now. Blocks on the database, so call it from a blocking task.
    pub fn refresh_total(&self) {
        self.core.draw_total();
    }
}

#[cfg(test)]
mod tests;
