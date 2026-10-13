use focus_core::LimitTarget;
use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum CommandErrorCode {
    DaemonUnavailable,
    VersionMismatch,
    RulesInvalid,
    LooseningLocked,
    UnlockNotReady,
    UnlockNotRequested,
    UnlockAlreadyPending,
    UnlockAlreadyActive,
    PhraseMismatch,
    DurationInvalid,
    InvalidInput,
    Storage,
    /// The folder for the export refused the file.
    ExportDenied,
    Platform,
    /// The installer is not bundled, as in a development build.
    ServiceInstallUnavailable,
    /// The installer ran and failed.
    ServiceInstallFailed,
    /// An earlier install is still running, so this one did not start.
    ServiceInstallBusy,
    Internal,
}

/// `message` is a diagnostic for logs and is never shown, because the UI words every failure
/// itself from `code` and, for invalid rules, from `rules_fault`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct CommandError {
    pub code: CommandErrorCode,
    pub message: String,
    /// What was wrong with the rules. Set with `rulesInvalid` when the app itself found the
    /// fault, and absent when the service refused rules the app had accepted.
    pub rules_fault: Option<RulesFault>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum RuleSection {
    Sites,
    Apps,
    Windows,
    Limits,
}

/// Why a blocked site was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum SiteProblem {
    Empty,
    NonAscii,
    TooLong,
    TooFewLabels,
    EmptyLabel,
    IpAddress,
    /// A suffix such as `co.uk` that many unrelated sites share.
    PublicSuffix,
    LabelTooLong,
    HyphenAtEdge,
    InvalidCharacter,
}

/// A specific way in which the rules were refused. Windows are counted from 1, in the order
/// the rules list them.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum RulesFault {
    /// The rules did not have the expected shape.
    Unreadable,
    InvalidSite {
        site: String,
        problem: SiteProblem,
    },
    InvalidApp,
    TooMany {
        section: RuleSection,
    },
    WindowWithoutDays {
        number: u32,
    },
    WindowNotForward {
        number: u32,
    },
    LimitOutOfRange {
        target: LimitTarget,
        minutes: u16,
    },
    DuplicateLimit {
        target: LimitTarget,
    },
    ProtectedApp {
        bundle_id: String,
    },
}
