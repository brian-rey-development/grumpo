//! Fakes shared by the tracking tests.

use std::collections::HashMap;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Duration;

use focus_core::test_support::app;
use focus_core::{AppRef, Domain};

use super::activity::Activity;
use super::sampler::SampleSink;

use crate::concurrency::LockExt;
use crate::platform::{
    ActivityProbe, Browser, ForegroundApp, Permission, PlatformError, PlatformErrorKind, TabReading,
};

pub const CHROME: &str = "com.google.Chrome";
pub const FOREGROUND_PID: i32 = 4242;

type Reply<T> = Result<T, PlatformErrorKind>;

struct Replies {
    foreground: Reply<Option<AppRef>>,
    idle: Reply<Duration>,
    locked: Reply<bool>,
    media: Reply<bool>,
    tabs: HashMap<Browser, Reply<TabReading>>,
}

pub struct FakeProbe {
    replies: Mutex<Replies>,
    pub tab_reads: AtomicUsize,
    pub media_reads: AtomicUsize,
    pub media_pid: AtomicI32,
}

impl FakeProbe {
    pub fn new() -> Self {
        Self {
            replies: Mutex::new(Replies {
                foreground: Ok(None),
                idle: Ok(Duration::ZERO),
                locked: Ok(false),
                media: Ok(false),
                tabs: HashMap::new(),
            }),
            tab_reads: AtomicUsize::new(0),
            media_reads: AtomicUsize::new(0),
            media_pid: AtomicI32::new(0),
        }
    }

    fn edit(&self, change: impl FnOnce(&mut Replies)) {
        change(&mut self.replies.lock_or_recover());
    }

    pub fn set_foreground(&self, bundle_id: Option<&str>) {
        self.edit(|r| r.foreground = Ok(bundle_id.map(app)));
    }

    pub fn fail_foreground(&self, kind: PlatformErrorKind) {
        self.edit(|r| r.foreground = Err(kind));
    }

    pub fn set_idle(&self, seconds: u64) {
        self.edit(|r| r.idle = Ok(Duration::from_secs(seconds)));
    }

    pub fn fail_idle(&self, kind: PlatformErrorKind) {
        self.edit(|r| r.idle = Err(kind));
    }

    pub fn set_locked(&self, locked: bool) {
        self.edit(|r| r.locked = Ok(locked));
    }

    pub fn fail_locked(&self, kind: PlatformErrorKind) {
        self.edit(|r| r.locked = Err(kind));
    }

    pub fn set_media(&self, playing: bool) {
        self.edit(|r| r.media = Ok(playing));
    }

    pub fn fail_media(&self, kind: PlatformErrorKind) {
        self.edit(|r| r.media = Err(kind));
    }

    pub fn set_host(&self, browser: Browser, host: Option<&str>) {
        self.set_tab(browser, host, false);
    }

    pub fn set_tab(&self, browser: Browser, host: Option<&str>, private: bool) {
        let reading = TabReading {
            domain: host.and_then(Domain::from_address),
            private,
        };
        self.edit(|r| {
            r.tabs.insert(browser, Ok(reading));
        });
    }

    pub fn fail_host(&self, browser: Browser, kind: PlatformErrorKind) {
        self.edit(|r| {
            r.tabs.insert(browser, Err(kind));
        });
    }

    fn read<T>(&self, pick: impl FnOnce(&Replies) -> Reply<T>) -> Result<T, PlatformError> {
        let reply = pick(&self.replies.lock_or_recover());
        reply.map_err(to_platform_error)
    }
}

fn to_platform_error(kind: PlatformErrorKind) -> PlatformError {
    match kind {
        PlatformErrorKind::Unsupported => PlatformError::Unsupported,
        PlatformErrorKind::PermissionDenied => {
            PlatformError::PermissionDenied(Permission::Automation(Browser::Chrome))
        }
        PlatformErrorKind::NeedsConsent => {
            PlatformError::NeedsConsent(Permission::Automation(Browser::Chrome))
        }
        PlatformErrorKind::Timeout => PlatformError::Timeout { operation: "fake" },
        PlatformErrorKind::Failed => PlatformError::Failed {
            operation: "fake",
            detail: "scripted failure".into(),
        },
    }
}

impl ActivityProbe for FakeProbe {
    fn foreground_app(&self) -> Result<Option<ForegroundApp>, PlatformError> {
        self.read(|r| r.foreground.clone()).map(|found| {
            found.map(|app| ForegroundApp {
                app,
                pid: FOREGROUND_PID,
            })
        })
    }

    fn idle_time(&self) -> Result<Duration, PlatformError> {
        self.read(|r| r.idle)
    }

    fn screen_locked(&self) -> Result<bool, PlatformError> {
        self.read(|r| r.locked)
    }

    fn media_playing(&self, frontmost_pid: i32) -> Result<bool, PlatformError> {
        self.media_reads.fetch_add(1, Ordering::SeqCst);
        self.media_pid.store(frontmost_pid, Ordering::SeqCst);
        self.read(|r| r.media)
    }

    fn active_tab(&self, browser: Browser, _pid: i32) -> Result<TabReading, PlatformError> {
        self.tab_reads.fetch_add(1, Ordering::SeqCst);
        self.read(|r| {
            r.tabs
                .get(&browser)
                .cloned()
                .unwrap_or(Ok(TabReading::default()))
        })
    }
}

#[derive(Debug, Default)]
pub struct RecordingSink {
    pub samples: Mutex<Vec<(Option<Activity>, i64)>>,
    pub usage_writes: AtomicUsize,
}

impl SampleSink for RecordingSink {
    fn on_sample(&self, activity: Option<&Activity>, at: i64) {
        let mut samples = self.samples.lock_or_recover();
        samples.push((activity.cloned(), at));
    }

    fn on_usage_written(&self) {
        self.usage_writes.fetch_add(1, Ordering::SeqCst);
    }
}
