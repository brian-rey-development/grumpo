use std::collections::BTreeSet;
use std::io;
use std::sync::{Arc, Mutex, MutexGuard};

use chrono::NaiveDate;
use focus_core::ipc::{DaemonStatus, IpcError, Request};
use focus_core::test_support::{now_at, site, site_target, window};
use focus_core::{Day, Domain, Limit, Rules, Schedule, TimeOfDay, UNLOCK_SENTENCE};

use crate::clock::test_support::FakeClock;
use crate::clock::Clock;
use crate::dns::PendingFlush;
use crate::hosts_writer::{HostsSink, HostsWrite};
use crate::service::{Daemon, Handled, Service};
use crate::state::{Snapshot, StateError, StateStore};

/// The wait before the first unlock of a day can be confirmed.
const UNLOCK_COOLDOWN_MINUTES: i64 = 5;

#[derive(Debug, Default)]
struct HostsInner {
    content: Vec<u8>,
    writes: usize,
    fail_reads: bool,
    fail_writes: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryHosts(Arc<Mutex<HostsInner>>);

impl MemoryHosts {
    fn inner(&self) -> MutexGuard<'_, HostsInner> {
        self.0.lock().unwrap()
    }

    pub fn content(&self) -> String {
        String::from_utf8_lossy(&self.content_bytes()).into_owned()
    }

    pub fn content_bytes(&self) -> Vec<u8> {
        self.inner().content.clone()
    }

    pub fn set_content(&self, content: &str) {
        self.set_content_bytes(content.as_bytes());
    }

    pub fn set_content_bytes(&self, content: &[u8]) {
        self.inner().content = content.to_vec();
    }

    pub fn writes(&self) -> usize {
        self.inner().writes
    }

    pub fn blocks(&self, host: &str) -> bool {
        self.content().contains(&format!("0.0.0.0 {host}\n"))
    }

    pub fn has_managed_block(&self) -> bool {
        self.content().contains("0.0.0.0 ")
    }

    pub fn fail_reads(&self, fail: bool) {
        self.inner().fail_reads = fail;
    }

    pub fn fail_writes(&self, fail: bool) {
        self.inner().fail_writes = fail;
    }
}

impl HostsSink for MemoryHosts {
    fn read(&self) -> io::Result<Vec<u8>> {
        let inner = self.inner();
        if inner.fail_reads {
            return Err(io::Error::other("read failed"));
        }
        Ok(inner.content.clone())
    }

    fn write_if_changed(&self, content: &[u8]) -> io::Result<HostsWrite> {
        let mut inner = self.inner();
        if inner.fail_writes {
            return Err(io::Error::other("write failed"));
        }
        if inner.content == content {
            return Ok(HostsWrite::UNCHANGED);
        }
        inner.content = content.to_vec();
        inner.writes += 1;
        Ok(HostsWrite {
            changed: true,
            flush: PendingFlush::NEEDED,
        })
    }
}

#[derive(Debug, Default)]
struct StoreInner {
    state: Option<Snapshot>,
    saves: usize,
    fail_saves: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MemoryStore(Arc<Mutex<StoreInner>>);

impl MemoryStore {
    fn inner(&self) -> MutexGuard<'_, StoreInner> {
        self.0.lock().unwrap()
    }

    pub fn seed(&self, state: Snapshot) {
        self.inner().state = Some(state);
    }

    pub fn saved(&self) -> Option<Snapshot> {
        self.inner().state.clone()
    }

    pub fn saves(&self) -> usize {
        self.inner().saves
    }

    pub fn fail_saves(&self, fail: bool) {
        self.inner().fail_saves = fail;
    }
}

impl StateStore for MemoryStore {
    fn load(&self, now: &focus_core::Now) -> Result<Snapshot, StateError> {
        Ok(self
            .inner()
            .state
            .clone()
            .unwrap_or_else(|| Snapshot::fresh(now)))
    }

