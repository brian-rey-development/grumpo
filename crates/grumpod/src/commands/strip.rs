use std::collections::BTreeSet;
use std::io;

use focus_core::hosts;

use crate::hosts_writer::{HostsSink, HostsWrite};

/// Removes the managed block (current and legacy), and says whether the file changed and
/// whether the resolver caches need a flush.
pub fn strip_hosts(hosts_file: &impl HostsSink) -> io::Result<HostsWrite> {
    let existing = match hosts_file.read() {
        Ok(content) => content,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(HostsWrite::UNCHANGED),
        Err(error) => return Err(error),
    };
    let stripped = hosts::render_bytes(&existing, &BTreeSet::new());
    if stripped == existing {
        return Ok(HostsWrite::UNCHANGED);
    }
    let flush = hosts_file.write(&stripped)?;
    Ok(HostsWrite {
        changed: true,
        flush,
    })
}

#[cfg(test)]
mod tests;
