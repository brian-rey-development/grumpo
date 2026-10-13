use std::sync::{Condvar, Mutex, MutexGuard, PoisonError, WaitTimeoutResult};
use std::time::Duration;

/// Locks that carry on after another thread panicked while holding them. Every lock in the
/// app guards state that a panic cannot leave half-written in a way later readers could not
/// cope with, so refusing to continue would only turn one failure into many.
pub trait LockExt<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T>;
}

impl<T> LockExt<T> for Mutex<T> {
    fn lock_or_recover(&self) -> MutexGuard<'_, T> {
        self.lock().unwrap_or_else(|poisoned: PoisonError<_>| {
            tracing::warn!("a lock was poisoned, recovering");
            poisoned.into_inner()
        })
    }
}

pub trait CondvarExt {
    fn wait_or_recover<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T>;

    fn wait_timeout_or_recover<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        limit: Duration,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult);
}

impl CondvarExt for Condvar {
    fn wait_or_recover<'a, T>(&self, guard: MutexGuard<'a, T>) -> MutexGuard<'a, T> {
        self.wait(guard).unwrap_or_else(PoisonError::into_inner)
    }

    fn wait_timeout_or_recover<'a, T>(
        &self,
        guard: MutexGuard<'a, T>,
        limit: Duration,
    ) -> (MutexGuard<'a, T>, WaitTimeoutResult) {
        self.wait_timeout(guard, limit)
            .unwrap_or_else(PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests;
