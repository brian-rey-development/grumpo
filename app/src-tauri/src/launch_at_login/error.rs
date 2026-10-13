use crate::storage::StorageError;

#[derive(Debug, thiserror::Error)]
pub enum LaunchAtLoginError {
    #[error("the system login item could not be changed: {0}")]
    System(#[from] tauri_plugin_autostart::Error),
    #[error("the login item of the earlier app could not be removed: {0}")]
    LegacyItem(#[source] std::io::Error),
    #[error(transparent)]
    Storage(#[from] StorageError),
}
