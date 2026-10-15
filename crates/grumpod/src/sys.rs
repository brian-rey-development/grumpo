//! The only module here that talks to libc, because std has no way to retarget the standard
//! descriptors. Peer credentials and the uid live in `focus_core::ipc::peer`, which the
//! clients need as well.

use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;

pub use focus_core::ipc::peer::effective_uid;

/// Points stdout and stderr at `file`, so launchd's log descriptors follow a rotated log.
pub fn redirect_stdio(file: &File) -> io::Result<()> {
    for target in [libc::STDOUT_FILENO, libc::STDERR_FILENO] {
        // SAFETY: `file` stays open for the whole call and the standard descriptors exist for
        // the life of the process, so dup2 only retargets descriptor numbers we own.
        if unsafe { libc::dup2(file.as_raw_fd(), target) } < 0 {
            return Err(io::Error::last_os_error());
        }
    }
    Ok(())
}
