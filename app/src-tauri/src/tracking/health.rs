use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use crate::concurrency::LockExt;
use crate::platform::Browser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProbeOutcome {
    Ok,
    /// The user has not been asked yet, so the tracker is not reading that browser.
    NeedsConsent,
    PermissionDenied,
    Unsupported,
    Failed,
}

/// The last tab-read outcome per browser, which the UI turns into permission hints.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObserverHealth {
    browsers: BTreeMap<Browser, ProbeOutcome>,
}

impl ObserverHealth {
    #[cfg(test)]
    pub fn from_outcomes(outcomes: &[(Browser, ProbeOutcome)]) -> Self {
        Self {
            browsers: outcomes.iter().copied().collect(),
        }
    }

    /// `None` until that browser has been the frontmost app at least once.
    #[cfg(test)]
    pub fn outcome(&self, browser: Browser) -> Option<ProbeOutcome> {
        self.browsers.get(&browser).copied()
    }

    pub fn permission_denied(&self) -> Vec<Browser> {
        self.with_outcome(ProbeOutcome::PermissionDenied)
    }

    #[cfg(test)]
    pub fn needs_consent(&self) -> Vec<Browser> {
        self.with_outcome(ProbeOutcome::NeedsConsent)
    }

    fn with_outcome(&self, wanted: ProbeOutcome) -> Vec<Browser> {
        self.browsers
            .iter()
            .filter(|(_, outcome)| **outcome == wanted)
            .map(|(browser, _)| *browser)
            .collect()
    }
}

/// Cheap clonable view of the observer's health, shared with the UI layer.
#[derive(Debug, Clone, Default)]
pub struct HealthHandle(Arc<Mutex<ObserverHealth>>);

impl HealthHandle {
    pub fn snapshot(&self) -> ObserverHealth {
        self.lock(|health| health.clone())
    }

    pub(super) fn record(&self, browser: Browser, outcome: ProbeOutcome) {
        self.lock(|health| {
            health.browsers.insert(browser, outcome);
        });
    }

    fn lock<T>(&self, work: impl FnOnce(&mut ObserverHealth) -> T) -> T {
        let mut guard = self.0.lock_or_recover();
        work(&mut guard)
    }
}

#[cfg(test)]
mod tests;
