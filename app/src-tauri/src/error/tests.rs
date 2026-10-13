use std::io;

use focus_core::ipc::IpcError;

use super::*;
use crate::dto::SiteProblem;
use crate::platform::{Browser, Permission};

fn map(error: AppError) -> CommandError {
    CommandError::from(error)
}

fn rejected(code: ErrorCode) -> AppError {
    AppError::Daemon(DaemonError::Rejected(IpcError::new(code, "daemon says so")))
}

#[test]
fn daemon_rejections_map_one_to_one() {
    let cases = [
        (
            ErrorCode::VersionMismatch,
            CommandErrorCode::VersionMismatch,
        ),
        (ErrorCode::RulesInvalid, CommandErrorCode::RulesInvalid),
        (
            ErrorCode::LooseningLocked,
            CommandErrorCode::LooseningLocked,
        ),
        (ErrorCode::UnlockNotReady, CommandErrorCode::UnlockNotReady),
        (
            ErrorCode::UnlockNotRequested,
            CommandErrorCode::UnlockNotRequested,
        ),
        (
            ErrorCode::UnlockAlreadyPending,
            CommandErrorCode::UnlockAlreadyPending,
        ),
        (
            ErrorCode::UnlockAlreadyActive,
            CommandErrorCode::UnlockAlreadyActive,
        ),
        (ErrorCode::PhraseMismatch, CommandErrorCode::PhraseMismatch),
        (
            ErrorCode::DurationInvalid,
            CommandErrorCode::DurationInvalid,
        ),
        (ErrorCode::Protocol, CommandErrorCode::Internal),
        (ErrorCode::Internal, CommandErrorCode::Internal),
    ];
    for (daemon, expected) in cases {
        assert_eq!(map(rejected(daemon)).code, expected, "{daemon:?}");
    }
}

#[test]
fn a_rejection_by_the_service_carries_no_fault_of_its_own() {
    assert_eq!(map(rejected(ErrorCode::RulesInvalid)).rules_fault, None);
}

#[test]
fn an_absent_silent_or_foreign_daemon_is_unavailable() {
    let silent = DaemonError::NotResponding(io::Error::from(io::ErrorKind::TimedOut));
    for error in [DaemonError::NotInstalled, silent, DaemonError::OtherUser] {
        assert_eq!(
            map(AppError::Daemon(error)).code,
            CommandErrorCode::DaemonUnavailable
        );
    }
}

#[test]
fn a_version_mismatch_is_its_own_code() {
    let mapped = map(AppError::Daemon(DaemonError::VersionMismatch {
        daemon: 99,
    }));
    assert_eq!(mapped.code, CommandErrorCode::VersionMismatch);
}

#[test]
fn a_refused_export_has_its_own_code() {
    assert_eq!(
        map(AppError::ExportDenied).code,
        CommandErrorCode::ExportDenied
    );
}

#[test]
fn a_protocol_failure_is_internal() {
    let mapped = map(AppError::Daemon(DaemonError::Protocol("bad frame".into())));
    assert_eq!(mapped.code, CommandErrorCode::Internal);
}

#[test]
fn local_failures_map_to_their_own_codes() {
    let storage = map(AppError::Storage(StorageError::SessionNotFound(1)));
    assert_eq!(storage.code, CommandErrorCode::Storage);

    let platform = map(AppError::Platform(PlatformError::PermissionDenied(
        Permission::Automation(Browser::Chrome),
    )));
    assert_eq!(platform.code, CommandErrorCode::Platform);

    let login = map(AppError::LaunchAtLogin(LaunchAtLoginError::System(
        tauri_plugin_autostart::Error::Anyhow("path missing".into()),
    )));
    assert_eq!(login.code, CommandErrorCode::Platform);

    let input = map(AppError::invalid_input("Pick a valid date."));
    assert_eq!(input.code, CommandErrorCode::InvalidInput);

    let internal = map(AppError::Internal("join failed".into()));
    assert_eq!(internal.code, CommandErrorCode::Internal);
}

#[test]
fn the_message_is_the_diagnostic_and_not_a_sentence_for_people() {
    let mapped = map(AppError::Internal("join failed".into()));
    assert_eq!(mapped.message, "join failed");
}

#[test]
fn invalid_rules_carry_a_typed_fault() {
    let error = AppError::from(RulesError::WindowWithoutDays { number: 2 });
    let mapped = map(error);
    assert_eq!(mapped.code, CommandErrorCode::RulesInvalid);
    assert_eq!(
        mapped.rules_fault,
        Some(RulesFault::WindowWithoutDays { number: 2 })
    );
    assert_eq!(
        mapped.message,
        "rules rejected: schedule window 2 has no days selected"
    );
}

#[test]
fn an_unreadable_site_keeps_the_text_that_was_refused() {
    let error = AppError::RulesInvalid {
        fault: RulesFault::InvalidSite {
            site: "co.uk".into(),
            problem: SiteProblem::PublicSuffix,
        },
        detail: "public suffix".into(),
    };
    assert_eq!(
        map(error).rules_fault,
        Some(RulesFault::InvalidSite {
            site: "co.uk".into(),
            problem: SiteProblem::PublicSuffix
        })
    );
}
