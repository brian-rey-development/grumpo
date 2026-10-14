use std::fs;
use std::io::{BufReader, Write};
use std::os::unix::fs::{MetadataExt, PermissionsExt};
use std::os::unix::net::{UnixListener, UnixStream};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use focus_core::ipc::{
    decode, encode, read_frame, DaemonStatus, ErrorCode, Request, Response, MAX_REQUEST_BYTES,
    MAX_RESPONSE_BYTES, PROTOCOL_VERSION,
};
use focus_core::{Exhausted, UnlockState, ValidatedRules};

use super::{Server, ServerError};
use crate::dns::PendingFlush;
use crate::service::{Daemon, Handled};
use crate::shared::Shared;
use crate::sys;
use focus_core::test_support::now_at;

struct FakeDaemon;

impl Daemon for FakeDaemon {
    fn handle(&mut self, _: Request) -> Handled {
        let now = now_at(2026, 10, 12, 9, 0);
        let status = DaemonStatus::build(
            &ValidatedRules::default(),
            &UnlockState::new(&now),
            &Exhausted::new(&now),
            &now,
        );
        Handled::new(Response::Ok { status })
    }

    fn reconcile(&mut self) -> PendingFlush {
        PendingFlush::NONE
    }
}

struct Running {
    _dir: tempfile::TempDir,
    socket: PathBuf,
}

fn start_for(owner_uid: u32) -> Running {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("s.sock");
    let server = Server::bind(&socket, owner_uid).unwrap();
    let shared = Shared::new(FakeDaemon);
    thread::spawn(move || {
        server.serve(&shared);
    });
    Running { _dir: dir, socket }
}

fn start() -> Running {
    start_for(sys::effective_uid())
}

fn exchange(socket: &Path, bytes: &[u8]) -> Option<Response> {
    let mut stream = UnixStream::connect(socket).unwrap();
    let _ = stream.write_all(bytes);
    let mut reader = BufReader::new(stream);
    let frame = read_frame(&mut reader, MAX_RESPONSE_BYTES).ok()?;
    Some(decode(&frame).unwrap())
}

fn call(socket: &Path, request: &Request) -> Response {
    exchange(socket, &encode(request).unwrap()).expect("a response")
}

fn error_code(response: Response) -> ErrorCode {
    response.into_result().unwrap_err().code
}

#[test]
fn answers_a_valid_request() {
    let server = start();
    assert!(call(&server.socket, &Request::Status).into_result().is_ok());
}

#[test]
fn serves_many_clients_in_a_row() {
    let server = start();
    for _ in 0..20 {
        assert!(call(&server.socket, &Request::Status).into_result().is_ok());
    }
}

#[test]
fn a_version_mismatch_gets_an_enveloped_error() {
    let server = start();
    let response = exchange(
        &server.socket,
        b"{\"protocolVersion\":99,\"body\":{\"kind\":\"status\"}}\n",
    );
    assert_eq!(error_code(response.unwrap()), ErrorCode::VersionMismatch);
}

#[test]
fn a_client_on_the_previous_protocol_is_told_to_update() {
    let server = start();
    let previous = format!(
        "{{\"protocolVersion\":{},\"body\":{{\"kind\":\"status\"}}}}\n",
        PROTOCOL_VERSION - 1
    );
    let response = exchange(&server.socket, previous.as_bytes());
    assert_eq!(error_code(response.unwrap()), ErrorCode::VersionMismatch);
}

#[test]
fn malformed_requests_get_a_protocol_error() {
    let server = start();
    let unknown_kind =
        format!("{{\"protocolVersion\":{PROTOCOL_VERSION},\"body\":{{\"kind\":\"nope\"}}}}\n");
    for bytes in [&b"not json\n"[..], unknown_kind.as_bytes(), b"\n"] {
        let response = exchange(&server.socket, bytes).unwrap();
        assert_eq!(error_code(response), ErrorCode::Protocol);
    }
}

#[test]
fn an_oversized_frame_is_rejected() {
    let server = start();
    let response = exchange(&server.socket, &vec![b'a'; MAX_REQUEST_BYTES + 1024]);
    assert_eq!(error_code(response.unwrap()), ErrorCode::Protocol);
    assert!(call(&server.socket, &Request::Status).into_result().is_ok());
}

#[test]
fn a_silent_client_cannot_block_the_next_one_for_long() {
    let server = start();
    let _silent = UnixStream::connect(&server.socket).unwrap();
    let started = Instant::now();
    assert!(call(&server.socket, &Request::Status).into_result().is_ok());
    assert!(started.elapsed() < Duration::from_secs(5));
}

#[test]
fn other_users_get_no_answer() {
    let server = start_for(sys::effective_uid().wrapping_add(1));
    assert!(
        sys::effective_uid() != 0,
        "this test needs an unprivileged user"
    );
    let response = exchange(&server.socket, &encode(&Request::Status).unwrap());
    assert!(response.is_none());
}

#[test]
fn the_socket_is_private_and_a_stale_one_is_replaced() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("s.sock");
    drop(UnixListener::bind(&socket).unwrap());
    assert!(socket.exists());
    let _server = Server::bind(&socket, sys::effective_uid()).unwrap();
    assert_eq!(
        fs::metadata(&socket).unwrap().permissions().mode() & 0o777,
        0o600
    );
}

#[test]
fn a_regular_file_at_the_socket_path_is_never_deleted() {
    let dir = tempfile::tempdir().unwrap();
    let path = dir.path().join("s.sock");
    fs::write(&path, "precious").unwrap();
    let error = Server::bind(&path, sys::effective_uid()).unwrap_err();
    assert!(matches!(error, ServerError::NotASocket(_)));
    assert_eq!(fs::read_to_string(&path).unwrap(), "precious");
}

#[test]
fn a_non_root_server_leaves_the_socket_with_the_user_that_bound_it() {
    let dir = tempfile::tempdir().unwrap();
    let socket = dir.path().join("s.sock");
    let other = sys::effective_uid().wrapping_add(1);

    let _server = Server::bind(&socket, other).unwrap();

    let owner = fs::metadata(&socket).unwrap().uid();
    let expected = if sys::effective_uid() == 0 {
        other
    } else {
        sys::effective_uid()
    };
    assert_eq!(owner, expected);
}
