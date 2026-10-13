use std::io;
use std::path::Path;

use thiserror::Error;

use crate::args::Command;
use crate::clock::{Clock, SystemClock};
use crate::commands::{self, ImportError, ImportOutcome, PingError, RunError};
use crate::hosts_writer::HostsFile;
use crate::instance_lock::{InstanceLock, LockError};
use crate::paths::Paths;
use crate::state::StateError;
use crate::state_dir::{ensure_state_dir, StateDirError};

#[derive(Debug, Error)]
pub enum CliError {
    #[error(transparent)]
    Run(#[from] RunError),
    #[error("{}", daemon_must_be_stopped(.0))]
    Locked(#[from] LockError),
    #[error(transparent)]
    Import(#[from] ImportError),
    #[error(transparent)]
    Ping(#[from] PingError),
    #[error(transparent)]
    State(#[from] StateError),
    #[error("could not update the hosts file: {0}")]
    Hosts(io::Error),
}

impl From<StateDirError> for CliError {
    fn from(error: StateDirError) -> Self {
        match error {
            StateDirError::State(state) => Self::State(state),
            StateDirError::Lock(lock) => Self::Locked(lock),
        }
    }
}

/// Runs a command to completion, with its outcome printed for scripts and an error on failure.
pub fn execute(command: Command, paths: &Paths) -> Result<(), CliError> {
    match command {
        Command::Run { owner_uid } => match commands::run(paths, owner_uid)? {},
        Command::ImportLegacy { script } => import_legacy(paths, &script)?,
        Command::StripHosts => strip_hosts(paths)?,
        Command::Ping => ping(paths)?,
    }
    Ok(())
}

fn daemon_must_be_stopped(error: &LockError) -> String {
    match error {
        LockError::Held(_) => {
            format!("{error}. Stop the daemon first, then run this command again")
        }
        LockError::Io { .. } => error.to_string(),
    }
}

fn ping(paths: &Paths) -> Result<(), CliError> {
    commands::ping(&paths.socket)?;
    println!("focusd is answering on {}", paths.socket.display());
    Ok(())
}

fn import_legacy(paths: &Paths, script: &Path) -> Result<(), CliError> {
    let (store, _lock) = ensure_state_dir(&paths.state_dir)?;
    let message = match commands::import_legacy(&store, script, &SystemClock.now())? {
        ImportOutcome::Imported { sites, windows } => {
            format!("imported {sites} blocked sites and {windows} schedule windows")
        }
        ImportOutcome::AlreadyInitialized => {
            "state already exists, legacy import skipped".to_owned()
        }
    };
    println!("{message}");
    Ok(())
}

fn strip_hosts(paths: &Paths) -> Result<(), CliError> {
    let _lock = InstanceLock::acquire_if_initialized(&paths.state_dir)?;
    let hosts = HostsFile::new(paths.hosts_file.clone(), paths.flushes_dns());
    let write = commands::strip_hosts(&hosts).map_err(CliError::Hosts)?;
    if write.changed {
        println!("removed the managed block from the hosts file");
        write.flush.run();
    } else {
        println!("the hosts file has no managed block");
    }
    Ok(())
}
