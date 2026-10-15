use std::io;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;

/// The uid of the process on the other end of `stream`, as the kernel recorded it at connect time.
#[cfg(not(any(target_os = "linux", target_os = "android")))]
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

/// Linux has no `getpeereid`, but `SO_PEERCRED` reports the same credentials.
#[cfg(any(target_os = "linux", target_os = "android"))]
pub fn peer_uid(stream: &UnixStream) -> io::Result<u32> {
    let mut credentials = libc::ucred {
        pid: 0,
        uid: 0,
        gid: 0,
    };
    let mut length =
        libc::socklen_t::try_from(std::mem::size_of::<libc::ucred>()).map_err(io::Error::other)?;
    // SAFETY: the descriptor is valid for the borrow of `stream`, `credentials` is a live
    // `ucred` with `length` set to its size, and getsockopt writes at most that many bytes
    // before returning.
    let status = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_SOCKET,
            libc::SO_PEERCRED,
            (&raw mut credentials).cast(),
            &raw mut length,
        )
    };
    if status == 0 {
        Ok(credentials.uid)
    } else {
        Err(io::Error::last_os_error())
    }
}

pub fn effective_uid() -> u32 {
    // SAFETY: geteuid takes no arguments, has no preconditions and cannot fail.
    unsafe { libc::geteuid() }
}
