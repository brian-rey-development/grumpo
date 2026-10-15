use std::fs::{self, File, OpenOptions};
use std::io;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::PathBuf;

use crate::sys;

const LOG_MODE: u32 = 0o644;

/// Keeps stderr pointed at the live log file across newsyslog rotations.
///
/// launchd opens the log once and the daemon inherits that descriptor, which after a rename
/// keeps writing into the archived file, and into an unlinked one once that is compressed.
/// Noticing the path now names a different file and reopening it keeps the logs flowing.
#[derive(Debug)]
pub struct RotatingLog {
    path: PathBuf,
    file: File,
}

impl RotatingLog {
    pub fn attach(path: PathBuf) -> io::Result<Self> {
        let log = Self::open(path)?;
        sys::redirect_stdio(&log.file)?;
        Ok(log)
    }

    /// Reopens the log if it was rotated and reports whether it did.
    pub fn reattach_if_rotated(&mut self) -> io::Result<bool> {
        self.reattach_with(sys::redirect_stdio)
    }

    /// A failed redirect leaves the old file in place, so the next call tries again.
    fn reattach_with(
        &mut self,
        redirect: impl FnOnce(&File) -> io::Result<()>,
    ) -> io::Result<bool> {
        if !self.is_rotated() {
            return Ok(false);
        }
        let fresh = Self::open(self.path.clone())?;
        redirect(&fresh.file)?;
        *self = fresh;
        Ok(true)
    }

    /// Refuses a symlink at the log path, so root never appends to a file an attacker chose.
    fn open(path: PathBuf) -> io::Result<Self> {
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .mode(LOG_MODE)
            .custom_flags(libc::O_NOFOLLOW)
            .open(&path)?;
        Ok(Self { path, file })
    }

    fn is_rotated(&self) -> bool {
        match (fs::metadata(&self.path), self.file.metadata()) {
            (Ok(named), Ok(held)) => (named.dev(), named.ino()) != (held.dev(), held.ino()),
            _ => true,
        }
    }
}

#[cfg(test)]
mod tests;
