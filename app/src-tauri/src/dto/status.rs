use focus_core::ipc::DaemonStatus;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum DaemonUnavailableReason {
    NotInstalled,
    NotResponding,
    /// The installed Grumpo service speaks an older protocol than this app.
    ServiceOutdated,
    /// The installed Grumpo service speaks a newer protocol than this app.
    AppOutdated,
    /// The socket exists but belongs to another user account.
    OtherUser,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum FocusStatus {
    Connected {
        status: Box<DaemonStatus>,
    },
    /// `message` is a diagnostic for logs and is never shown, because the UI words each
    /// reason itself.
    Unavailable {
        reason: DaemonUnavailableReason,
        message: String,
    },
}

impl FocusStatus {
    pub fn daemon(&self) -> Option<&DaemonStatus> {
        match self {
            Self::Connected { status } => Some(status),
            Self::Unavailable { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests;
