use super::*;
use crate::dto::BlockReason;
use crate::enforcement::memory::Outcome;
use crate::enforcement::test_support::{app, app_target, daemon_status, site, with_limit, END, T0};
use focus_core::ipc::DaemonStatus;
use focus_core::LimitTarget;

pub const STEAM: &str = "com.valvesoftware.steam";

pub fn memory() -> EnforcementMemory {
    EnforcementMemory::new(chrono::NaiveDate::from_ymd_opt(2027, 1, 15).unwrap())
}

pub const MIDNIGHT: i64 = T0 + 7_200;

pub fn ends() -> BlockEnds {
    BlockEnds {
        schedule: Some(END),
        midnight: MIDNIGHT,
    }
}

pub fn input_for<'a>(
    activity: Option<&'a Activity>,
    status: Option<&'a DaemonStatus>,
    usage: &'a [(LimitTarget, u32)],
    memory: &'a EnforcementMemory,
    now: i64,
) -> PolicyInput<'a> {
    PolicyInput {
        activity,
        status,
        usage,
        memory,
        now,
        status_predates_today: false,
        ends: ends(),
    }
}

pub fn decide_with(
    activity: Option<&Activity>,
    status: Option<&DaemonStatus>,
    usage: &[(LimitTarget, u32)],
    memory: &EnforcementMemory,
    now: i64,
) -> Vec<Action> {
    decide(&input_for(activity, status, usage, memory, now))
}

pub fn decide_stale(activity: Option<&Activity>, status: &DaemonStatus) -> Vec<Action> {
    let memory = memory();
    let mut input = input_for(activity, Some(status), &[], &memory, T0);
    input.status_predates_today = true;
    decide(&input)
}

pub fn sweep_now(
    running: &[AppRef],
    status: &DaemonStatus,
    memory: &EnforcementMemory,
) -> Vec<Action> {
    sweep(&input_for(None, Some(status), &[], memory, T0), running)
}

pub fn sweep_stale(running: &[AppRef], status: &DaemonStatus) -> Vec<Action> {
    let memory = memory();
    let mut input = input_for(None, Some(status), &[], &memory, T0);
    input.status_predates_today = true;
    sweep(&input, running)
}

pub fn decide_now(activity: Option<&Activity>, status: &DaemonStatus) -> Vec<Action> {
    decide_with(activity, Some(status), &[], &memory(), T0)
}

pub fn note_all(memory: &mut EnforcementMemory, actions: &[Action], now: i64) {
    for action in actions {
        let outcome = Outcome {
            now,
            succeeded: true,
        };
        memory.note(action, outcome);
    }
}

pub fn app_blocked_notice() -> Action {
    Action::Notify(NoticeKind::AppBlocked {
        app: app(STEAM),
        until: Some(END),
        reason: BlockReason::Schedule,
        strict: true,
    })
}

pub fn site_blocked_notice(host: &str) -> Action {
    Action::Notify(NoticeKind::SiteBlocked {
        domain: site(host),
        until: Some(END),
        reason: BlockReason::Schedule,
        strict: true,
    })
}

pub fn status_with_limit(minutes: u16) -> DaemonStatus {
    with_limit(daemon_status(), app_target(STEAM), minutes)
}

pub fn reached(minutes: u16) -> Action {
    Action::ReportLimitReached {
        target: app_target(STEAM),
        daily_minutes: minutes,
    }
}

pub fn used(seconds: u32) -> Vec<(LimitTarget, u32)> {
    vec![(app_target(STEAM), seconds)]
}
