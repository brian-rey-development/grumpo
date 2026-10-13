use std::sync::{Arc, Mutex};

use focus_core::ipc::UnlockPhaseView;

use crate::concurrency::LockExt;
use crate::dto::FocusStatus;
use crate::notify::{Notice, Notifier};
use crate::time::ClockFormatter;

const TITLE: &str = "Unlock ready";
const BODY_WITHOUT_TIME: &str = "Confirm in Grumpo before the time runs out.";

/// Tells the person when a cooldown ends, since they likely went to do something else while
/// they waited and the confirmation window is short.
pub struct UnlockReadyAlert {
    notifier: Arc<dyn Notifier>,
    clock_text: ClockFormatter,
    last_phase: Mutex<Option<UnlockPhaseView>>,
}

impl std::fmt::Debug for UnlockReadyAlert {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("UnlockReadyAlert").finish_non_exhaustive()
    }
}

impl UnlockReadyAlert {
    pub fn new(notifier: Arc<dyn Notifier>, clock_text: ClockFormatter) -> Self {
        Self {
            notifier,
            clock_text,
            last_phase: Mutex::new(None),
        }
    }

    /// Call with every status the hub delivers, in order. The last phase the daemon reported
    /// is remembered across unavailable stretches, so a cooldown that ends while the service
    /// is briefly out of reach still alerts once it is back.
    pub fn observe(&self, status: &FocusStatus) {
        let Some(phase) = status.daemon().map(|daemon| daemon.unlock.phase) else {
            return;
        };
        let before = self.last_phase.lock_or_recover().replace(phase);
        if let (
            Some(UnlockPhaseView::CoolingDown { .. }),
            UnlockPhaseView::Ready { expires_at, .. },
        ) = (before, phase)
        {
            self.notifier.notify(Notice {
                title: TITLE.to_owned(),
                body: self.body(expires_at),
            });
        }
    }

    fn body(&self, expires_at: i64) -> String {
        self.clock_text.clock(expires_at).map_or_else(
            || BODY_WITHOUT_TIME.to_owned(),
            |time| format!("Confirm by {time} in Grumpo."),
        )
    }
}

#[cfg(test)]
mod tests;
