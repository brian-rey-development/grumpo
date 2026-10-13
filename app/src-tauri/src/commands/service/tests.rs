use focus_core::ipc::Request;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use super::test_support::FakeInstaller;
use super::*;
use crate::daemon_client::test_support::FakeDaemon;
use crate::dto::CommandErrorCode;
use crate::platform::ServiceInstallError;

fn install_with(
    installer: &FakeInstaller,
    daemon: &FakeDaemon,
) -> Result<ServiceInstallOutcome, AppError> {
    install_while(installer, daemon, &AtomicBool::new(false))
}

fn install_while(
    installer: &FakeInstaller,
    daemon: &FakeDaemon,
    installing: &AtomicBool,
) -> Result<ServiceInstallOutcome, AppError> {
    install(
        installer,
        &StatusHub::new(Arc::new(daemon.client())),
        installing,
    )
}

fn status_requests(daemon: &FakeDaemon) -> usize {
    daemon
        .requests()
        .iter()
        .filter(|request| matches!(request, Request::Status))
        .count()
}

#[test]
fn a_finished_install_asks_the_daemon_for_its_status_at_once() {
    let daemon = FakeDaemon::start();
    let installer = FakeInstaller::ending_with(Ok(InstallOutcome::Installed));

    let outcome = install_with(&installer, &daemon).unwrap();

    assert_eq!(outcome, ServiceInstallOutcome::Installed);
    assert_eq!(installer.runs(), 1);
    assert_eq!(status_requests(&daemon), 1);
}

#[test]
fn a_dismissed_prompt_is_an_outcome_and_leaves_the_daemon_alone() {
    let daemon = FakeDaemon::start();
    let installer = FakeInstaller::ending_with(Ok(InstallOutcome::Cancelled));

    let outcome = install_with(&installer, &daemon).unwrap();

    assert_eq!(outcome, ServiceInstallOutcome::Cancelled);
    assert_eq!(status_requests(&daemon), 0);
}

#[test]
fn a_missing_installer_is_reported_as_unavailable() {
    let daemon = FakeDaemon::start();
    let installer = FakeInstaller::ending_with(Err(ServiceInstallError::Unavailable));

    let error = CommandError::from(install_with(&installer, &daemon).unwrap_err());

    assert_eq!(error.code, CommandErrorCode::ServiceInstallUnavailable);
    assert_eq!(status_requests(&daemon), 0);
}

#[test]
fn a_failed_installer_is_reported_as_failed_without_asking_the_daemon() {
    let daemon = FakeDaemon::start();
    let installer = FakeInstaller::ending_with(Err(ServiceInstallError::Failed("boom".into())));

    let error = CommandError::from(install_with(&installer, &daemon).unwrap_err());

    assert_eq!(error.code, CommandErrorCode::ServiceInstallFailed);
    assert_eq!(status_requests(&daemon), 0);
}

#[test]
fn the_outcomes_are_sent_as_installed_and_cancelled() {
    for (outcome, wire) in [
        (ServiceInstallOutcome::Installed, "\"installed\""),
        (ServiceInstallOutcome::Cancelled, "\"cancelled\""),
    ] {
        assert_eq!(serde_json::to_string(&outcome).unwrap(), wire);
    }
}

#[test]
fn the_install_error_codes_are_sent_in_camel_case() {
    for (code, wire) in [
        (
            CommandErrorCode::ServiceInstallUnavailable,
            "\"serviceInstallUnavailable\"",
        ),
        (
            CommandErrorCode::ServiceInstallFailed,
            "\"serviceInstallFailed\"",
        ),
    ] {
        assert_eq!(serde_json::to_string(&code).unwrap(), wire);
    }
}

#[test]
fn a_second_install_while_one_runs_is_busy_and_never_starts_the_installer() {
    let daemon = FakeDaemon::start();
    let installer = FakeInstaller::ending_with(Ok(InstallOutcome::Installed));
    let installing = AtomicBool::new(true);

    let error = CommandError::from(install_while(&installer, &daemon, &installing).unwrap_err());

    assert_eq!(error.code, CommandErrorCode::ServiceInstallBusy);
    assert_eq!(installer.runs(), 0);
    assert!(
        installing.load(Ordering::SeqCst),
        "the first install keeps its claim"
    );
}

#[test]
fn the_claim_is_released_after_every_kind_of_ending() {
    let daemon = FakeDaemon::start();
    let installing = AtomicBool::new(false);
    for result in [
        Ok(InstallOutcome::Installed),
        Ok(InstallOutcome::Cancelled),
        Err(ServiceInstallError::Failed("boom".into())),
        Err(ServiceInstallError::Busy),
    ] {
        let installer = FakeInstaller::ending_with(result);

        let _ = install_while(&installer, &daemon, &installing);

        assert!(!installing.load(Ordering::SeqCst));
    }
}

#[test]
fn the_busy_code_is_sent_in_camel_case() {
    assert_eq!(
        serde_json::to_string(&CommandErrorCode::ServiceInstallBusy).unwrap(),
        "\"serviceInstallBusy\""
    );
}
