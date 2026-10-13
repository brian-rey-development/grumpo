use std::sync::Arc;
use std::time::Duration;

use super::activity::{Activity, IDLE_THRESHOLD};
use super::health::{HealthHandle, ProbeOutcome};
use super::probe_errors::{ErrorLog, Source};
use super::tab_memory::{Decision, TabMemory};
use crate::platform::{
    ActivityProbe, Browser, ForegroundApp, PlatformError, PlatformErrorKind, TabReading,
};

/// Reads the platform probe once per sample and reduces it to an [`Activity`].
///
/// Every failure degrades instead of aborting the sample. No foreground app means no
/// activity, a failed idle or lock read counts as an active, unlocked user so tracking and
/// limits keep working, and a failed media read counts as no media. Idle and lock state are
/// read first, because a disengaged sample skips the costly tab read and reuses the last
/// known tab.
pub struct Observer {
    probe: Arc<dyn ActivityProbe>,
    errors: ErrorLog,
    health: HealthHandle,
    tabs: TabMemory,
}

impl std::fmt::Debug for Observer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Observer").finish_non_exhaustive()
    }
}

impl Observer {
    pub fn new(probe: Arc<dyn ActivityProbe>) -> Self {
        Self {
            probe,
            errors: ErrorLog::default(),
            health: HealthHandle::default(),
            tabs: TabMemory::default(),
        }
    }

    pub fn health(&self) -> HealthHandle {
        self.health.clone()
    }

    /// `now` is the sample time in unix seconds, which paces the tab reads.
    pub fn sample(&mut self, now: i64) -> Option<Activity> {
        let ForegroundApp { app, pid } = self.foreground()?;
        let idle = self.idle();
        let engaged = !self.screen_locked() && self.engaged(idle, pid);
        let browser = Browser::from_bundle_id(app.bundle_id());
        let tab = browser.map_or_else(TabReading::default, |browser| {
            self.tab(browser, pid, engaged, now)
        });
        Some(Activity {
            app,
            domain: tab.domain,
            browser,
            engaged,
            private: tab.private,
            idle,
        })
    }

    fn foreground(&mut self) -> Option<ForegroundApp> {
        let result = self.probe.foreground_app();
        self.settle(Source::Foreground, result).flatten()
    }

    fn idle(&mut self) -> Duration {
        let result = self.probe.idle_time();
        self.settle(Source::Idle, result).unwrap_or(Duration::ZERO)
    }

    fn screen_locked(&mut self) -> bool {
        let result = self.probe.screen_locked();
        self.settle(Source::ScreenLock, result).unwrap_or(false)
    }

    fn engaged(&mut self, idle: Duration, pid: i32) -> bool {
        idle < IDLE_THRESHOLD || self.media_playing(pid)
    }

    fn media_playing(&mut self, pid: i32) -> bool {
        let result = self.probe.media_playing(pid);
        self.settle(Source::Media, result).unwrap_or(false)
    }

    fn tab(&mut self, browser: Browser, pid: i32, engaged: bool, now: i64) -> TabReading {
        match self.tabs.decide(browser, pid, engaged, now) {
            Decision::Reuse(reading) => reading,
            Decision::Read => self.read_tab(browser, pid, now),
        }
    }

    fn read_tab(&mut self, browser: Browser, pid: i32, now: i64) -> TabReading {
        let result = self.probe.active_tab(browser, pid);
        self.health.record(browser, outcome_of(&result));
        let reading = self.settle(Source::BrowserTab(browser), result);
        self.tabs.record(browser, pid, now, reading)
    }

    fn settle<T>(&mut self, source: Source, result: Result<T, PlatformError>) -> Option<T> {
        self.errors.log(source, result.as_ref().err());
        result.ok()
    }
}

fn outcome_of<T>(result: &Result<T, PlatformError>) -> ProbeOutcome {
    match result.as_ref().map_err(PlatformError::kind) {
        Ok(_) => ProbeOutcome::Ok,
        Err(PlatformErrorKind::PermissionDenied) => ProbeOutcome::PermissionDenied,
        Err(PlatformErrorKind::NeedsConsent) => ProbeOutcome::NeedsConsent,
        Err(PlatformErrorKind::Unsupported) => ProbeOutcome::Unsupported,
        Err(PlatformErrorKind::Timeout | PlatformErrorKind::Failed) => ProbeOutcome::Failed,
    }
}

#[cfg(test)]
mod tab_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
