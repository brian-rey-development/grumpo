//! Builds the app's long-lived parts once the Tauri runtime exists and wires them together.

mod database;

use std::error::Error;
use std::sync::Arc;
use std::time::Duration;

use tauri::{App, AppHandle, Manager};

use crate::block_screen::{BlockScreenHub, TauriBlockSurface};
use crate::daemon_client::DaemonClient;
use crate::enforcement::{EnforcementDeps, EngineSink};
use crate::events::AppEvents;
use crate::icon_cache::IconCache;
use crate::launch_at_login::{self, Autostart, LaunchAtLogin};
use crate::legacy_data::{self, LegacyAppProbe};
use crate::notify::{Notifier, TauriNotifier};
use crate::platform::{self, AppControl, Platform};
use crate::state::{AppPaths, AppState, Workers};
use crate::status::{StatusHub, UnlockReadyAlert};
use crate::storage::Store;
use crate::time::{Clock, ClockFormatter, SystemCalendar, SystemClock};
use crate::tracking::observer::Observer;
use crate::tracking::sampler::Sampler;
use crate::tray::TrayHandle;
use crate::window;
use database::open_database;

const DATABASE_FILE: &str = "focus.db";
const DOWNLOADS_FROM_HOME: &str = "Downloads";
const STATUS_POLL_INTERVAL: Duration = Duration::from_secs(5);

type SetupResult = Result<(), Box<dyn Error>>;

/// The shared pieces everything else is wired from.
struct Parts {
    handle: AppHandle,
    paths: Arc<AppPaths>,
    store: Arc<Store>,
    status: Arc<StatusHub>,
    platform: Platform,
    events: Arc<AppEvents>,
    block_screens: Arc<BlockScreenHub>,
    notifier: Arc<dyn Notifier>,
    launch: Arc<dyn LaunchAtLogin>,
    clock_text: ClockFormatter,
}

pub fn setup(app: &mut App) -> SetupResult {
    let parts = build_parts(app.handle())?;
    let observer = Observer::new(Arc::clone(&parts.platform.probe));
    app.manage(app_state(&parts, &observer));
    wire_status(&parts);
    sync_launch_at_login(&parts);
    app.manage(start_workers(&parts, observer)?);
    show_window_unless_autostarted(&parts.handle);
    Ok(())
}

fn build_parts(app: &AppHandle) -> Result<Parts, Box<dyn Error>> {
    let handle = app.clone();
    let platform = platform::current();
    let notifier: Arc<dyn Notifier> = Arc::new(TauriNotifier::new(handle.clone()));
    let paths = Arc::new(resolve_paths(&handle, platform.control.as_ref())?);
    let store = open_store(&paths, &platform, notifier.as_ref())?;
    let status = StatusHub::new(Arc::new(DaemonClient::from_env()));
    let calendar = Arc::new(SystemCalendar);
    let clock_text = ClockFormatter::new(calendar, Arc::clone(&platform.hour_cycle));
    let tray = TrayHandle::build(
        &handle,
        Arc::clone(&store),
        &status.current(),
        clock_text.clone(),
    )?;
    Ok(Parts {
        events: Arc::new(AppEvents::new(handle.clone(), tray)),
        block_screens: Arc::new(BlockScreenHub::new(Arc::new(TauriBlockSurface::new(
            handle.clone(),
        )))),
        launch: Arc::new(Autostart::new(handle.clone())),
        handle,
        paths,
        store,
        status,
        platform,
        notifier,
        clock_text,
    })
}

fn app_state(parts: &Parts, observer: &Observer) -> AppState {
    AppState {
        store: Arc::clone(&parts.store),
        status: Arc::clone(&parts.status),
        platform: parts.platform.clone(),
        icons: Arc::new(IconCache::default()),
        health: observer.health(),
        events: Arc::clone(&parts.events),
        block_screens: Arc::clone(&parts.block_screens),
        launch: Arc::clone(&parts.launch),
        installing: Arc::default(),
        paths: Arc::clone(&parts.paths),
        own_bundle_id: Arc::from(parts.handle.config().identifier.as_str()),
    }
}

