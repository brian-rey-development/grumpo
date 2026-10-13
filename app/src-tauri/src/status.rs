//! The app's view of the daemon: the latest status, who is listening, and a poller that
//! keeps it fresh. Every daemon exchange goes through the hub, so state and exchanges stay
//! in the same order.

mod exchange;
mod poller;
mod ready_alert;
mod unavailable;

use std::io;
use std::panic::{self, AssertUnwindSafe};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use focus_core::ipc::Request;

use crate::concurrency::LockExt;
use crate::daemon_client::{DaemonClient, DaemonError};
use crate::dto::{DaemonUnavailableReason, FocusStatus};
use crate::time::{Clock, SystemClock};
pub use poller::PollerHandle;
pub use ready_alert::UnlockReadyAlert;

type Listener = Arc<dyn Fn(&FocusStatus) + Send + Sync>;

const NOT_CHECKED_YET: &str = "the Grumpo service has not been asked yet";

/// The latest daemon answer, how many exchanges in a row have failed since, and when the
/// status was last confirmed by the daemon (unix seconds).
#[derive(Debug)]
struct State {
    current: FocusStatus,
    failures_in_a_row: u8,
    confirmed_at: Option<i64>,
}

pub struct StatusHub {
    client: Arc<DaemonClient>,
    clock: Arc<dyn Clock>,
    /// Held across a daemon call and the store of its result, so a slow poll can never
    /// overwrite the answer to a newer mutation.
    exchange: Mutex<()>,
    state: Mutex<State>,
    /// Held while the newest status is read and handed to the listeners, so deliveries are
    /// ordered: a delivery that starts later always reads a status at least as new, and a stale
    /// one can never land last. Listeners must not call back into the hub's `refresh` or `send`.
    delivery: Mutex<()>,
    listeners: Mutex<Vec<Listener>>,
}

impl std::fmt::Debug for StatusHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("StatusHub").finish_non_exhaustive()
    }
}

impl StatusHub {
    pub fn new(client: Arc<DaemonClient>) -> Arc<Self> {
        Self::with_clock(client, Arc::new(SystemClock))
    }

    pub fn with_clock(client: Arc<DaemonClient>, clock: Arc<dyn Clock>) -> Arc<Self> {
        Arc::new(Self {
            client,
            clock,
            exchange: Mutex::new(()),
            state: Mutex::new(State {
                current: FocusStatus::Unavailable {
                    reason: DaemonUnavailableReason::NotResponding,
                    message: NOT_CHECKED_YET.to_owned(),
                },
                failures_in_a_row: 0,
                confirmed_at: None,
            }),
            delivery: Mutex::new(()),
            listeners: Mutex::new(Vec::new()),
        })
    }

    pub fn current(&self) -> FocusStatus {
        self.state.lock_or_recover().current.clone()
    }

    /// The status together with the moment the daemon last confirmed it, which tells a reader
    /// whether it may predate a local midnight.
    pub fn snapshot(&self) -> (FocusStatus, Option<i64>) {
        let state = self.state.lock_or_recover();
        (state.current.clone(), state.confirmed_at)
    }

    pub fn now(&self) -> i64 {
        self.clock.now_unix()
    }

    /// Asks the daemon for its status. Listeners hear about it only when it changed.
    pub fn refresh(&self) -> FocusStatus {
        let (outcome, changed) = self.exchange(&Request::Status);
        if changed {
            self.notify();
        }
        outcome.unwrap_or_else(|_| self.current())
    }

    /// Applies a mutation. A refused request leaves the status alone and refreshes it
    /// instead, because a refusal often means the app's picture of the daemon was stale.
    pub fn send(&self, request: &Request) -> Result<FocusStatus, DaemonError> {
        let (outcome, changed) = self.exchange(request);
        match outcome {
            Ok(status) => {
                self.notify();
                Ok(status)
            }
            Err(error @ DaemonError::Rejected(_)) => {
                self.refresh();
                Err(error)
            }
            Err(error) => {
                if changed {
                    self.notify();
                }
                Err(error)
            }
        }
    }

    pub fn on_change(&self, listener: Box<dyn Fn(&FocusStatus) + Send + Sync>) {
        self.listeners.lock_or_recover().push(Arc::from(listener));
    }

    /// Drops every listener, so a worker that is still finishing an exchange cannot reach
    /// the UI or the tray while the app is shutting down.
    pub fn clear_listeners(&self) {
        self.listeners.lock_or_recover().clear();
    }

    pub fn start_polling(self: &Arc<Self>, interval: Duration) -> io::Result<PollerHandle> {
        PollerHandle::spawn(Arc::clone(self), interval)
    }

    fn notify(&self) {
        let _ordered = self.delivery.lock_or_recover();
        let status = self.current();
        let listeners: Vec<Listener> = self.listeners.lock_or_recover().clone();
        for listener in &listeners {
            if panic::catch_unwind(AssertUnwindSafe(|| listener(&status))).is_err() {
                tracing::error!("a status listener panicked");
            }
        }
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
