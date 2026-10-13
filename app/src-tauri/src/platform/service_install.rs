#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallOutcome {
    Installed,
    /// The person dismissed the administrator prompt.
    Cancelled,
}

#[derive(Debug, thiserror::Error)]
pub enum ServiceInstallError {
    /// A development build, or a system without an installer, has nothing to run.
    #[error("the installer is not bundled with this app")]
    Unavailable,
    /// Another install is still running, here or as root after this app stopped waiting.
    #[error("another install is still running")]
    Busy,
    #[error("the installer failed: {0}")]
    Failed(String),
}
