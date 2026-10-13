//! Test doubles for the ports enforcement depends on.

use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use focus_core::{AppRef, Domain};

use crate::concurrency::LockExt;
use crate::enforcement::UsageEvents;
use crate::platform::{AppControl, Browser, PlatformError};

#[derive(Debug, Default)]
pub struct FakeControl {
    quits: Mutex<Vec<String>>,
    closes: Mutex<Vec<(Browser, String)>>,
    failing: AtomicBool,
    delay: Duration,
    running: Mutex<Vec<AppRef>>,
    listings: AtomicUsize,
}

impl FakeControl {
    pub fn failing() -> Self {
        let control = Self::default();
        control.failing.store(true, Ordering::SeqCst);
        control
    }

    /// Every quit takes `delay`, like a slow `AppleScript`.
    pub fn slow(delay: Duration) -> Self {
        Self {
            delay,
            ..Self::default()
        }
    }

    /// The apps `running_apps` reports.
    pub fn with_running(running: Vec<AppRef>) -> Self {
        Self {
            running: Mutex::new(running),
            ..Self::default()
        }
    }

    /// An app that starts running after the control was built.
    pub fn launch(&self, app: AppRef) {
        self.running.lock_or_recover().push(app);
    }

    /// How many times the running apps were listed.
    pub fn listings(&self) -> usize {
        self.listings.load(Ordering::SeqCst)
    }

    pub fn quits(&self) -> Vec<String> {
        self.quits.lock_or_recover().clone()
    }

    pub fn closes(&self) -> Vec<(Browser, String)> {
        self.closes.lock_or_recover().clone()
    }
}

impl AppControl for FakeControl {
    fn running_apps(&self) -> Result<Vec<AppRef>, PlatformError> {
        self.listings.fetch_add(1, Ordering::SeqCst);
        if self.failing.load(Ordering::SeqCst) {
            return Err(PlatformError::Timeout {
                operation: "list apps",
            });
        }
        Ok(self.running.lock_or_recover().clone())
    }

    fn is_running(&self, bundle_id: &str) -> Result<bool, PlatformError> {
        Ok(self
            .running
            .lock_or_recover()
            .iter()
            .any(|app| app.bundle_id() == bundle_id))
    }

    fn quit(&self, app: &AppRef) -> Result<(), PlatformError> {
        std::thread::sleep(self.delay);
        self.quits
            .lock_or_recover()
            .push(app.bundle_id().to_owned());
        if self.failing.load(Ordering::SeqCst) {
            return Err(PlatformError::Timeout { operation: "quit" });
        }
        Ok(())
    }

    fn close_active_tab_if(
        &self,
        browser: Browser,
        domain: &Domain,
    ) -> Result<bool, PlatformError> {
        self.closes
            .lock_or_recover()
            .push((browser, domain.as_str().to_owned()));
        if self.failing.load(Ordering::SeqCst) {
            return Err(PlatformError::Timeout { operation: "close" });
        }
        Ok(true)
    }
}

#[derive(Debug, Default)]
pub struct CountingEvents(pub AtomicUsize);

impl CountingEvents {
    pub fn count(&self) -> usize {
        self.0.load(Ordering::SeqCst)
    }
}

impl UsageEvents for CountingEvents {
    fn usage_changed(&self) {
        self.0.fetch_add(1, Ordering::SeqCst);
    }
}
