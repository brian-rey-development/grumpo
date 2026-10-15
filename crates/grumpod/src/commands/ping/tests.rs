use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::thread;

use focus_core::ipc::transport::read_message;
use focus_core::ipc::{encode, ErrorCode};
use focus_core::test_support::now_at;

use super::*;
use crate::state::Snapshot;

fn answer_once(listener: UnixListener, response: Response) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        assert_eq!(read_message::<Request>(&stream).unwrap(), Request::Status);
        (&stream).write_all(&encode(&response).unwrap()).unwrap();
    })
}

fn listener() -> (tempfile::TempDir, PathBuf, UnixListener) {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("s.sock");
    let listener = UnixListener::bind(&socket).unwrap();
    (dir, socket, listener)
}

#[test]
fn an_ok_status_is_a_successful_ping() {
    let (_dir, socket, listener) = listener();
    let now = now_at(2026, 10, 12, 9, 0);
    let status = Snapshot::fresh(&now).status(&now);
    let server = answer_once(listener, Response::Ok { status });
    assert!(ping(&socket).is_ok());
    server.join().unwrap();
}

#[test]
fn an_error_response_is_a_failed_ping() {
    let (_dir, socket, listener) = listener();
    let error = IpcError::new(ErrorCode::Internal, "broken");
    let server = answer_once(listener, Response::Err { error });
    let failure = ping(&socket).unwrap_err();
    assert!(matches!(failure, PingError::Rejected(_)), "{failure}");
    server.join().unwrap();
}

#[test]
fn nothing_listening_is_a_failed_ping() {
    let dir = tempfile::tempdir().unwrap();
    let failure = ping(&dir.path().join("none.sock")).unwrap_err();
    assert!(matches!(failure, PingError::Connect { .. }), "{failure}");
}

#[test]
fn a_peer_that_hangs_up_is_a_failed_ping() {
    let (_dir, socket, listener) = listener();
    let server = thread::spawn(move || drop(listener.accept().unwrap()));
    assert!(ping(&socket).is_err());
    server.join().unwrap();
}

#[test]
fn a_socket_served_by_a_stranger_is_refused_before_anything_is_sent() {
    let (_dir, socket, listener) = listener();
    let server = thread::spawn(move || {
        let (stream, _) = listener.accept().unwrap();
        stream.set_nonblocking(true).unwrap();
        let mut sent = Vec::new();
        let _ = (&stream).read_to_end(&mut sent);
        sent
    });

    let failure = ping_verified(&socket, |_| Err(ServerIdentityError::Impostor { uid: 502 }));

    assert!(
        matches!(failure, Err(PingError::Untrusted(_))),
        "{failure:?}"
    );
    assert_eq!(server.join().unwrap(), Vec::<u8>::new());
}

#[test]
fn a_socket_whose_server_cannot_be_identified_is_refused() {
    let (_dir, socket, listener) = listener();
    let server = thread::spawn(move || drop(listener.accept().unwrap()));

    let failure = ping_verified(&socket, |_| {
        Err(ServerIdentityError::Unverifiable(io::Error::other(
            "no credentials",
        )))
    });

    assert!(
        matches!(failure, Err(PingError::Untrusted(_))),
        "{failure:?}"
    );
    server.join().unwrap();
}
