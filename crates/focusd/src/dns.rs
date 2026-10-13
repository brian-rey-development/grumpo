use std::io;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use tracing::warn;

const FLUSH_TIMEOUT: Duration = Duration::from_secs(5);
const REAPER_THREAD: &str = "dns-reaper";
const POLL_INTERVAL: Duration = Duration::from_millis(25);

const FLUSH_COMMANDS: [(&str, &[&str]); 2] = [
    ("/usr/bin/dscacheutil", &["-flushcache"]),
    ("/usr/bin/killall", &["-HUP", "mDNSResponder"]),
];

/// A resolver cache flush that follows a hosts change. It can take seconds, so the caller
/// runs it only after releasing the service lock.
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
    let mut child = Command::new(program)
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?;
    let deadline = Instant::now() + timeout;
    loop {
        if let Some(status) = child.try_wait()? {
            return if status.success() {
                Ok(())
            } else {
                Err(io::Error::other(format!("exited with {status}")))
            };
        }
        if Instant::now() >= deadline {
            kill_and_reap(child, program);
            return Err(io::Error::new(io::ErrorKind::TimedOut, "command timed out"));
        }
        thread::sleep(POLL_INTERVAL);
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
