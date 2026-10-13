//! Shared handles that commands, the tray and the background workers use.

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};

use crate::block_screen::BlockScreenHub;
use crate::concurrency::LockExt;
use crate::enforcement::EngineHandle;
use crate::events::AppEvents;
use crate::icon_cache::IconCache;
use crate::launch_at_login::LaunchAtLogin;
use crate::platform::{Platform, TrackingActivity};
use crate::status::{PollerHandle, StatusHub};
use crate::storage::Store;
use crate::tracking::health::HealthHandle;
use crate::tracking::sampler::SamplerHandle;

#[derive(Debug, Clone)]
pub struct AppPaths {
    pub database: PathBuf,
    pub exports: PathBuf,
}

/// Cheap to clone, since every field is shared, so a command can move a copy into a
/// blocking task.
#[derive(Clone)]
pub struct AppState {
    pub store: Arc<Store>,
    pub status: Arc<StatusHub>,
    pub platform: Platform,
    pub icons: Arc<IconCache>,
    pub health: HealthHandle,
    pub events: Arc<AppEvents>,
    pub block_screens: Arc<BlockScreenHub>,
    pub launch: Arc<dyn LaunchAtLogin>,
    /// Set while the service installer runs, so a second request fails instead of racing it.
    pub installing: Arc<AtomicBool>,
    pub paths: Arc<AppPaths>,
    /// The bundle identifier of this app, which is never offered for blocking.
    pub own_bundle_id: Arc<str>,
}

impl std::fmt::Debug for AppState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppState").finish_non_exhaustive()
    }
}

/// The long-running workers, kept so exit can stop them in order and flush what they hold.
pub struct Workers {
    sampler: Mutex<Option<SamplerHandle>>,
    engine: Mutex<Option<EngineHandle>>,
    poller: Mutex<Option<PollerHandle>>,
    status: Arc<StatusHub>,
    activity: Mutex<Option<TrackingActivity>>,
}

impl std::fmt::Debug for Workers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Workers").finish_non_exhaustive()
    }
}

impl Workers {
    pub fn new(
        sampler: SamplerHandle,
        engine: EngineHandle,
        poller: PollerHandle,
        status: Arc<StatusHub>,
        activity: TrackingActivity,
    ) -> Self {
        Self {
            sampler: Mutex::new(Some(sampler)),
            engine: Mutex::new(Some(engine)),
            poller: Mutex::new(Some(poller)),
            status,
            activity: Mutex::new(Some(activity)),
        }
    }

    /// Stops the poller first and cuts the status listeners loose, so that no worker can
    /// push a status update into the tray or the webview while the app is going away. Then
    /// the sampler stops, which flushes the open session, then the enforcement engine that
    /// it feeds. Safe to call twice. Exit runs on the main thread, so every step here has to
    /// be something that cannot be waiting for the main thread, and the tray guarantees that
    /// by only ever queueing its updates.
    pub fn stop(&self) {
        drop(take(&self.poller));
        self.status.clear_listeners();
        if let Some(sampler) = take(&self.sampler) {
            sampler.stop();
        }
        drop(take(&self.engine));
        drop(take(&self.activity));
    }
}

fn take<T>(slot: &Mutex<Option<T>>) -> Option<T> {
    slot.lock_or_recover().take()
}