    fn save(&self, state: &Snapshot) -> Result<(), StateError> {
        let mut inner = self.inner();
        if inner.fail_saves {
            return Err(StateError::Io {
                action: "write the state file",
                path: "memory".into(),
                source: io::Error::other("disk full"),
            });
        }
        inner.state = Some(state.clone());
        inner.saves += 1;
        Ok(())
    }
}

pub type TestService = Service<FakeClock, MemoryHosts, MemoryStore>;

pub struct Harness {
    pub service: TestService,
    pub clock: FakeClock,
    pub hosts: MemoryHosts,
    pub store: MemoryStore,
}

/// Monday 2026-10-12 at the given time, with `rules` already saved and reconciled.
pub fn monday(hour: u32, minute: u32, rules: Rules) -> Harness {
    let clock = FakeClock::at(2026, 10, 12, hour, minute);
    let store = MemoryStore::default();
    let now = now_at(2026, 10, 12, hour, minute);
    store.seed(Snapshot {
        rules: rules.validate().unwrap(),
        ..Snapshot::fresh(&now)
    });
    let hosts = MemoryHosts::default();
    let mut service = Service::new(clock.clone(), hosts.clone(), store.clone()).unwrap();
    let _flush = service.reconcile();
    Harness {
        service,
        clock,
        hosts,
        store,
    }
}

impl Harness {
    pub fn send(&mut self, request: Request) -> Result<DaemonStatus, IpcError> {
        self.service.handle(request).response.into_result()
    }

    pub fn handle(&mut self, request: Request) -> Handled {
        self.service.handle(request)
    }

    pub fn status(&mut self) -> DaemonStatus {
        self.send(Request::Status).unwrap()
    }

    pub fn update(&mut self, rules: Rules) -> Result<DaemonStatus, IpcError> {
        self.send(Request::UpdateRules { rules })
    }

    pub fn reconcile(&mut self) {
        let _flush = self.service.reconcile();
    }

    pub fn reconcile_flush(&mut self) -> PendingFlush {
        self.service.reconcile()
    }

    pub fn advance(&self, minutes: i64) {
        self.clock.advance_minutes(minutes);
    }

    pub fn today(&self) -> NaiveDate {
        self.clock.now().local().date()
    }

    /// Reports `host` out of `minutes` on the current day, like the app does.
    pub fn report_limit(&mut self, host: &str, minutes: u16) -> Result<DaemonStatus, IpcError> {
        let day = self.today();
        self.report_limit_on(host, minutes, day)
    }

    pub fn report_limit_on(
        &mut self,
        host: &str,
        minutes: u16,
        day: NaiveDate,
    ) -> Result<DaemonStatus, IpcError> {
        self.send(Request::ReportLimitReached {
            target: site_target(host),
            daily_minutes: minutes,
            day,
        })
    }

    /// Runs the whole unlock flow for `minutes`, so loosening is allowed afterwards.
    pub fn unlock(&mut self, minutes: u16) {
        self.send(Request::RequestUnlock).unwrap();
        self.advance(UNLOCK_COOLDOWN_MINUTES);
        self.send(Request::ConfirmUnlock {
            phrase: UNLOCK_SENTENCE.to_owned(),
            minutes,
        })
        .unwrap();
    }
}

pub fn sites(hosts: &[&str]) -> BTreeSet<Domain> {
    hosts.iter().map(|host| site(host)).collect()
}

/// Mondays from midnight to 19:00.
pub fn monday_until_seven() -> Schedule {
    Schedule {
        windows: vec![window(&[Day::Mon], 0, 19 * 60)],
    }
}

pub fn always_on() -> Schedule {
    Schedule {
        windows: vec![window(&Day::ALL, 0, TimeOfDay::END_OF_DAY.minutes())],
    }
}

pub fn rules(hosts: &[&str], schedule: Schedule) -> Rules {
    Rules {
        blocked_sites: sites(hosts),
        schedule,
        ..Rules::default()
    }
}

pub fn with_limit(mut rules: Rules, host: &str, minutes: u16) -> Rules {
    rules.limits.push(Limit {
        target: site_target(host),
        daily_minutes: minutes,
    });
    rules
}
