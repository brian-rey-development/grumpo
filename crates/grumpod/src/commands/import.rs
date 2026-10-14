use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use focus_core::{legacy, Now, RulesError};
use thiserror::Error;

use crate::state::{FileStateStore, Snapshot, StateError, StateStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImportOutcome {
    Imported { sites: usize, windows: usize },
    AlreadyInitialized,
}

#[derive(Debug, Error)]
pub enum ImportError {
    #[error("could not read {}: {source}", path.display())]
    ReadScript { path: PathBuf, source: io::Error },
    #[error("no blocked domains were found in {}, is it the old focus-hosts script?", .0.display())]
    NoDomains(PathBuf),
    #[error("the legacy rules are not valid: {0}")]
    Rules(#[from] RulesError),
    #[error(transparent)]
    State(#[from] StateError),
}

/// Creates the state from the old script's domains and schedule, unless a state exists.
///
/// An existing state always wins, even a damaged one, because the daemon sets damaged
/// state aside itself and an import must never overwrite rules the user has since changed.
pub fn import_legacy(
    store: &FileStateStore,
    script: &Path,
    now: &Now,
) -> Result<ImportOutcome, ImportError> {
    if store.exists() {
        return Ok(ImportOutcome::AlreadyInitialized);
    }
    let bytes = fs::read(script).map_err(|source| ImportError::ReadScript {
        path: script.to_path_buf(),
        source,
    })?;
    let rules = legacy::rules(&String::from_utf8_lossy(&bytes));
    if rules.blocked_sites.is_empty() {
        return Err(ImportError::NoDomains(script.to_path_buf()));
    }
    let outcome = ImportOutcome::Imported {
        sites: rules.blocked_sites.len(),
        windows: rules.schedule.windows.len(),
    };
    store.save(&Snapshot {
        rules: rules.validate()?,
        ..Snapshot::fresh(now)
    })?;
    Ok(outcome)
}

#[cfg(test)]
mod tests;
