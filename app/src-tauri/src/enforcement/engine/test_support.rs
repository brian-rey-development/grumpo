use crate::enforcement::test_support::Rig;
use crate::enforcement::test_support::{app_target, daemon_status, with_limit};
use crate::storage::test_support::record;
use focus_core::ipc::Request;

pub const STEAM: &str = "com.valvesoftware.steam";

pub const DAY: i64 = 86_400;

pub fn reports(rig: &Rig) -> usize {
    rig.daemon
        .requests()
        .iter()
        .filter(|request| matches!(request, Request::ReportLimitReached { .. }))
        .count()
}

pub fn steam_limited(minutes: u16) -> focus_core::ipc::DaemonStatus {
    with_limit(daemon_status(), app_target(STEAM), minutes)
}

pub fn spend_on_steam(rig: &Rig, seconds: i64, ending_at: i64) {
    record(
        &rig.store,
        STEAM,
        "steam",
        None,
        (ending_at - seconds, ending_at),
    );
}
