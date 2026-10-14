use focus_core::ipc::Request;
use focus_core::test_support::eventually;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use super::*;
use crate::clock::test_support::FakeClock;
use crate::dns::PendingFlush;
use crate::service::Handled;

struct Counting {
    ticks: Arc<AtomicUsize>,
    panic_on_first: bool,
}

impl Daemon for Counting {
    fn handle(&mut self, _: Request) -> Handled {
        unreachable!("the reconciler never handles requests")
    }

    fn reconcile(&mut self) -> PendingFlush {
        let seen = self.ticks.fetch_add(1, Ordering::SeqCst);
        assert!(!(self.panic_on_first && seen == 0), "first tick fails");
        PendingFlush::NONE
    }
}

#[test]
fn ticks_repeatedly() {
    let ticks = Arc::new(AtomicUsize::new(0));
    let daemon = Counting {
        ticks: Arc::clone(&ticks),
        panic_on_first: false,
    };
    spawn(
        Shared::new(daemon),
        FakeClock::at(2026, 10, 12, 9, 0),
        Duration::from_millis(5),
        || {},
    )
    .unwrap();
    eventually(|| ticks.load(Ordering::SeqCst) >= 3);
}

#[test]
fn keeps_ticking_after_a_tick_panics() {
    let ticks = Arc::new(AtomicUsize::new(0));
    let daemon = Counting {
        ticks: Arc::clone(&ticks),
        panic_on_first: true,
    };
    spawn(
        Shared::new(daemon),
        FakeClock::at(2026, 10, 12, 9, 0),
        Duration::from_millis(5),
        || {},
    )
    .unwrap();
    eventually(|| ticks.load(Ordering::SeqCst) >= 3);
}

#[test]
fn keeps_ticking_after_housekeeping_panics() {
    let ticks = Arc::new(AtomicUsize::new(0));
    let beats = Arc::new(AtomicUsize::new(0));
    let daemon = Counting {
        ticks: Arc::clone(&ticks),
        panic_on_first: false,
    };
    let counted = Arc::clone(&beats);
    spawn(
        Shared::new(daemon),
        FakeClock::at(2026, 10, 12, 9, 0),
        Duration::from_millis(5),
        move || {
            let seen = counted.fetch_add(1, Ordering::SeqCst);
            assert!(seen != 0, "first housekeeping fails");
        },
    )
    .unwrap();
    eventually(|| beats.load(Ordering::SeqCst) >= 3);
}
