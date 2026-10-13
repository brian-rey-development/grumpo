//! Blocking messages over a Unix socket, where every read and write draws on one time budget.

use std::io::{self, Read, Write};
use std::os::unix::net::UnixStream;
use std::time::{Duration, Instant};

use thiserror::Error;

use super::{decode, encode, read_frame, IpcCodecError, Message};

/// A stream whose reads and writes all count against one expiry.
///
/// A plain socket timeout restarts on every call, so a peer trickling one byte at a time
/// could hold the caller for as long as it likes. Here the time left shrinks, and the expiry
/// is fixed by the caller, so a write and the read that follows it can share it.
#[derive(Debug)]
pub struct Deadline<'a> {
    stream: &'a UnixStream,
    expires: Instant,
}

impl<'a> Deadline<'a> {
    pub fn until(stream: &'a UnixStream, expires: Instant) -> Self {
        Self { stream, expires }
    }

    pub fn within(stream: &'a UnixStream, budget: Duration) -> Self {
        Self::until(stream, Instant::now() + budget)
    }

    fn remaining(&self) -> io::Result<Duration> {
        let left = self.expires.saturating_duration_since(Instant::now());
        if left.is_zero() {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "the time budget ran out",
            ));
        }
        Ok(left)
    }
}

impl Read for Deadline<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.stream.set_read_timeout(Some(self.remaining()?))?;
        self.stream.read(buf)
    }
}

impl Write for Deadline<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stream.set_write_timeout(Some(self.remaining()?))?;
        self.stream.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[derive(Debug, Error)]
pub enum ExchangeError {
    #[error("could not send the request: {0}")]
    Send(io::Error),
    #[error(transparent)]
    Codec(#[from] IpcCodecError),
}

/// Reads one message of the largest size its type allows.
pub fn read_message<T: Message>(reader: impl Read) -> Result<T, IpcCodecError> {
    let mut reader = io::BufReader::new(reader);
    decode(&read_frame(&mut reader, T::MAX_BYTES)?)
}

/// Sends `request` and reads the reply, with both sharing one `budget` that starts here.
pub fn exchange<Q: Message, A: Message>(
    stream: &UnixStream,
    request: &Q,
    budget: Duration,
) -> Result<A, ExchangeError> {
    let expires = Instant::now() + budget;
    Deadline::until(stream, expires)
        .write_all(&encode(request)?)
        .map_err(ExchangeError::Send)?;
    Ok(read_message(Deadline::until(stream, expires))?)
}

#[cfg(test)]
mod tests;
