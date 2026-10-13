use std::io;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::thread::{self, JoinHandle};
use std::time::Duration;

use tracing::error;

use crate::clock::Clock;
use crate::service::Daemon;
use crate::shared::Shared;

mod wait;

use wait::next_wait;

pub const TICK_INTERVAL: Duration = Duration::from_secs(15);
const THREAD_NAME: &str = "reconciler";

/// Re-applies the plan on a regular beat, which is what ends unlocks and rolls over days
/// when no client is talking to the daemon. A beat is also scheduled just after each local
/// midnight, so exhausted limits clear promptly. The resolver flush and `housekeeping` run
/// after each beat, outside the lock.
///
/// A panic in a beat is contained and logged, because nothing else would retry the work.
/// Requests are not guarded the same way, since a client can simply ask again.
pub fn spawn<D: Daemon>(
    service: Shared<D>,
    clock: impl Clock + Send + 'static,
    interval: Duration,
    mut housekeeping: impl FnMut() + Send + 'static,
) -> io::Result<JoinHandle<()>> {
    thread::Builder::new()
        .name(THREAD_NAME.to_owned())
        .spawn(move || loop {
            thread::sleep(next_wait(&clock.now(), interval));
            let beat = catch_unwind(AssertUnwindSafe(|| {
                service.with(Daemon::reconcile).run();
                housekeeping();
            }));
            if beat.is_err() {
                error!("panic contained, the daemon keeps running");
            }
        })
}

#[cfg(test)]
mod tests;
