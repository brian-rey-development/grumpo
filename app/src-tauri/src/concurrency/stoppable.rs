use std::fmt;
use std::io;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

type StopHook = Box<dyn FnOnce() + Send + Sync>;

/// A worker thread that is asked to stop and then joined, when told to or when dropped.
pub struct StoppableThread {
    name: &'static str,
    stop: Option<StopHook>,
    join: Option<JoinHandle<()>>,
}

impl fmt::Debug for StoppableThread {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StoppableThread")
            .field("name", &self.name)
            .finish_non_exhaustive()
    }
}

/// What a worker waits on between rounds, so that a stop request ends the wait at once.
#[derive(Debug)]
pub struct StopSignal(mpsc::Receiver<()>);

impl StopSignal {
    /// Sleeps for `timeout` and answers whether a stop was requested meanwhile.
    pub fn wait(&self, timeout: Duration) -> bool {
        match self.0.recv_timeout(timeout) {
            Err(RecvTimeoutError::Timeout) => false,
            Ok(()) | Err(RecvTimeoutError::Disconnected) => true,
        }
    }
}

impl StoppableThread {
    /// Runs `body` on a thread named `name`. `stop` is what makes `body` return, for a worker
    /// that waits on something other than a [`StopSignal`].
    pub fn spawn(
        name: &'static str,
        stop: impl FnOnce() + Send + Sync + 'static,
        body: impl FnOnce() + Send + 'static,
    ) -> io::Result<Self> {
        let join = thread::Builder::new().name(name.into()).spawn(body)?;
        Ok(Self {
            name,
            stop: Some(Box::new(stop)),
            join: Some(join),
        })
    }

    /// Runs `body` with a signal that fires when the thread is told to stop.
    pub fn spawn_signalled(
        name: &'static str,
        body: impl FnOnce(&StopSignal) + Send + 'static,
    ) -> io::Result<Self> {
        let (sender, receiver) = mpsc::channel();
        let signal = StopSignal(receiver);
        Self::spawn(name, move || drop(sender), move || body(&signal))
    }

    /// Asks the thread to stop and detaches it instead of joining.
    pub fn stop_without_waiting(mut self) {
        self.request_stop();
        self.join = None;
    }

    /// Asks the thread to stop and waits for it. Safe to call twice.
    pub fn stop(&mut self) {
        self.request_stop();
        if let Some(join) = self.join.take() {
            if join.join().is_err() {
                tracing::error!(thread = self.name, "a worker thread panicked");
            }
        }
    }

    fn request_stop(&mut self) {
        if let Some(stop) = self.stop.take() {
            stop();
        }
    }
}

impl Drop for StoppableThread {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests;
