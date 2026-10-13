//! A fully faked enforcement environment around a scripted daemon.

use std::path::PathBuf;
use std::sync::Arc;

use focus_core::ipc::DaemonStatus;

use super::{CountingEvents, FakeControl, T0};
use crate::block_screen::test_support::FakeSurface;
use crate::block_screen::BlockScreenHub;
use crate::daemon_client::test_support::FakeDaemon;
use crate::daemon_client::DaemonClient;
use crate::enforcement::engine::Engine;
use crate::enforcement::{EnforcementDeps, EngineHandle, EngineSink};
use crate::notify::test_support::RecordingNotifier;
use crate::platform::HourCycle;
use crate::status::StatusHub;
use crate::storage::test_support::store;
use crate::storage::Store;
use crate::time::test_support::{FakeClock, FakeHourCycle, FixedCalendar};

#[derive(Debug)]
pub struct Rig {
    pub daemon: FakeDaemon,
    pub hub: Arc<StatusHub>,
    pub store: Arc<Store>,
    pub control: Arc<FakeControl>,
    pub notifier: Arc<RecordingNotifier>,
    pub surface: Arc<FakeSurface>,
    pub block_screens: Arc<BlockScreenHub>,
    pub events: Arc<CountingEvents>,
    pub clock: Arc<FakeClock>,
}

impl Rig {
    /// The daemon answers every request with `status`, and the hub has already read it.
    pub fn new(status: DaemonStatus) -> Self {
        Self::with_control(status, FakeControl::default())
    }

    pub fn with_control(status: DaemonStatus, control: FakeControl) -> Self {
        let daemon = FakeDaemon::start();
        daemon.respond_with(status);
        let clock = Arc::new(FakeClock::at(T0));
        let hub = StatusHub::with_clock(Arc::new(daemon.client()), clock.clone());
        hub.refresh();
        Self::around(daemon, hub, control, clock)
    }

    /// The hub cannot reach any daemon.
    pub fn offline() -> Self {
        let clock = Arc::new(FakeClock::at(T0));
        let hub = StatusHub::with_clock(
            Arc::new(DaemonClient::new(PathBuf::from("/nonexistent/focusd.sock"))),
            clock.clone(),
        );
        hub.refresh();
        Self::around(FakeDaemon::start(), hub, FakeControl::default(), clock)
    }

    fn around(
        daemon: FakeDaemon,
        hub: Arc<StatusHub>,
        control: FakeControl,
        clock: Arc<FakeClock>,
    ) -> Self {
        let surface = Arc::new(FakeSurface::default());
        Self {
            daemon,
            hub,
            store: Arc::new(store()),
            control: Arc::new(control),
            notifier: Arc::new(RecordingNotifier::default()),
            block_screens: Arc::new(BlockScreenHub::new(surface.clone())),
            surface,
            events: Arc::new(CountingEvents::default()),
            clock,
        }
    }

    /// The block screen window cannot be shown, so every block falls back to a notification.
    pub fn without_window(mut self) -> Self {
        let surface = Arc::new(FakeSurface::failing());
        self.block_screens = Arc::new(BlockScreenHub::new(surface.clone()));
        self.surface = surface;
        self
    }

    pub fn deps(&self) -> EnforcementDeps {
        EnforcementDeps {
            control: self.control.clone(),
            store: Arc::clone(&self.store),
            status: Arc::clone(&self.hub),
            notifier: self.notifier.clone(),
            block_screens: Arc::clone(&self.block_screens),
            events: self.events.clone(),
            clock: self.clock.clone(),
            hour_cycle: Arc::new(FakeHourCycle::new(HourCycle::TwentyFour)),
        }
    }

    pub fn engine(&self) -> Engine {
        Engine::new(self.deps(), Arc::new(FixedCalendar::utc()))
    }

    pub fn sink(&self) -> (Arc<EngineSink>, EngineHandle) {
        EngineSink::start_in(self.deps(), Arc::new(FixedCalendar::utc())).unwrap()
    }

    pub fn titles(&self) -> Vec<String> {
        self.notifier
            .notices()
            .into_iter()
            .map(|n| n.title)
            .collect()
    }
}
