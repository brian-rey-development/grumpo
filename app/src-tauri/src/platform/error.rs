use std::fmt;

use super::Permission;

#[derive(Debug, thiserror::Error)]
pub enum PlatformError {
    #[error("not supported on this platform")]
    #[cfg_attr(
        all(target_os = "macos", not(test)),
        expect(
            dead_code,
            reason = "only the stub adapters for other systems report it"
        )
    )]
    Unsupported,
    #[error("permission denied: {0}")]
    PermissionDenied(Permission),
    #[error("permission not requested yet: {0}")]
    NeedsConsent(Permission),
    #[error("{operation} timed out")]
    Timeout { operation: &'static str },
    #[error("{operation} failed: {detail}")]
    Failed {
        operation: &'static str,
        detail: String,
    },
}

/// What went wrong without saying where, so it is safe to log above debug level: it never
/// carries the browser, the app or the detail that the full message may name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformErrorKind {
    Unsupported,
    PermissionDenied,
    NeedsConsent,
    Timeout,
    Failed,
}

impl fmt::Display for PlatformErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Unsupported => "unsupported",
            Self::PermissionDenied => "permission_denied",
            Self::NeedsConsent => "needs_consent",
            Self::Timeout => "timeout",
            Self::Failed => "failed",
        })
    }
}

impl PlatformError {
    pub fn kind(&self) -> PlatformErrorKind {
        match self {
            Self::Unsupported => PlatformErrorKind::Unsupported,
            Self::PermissionDenied(_) => PlatformErrorKind::PermissionDenied,
            Self::NeedsConsent(_) => PlatformErrorKind::NeedsConsent,
            Self::Timeout { .. } => PlatformErrorKind::Timeout,
            Self::Failed { .. } => PlatformErrorKind::Failed,
        }
    }
}
