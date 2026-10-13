//! Why the daemon is out of reach, as the reason the UI words itself. The text kept beside it
//! is a diagnostic for logs and is never shown.

use focus_core::ipc::PROTOCOL_VERSION;

use crate::daemon_client::DaemonError;
use crate::dto::{DaemonUnavailableReason, FocusStatus};

/// Which side is behind when the protocol versions differ. A service that speaks an older
/// version needs the install refreshed, and one that speaks a newer version means this app
/// is the old one.
pub fn version_reason(daemon_version: u32) -> DaemonUnavailableReason {
    if daemon_version < PROTOCOL_VERSION {
        DaemonUnavailableReason::ServiceOutdated
    } else {
        DaemonUnavailableReason::AppOutdated
    }
}

pub fn reason_of(error: &DaemonError) -> DaemonUnavailableReason {
    match error {
        DaemonError::NotInstalled => DaemonUnavailableReason::NotInstalled,
        DaemonError::NotResponding(_) | DaemonError::Rejected(_) | DaemonError::Protocol(_) => {
            DaemonUnavailableReason::NotResponding
        }
        DaemonError::VersionMismatch { daemon } => version_reason(*daemon),
        DaemonError::OtherUser => DaemonUnavailableReason::OtherUser,
    }
}

/// The status for an error that left the app without a daemon status.
pub fn unavailable(error: &DaemonError) -> FocusStatus {
    FocusStatus::Unavailable {
        reason: reason_of(error),
        message: error.to_string(),
    }
}

#[cfg(test)]
mod tests;
