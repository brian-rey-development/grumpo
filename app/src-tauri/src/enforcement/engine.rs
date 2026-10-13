//! One enforcement step per sample: read the world, ask the policy, run what it chose.
//! The engine is plain synchronous code, and the worker thread only feeds it.

mod frame;
mod sweep;

use std::sync::Arc;

use focus_core::ipc::DaemonStatus;
use focus_core::Coverage;

use super::action::Action;
use super::executor::Executor;
use super::memory::EnforcementMemory;
use super::policy::{decide, BlockEnds, PolicyInput};
use super::usage::UsageTracker;
use super::EnforcementDeps;
use crate::platform::AppControl;
use crate::status::StatusHub;
use crate::storage::Store;
use crate::time::{Calendar, ClockFormatter, LocalDay};
use crate::tracking::activity::Activity;
use frame::Frame;
use sweep::Swept;

pub struct Engine {
    store: Arc<Store>,
    status: Arc<StatusHub>,
    control: Arc<dyn AppControl>,
    calendar: Arc<dyn Calendar>,
    executor: Executor,
    memory: EnforcementMemory,
    usage: UsageTracker,
    day: LocalDay,
    /// When and against which status the running apps were last checked, so that a blocked
    /// app running behind the front window is closed when blocking starts, when its rules
    /// change and periodically while it lasts, without listing every process on each sample.
    swept: Option<Swept>,
}

impl Engine {
    pub fn new(deps: EnforcementDeps, calendar: Arc<dyn Calendar>) -> Self {
        let day = calendar.day_containing(deps.clock.now_unix());
        let clock_text = ClockFormatter::new(Arc::clone(&calendar), deps.hour_cycle);
        Self {
            store: deps.store,
            control: Arc::clone(&deps.control),
            executor: Executor::new(
                deps.control,
                Arc::clone(&deps.status),
                deps.notifier,
                deps.block_screens,
                Arc::clone(&calendar),
                clock_text,
            ),
            status: deps.status,
            calendar,
            memory: EnforcementMemory::new(day.date),
            usage: UsageTracker::default(),
            day,
            swept: None,
        }
    }

    pub fn usage_written(&mut self, at: i64) {
        self.usage.mark_written(at);
    }

    pub fn sample(&mut self, activity: Option<&Activity>, at: i64) {
        self.advance_day(at);
        let (focus, confirmed_at) = self.status.snapshot();
        let status = focus.daemon();
        let frame = Frame {
            status,
            usage: self.usage_today(status, activity, at),
            at,
            predates_today: confirmed_at.is_none_or(|confirmed| confirmed < self.day.start),
            ends: self.block_ends(status, at),
        };
        let actions = decide(&self.input(&frame, activity));
        self.run(&actions, at);
        self.sweep_if_due(&frame);
    }

    fn input<'a>(
        &'a self,
        frame: &'a Frame<'a>,
        activity: Option<&'a Activity>,
    ) -> PolicyInput<'a> {
        PolicyInput {
            activity,
            status: frame.status,
            usage: &frame.usage,
            memory: &self.memory,
            now: frame.at,
            status_predates_today: frame.predates_today,
            ends: frame.ends,
        }
    }

    fn run(&mut self, actions: &[Action], at: i64) {
        if !actions.is_empty() {
            self.executor.run(actions, &mut self.memory, at);
        }
    }

    fn usage_today(
        &mut self,
        status: Option<&DaemonStatus>,
        activity: Option<&Activity>,
        at: i64,
    ) -> Vec<(focus_core::LimitTarget, u32)> {
        let targets = frame::limit_targets(status);
        self.usage
            .today(&self.store, self.day, &targets, activity, at)
    }

    fn block_ends(&self, status: Option<&DaemonStatus>, at: i64) -> BlockEnds {
        BlockEnds {
            schedule: status.and_then(|status| self.schedule_end(status, at)),
            midnight: self.day.end,
        }
    }

    fn schedule_end(&self, status: &DaemonStatus, at: i64) -> Option<i64> {
        let now = self.calendar.instant(at)?;
        match status.rules.schedule.active_until(now.local()) {
            Coverage::Until(end) => now.local_to_utc(end).map(|utc| utc.timestamp()),
            Coverage::Always | Coverage::Inactive => None,
        }
    }

    fn advance_day(&mut self, at: i64) {
        if (self.day.start..self.day.end).contains(&at) {
            return;
        }
        self.day = self.calendar.day_containing(at);
        self.memory.roll_to(self.day.date);
        self.usage.reset();
        self.swept = None;
    }
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
