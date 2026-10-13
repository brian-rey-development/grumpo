use super::*;
use crate::tracking::tab_memory::TAB_REFRESH_INTERVAL;
use crate::tracking::test_support::FakeProbe;
use focus_core::Domain;
use std::sync::atomic::Ordering;
use std::sync::Arc;

pub const T0: i64 = 1_000;

pub fn observer(probe: &Arc<FakeProbe>) -> Observer {
    Observer::new(Arc::clone(probe) as Arc<dyn ActivityProbe>)
}

pub fn refresh() -> i64 {
    i64::try_from(TAB_REFRESH_INTERVAL.as_secs()).unwrap()
}

pub fn tab_reads(probe: &FakeProbe) -> usize {
    probe.tab_reads.load(Ordering::SeqCst)
}

pub fn domain_of(activity: &Activity) -> Option<&str> {
    activity.domain.as_ref().map(Domain::as_str)
}
