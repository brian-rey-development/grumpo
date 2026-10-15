use std::convert::Infallible;
use std::io;
use std::path::PathBuf;

use thiserror::Error;
use tracing::{info, warn};

use crate::clock::SystemClock;
use crate::dns::FlushWorker;
use crate::hosts_writer::HostsFile;
use crate::log_rotation::RotatingLog;
use crate::paths::Paths;
use crate::reconciler::{self, TICK_INTERVAL};
use crate::server::{Server, ServerError};
use crate::service::{Daemon, Service};
use crate::shared::Shared;
use crate::state::StateError;
use crate::state_dir::{ensure_state_dir, StateDirError};

#[derive(Debug, Error)]
pub enum RunError {
    #[error(transparent)]
    StateDir(#[from] StateDirError),
    #[error(transparent)]
    State(#[from] StateError),
    #[error(transparent)]
    Server(#[from] ServerError),
    #[error("could not start the reconciler thread: {0}")]
    Reconciler(io::Error),
    #[error("could not start the resolver flush thread: {0}")]
    Flusher(io::Error),
}

/// Serves until the process is stopped, so it only ever returns an error.
///
/// The instance lock comes first and is held until the process ends, so a second daemon fails
/// before it can touch the socket. The hosts block is reconciled before the socket opens, so a
/// reboot re-applies it before any client can connect, and even a daemon that then fails to bind
/// has enforced the rules. Resolver flushes all go through one background thread, so neither
/// the accept loop nor the reconciler waits for one.
pub fn run(paths: &Paths, owner_uid: u32) -> Result<Infallible, RunError> {
    info!(
        version = env!("CARGO_PKG_VERSION"),
        owner_uid, "grumpod starting"
    );
    let log = paths.log_file.clone().and_then(attach_log);
    let (store, _instance) = ensure_state_dir(&paths.state_dir)?;
    let hosts = HostsFile::new(paths.hosts_file.clone(), paths.flushes_dns());
    hosts.remove_stale_temps();
    let mut service = Service::new(SystemClock, hosts, store)?;
    let flusher = FlushWorker::spawn().map_err(RunError::Flusher)?;
    flusher.submit(service.reconcile());
    let server = Server::bind(&paths.socket, owner_uid)?;
    let shared = Shared::new(service);
    reconciler::spawn(
        shared.clone(),
        SystemClock,
        TICK_INTERVAL,
        flusher.clone(),
        follow_rotation(log),
    )
    .map_err(RunError::Reconciler)?;
    server.serve(&shared, &flusher)
}

fn attach_log(path: PathBuf) -> Option<RotatingLog> {
    RotatingLog::attach(path)
        .inspect_err(|error| warn!(%error, "could not attach to the log file, keeping stderr"))
        .ok()
}

fn follow_rotation(mut log: Option<RotatingLog>) -> impl FnMut() + Send + 'static {
    move || {
        let Some(log) = log.as_mut() else { return };
        match log.reattach_if_rotated() {
            Ok(true) => info!("log file was rotated, reopened it"),
            Ok(false) => {}
            Err(error) => warn!(%error, "could not reopen the rotated log file"),
        }
    }
}
