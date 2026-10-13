//! Child processes with a hard deadline, so no external tool can stall a probe.

use std::io::{self, ErrorKind, Read};
use std::process::{Child, Command, ExitStatus, Output, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::{Duration, Instant};

const POLL_INTERVAL: Duration = Duration::from_millis(10);
const REAP_GRACE: Duration = Duration::from_millis(500);
const DRAIN_GRACE: Duration = Duration::from_millis(500);
const READ_CHUNK_BYTES: usize = 8 * 1024;
/// Keeps a runaway tool from filling memory. The excess is read and thrown away so the
/// child never blocks on a full pipe.
pub(super) const MAX_OUTPUT_BYTES: usize = 64 * 1024;

#[derive(Debug)]
pub(super) enum RunError {
    Spawn(io::Error),
    Wait(io::Error),
    Timeout,
}

pub(super) fn run_bounded(command: &mut Command, timeout: Duration) -> Result<Output, RunError> {
    let mut child = command
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(RunError::Spawn)?;
    let (stdout, stderr) = match start_drains(&mut child) {
        Ok(drains) => drains,
        Err(error) => {
            kill_and_reap(child);
            return Err(RunError::Spawn(error));
        }
    };
    let status = wait_until(child, Instant::now() + timeout)?;
    Ok(Output {
        status,
        stdout: stdout.finish(),
        stderr: stderr.finish(),
    })
}

fn start_drains(child: &mut Child) -> io::Result<(Drain, Drain)> {
    Ok((drain(child.stdout.take())?, drain(child.stderr.take())?))
}

fn wait_until(mut child: Child, deadline: Instant) -> Result<ExitStatus, RunError> {
    loop {
        match child.try_wait() {
            Ok(Some(status)) => return Ok(status),
            Ok(None) if Instant::now() >= deadline => {
                kill_and_reap(child);
                return Err(RunError::Timeout);
            }
            Ok(None) => thread::sleep(POLL_INTERVAL),
            Err(error) => {
                kill_and_reap(child);
                return Err(RunError::Wait(error));
            }
        }
    }
}

/// Collects one pipe on its own thread while the caller polls the child.
struct Drain(Option<Receiver<Vec<u8>>>);

impl Drain {
    fn finish(self) -> Vec<u8> {
        let Some(receiver) = self.0 else {
            return Vec::new();
        };
        receiver.recv_timeout(DRAIN_GRACE).unwrap_or_else(|_| {
            tracing::warn!("child output was not closed in time, dropping it");
            Vec::new()
        })
    }
}

fn drain(pipe: Option<impl Read + Send + 'static>) -> io::Result<Drain> {
    let Some(pipe) = pipe else {
        return Ok(Drain(None));
    };
    let (sender, receiver) = mpsc::channel();
    thread::Builder::new()
        .name("focus-child-output".into())
        .spawn(move || {
            if sender.send(read_capped(pipe)).is_err() {
                tracing::debug!("child output finished after the caller gave up");
            }
        })?;
    Ok(Drain(Some(receiver)))
}

fn read_capped(mut pipe: impl Read) -> Vec<u8> {
    let mut kept = Vec::new();
    let mut chunk = [0_u8; READ_CHUNK_BYTES];
    loop {
        match pipe.read(&mut chunk) {
            Ok(0) => return kept,
            Ok(read) => {
                let room = MAX_OUTPUT_BYTES.saturating_sub(kept.len());
                kept.extend_from_slice(&chunk[..read.min(room)]);
            }
            Err(error) if error.kind() == ErrorKind::Interrupted => {}
            Err(error) => {
                tracing::debug!(%error, "child output pipe failed");
                return kept;
            }
        }
    }
}

/// Kills the child and waits briefly for it to exit. A process stuck in the kernel (for
/// example behind a consent prompt) may ignore SIGKILL for a while, so after the grace
/// period a detached thread reaps it later instead of blocking the caller.
fn kill_and_reap(mut child: Child) {
    if let Err(error) = child.kill() {
        tracing::debug!(%error, "child already gone before kill");
    }
    let grace_end = Instant::now() + REAP_GRACE;
    while Instant::now() < grace_end {
        if !matches!(child.try_wait(), Ok(None)) {
            return;
        }
        thread::sleep(POLL_INTERVAL);
    }
    tracing::warn!("child ignored kill, reaping in the background");
    thread::spawn(move || {
        if let Err(error) = child.wait() {
            tracing::warn!(%error, "failed to reap child process");
        }
    });
}

#[cfg(test)]
mod tests;
