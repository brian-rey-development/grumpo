use std::io;

#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("database error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("{context}: {source}")]
    Io {
        context: &'static str,
        #[source]
        source: io::Error,
    },
    #[error("export serialization failed: {0}")]
    Json(#[from] serde_json::Error),
    #[error("database schema version {found} is newer than the {supported} this app supports")]
    SchemaTooNew { found: u32, supported: u32 },
    #[error("session {0} does not exist")]
    SessionNotFound(i64),
    #[error("the database failed its integrity check: {0}")]
    Corrupt(String),
}

impl StorageError {
    /// True when the file is not a usable database at all, so starting over is the only way
    /// forward. A newer schema, a busy file or a full disk are not damage.
    pub fn is_damaged(&self) -> bool {
        match self {
            Self::Corrupt(_) => true,
            Self::Sqlite(rusqlite::Error::SqliteFailure(failure, _)) => matches!(
                failure.code,
                rusqlite::ErrorCode::DatabaseCorrupt | rusqlite::ErrorCode::NotADatabase
            ),
            _ => false,
        }
    }

    pub(crate) fn io(context: &'static str) -> impl FnOnce(io::Error) -> Self {
        move |source| Self::Io { context, source }
    }
}
