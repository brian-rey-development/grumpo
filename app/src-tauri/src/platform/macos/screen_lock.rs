#![expect(
    unsafe_code,
    reason = "typing the CF session dictionary needs an unchecked retained cast"
)]

//! Whether the login session's screen is locked, from the session dictionary.

use objc2_core_foundation::{CFBoolean, CFDictionary, CFRetained, CFString, CFType};
use objc2_core_graphics::CGSessionCopyCurrentDictionary;

use crate::platform::PlatformError;

const LOCKED_KEY: &str = "CGSSessionScreenIsLocked";

pub(super) fn screen_locked() -> Result<bool, PlatformError> {
    let session = CGSessionCopyCurrentDictionary()
        .ok_or_else(|| super::failed("screen lock", "no login session"))?;
    // SAFETY: every CF object is a CFType, and the value is checked with `downcast_ref`
    // before it is read as a boolean.
    let session = unsafe { CFRetained::cast_unchecked::<CFDictionary<CFString, CFType>>(session) };
    Ok(session
        .get(&CFString::from_static_str(LOCKED_KEY))
        .and_then(|value| value.downcast_ref::<CFBoolean>().map(CFBoolean::as_bool))
        .unwrap_or(false))
}
