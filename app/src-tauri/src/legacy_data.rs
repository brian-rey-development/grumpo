//! The app's data folder is named after its bundle identifier, so the identifier change
//! would leave the usage history and the settings behind. The first start under
//! the new identifier moves the folder across.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The identifier of the app before it was renamed to Grumpo.
pub const LEGACY_IDENTIFIER: &str = "io.github.brian-rey-development.focus";

/// Tells whether the earlier app is running right now.
pub trait LegacyAppProbe {
    fn is_running(&self) -> bool;
}

/// Returns the folder the app should keep its data in. That is `current` once the earlier
/// folder has been moved there, and the earlier folder itself when the move failed, so a
/// refusal never hides the person's history behind a fresh database and the next start
/// tries again.
///
/// The move is a single rename, which the file system applies completely or not at all, so a
/// crash leaves the data in exactly one of the two places. When both exist the new folder
/// holds data the new app already wrote, and the earlier one is left untouched.
///
/// While the earlier app runs its open database files would move out from under it and it
/// would keep writing sessions into the folder this app opens. So the move waits for a start
/// when it is gone, and this start keeps using the earlier folder, which also keeps the
/// history out of a fresh database that would later hide it behind "both exist".
pub fn adopt_legacy_data(current: &Path, earlier_app: &dyn LegacyAppProbe) -> PathBuf {
    let Some(legacy) = legacy_folder(current) else {
        return current.to_owned();
    };
    match move_folder(&legacy, current, earlier_app) {
        Ok(Move::Settled) => current.to_owned(),
        Ok(Move::WaitsForEarlierApp) => legacy,
        Err(error) => {
            tracing::warn!("could not move the data folder of the earlier app, using it in place");
            tracing::debug!(%error, "data folder move failure detail");
            legacy
        }
    }
}

fn legacy_folder(current: &Path) -> Option<PathBuf> {
    Some(current.parent()?.join(LEGACY_IDENTIFIER))
}

enum Move {
    Settled,
    WaitsForEarlierApp,
}

fn move_folder(
    legacy: &Path,
    current: &Path,
    earlier_app: &dyn LegacyAppProbe,
) -> io::Result<Move> {
    if !legacy.is_dir() {
        return Ok(Move::Settled);
    }
    if current.exists() && !is_empty_dir(current)? {
        tracing::info!("found data folders of both app identities, keeping the new one");
        return Ok(Move::Settled);
    }
    if earlier_app.is_running() {
        tracing::info!("the earlier app is still running, moving its data folder next start");
        return Ok(Move::WaitsForEarlierApp);
    }
    fs::rename(legacy, current)?;
    tracing::info!("moved the data folder of the earlier app identity");
    tracing::debug!(from = %legacy.display(), to = %current.display(), "data folder moved");
    Ok(Move::Settled)
}

fn is_empty_dir(path: &Path) -> io::Result<bool> {
    Ok(path.is_dir() && fs::read_dir(path)?.next().is_none())
}

#[cfg(test)]
mod tests;
