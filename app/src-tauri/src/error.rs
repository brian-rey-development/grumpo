//! The one place where internal failures turn into the errors the UI sees. The UI words every
//! failure from the code, so the message carried along is a diagnostic only.

mod rules_fault;

use focus_core::ipc::ErrorCode;
use focus_core::RulesError;

use crate::daemon_client::DaemonError;
use crate::dto::{CommandError, CommandErrorCode, RulesFault};
use crate::launch_at_login::LaunchAtLoginError;
use crate::platform::{PlatformError, ServiceInstallError};
use crate::storage::StorageError;

pub use rules_fault::diagnose_unreadable_rules;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Daemon(#[from] DaemonError),
    #[error(transparent)]
    Storage(#[from] StorageError),
    #[error(transparent)]
    Platform(#[from] PlatformError),
    #[error("rules rejected: {detail}")]
    RulesInvalid { fault: RulesFault, detail: String },
    #[error("the export folder refused the file")]
    ExportDenied,
    #[error(transparent)]
    ServiceInstall(#[from] ServiceInstallError),
    #[error(transparent)]
    LaunchAtLogin(#[from] LaunchAtLoginError),
    #[error("{0}")]
    InvalidInput(String),
    #[error("{0}")]
    Internal(String),
}

impl From<RulesError> for AppError {
    fn from(error: RulesError) -> Self {
        Self::RulesInvalid {
            detail: error.to_string(),
            fault: RulesFault::from(&error),
        }
    }
}

impl AppError {
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput(message.into())
    }

    fn code(&self) -> CommandErrorCode {
        match self {
            Self::Daemon(error) => daemon_code(error),
            Self::Storage(_) => CommandErrorCode::Storage,
            Self::Platform(_) | Self::LaunchAtLogin(_) => CommandErrorCode::Platform,
            Self::ServiceInstall(error) => install_code(error),
            Self::ExportDenied => CommandErrorCode::ExportDenied,
            Self::RulesInvalid { .. } => CommandErrorCode::RulesInvalid,
            Self::InvalidInput(_) => CommandErrorCode::InvalidInput,
            Self::Internal(_) => CommandErrorCode::Internal,
        }
    }

    fn rules_fault(&self) -> Option<RulesFault> {
        match self {
            Self::RulesInvalid { fault, .. } => Some(fault.clone()),
            _ => None,
        }
    }
}

impl From<AppError> for CommandError {
    fn from(error: AppError) -> Self {
        let code = error.code();
        tracing::warn!(?code, "command failed");
        tracing::debug!(%error, "command failure detail");
        Self {
            code,
            message: error.to_string(),
            rules_fault: error.rules_fault(),
        }
    }
}

fn install_code(error: &ServiceInstallError) -> CommandErrorCode {
    match error {
        ServiceInstallError::Unavailable => CommandErrorCode::ServiceInstallUnavailable,
        ServiceInstallError::Failed(_) => CommandErrorCode::ServiceInstallFailed,
        ServiceInstallError::Busy => CommandErrorCode::ServiceInstallBusy,
    }
}

fn daemon_code(error: &DaemonError) -> CommandErrorCode {
    match error {
        DaemonError::NotInstalled | DaemonError::NotResponding(_) | DaemonError::OtherUser => {
            CommandErrorCode::DaemonUnavailable
        }
        DaemonError::VersionMismatch { .. } => CommandErrorCode::VersionMismatch,
        DaemonError::Rejected(rejection) => rejected_code(rejection.code),
        DaemonError::Protocol(_) => CommandErrorCode::Internal,
    }
}

fn rejected_code(code: ErrorCode) -> CommandErrorCode {
    match code {
        ErrorCode::VersionMismatch => CommandErrorCode::VersionMismatch,
        ErrorCode::RulesInvalid => CommandErrorCode::RulesInvalid,
        ErrorCode::LooseningLocked => CommandErrorCode::LooseningLocked,
        ErrorCode::UnlockNotReady => CommandErrorCode::UnlockNotReady,
        ErrorCode::UnlockNotRequested => CommandErrorCode::UnlockNotRequested,
        ErrorCode::UnlockAlreadyPending => CommandErrorCode::UnlockAlreadyPending,
        ErrorCode::UnlockAlreadyActive => CommandErrorCode::UnlockAlreadyActive,
        ErrorCode::PhraseMismatch => CommandErrorCode::PhraseMismatch,
        ErrorCode::DurationInvalid => CommandErrorCode::DurationInvalid,
        ErrorCode::Protocol | ErrorCode::Internal => CommandErrorCode::Internal,
    }
}

#[cfg(test)]
mod tests;
