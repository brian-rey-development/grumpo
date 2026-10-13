use std::io;

use super::*;

#[test]
fn a_service_on_an_older_protocol_is_outdated_and_a_newer_one_means_the_app_is() {
    assert_eq!(
        version_reason(PROTOCOL_VERSION - 1),
        DaemonUnavailableReason::ServiceOutdated
    );
    assert_eq!(
        version_reason(PROTOCOL_VERSION + 1),
        DaemonUnavailableReason::AppOutdated
    );
}

#[test]
fn errors_map_to_reasons() {
    let silent = DaemonError::NotResponding(io::Error::from(io::ErrorKind::TimedOut));
    assert_eq!(reason_of(&silent), DaemonUnavailableReason::NotResponding);
    assert_eq!(
        reason_of(&DaemonError::NotInstalled),
        DaemonUnavailableReason::NotInstalled
    );
    assert_eq!(
        reason_of(&DaemonError::OtherUser),
        DaemonUnavailableReason::OtherUser
    );
    assert_eq!(
        reason_of(&DaemonError::Protocol(String::new())),
        DaemonUnavailableReason::NotResponding
    );
}

#[test]
fn the_message_is_the_diagnostic_of_the_error() {
    let FocusStatus::Unavailable { message, .. } = unavailable(&DaemonError::NotInstalled) else {
        panic!("an error leaves the status unavailable");
    };
    assert_eq!(message, "the Grumpo service is not installed");
}
