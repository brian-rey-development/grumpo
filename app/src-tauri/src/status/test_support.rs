use super::StatusHub;
use crate::daemon_client::test_support::{daemon_status, FakeDaemon, Reply};
use crate::dto::{DaemonUnavailableReason, FocusStatus};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use std::time::Duration;

pub const POLL: Duration = Duration::from_millis(15);

pub fn hub_for(daemon: &FakeDaemon) -> Arc<StatusHub> {
    StatusHub::new(Arc::new(daemon.client()))
}

pub fn counting(hub: &StatusHub) -> Arc<AtomicUsize> {
    let heard = Arc::new(AtomicUsize::new(0));
    let counter = Arc::clone(&heard);
    hub.on_change(Box::new(move |_| {
        counter.fetch_add(1, Ordering::SeqCst);
    }));
    heard
}

pub fn reason(status: &FocusStatus) -> DaemonUnavailableReason {
    match status {
        FocusStatus::Unavailable { reason, .. } => *reason,
        FocusStatus::Connected { .. } => panic!("expected an unavailable status"),
    }
}

/// Makes the next `exchanges` connections end without an answer.
pub fn hang_up(daemon: &FakeDaemon, exchanges: usize) {
    for _ in 0..exchanges {
        daemon.push(Reply::HangUp);
    }
}

pub fn changed_status() -> focus_core::ipc::DaemonStatus {
    let mut status = daemon_status();
    status.enforcement.active = true;
    status
}
