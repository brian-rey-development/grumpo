use std::io;
use std::time::{Duration, Instant};

use focus_core::ipc::{ErrorCode, IpcError, Request, Response, MAX_RESPONSE_BYTES};

use super::test_support::{daemon_status, FakeDaemon, Reply};
use super::{DaemonClient, DaemonError};

const OVERSIZE_BUDGET: Duration = Duration::from_secs(30);

#[test]
fn a_status_reply_is_returned_and_the_request_arrives_intact() {
    let daemon = FakeDaemon::start();

    let status = daemon.client().send(&Request::Status).unwrap();

    assert_eq!(status, daemon_status());
    assert_eq!(daemon.requests(), [Request::Status]);
}

#[test]
fn an_error_reply_is_rejected_with_its_code() {
    let daemon = FakeDaemon::start();
    let error = IpcError::new(ErrorCode::LooseningLocked, "locked");
    daemon.push(Reply::Respond(Box::new(Response::Err { error })));

    let failure = daemon.client().send(&Request::Relock).unwrap_err();

    assert!(
        matches!(&failure, DaemonError::Rejected(e) if e.code == ErrorCode::LooseningLocked),
        "{failure:?}"
    );
}

#[test]
fn a_reply_from_another_protocol_version_names_that_version() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::Raw(
        b"{\"protocolVersion\":99,\"body\":{}}\n".to_vec(),
    ));

    let failure = daemon.client().send(&Request::Status).unwrap_err();

    assert!(
        matches!(failure, DaemonError::VersionMismatch { daemon: 99 }),
        "{failure:?}"
    );
}

#[test]
fn a_reply_over_the_size_limit_is_a_protocol_error() {
    let daemon = FakeDaemon::start();
    let mut oversize = vec![b'x'; MAX_RESPONSE_BYTES + 10];
    oversize.push(b'\n');
    daemon.push(Reply::Raw(oversize));
    // Streaming a megabyte through the fake can outlast the real budget on a loaded machine.
    let client = DaemonClient {
        budget: OVERSIZE_BUDGET,
        ..daemon.client()
    };

    let failure = client.send(&Request::Status).unwrap_err();

    assert!(matches!(failure, DaemonError::Protocol(_)), "{failure:?}");
}

#[test]
fn a_reply_that_is_not_json_is_a_protocol_error() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::Raw(b"hello\n".to_vec()));

    let failure = daemon.client().send(&Request::Status).unwrap_err();

    assert!(matches!(failure, DaemonError::Protocol(_)), "{failure:?}");
}

#[test]
fn a_daemon_that_hangs_up_mid_reply_is_not_responding() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::HangUp);

    let failure = daemon.client().send(&Request::Status).unwrap_err();

    assert!(
        matches!(&failure, DaemonError::NotResponding(e) if e.kind() == io::ErrorKind::UnexpectedEof),
        "{failure:?}"
    );
}

#[test]
fn a_missing_socket_means_not_installed() {
    let dir = tempfile::tempdir().unwrap();
    let client = DaemonClient::new(dir.path().join("absent.sock"));

    let failure = client.send(&Request::Status).unwrap_err();

    assert!(matches!(failure, DaemonError::NotInstalled), "{failure:?}");
}

#[test]
fn a_socket_file_nobody_listens_on_is_not_responding() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("stale.sock");
    drop(std::os::unix::net::UnixListener::bind(&path).unwrap());

    let failure = DaemonClient::new(path).send(&Request::Status).unwrap_err();

    assert!(
        matches!(&failure, DaemonError::NotResponding(e) if e.kind() == io::ErrorKind::ConnectionRefused),
        "{failure:?}"
    );
}

#[test]
fn a_socket_in_a_directory_the_user_cannot_enter_belongs_to_another_user() {
    use std::os::unix::fs::PermissionsExt;

    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("private.sock");
    let _listener = std::os::unix::net::UnixListener::bind(&path).unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o000)).unwrap();
    let still_readable = std::fs::read_dir(dir.path()).is_ok();

    let failure = DaemonClient::new(path).send(&Request::Status);

    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    if still_readable {
        return;
    }
    assert!(
        matches!(failure, Err(DaemonError::OtherUser)),
        "{failure:?}"
    );
}

#[test]
fn a_silent_daemon_times_out_within_the_budget() {
    let daemon = FakeDaemon::start();
    daemon.push(Reply::Silent);
    let client = DaemonClient {
        budget: Duration::from_millis(200),
        ..daemon.client()
    };
    let started = Instant::now();

    let failure = client.send(&Request::Status).unwrap_err();

    assert!(
        matches!(&failure, DaemonError::NotResponding(e) if matches!(e.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut)),
        "{failure:?}"
    );
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn the_socket_defaults_to_the_system_path() {
    let client = DaemonClient::from_lookup(|_| None);
    assert_eq!(client.socket.to_str(), Some("/var/run/focusd.sock"));
}

#[test]
fn focusd_socket_overrides_the_default_unless_empty() {
    let set = DaemonClient::from_lookup(|_| Some("/tmp/x.sock".into()));
    assert_eq!(set.socket.to_str(), Some("/tmp/x.sock"));
    let empty = DaemonClient::from_lookup(|_| Some(String::new().into()));
    assert_eq!(empty.socket.to_str(), Some("/var/run/focusd.sock"));
}
