use std::io::{self, Write};
use std::os::unix::net::UnixStream;
use std::time::Duration;

use focus_core::ipc::peer::ROOT_UID;
use focus_core::ipc::transport::{read_message, Deadline};
use focus_core::ipc::{encode, IpcCodecError, IpcError, Request, Response};
use tracing::{debug, warn};

use crate::dns::FlushWorker;
use crate::service::{Daemon, Handled};
use crate::shared::Shared;

const REQUEST_BUDGET: Duration = Duration::from_secs(2);
const RESPONSE_BUDGET: Duration = Duration::from_secs(2);

/// How the server learns who is on the other end, which tests replace so that the checks on
/// peers do not depend on the user running the suite.
pub type PeerLookup = fn(&UnixStream) -> io::Result<u32>;

/// Handles one connection: one request in, exactly one response out, then close. The resolver
/// flush the request calls for is queued on `flusher` after the response, never run here.
///
/// A panic while handling the request is deliberately not caught. It takes the process down,
/// launchd restarts it, and the state on disk is consistent because every save precedes the
/// adoption of what it saved.
pub fn serve<D: Daemon>(
    stream: &UnixStream,
    owner_uid: u32,
    peer_lookup: PeerLookup,
    service: &Shared<D>,
    flusher: &FlushWorker,
) {
    if !is_authorized(peer_lookup(stream), owner_uid) {
        return;
    }
    let Handled { response, flush } = respond(stream, service);
    if let Err(error) = write_response(stream, &response) {
        debug!(%error, "could not deliver the response, the client is gone");
    }
    flusher.submit(flush);
}

pub(super) fn is_authorized(peer: io::Result<u32>, owner_uid: u32) -> bool {
    match peer {
        Ok(uid) if uid == owner_uid || uid == ROOT_UID => true,
        Ok(uid) => {
            warn!(uid, "rejected a connection from an unauthorized user");
            false
        }
        Err(error) => {
            warn!(%error, "could not identify the connecting process, rejecting it");
            false
        }
    }
}

fn respond<D: Daemon>(stream: &UnixStream, service: &Shared<D>) -> Handled {
    match read_message::<Request>(Deadline::within(stream, REQUEST_BUDGET)) {
        Ok(request) => service.with(|daemon| daemon.handle(request)),
        Err(error) => Handled::new(rejected(error)),
    }
}

/// The text of a malformed request can echo what the peer sent, so only the error code is
/// logged above debug.
fn rejected(error: IpcCodecError) -> Response {
    let error = IpcError::from(error);
    warn!(code = ?error.code, "rejected a malformed request");
    debug!(detail = %error.message, "malformed request in full");
    Response::Err { error }
}

fn write_response(stream: &UnixStream, response: &Response) -> std::io::Result<()> {
    let bytes = encode(response).or_else(|error| {
        warn!(%error, "response could not be encoded");
        encode(&Response::Err {
            error: IpcError::internal("the response could not be encoded"),
        })
    });
    let bytes = bytes.map_err(std::io::Error::other)?;
    Deadline::within(stream, RESPONSE_BUDGET).write_all(&bytes)
}
