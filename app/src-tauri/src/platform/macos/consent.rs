//! Gate in front of `osascript`. Scripting a browser that the user has not approved makes
//! macOS pop a consent dialog from a background poll, so the state is checked first and the
//! script only runs once consent is granted.

use std::collections::BTreeMap;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use super::apple_events;
use super::os_status::{OsStatus, ERR_NOT_PERMITTED, ERR_WOULD_REQUIRE_CONSENT, NO_ERR};
use crate::concurrency::LockExt;
use crate::platform::{Browser, Permission, PlatformError};

/// Asking costs an Apple Event round trip and the tracker asks every few seconds.
const CACHE_TTL: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Consent {
    Granted,
    Unasked,
    Denied,
}

/// Anything unexpected is let through, so `osascript` reports the real error instead of a
/// guess made here.
fn consent_of(status: OsStatus) -> Consent {
    match status {
        ERR_NOT_PERMITTED => Consent::Denied,
        ERR_WOULD_REQUIRE_CONSENT => Consent::Unasked,
        NO_ERR => Consent::Granted,
        other => {
            tracing::debug!(
                status = other,
                "unexpected automation status, letting it run"
            );
            Consent::Granted
        }
    }
}

#[derive(Debug)]
struct Cache {
    entries: BTreeMap<Browser, (Instant, Consent)>,
}

static CACHE: Mutex<Cache> = Mutex::new(Cache {
    entries: BTreeMap::new(),
});

impl Cache {
    fn fresh(&self, browser: Browser, now: Instant) -> Option<Consent> {
        let (checked_at, consent) = self.entries.get(&browser)?;
        (now.saturating_duration_since(*checked_at) < CACHE_TTL).then_some(*consent)
    }
}

pub(super) fn require(browser: Browser) -> Result<(), PlatformError> {
    let consent = check(&CACHE, browser, Instant::now(), || {
        apple_events::automation_status(browser.bundle_id())
    });
    gate(browser, consent)
}

/// Drops the cached state, used after the user answered a prompt so the answer applies now.
pub(super) fn forget(browser: Browser) {
    let mut cache = CACHE.lock_or_recover();
    cache.entries.remove(&browser);
}

fn check(
    cache: &Mutex<Cache>,
    browser: Browser,
    now: Instant,
    read_status: impl FnOnce() -> OsStatus,
) -> Consent {
    let mut cache = cache.lock_or_recover();
    if let Some(consent) = cache.fresh(browser, now) {
        return consent;
    }
    let consent = consent_of(read_status());
    cache.entries.insert(browser, (now, consent));
    consent
}

fn gate(browser: Browser, consent: Consent) -> Result<(), PlatformError> {
    let permission = Permission::Automation(browser);
    match consent {
        Consent::Granted => Ok(()),
        Consent::Unasked => Err(PlatformError::NeedsConsent(permission)),
        Consent::Denied => Err(PlatformError::PermissionDenied(permission)),
    }
}

#[cfg(test)]
mod tests;
