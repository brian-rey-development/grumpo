mod requests;
#[cfg(test)]
pub(crate) mod test_support;
#[cfg(test)]
mod tests;

use std::io;

use focus_core::hosts;
use focus_core::ipc::{ErrorCode, IpcError, Request, Response};
use focus_core::Now;
use tracing::{debug, error, info, warn};

use crate::clock::Clock;
use crate::dns::PendingFlush;
use crate::hosts_writer::HostsSink;
use crate::state::{Snapshot, StateError, StateStore};
use requests::Applied;

/// A response and the resolver flush its hosts change calls for, which the caller runs
/// after letting go of the service lock.
#[derive(Debug)]
pub struct Handled {
    pub response: Response,
    pub flush: PendingFlush,
}

impl Handled {
    pub fn new(response: Response) -> Self {
        Self {
            response,
            flush: PendingFlush::NONE,
        }
    }
}

/// What the server and reconciler need from the service, so both can be tested without I/O.
pub trait Daemon: Send + 'static {
    fn handle(&mut self, request: Request) -> Handled;
    fn reconcile(&mut self) -> PendingFlush;
}

#[derive(Debug)]
pub struct Service<C, H, S> {
    clock: C,
    hosts: H,
    store: S,
    state: Snapshot,
    unsaved: bool,
}

impl<C: Clock, H: HostsSink, S: StateStore> Service<C, H, S> {
    pub fn new(clock: C, hosts: H, store: S) -> Result<Self, StateError> {
        let state = store.load(&clock.now())?;
        Ok(Self {
            clock,
            hosts,
            store,
            state,
            unsaved: false,
        })
    }

    fn respond(
        &mut self,
        request: Request,
        now: &Now,
    ) -> Result<(Response, PendingFlush), IpcError> {
        self.adopt_time(now);
        let applied = requests::apply(&self.state, request, now)?;
        self.commit(applied)?;
        let flush = self.apply_hosts(now);
        let status = self.state.status(now);
        Ok((Response::Ok { status }, flush))
    }

    /// Time moves state on its own (unlocks expire, days roll over), so the present is always
    /// adopted. It is written out with the next save instead of being a reason to fail.
    fn adopt_time(&mut self, now: &Now) {
        let advanced = self.state.advanced(now);
        if advanced != self.state {
            self.state = advanced;
            self.unsaved = true;
        }
    }

    /// Saves before adopting, so a rejected change leaves memory and disk both at the old state.
    ///
    /// The exception is a state that must be kept. Refusing it because the disk is failing
    /// would let a full disk switch enforcement off, so it is adopted anyway and the save is
    /// retried on the next tick.
    fn commit(&mut self, applied: Applied) -> Result<(), IpcError> {
        let Applied {
            next,
            keep_on_save_failure,
        } = applied;
        if next == self.state {
            return Ok(());
        }
        match self.store.save(&next) {
            Ok(()) => self.unsaved = false,
            Err(error) if keep_on_save_failure => {
                error!(reason = %error.summary(), "could not save a state that must stay in force, keeping it in memory");
                self.unsaved = true;
            }
            Err(error) => {
                error!(reason = %error.summary(), "could not save state, rejecting the change");
                return Err(IpcError::internal("the daemon could not save its state"));
            }
        }
        self.state = next;
        Ok(())
    }

    fn retry_unsaved(&mut self) {
        if !self.unsaved {
            return;
        }
        match self.store.save(&self.state) {
            Ok(()) => {
                info!("pending state saved");
                self.unsaved = false;
            }
            Err(error) => {
                error!(reason = %error.summary(), "could not save pending state, will retry on the next tick");
            }
        }
    }

    fn apply_hosts(&self, now: &Now) -> PendingFlush {
        let sites = self.state.blocked_sites(now);
        let existing = match self.hosts.read() {
            Ok(content) => content,
            Err(error) if error.kind() == io::ErrorKind::NotFound => Vec::new(),
            Err(error) => {
                error!(%error, "could not read the hosts file, leaving it alone");
                return PendingFlush::NONE;
            }
        };
        match self
            .hosts
            .write_if_changed(&hosts::render_bytes(&existing, &sites))
        {
            Ok(write) if write.changed => {
                info!(blocked_sites = sites.len(), "hosts file updated");
                write.flush
            }
            Ok(_) => {
                debug!(blocked_sites = sites.len(), "hosts file already current");
                PendingFlush::NONE
            }
            Err(error) => {
                error!(%error, "could not write the hosts file");
                PendingFlush::NONE
            }
        }
    }
}

impl<C, H, S> Daemon for Service<C, H, S>
where
    C: Clock + Send + 'static,
    H: HostsSink + Send + 'static,
    S: StateStore + Send + 'static,
{
    fn handle(&mut self, request: Request) -> Handled {
        let kind = requests::kind(&request);
        let now = self.clock.now();
        match self.respond(request, &now) {
            Ok((response, flush)) => {
                debug!(request = kind, "request handled");
                Handled { response, flush }
            }
            Err(error) => {
                log_rejection(kind, &error);
                Handled::new(Response::Err { error })
            }
        }
    }

    /// Moves state to the present, saves it if it changed and re-applies the hosts block.
    ///
    /// The hosts file is rewritten whenever it differs from the plan, which also repairs
    /// manual edits to the managed block.
    fn reconcile(&mut self) -> PendingFlush {
        let now = self.clock.now();
        self.adopt_time(&now);
        self.retry_unsaved();
        self.apply_hosts(&now)
    }
}

fn log_rejection(kind: &'static str, error: &IpcError) {
    if error.code == ErrorCode::Internal {
        error!(request = kind, "request failed");
    } else {
        warn!(request = kind, code = ?error.code, "request rejected");
    }
}
