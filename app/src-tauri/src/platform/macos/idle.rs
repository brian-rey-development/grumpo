//! Time since the last keyboard, mouse or trackpad event in the login session.

use std::time::Duration;

use objc2_core_graphics::{CGEventSource, CGEventSourceStateID, CGEventType};

use crate::platform::PlatformError;

/// `kCGAnyInputEventType`, which is `~0` and not a member of the `CGEventType` enum.
const ANY_INPUT_EVENT_TYPE: CGEventType = CGEventType(u32::MAX);

pub(super) fn idle_time() -> Result<Duration, PlatformError> {
    let seconds = CGEventSource::seconds_since_last_event_type(
        CGEventSourceStateID::CombinedSessionState,
        ANY_INPUT_EVENT_TYPE,
    );
    to_duration(seconds)
}

fn to_duration(seconds: f64) -> Result<Duration, PlatformError> {
    if !seconds.is_finite() {
        return Err(super::failed(
            "idle time",
            format!("invalid reading {seconds}"),
        ));
    }
    Duration::try_from_secs_f64(seconds.max(0.0))
        .map_err(|_| super::failed("idle time", format!("invalid reading {seconds}")))
}

#[cfg(test)]
mod tests;
