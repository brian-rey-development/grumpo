#![expect(
    unsafe_code,
    reason = "proc_pidinfo and the private responsibility call are raw libSystem calls"
)]

//! Decides whether a helper process works for an app. Browsers run their media in helper
//! processes (`WebKit`'s GPU and `WebContent` services for Safari, renderer helpers for Chrome),
//! and the power assertion is held by the helper, not by the app in the foreground.

use std::ffi::{c_char, c_int, c_void};
use std::mem::size_of;
use std::sync::OnceLock;

/// Chains are short (helper, XPC service, app), and the bound keeps a cycle from looping.
const MAX_HOPS: usize = 8;
const LAUNCHD_PID: i32 = 1;
const RESPONSIBLE_SYMBOL: &[u8] = b"responsibility_get_pid_responsible_for_pid\0";

pub(super) trait ProcessTree {
    /// The process macOS holds accountable for `pid`, when it can say.
    fn responsible(&self, pid: i32) -> Option<i32>;
    fn parent(&self, pid: i32) -> Option<i32>;
}

pub(super) fn works_for(pid: i32, app_pid: i32) -> bool {
    belongs_to(&System, pid, app_pid)
}

fn belongs_to(tree: &impl ProcessTree, pid: i32, app_pid: i32) -> bool {
    let mut current = pid;
    for _ in 0..=MAX_HOPS {
        if current == app_pid {
            return true;
        }
        let next = tree
            .responsible(current)
            .filter(|responsible| *responsible != current)
            .or_else(|| tree.parent(current));
        match next {
            Some(next) if next > LAUNCHD_PID => current = next,
            _ => return false,
        }
    }
    false
}

struct System;

type ResponsibleFn = unsafe extern "C" fn(c_int) -> c_int;

impl ProcessTree for System {
    fn responsible(&self, pid: i32) -> Option<i32> {
        let function = responsible_function()?;
        // SAFETY: the symbol has the signature `pid_t f(pid_t)`, and any pid is valid input.
        let responsible = unsafe { function(pid) };
        (responsible > 0).then_some(responsible)
    }

    fn parent(&self, pid: i32) -> Option<i32> {
        // SAFETY: an all-zero proc_bsdinfo is a valid value for this plain C struct.
        let mut info: libc::proc_bsdinfo = unsafe { std::mem::zeroed() };
        let size = c_int::try_from(size_of::<libc::proc_bsdinfo>()).ok()?;
        // SAFETY: the buffer is `size` writable bytes, and the call fills at most that many.
        let written = unsafe {
            libc::proc_pidinfo(
                pid,
                libc::PROC_PIDTBSDINFO,
                0,
                (&raw mut info).cast::<c_void>(),
                size,
            )
        };
        (written == size)
            .then(|| i32::try_from(info.pbi_ppid).ok())
            .flatten()
    }
}

/// The call is private API, so it is looked up at runtime and its absence just disables
/// this strategy in favour of the parent chain.
fn responsible_function() -> Option<ResponsibleFn> {
    static FUNCTION: OnceLock<Option<ResponsibleFn>> = OnceLock::new();
    *FUNCTION.get_or_init(|| {
        // SAFETY: the name is NUL terminated and RTLD_DEFAULT searches loaded images.
        let symbol = unsafe {
            libc::dlsym(
                libc::RTLD_DEFAULT,
                RESPONSIBLE_SYMBOL.as_ptr().cast::<c_char>(),
            )
        };
        // SAFETY: a non-null result is the address of the function with this signature.
        (!symbol.is_null())
            .then(|| unsafe { std::mem::transmute::<*mut c_void, ResponsibleFn>(symbol) })
    })
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
