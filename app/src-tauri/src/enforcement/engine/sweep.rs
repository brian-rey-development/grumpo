//! The periodic look at the apps running behind the front window.

use focus_core::ipc::DaemonStatus;

use super::{Engine, Frame};
use crate::enforcement::policy::sweep;

/// How long a listing of the running apps stays fresh while blocking is active. An app that
/// was launched in the background after the last listing is found and quit within this long.
pub const BACKGROUND_SWEEP_INTERVAL_SECONDS: i64 = 30;

/// The listing of running apps that enforcement last acted on.
#[derive(Debug)]
pub(super) struct Swept {
    status: DaemonStatus,
    at: i64,
}

impl Swept {
    /// A clock that moved backwards counts as due, since the listing is then of unknown age.
    fn is_current(&self, status: &DaemonStatus, at: i64) -> bool {
        let age = at.saturating_sub(self.at);
        self.status == *status && (0..BACKGROUND_SWEEP_INTERVAL_SECONDS).contains(&age)
    }
}

impl Engine {
    pub(super) fn sweep_if_due(&mut self, frame: &Frame<'_>) {
        let Some(status) = frame.status.filter(|status| status.enforcement.active) else {
            self.swept = None;
            return;
        };
        if self
            .swept
            .as_ref()
            .is_some_and(|swept| swept.is_current(status, frame.at))
        {
            return;
        }
        let Some(running) = self.running_apps() else {
            return;
        };
        self.swept = Some(Swept {
            status: status.clone(),
            at: frame.at,
        });
        let actions = sweep(&self.input(frame, None), &running);
        self.run(&actions, frame.at);
    }

    fn running_apps(&self) -> Option<Vec<focus_core::AppRef>> {
        match self.control.running_apps() {
            Ok(running) => Some(running),
            Err(error) => {
                tracing::warn!(
                    kind = %error.kind(),
                    "could not list running apps, will look again"
                );
                tracing::debug!(%error, "running apps listing failure detail");
                None
            }
        }
    }
}
