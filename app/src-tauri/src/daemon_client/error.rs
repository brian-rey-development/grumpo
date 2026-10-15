use std::io;

use focus_core::ipc::peer::ServerIdentityError;
use focus_core::ipc::{IpcCodecError, IpcError};

#[derive(Debug, thiserror::Error)]
pub enum DaemonError {
    #[error("the Grumpo service is not installed")]
    NotInstalled,
    #[error("the Grumpo service is still installed under its earlier name, focusd")]
    Renamed,
    #[error("the Grumpo service is not responding: {0}")]
    NotResponding(io::Error),
    #[error("the Grumpo service speaks protocol version {daemon}")]
    VersionMismatch { daemon: u32 },
    #[error("the Grumpo service belongs to another user")]
    OtherUser,
    #[error("the socket is not served by the Grumpo service: {0}")]
    Untrusted(#[from] ServerIdentityError),
    #[error("the Grumpo service rejected the request: {0}")]
    Rejected(IpcError),
    #[error("the Grumpo service broke the protocol: {0}")]
    Protocol(String),
}

impl From<IpcCodecError> for DaemonError {
    fn from(error: IpcCodecError) -> Self {
        match error {
            IpcCodecError::VersionMismatch { found, .. } => Self::VersionMismatch { daemon: found },
            IpcCodecError::Io(source) => Self::NotResponding(source),
            IpcCodecError::MissingTerminator => Self::NotResponding(io::Error::new(
                io::ErrorKind::UnexpectedEof,
                "the connection closed before the reply was complete",
            )),
            IpcCodecError::TooLarge { .. } | IpcCodecError::Json(_) => {
                Self::Protocol(error.to_string())
            }
        }
    }
}

impl From<IpcError> for DaemonError {
    fn from(error: IpcError) -> Self {
        Self::Rejected(error)
    }
}
