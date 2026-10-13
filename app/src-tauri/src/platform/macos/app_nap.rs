#![expect(
    unsafe_code,
    reason = "NSProcessInfo.endActivity is unsafe-marked and the token crosses threads"
)]

//! Keeps macOS from throttling the process while its window is hidden. Without an activity
//! assertion App Nap stretches timers of a background app, which delays the 2 s sampler and
//! the status poller by seconds to minutes and makes tracking and enforcement unreliable.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_foundation::{NSActivityOptions, NSObjectProtocol, NSProcessInfo, NSString};

const REASON: &str = "Tracking screen time";

/// The activity returned by `NSProcessInfo`, ended when dropped.
pub(in crate::platform) struct ActivityToken(Retained<ProtocolObject<dyn NSObjectProtocol>>);

impl std::fmt::Debug for ActivityToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ActivityToken").finish_non_exhaustive()
    }
}

// SAFETY: the token is an opaque object that is only retained, released and handed back to
// the thread-safe `NSProcessInfo`. Nothing sends it any other message.
unsafe impl Send for ActivityToken {}
// SAFETY: see above, a shared reference offers no way to message the token.
unsafe impl Sync for ActivityToken {}

/// Starts the activity. System sleep stays allowed, so the Mac still sleeps when idle.
pub(in crate::platform) fn begin() -> ActivityToken {
    let activity = NSProcessInfo::processInfo().beginActivityWithOptions_reason(
        NSActivityOptions::UserInitiatedAllowingIdleSystemSleep,
        &NSString::from_str(REASON),
    );
    ActivityToken(activity)
}

impl Drop for ActivityToken {
    fn drop(&mut self) {
        // SAFETY: the token came from `beginActivityWithOptions:reason:` and is ended once.
        unsafe { NSProcessInfo::processInfo().endActivity(&self.0) };
    }
}

#[cfg(test)]
mod tests;
