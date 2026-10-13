use super::*;
use crate::daemon_client::test_support::daemon_status;

#[test]
fn only_a_connected_status_has_a_daemon() {
    let connected = FocusStatus::Connected {
        status: Box::new(daemon_status()),
    };
    let down = FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::OtherUser,
        message: String::new(),
    };

    assert_eq!(connected.daemon(), Some(&daemon_status()));
    assert_eq!(down.daemon(), None);
}
