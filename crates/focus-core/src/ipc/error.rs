use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::IpcCodecError;
use crate::rules::{Loosening, RulesError};
use crate::unlock::UnlockError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ErrorCode {
    VersionMismatch,
    RulesInvalid,
    LooseningLocked,
    UnlockNotReady,
    UnlockNotRequested,
    UnlockAlreadyPending,
    UnlockAlreadyActive,
    PhraseMismatch,
    DurationInvalid,
    Protocol,
    Internal,
}

#[derive(Debug, Clone, PartialEq, Eq, Error, Serialize, Deserialize)]
#[error("{message}")]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct IpcError {
    pub code: ErrorCode,
    pub message: String,
}

impl IpcError {
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    pub fn loosening_locked(loosenings: &[Loosening]) -> Self {
        let changes: Vec<String> = loosenings.iter().map(ToString::to_string).collect();
        Self::new(
            ErrorCode::LooseningLocked,
            format!(
                "This change needs an unlock first because it {}",
                changes.join(", ")
            ),
        )
    }

    pub fn protocol(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Protocol, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(ErrorCode::Internal, message)
    }
}

impl From<UnlockError> for IpcError {
    fn from(error: UnlockError) -> Self {
        let code = match error {
            UnlockError::NotRequested => ErrorCode::UnlockNotRequested,
            UnlockError::AlreadyPending => ErrorCode::UnlockAlreadyPending,
            UnlockError::AlreadyUnlocked => ErrorCode::UnlockAlreadyActive,
            UnlockError::NotReady => ErrorCode::UnlockNotReady,
            UnlockError::PhraseMismatch => ErrorCode::PhraseMismatch,
            UnlockError::DurationInvalid => ErrorCode::DurationInvalid,
        };
        Self::new(code, error.to_string())
    }
}

impl From<RulesError> for IpcError {
    fn from(error: RulesError) -> Self {
        Self::new(ErrorCode::RulesInvalid, error.to_string())
    }
}

impl From<IpcCodecError> for IpcError {
    fn from(error: IpcCodecError) -> Self {
        let code = match error {
            IpcCodecError::VersionMismatch { .. } => ErrorCode::VersionMismatch,
            _ => ErrorCode::Protocol,
        };
        Self::new(code, error.to_string())
    }
}

#[cfg(test)]
mod tests;
