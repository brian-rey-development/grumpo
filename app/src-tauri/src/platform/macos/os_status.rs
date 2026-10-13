//! The `OSStatus` and `AppleScript` error numbers the adapters interpret, in one place.

pub(super) type OsStatus = i32;

pub(super) const NO_ERR: OsStatus = 0;
pub(super) const PROC_NOT_FOUND: OsStatus = -600;
pub(super) const ERR_INVALID_INDEX: OsStatus = -1719;
pub(super) const ERR_CANT_GET: OsStatus = -1728;
/// `errAEEventNotPermitted`, the user refused Automation.
pub(super) const ERR_NOT_PERMITTED: OsStatus = -1743;
/// `errAEEventWouldRequireUserConsent`, the user has not been asked yet.
pub(super) const ERR_WOULD_REQUIRE_CONSENT: OsStatus = -1744;
