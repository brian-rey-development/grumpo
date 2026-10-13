use std::fs;
use std::io;
use std::os::unix::fs::MetadataExt;
use std::path::PathBuf;

use crate::atomic_write::{write_atomic, Owner};
use crate::dns::PendingFlush;

const DEFAULT_MODE: u32 = 0o644;
const MODE_MASK: u32 = 0o7777;

/// What a hosts write did, so the caller can run the slow resolver flush outside its locks.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HostsWrite {
    pub changed: bool,
    pub flush: PendingFlush,
}

impl HostsWrite {
    pub const UNCHANGED: Self = Self {
        changed: false,
        flush: PendingFlush::NONE,
    };
}

pub trait HostsSink {
    /// The raw content, since a hosts file is not guaranteed to be UTF-8.
    fn read(&self) -> io::Result<Vec<u8>>;

    /// Replaces the hosts content unless it is already identical.
    fn write_if_changed(&self, content: &[u8]) -> io::Result<HostsWrite>;
}

#[derive(Debug, Clone)]
pub struct HostsFile {
    path: PathBuf,
    flush_dns: bool,
}

impl HostsFile {
    pub fn new(path: PathBuf, flush_dns: bool) -> Self {
        Self { path, flush_dns }
    }

    /// `/etc/hosts` is a symlink to `/private/etc/hosts`, and replacing the link itself
    /// would break it, so writes go to the file it resolves to.
    fn target(&self) -> io::Result<PathBuf> {
        match fs::canonicalize(&self.path) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(self.path.clone()),
            other => other,
        }
    }

    fn flush_after_write(&self) -> PendingFlush {
        if self.flush_dns {
            PendingFlush::NEEDED
        } else {
            PendingFlush::NONE
        }
    }
}

impl HostsSink for HostsFile {
    fn read(&self) -> io::Result<Vec<u8>> {
        fs::read(self.target()?)
    }

    fn write_if_changed(&self, content: &[u8]) -> io::Result<HostsWrite> {
        let target = self.target()?;
        let current = match fs::read(&target) {
            Ok(bytes) => Some(bytes),
            Err(error) if error.kind() == io::ErrorKind::NotFound => None,
            Err(error) => return Err(error),
        };
        if current.as_deref().unwrap_or_default() == content {
            return Ok(HostsWrite::UNCHANGED);
        }
        let (mode, owner) = match fs::metadata(&target) {
            Ok(metadata) => (metadata.mode() & MODE_MASK, Some(Owner::of(&target)?)),
            Err(_) => (DEFAULT_MODE, None),
        };
        write_atomic(&target, content, mode, owner)?;
        Ok(HostsWrite {
            changed: true,
            flush: self.flush_after_write(),
        })
    }
}

#[cfg(test)]
mod tests;
