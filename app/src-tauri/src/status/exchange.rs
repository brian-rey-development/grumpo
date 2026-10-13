//! One request to the daemon and what it does to the hub's state: a retry for the failures
//! that are usually a blip, and a run of failures before a connected status is given up.

use std::thread;
use std::time::Duration;

use focus_core::ipc::{DaemonStatus, Request};

use super::unavailable::unavailable;
use super::{State, StatusHub};
use crate::concurrency::LockExt;
use crate::daemon_client::DaemonError;
use crate::dto::FocusStatus;

/// A busy daemon or one restarting can miss a single exchange, so a failed one is tried again
/// after this pause before it counts as a failure.
const RETRY_DELAY: Duration = Duration::from_millis(150);
/// A connected status survives one failed exchange, because the daemon answered a moment
/// before and one miss is far more often a blip than an outage.
const FAILURES_BEFORE_UNAVAILABLE: u8 = 2;

impl StatusHub {
    /// Returns the outcome and whether the stored status changed because of it.
    pub(super) fn exchange(&self, request: &Request) -> (Result<FocusStatus, DaemonError>, bool) {
        let _turn = self.exchange.lock_or_recover();
        match self.attempt(request) {
            Ok(status) => {
                let status = FocusStatus::Connected {
                    status: Box::new(status),
                };
                let changed = self.confirm(status.clone());
                (Ok(status), changed)
            }
            Err(error @ DaemonError::Rejected(_)) => {
                self.state.lock_or_recover().failures_in_a_row = 0;
                (Err(error), false)
            }
            Err(error) => {
                let changed = self.fail(&error);
                (Err(error), changed)
            }
        }
    }

    /// One exchange, tried a second time after a short pause when it failed in a way that
    /// can be a blip and repeating it cannot do harm.
    fn attempt(&self, request: &Request) -> Result<DaemonStatus, DaemonError> {
        match self.client.send(request) {
            Err(error) if is_repeatable(request) && is_transient(&error) => {
                tracing::debug!(%error, "daemon exchange failed, trying once more");
                thread::sleep(RETRY_DELAY);
                self.client.send(request)
            }
            result => result,
        }
    }

    fn confirm(&self, status: FocusStatus) -> bool {
        let mut state = self.state.lock_or_recover();
        state.failures_in_a_row = 0;
        state.confirmed_at = Some(self.clock.now_unix());
        replace_current(&mut state, status)
    }

    fn fail(&self, error: &DaemonError) -> bool {
        let mut state = self.state.lock_or_recover();
        state.failures_in_a_row = state.failures_in_a_row.saturating_add(1);
        tracing::debug!(%error, failures = state.failures_in_a_row, "daemon exchange failed");
        let connected = state.current.daemon().is_some();
        if connected && state.failures_in_a_row < FAILURES_BEFORE_UNAVAILABLE {
            return false;
        }
        replace_current(&mut state, unavailable(error))
    }
}

fn replace_current(state: &mut State, status: FocusStatus) -> bool {
    if state.current == status {
        return false;
    }
    log_transition(&state.current, &status);
    state.current = status;
    true
}

/// Requests that give the same result when they arrive twice. An unlock request or
/// confirmation would fail or change meaning the second time, so those are never repeated.
fn is_repeatable(request: &Request) -> bool {
    matches!(
        request,
        Request::Status
            | Request::UpdateRules { .. }
            | Request::Relock
            | Request::ReportLimitReached { .. }
    )
}

fn is_transient(error: &DaemonError) -> bool {
    matches!(
        error,
        DaemonError::NotInstalled | DaemonError::NotResponding(_)
    )
}

fn log_transition(from: &FocusStatus, to: &FocusStatus) {
    match (from, to) {
        (_, FocusStatus::Unavailable { reason, .. }) => {
            tracing::warn!(?reason, "the Grumpo service is unavailable");
        }
        (FocusStatus::Unavailable { .. }, FocusStatus::Connected { .. }) => {
            tracing::info!("connected to the Grumpo service");
        }
        (FocusStatus::Connected { .. }, FocusStatus::Connected { .. }) => {
            tracing::debug!("the Grumpo service's status changed");
        }
    }
}
