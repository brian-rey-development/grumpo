//! Who stands behind a Unix socket, as the kernel recorded it, shared by the daemon (to vet
//! its clients) and by the clients (to vet the daemon).

use std::io;
use std::os::unix::net::UnixStream;

use thiserror::Error;

#[expect(
    unsafe_code,
    reason = "peer credentials and the effective uid have no stable std API"
)]
mod os;

pub use os::{effective_uid, peer_uid};

pub const ROOT_UID: u32 = 0;

#[derive(Debug, Error)]
pub enum ServerIdentityError {
    #[error("could not tell which process serves the socket: {0}")]
    Unverifiable(io::Error),
    #[error("the socket is served by user {uid}, who is neither root nor you")]
    Impostor { uid: u32 },
}

/// Checks that the process serving `stream` is root or the user running this process.
///
/// The socket lives in a directory that members of the `daemon` group can write, so one of
/// them could delete the real socket and bind a look-alike in its place. Root is the installed
/// daemon. Our own uid is the development and test setups, where the daemon runs unprivileged
/// beside its client, and it costs nothing because a process with our uid can already do to us
/// whatever a fake daemon could. Any other uid is refused, and so is a lookup that fails,
/// because a peer that cannot be identified cannot be trusted.
pub fn verify_server(stream: &UnixStream) -> Result<(), ServerIdentityError> {
    judge_server(peer_uid(stream), effective_uid())
}

fn judge_server(peer: io::Result<u32>, own_uid: u32) -> Result<(), ServerIdentityError> {
    match peer {
        Ok(uid) if uid == ROOT_UID || uid == own_uid => Ok(()),
        Ok(uid) => Err(ServerIdentityError::Impostor { uid }),
        Err(source) => Err(ServerIdentityError::Unverifiable(source)),
    }
}

#[cfg(test)]
mod tests;
