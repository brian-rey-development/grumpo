use std::sync::{Arc, Mutex};

use tracing::error;

/// A value shared between the server and the reconciler, which survives a reconciler panic while held.
#[derive(Debug)]
pub struct Shared<T>(Arc<Mutex<T>>);

impl<T> Clone for Shared<T> {
    fn clone(&self) -> Self {
        Self(Arc::clone(&self.0))
    }
}

impl<T> Shared<T> {
    pub fn new(value: T) -> Self {
        Self(Arc::new(Mutex::new(value)))
    }

    /// Runs `f` with exclusive access. A poisoned lock is recovered rather than propagated,
    /// because the service only assigns whole new states, so the inner value is consistent.
    /// Only a contained reconciler tick can poison it, since a panic elsewhere ends the process.
    pub fn with<R>(&self, f: impl FnOnce(&mut T) -> R) -> R {
        let mut guard = self.0.lock().unwrap_or_else(|poisoned| {
            error!("lock was poisoned by a panic, continuing with the last state");
            self.0.clear_poison();
            poisoned.into_inner()
        });
        f(&mut guard)
    }
}

#[cfg(test)]
mod tests;
