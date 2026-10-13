use std::thread;

use super::*;
use crate::ipc::{DaemonStatus, ErrorCode, IpcError, Request, Response};
use crate::test_support::{now_at, sample_status};

fn expired() -> Instant {
    Instant::now()
        .checked_sub(Duration::from_millis(1))
        .unwrap()
}

fn status() -> DaemonStatus {
    sample_status(&now_at(2026, 10, 12, 9, 0))
}

#[test]
fn a_spent_budget_fails_both_directions_without_touching_the_socket() {
    let (stream, _peer) = UnixStream::pair().unwrap();
    let mut deadline = Deadline::until(&stream, expired());

    let write = deadline.write(b"x").unwrap_err();
    let read = deadline.read(&mut [0; 1]).unwrap_err();

    assert_eq!(write.kind(), io::ErrorKind::TimedOut);
    assert_eq!(read.kind(), io::ErrorKind::TimedOut);
}

#[test]
fn a_live_budget_lets_data_through() {
    let (stream, mut peer) = UnixStream::pair().unwrap();
    peer.write_all(b"hello").unwrap();
    let mut buf = [0_u8; 5];

    Deadline::within(&stream, Duration::from_secs(2))
        .read_exact(&mut buf)
        .unwrap();

    assert_eq!(&buf, b"hello");
}

#[test]
fn a_silent_peer_times_out() {
    let (stream, _peer) = UnixStream::pair().unwrap();
    let error = Deadline::within(&stream, Duration::from_millis(100))
        .read(&mut [0_u8; 1])
        .unwrap_err();
    assert!(matches!(
        error.kind(),
        io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut
    ));
}

#[test]
fn the_read_gets_only_what_the_write_left_of_the_shared_expiry() {
    let (stream, _peer) = UnixStream::pair().unwrap();
    let expires = Instant::now() + Duration::from_millis(150);

    Deadline::until(&stream, expires).write_all(b"x").unwrap();
    thread::sleep(Duration::from_millis(200));
    let read = Deadline::until(&stream, expires)
        .read(&mut [0; 1])
        .unwrap_err();

    assert_eq!(read.kind(), io::ErrorKind::TimedOut);
}

#[test]
fn a_trickling_peer_cannot_extend_the_budget() {
    let (mut client, server) = UnixStream::pair().unwrap();
    let trickle = thread::spawn(move || {
        for _ in 0..40 {
            if client.write_all(b"x").is_err() {
                break;
            }
            thread::sleep(Duration::from_millis(25));
        }
    });
    let mut reader = Deadline::within(&server, Duration::from_millis(200));
    let started = Instant::now();
    let mut sink = [0_u8; 1];
    let outcome = loop {
        if let Err(error) = reader.read(&mut sink) {
            break error;
        }
    };
    assert!(started.elapsed() < Duration::from_millis(600), "{outcome}");
    drop(server);
    trickle.join().unwrap();
}

fn answer_once(stream: UnixStream, response: &Response) -> thread::JoinHandle<()> {
    let bytes = encode(response).unwrap();
    thread::spawn(move || {
        let request: Request = read_message(&stream).unwrap();
        assert_eq!(request, Request::Status);
        (&stream).write_all(&bytes).unwrap();
    })
}

#[test]
fn an_exchange_returns_the_reply_to_the_request() {
    let (client, server) = UnixStream::pair().unwrap();
    let reply = Response::Ok { status: status() };
    let peer = answer_once(server, &reply);

    let received: Response = exchange(&client, &Request::Status, Duration::from_secs(2)).unwrap();

    assert_eq!(received, reply);
    peer.join().unwrap();
}

#[test]
fn an_exchange_returns_an_error_reply_as_a_message() {
    let (client, server) = UnixStream::pair().unwrap();
    let error = IpcError::new(ErrorCode::Internal, "broken");
    let peer = answer_once(
        server,
        &Response::Err {
            error: error.clone(),
        },
    );

    let received: Response = exchange(&client, &Request::Status, Duration::from_secs(2)).unwrap();

    assert_eq!(received, Response::Err { error });
    peer.join().unwrap();
}

#[test]
fn an_exchange_with_a_silent_peer_gives_up_within_the_budget() {
    let (client, _server) = UnixStream::pair().unwrap();
    let started = Instant::now();

    let failure =
        exchange::<Request, Response>(&client, &Request::Status, Duration::from_millis(150))
            .unwrap_err();

    assert!(started.elapsed() < Duration::from_secs(2), "{failure}");
    assert!(
        matches!(&failure, ExchangeError::Codec(IpcCodecError::Io(_))),
        "{failure:?}"
    );
}

#[test]
fn an_exchange_shares_one_budget_between_the_write_and_the_read() {
    let (client, server) = UnixStream::pair().unwrap();
    let peer = thread::spawn(move || {
        let _: Request = read_message(&server).unwrap();
        thread::sleep(Duration::from_millis(400));
    });
    let started = Instant::now();

    let failure =
        exchange::<Request, Response>(&client, &Request::Status, Duration::from_millis(200))
            .unwrap_err();

    assert!(started.elapsed() < Duration::from_millis(380), "{failure}");
    peer.join().unwrap();
}

#[test]
fn a_reply_from_another_protocol_version_is_named() {
    let (client, mut server) = UnixStream::pair().unwrap();
    server
        .write_all(b"{\"protocolVersion\":99,\"body\":{}}\n")
        .unwrap();

    let failure = exchange::<Request, Response>(&client, &Request::Status, Duration::from_secs(2))
        .unwrap_err();

    assert!(
        matches!(
            failure,
            ExchangeError::Codec(IpcCodecError::VersionMismatch { found: 99, .. })
        ),
        "{failure:?}"
    );
}

#[test]
fn a_reply_over_the_size_limit_is_refused() {
    let (client, mut server) = UnixStream::pair().unwrap();
    let writer = thread::spawn(move || {
        let oversize = vec![b'x'; Response::MAX_BYTES + 10];
        drop(server.write_all(&oversize));
    });

    let failure = exchange::<Request, Response>(&client, &Request::Status, Duration::from_secs(2))
        .unwrap_err();

    assert!(
        matches!(
            failure,
            ExchangeError::Codec(IpcCodecError::TooLarge { .. })
        ),
        "{failure:?}"
    );
    drop(client);
    writer.join().unwrap();
}
