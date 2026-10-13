//! Detects media playback from the system power assertions. Players hold a "prevent display
//! sleep" assertion while a video or call is on screen. Only assertions that belong to the
//! frontmost app count, so a video playing in a background window never keeps a different
//! app's session alive.

use super::power_assertions::{self, Assertion};
use super::process_tree;
use crate::platform::PlatformError;

const DISPLAY_SLEEP_KINDS: [&str; 2] = ["PreventUserIdleDisplaySleep", "NoDisplaySleepAssertion"];

pub(super) fn media_playing(frontmost_pid: i32) -> Result<bool, PlatformError> {
    let assertions = power_assertions::snapshot()?;
    Ok(frontmost_holds_display_assertion(
        &assertions,
        frontmost_pid,
        process_tree::works_for,
    ))
}

/// `works_for(pid, app)` answers whether `pid` is a helper of `app`. It runs only for
/// display-sleep assertions that are on, because resolving it costs system calls.
fn frontmost_holds_display_assertion(
    assertions: &[Assertion],
    frontmost_pid: i32,
    works_for: impl Fn(i32, i32) -> bool,
) -> bool {
    assertions
        .iter()
        .filter(|assertion| assertion.level > 0)
        .filter(|assertion| DISPLAY_SLEEP_KINDS.contains(&assertion.kind.as_str()))
        .any(|assertion| {
            [Some(assertion.holder_pid), assertion.on_behalf_of_pid]
                .into_iter()
                .flatten()
                .any(|pid| pid == frontmost_pid || works_for(pid, frontmost_pid))
        })
}

#[cfg(test)]
mod tests;
