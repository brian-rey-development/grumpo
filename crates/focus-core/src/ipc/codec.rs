use std::io::{self, BufRead, Read};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use thiserror::Error;

use super::{Envelope, Request, Response, PROTOCOL_VERSION};

pub const MAX_REQUEST_BYTES: usize = 256 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
const FRAME_TERMINATOR: u8 = b'\n';

#[derive(Debug, Error)]
pub enum IpcCodecError {
    #[error("message is larger than the {max} byte limit")]
    TooLarge { max: usize },
    #[error("the message ended before its terminating newline")]
    MissingTerminator,
    #[error("the peer speaks protocol version {found}, this side speaks {expected}")]
    VersionMismatch { expected: u32, found: u32 },
    #[error("invalid JSON message: {0}")]
    Json(#[from] serde_json::Error),
    #[error("could not read the message: {0}")]
    Io(#[from] io::Error),
}

/// A message type that travels in an [`Envelope`], with the largest frame it may occupy.
pub trait Message: Serialize + DeserializeOwned {
    const MAX_BYTES: usize;
}

impl Message for Request {
    const MAX_BYTES: usize = MAX_REQUEST_BYTES;
}

impl Message for Response {
    const MAX_BYTES: usize = MAX_RESPONSE_BYTES;
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct VersionProbe {
    protocol_version: u32,
}

/// Serializes `message` in an envelope as one JSON line, newline included, within its size limit.
pub fn encode<T: Message>(message: &T) -> Result<Vec<u8>, IpcCodecError> {
    let mut bytes = serde_json::to_vec(&Envelope::new(message))?;
    bytes.push(FRAME_TERMINATOR);
    ensure_within_limit(bytes.len(), T::MAX_BYTES)?;
    Ok(bytes)
}

/// Parses one frame, with or without its trailing newline.
///
/// The protocol version is read on its own first, so a peer from another version gets
/// a version error instead of whatever its different body shape would fail with.
pub fn decode<T: Message>(frame: &[u8]) -> Result<T, IpcCodecError> {
    ensure_within_limit(frame.len(), T::MAX_BYTES)?;
    let probe: VersionProbe = serde_json::from_slice(frame)?;
    if probe.protocol_version != PROTOCOL_VERSION {
        return Err(IpcCodecError::VersionMismatch {
            expected: PROTOCOL_VERSION,
            found: probe.protocol_version,
        });
    }
    Ok(serde_json::from_slice::<Envelope<T>>(frame)?.body)
}

/// Reads one newline-terminated frame of at most `limit` bytes, newline included.
///
/// Reading stops one byte past the limit, so a peer that never sends a newline cannot
/// make the reader buffer more than that.
pub fn read_frame<R: BufRead>(reader: &mut R, limit: usize) -> Result<Vec<u8>, IpcCodecError> {
    let mut frame = Vec::new();
    let bound = u64::try_from(limit).map_or(u64::MAX, |bound| bound.saturating_add(1));
    reader
        .take(bound)
        .read_until(FRAME_TERMINATOR, &mut frame)?;
    ensure_within_limit(frame.len(), limit)?;
    if frame.last() != Some(&FRAME_TERMINATOR) {
        return Err(IpcCodecError::MissingTerminator);
    }
    Ok(frame)
}

fn ensure_within_limit(size: usize, max: usize) -> Result<(), IpcCodecError> {
    if size > max {
        return Err(IpcCodecError::TooLarge { max });
    }
    Ok(())
}

#[cfg(test)]
mod tests;
