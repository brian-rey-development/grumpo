mod file_store;
mod migrate;
mod persisted;
mod snapshot;

use std::borrow::Cow;
use std::io;
use std::path::PathBuf;

use focus_core::{Now, RulesError};
use thiserror::Error;

pub use file_store::FileStateStore;
pub use snapshot::Snapshot;

#[derive(Debug, Error)]
pub enum StateError {
    #[error("could not {action} {}: {source}", path.display())]
    Io {
        action: &'static str,
        path: PathBuf,
        source: io::Error,
    },
    #[error("state file is not valid: {0}")]
    Json(#[from] serde_json::Error),
    #[error("state file has no readable schema version")]
    MissingVersion,
    #[error("state file has schema version {found}, this daemon supports up to {supported}")]
    UnsupportedVersion { found: u64, supported: u32 },
    #[error("state file holds rules that no longer validate: {0}")]
    InvalidRules(#[from] RulesError),
    #[error("{} is a symlink, so the daemon will not keep its state behind it", .0.display())]
    Symlink(PathBuf),
    #[error(
        "{} belongs to uid {found} and the daemon runs as uid {expected}, so it will not trust it",
        path.display()
    )]
    ForeignOwner {
        path: PathBuf,
        found: u32,
        expected: u32,
    },
    /// Without the hosts file there is no telling which sites to keep blocking, and starting
    /// anyway would strip the managed block.
    #[error("could not read the hosts file to keep its blocked sites: {0}")]
    HostsUnreadable(io::Error),
}

impl StateError {
    /// Whether the saved file itself is at fault, so setting it aside and starting over is safe.
    pub fn is_unusable_file(&self) -> bool {
        matches!(
            self,
            Self::Json(_) | Self::MissingVersion | Self::InvalidRules(_)
        )
    }

    /// A description that never quotes the state file, whose text can name sites and apps,
    /// so it is safe above debug level.
    pub fn summary(&self) -> Cow<'static, str> {
        match self {
            Self::Json(_) => Cow::Borrowed("state file could not be encoded or decoded as JSON"),
            Self::InvalidRules(_) => {
                Cow::Borrowed("state file holds rules that no longer validate")
            }
            other => Cow::Owned(other.to_string()),
        }
    }
}

pub trait StateStore {
    /// Loads the saved state, or `None` when there is no usable one, so the caller decides what
    /// to start from instead of this layer quietly picking empty rules.
    ///
    /// A saved state that cannot be read as one (damaged JSON, no version, rules that no longer
    /// validate) is set aside and reported as `None`. A state that cannot be read at all, or
    /// that a newer daemon wrote, is an error instead, because starting without it would end
    /// enforcement while the hosts block from the saved rules still stands.
    fn load(&self, now: &Now) -> Result<Option<Snapshot>, StateError>;

    fn save(&self, state: &Snapshot) -> Result<(), StateError>;
}