fn open_store(
    paths: &AppPaths,
    platform: &Platform,
    notifier: &dyn Notifier,
) -> Result<Arc<Store>, Box<dyn Error>> {
    let now = SystemClock.now_unix();
    let store = open_database(&paths.database, platform.dialogs.as_ref(), notifier, now)?;
    Ok(Arc::new(store))
}

/// An error while asking counts as running, because a wrongly skipped move is retried at the
/// next start while a wrongly made one splits the history between two apps.
struct EarlierApp<'a>(&'a dyn AppControl);

impl LegacyAppProbe for EarlierApp<'_> {
    fn is_running(&self) -> bool {
        self.0
            .is_running(legacy_data::LEGACY_IDENTIFIER)
            .unwrap_or_else(|error| {
                tracing::debug!(%error, "earlier app lookup failure detail");
                true
            })
    }
}

fn resolve_paths(app: &AppHandle, control: &dyn AppControl) -> Result<AppPaths, Box<dyn Error>> {
    let resolver = app.path();
    let exports = match resolver.download_dir() {
        Ok(dir) => dir,
        Err(_) => resolver.home_dir()?.join(DOWNLOADS_FROM_HOME),
    };
    Ok(AppPaths {
        database: legacy_data::adopt_legacy_data(&resolver.app_data_dir()?, &EarlierApp(control))
            .join(DATABASE_FILE),
        exports,
    })
}

fn wire_status(parts: &Parts) {
    let events = Arc::clone(&parts.events);
    parts
        .status
        .on_change(Box::new(move |status| events.status_changed(status)));
    let alert = UnlockReadyAlert::new(Arc::clone(&parts.notifier), parts.clock_text.clone());
    parts
        .status
        .on_change(Box::new(move |status| alert.observe(status)));
}

/// A development build must not register itself as a login item, since its path is the
/// build directory. The error text can hold a path, so only a fixed description is logged
/// above debug.
fn sync_launch_at_login(parts: &Parts) {
    if cfg!(debug_assertions) {
        return;
    }
    let home = match parts.handle.path().home_dir() {
        Ok(home) => home,
        Err(error) => {
            tracing::warn!(
                "could not tell where the login items live, leaving launch at login alone"
            );
            tracing::debug!(%error, "home directory lookup failure detail");
            return;
        }
    };
    let executable = match std::env::current_exe() {
        Ok(executable) => executable,
        Err(error) => {
            tracing::warn!("could not tell where the app runs from, leaving launch at login alone");
            tracing::debug!(%error, "current executable lookup failure detail");
            return;
        }
    };
    if let Err(error) = launch_at_login::sync_on_start(
        &parts.store,
        parts.launch.as_ref(),
        &executable,
        &launch_at_login::agents_dir(&home),
    ) {
        tracing::warn!("could not sync launch at login");
        tracing::debug!(%error, "launch at login failure detail");
    }
}

/// The login item starts the app with a flag, and then the window stays hidden so the app
/// comes up quietly in the menu bar. Any other start shows it.
fn show_window_unless_autostarted(app: &AppHandle) {
    if !launch_at_login::was_autostarted(std::env::args()) {
        window::show_main(app);
    }
}

fn start_workers(parts: &Parts, observer: Observer) -> Result<Workers, Box<dyn Error>> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let (sink, engine) = EngineSink::start(EnforcementDeps {
        control: Arc::clone(&parts.platform.control),
        store: Arc::clone(&parts.store),
        status: Arc::clone(&parts.status),
        notifier: Arc::clone(&parts.notifier),
        block_screens: Arc::clone(&parts.block_screens),
        events: Arc::clone(&parts.events) as _,
        clock: Arc::clone(&clock),
        hour_cycle: Arc::clone(&parts.platform.hour_cycle),
    })?;
    let sampler = Sampler::new(Arc::clone(&parts.store), observer, clock, sink).start()?;
    let poller = parts.status.start_polling(STATUS_POLL_INTERVAL)?;
    let activity = platform::begin_tracking_activity();
    Ok(Workers::new(
        sampler,
        engine,
        poller,
        Arc::clone(&parts.status),
        activity,
    ))
}
