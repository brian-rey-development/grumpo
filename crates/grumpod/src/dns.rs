use std::io;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::sync::mpsc::{self, SyncSender, TrySendError};
use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
const REAPER_THREAD: &str = "dns-reaper";
const FLUSH_THREAD: &str = "dns-flush";
const QUEUED_RUNS: usize = 1;
const POLL_INTERVAL: Duration = Duration::from_millis(25);

const FLUSH_COMMANDS: [(&str, &[&str]); 2] = [
    ("/usr/bin/dscacheutil", &["-flushcache"]),
    ("/usr/bin/killall", &["-HUP", "mDNSResponder"]),
];

/// A resolver cache flush that follows a hosts change. It can take seconds, so the caller
/// hands it to a `FlushWorker` after releasing the service lock. Only a one-shot command with
/// nothing else to serve runs it inline.
#[must_use]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PendingFlush(bool);

impl PendingFlush {
    pub const NONE: Self = Self(false);
    pub const NEEDED: Self = Self(true);

    pub fn run(self) {
        if self.0 {
            flush_resolver_caches();
        }
    }
}

/// Runs resolver flushes on one long-lived thread, so no caller waits for the seconds a flush
/// can take. The queue holds a single pending run: a request that arrives while a flush is
/// running and another is already waiting adds nothing, because the waiting run starts after
/// the hosts change that request announces and so covers it.
#[derive(Debug, Clone)]
pub struct FlushWorker {
    queue: SyncSender<()>,
}

impl FlushWorker {
    pub fn spawn() -> io::Result<Self> {
        Self::with_action(|| PendingFlush::NEEDED.run())
    }

    pub(crate) fn with_action(mut action: impl FnMut() + Send + 'static) -> io::Result<Self> {
        let (queue, requests) = mpsc::sync_channel(QUEUED_RUNS);
        thread::Builder::new()
            .name(FLUSH_THREAD.to_owned())
            .spawn(move || {
                while requests.recv().is_ok() {
                    action();
                }
            })?;
        Ok(Self { queue })
    }

    /// Returns at once. The flush runs in the background if `flush` asks for one.
    pub fn submit(&self, flush: PendingFlush) {
        if flush == PendingFlush::NONE {
            return;
        }
        if let Err(TrySendError::Disconnected(())) = self.queue.try_send(()) {
            warn!("the resolver flush thread has stopped, the caches will expire on their own");
        }
    }
}

/// Makes the resolver notice a hosts change. A failure is logged and never undoes the write,
/// since the file is already correct and the caches expire on their own.
fn flush_resolver_caches() {
    for (program, args) in FLUSH_COMMANDS {
        if let Err(error) = run_with_timeout(program, args, FLUSH_TIMEOUT) {
            warn!(program, %error, "resolver cache flush step failed");
        }
    }
}

fn run_with_timeout(program: &str, args: &[&str], timeout: Duration) -> io::Result<()> {
    let child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    wait_for(child, program, timeout, Child::try_wait)
}

/// Polls `child` until it exits or `timeout` passes. Every way out that leaves it running
/// kills it, including a failed poll, because dropping a `Child` neither kills nor reaps it.
fn wait_for(
    mut child: Child,
    program: &str,
    timeout: Duration,
    mut poll: impl FnMut(&mut Child) -> io::Result<Option<ExitStatus>>,
) -> io::Result<()> {
    let deadline = Instant::now() + timeout;
    loop {
        match poll(&mut child) {
            Ok(Some(status)) => return exit_result(status),
            Ok(None) if Instant::now() < deadline => thread::sleep(POLL_INTERVAL),
            Ok(None) => {
                kill_and_reap(child, program);
                return Err(io::Error::new(io::ErrorKind::TimedOut, "command timed out"));
            }
            Err(error) => {
                kill_and_reap(child, program);
                return Err(error);
            }
        }
    }
}

fn exit_result(status: ExitStatus) -> io::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(io::Error::other(format!("exited with {status}")))
    }
}

/// Kills the command and leaves reaping it to another thread, because a process stuck in the
/// kernel can ignore the kill for a while and the caller must not wait for it.
fn kill_and_reap(mut child: Child, program: &str) {
    if let Err(error) = child.kill() {
        warn!(program, %error, "could not kill the timed-out command");
    }
    let program = program.to_owned();
    let reaper = thread::Builder::new()
        .name(REAPER_THREAD.to_owned())
        .spawn(move || {
            if let Err(error) = child.wait() {
                warn!(program, %error, "could not reap the killed command");
            }
        });
    if let Err(error) = reaper {
        warn!(%error, "could not start a reaper thread, the command stays a zombie");
    }
}

#[cfg(test)]
mod tests;
