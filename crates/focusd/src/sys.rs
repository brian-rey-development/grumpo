//! The only module that talks to libc, because std exposes neither peer credentials, the uid
//! nor a way to retarget the standard descriptors.

use std::fs::File;
use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

/// The uid of the process on the other end of `stream`, as the kernel recorded it at connect time.
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    let mut uid: libc::uid_t = 0;
    let mut gid: libc::gid_t = 0;
    // SAFETY: the descriptor is valid for the borrow of `stream`, and both out pointers
    // reference live locals that getpeereid fills in before returning.
    let status = unsafe { libc::getpeereid(stream.as_raw_fd(), &raw mut uid, &raw mut gid) };
    if status == 0 {
        Ok(uid)
    } else {
        Err(io::Error::last_os_error())
    }
}

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

pub fn effective_uid() -> u32 {
    // SAFETY: geteuid takes no arguments, has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}
