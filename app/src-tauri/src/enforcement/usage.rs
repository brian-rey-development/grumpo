//! Today's usage per limit target, read from the store sparingly and topped up with the
//! time of the session in progress, which the store only learns about in batches, and with
//! private browsing time, which the store never learns about at all.

use std::collections::BTreeMap;

use focus_core::LimitTarget;

use crate::storage::Store;
use crate::time::LocalDay;
use crate::tracking::activity::Activity;
use crate::tracking::timing::MAX_SAMPLE_GAP;

const REFRESH_SECONDS: i64 = 10;
/// The writer flushes every 10 s, so a trail much longer than that means a write failed.
/// Crediting it anyway could block a target on time that was never spent.
const MAX_IN_FLIGHT_SECONDS: i64 = 30;

#[derive(Debug, Default)]
pub struct UsageTracker {
    stored: Vec<(LimitTarget, u32)>,
    queried_at: Option<i64>,
    dirty: bool,
    last_write_at: Option<i64>,
    /// Engaged private browsing seconds per limit target, for today only. It lives in memory
    /// alone, so a restart or the next midnight forgets it, which is the privacy promise: the
    /// time counts toward limits while Grumpo watches, but no record of it is ever kept.
    private: BTreeMap<LimitTarget, u32>,
    last_sample_at: Option<i64>,
}

impl UsageTracker {
    pub fn mark_written(&mut self, at: i64) {
        self.dirty = true;
        self.last_write_at = Some(at);
    }

    pub fn reset(&mut self) {
        *self = Self::default();
    }

    pub fn today(
        &mut self,
        store: &Store,
        day: LocalDay,
        targets: &[LimitTarget],
        activity: Option<&Activity>,
        at: i64,
    ) -> Vec<(LimitTarget, u32)> {
        let gap = self.advance(at);
        if targets.is_empty() {
            return Vec::new();
        }
        if let Some(activity) = activity {
            self.tally_private(targets, activity, gap);
        }
        if self.needs_refresh(targets, at) {
            self.refresh(store, day, targets, at);
        }
        let mut usage = self.stored.clone();
        self.add_private(&mut usage);
        if let Some(activity) = activity {
            credit(&mut usage, activity, self.in_flight(at));
        }
        usage
    }

    /// Seconds since the previous sample, capped at the longest gap that still counts as one
    /// continuous run, so a stalled worker or a sleeping Mac cannot credit time it did not see.
    fn advance(&mut self, at: i64) -> u32 {
        let gap = self
            .last_sample_at
            .map_or(0, |previous| at.saturating_sub(previous));
        self.last_sample_at = Some(at);
        u32::try_from(gap.clamp(0, MAX_SAMPLE_GAP)).unwrap_or_default()
    }

    fn tally_private(&mut self, targets: &[LimitTarget], activity: &Activity, seconds: u32) {
        if seconds == 0 || !activity.engaged || !activity.private {
            return;
        }
        for target in targets.iter().filter(|target| counts_for(target, activity)) {
            let tally = self.private.entry(target.clone()).or_default();
            *tally = tally.saturating_add(seconds);
        }
    }

    fn add_private(&self, usage: &mut [(LimitTarget, u32)]) {
        for (target, used) in usage.iter_mut() {
            if let Some(private) = self.private.get(target) {
                *used = used.saturating_add(*private);
            }
        }
    }

    fn needs_refresh(&self, targets: &[LimitTarget], at: i64) -> bool {
        let stale = self
            .queried_at
            .is_none_or(|queried| !(0..REFRESH_SECONDS).contains(&at.saturating_sub(queried)));
        let same_targets = self.stored.len() == targets.len()
            && self
                .stored
                .iter()
                .zip(targets)
                .all(|((held, _), wanted)| held == wanted);
        self.dirty || stale || !same_targets
    }

    fn refresh(&mut self, store: &Store, day: LocalDay, targets: &[LimitTarget], at: i64) {
        self.queried_at = Some(at);
        self.dirty = false;
        match store.usage_seconds(day, targets) {
            Ok(rows) => self.stored = rows,
            Err(error) => {
                tracing::warn!(%error, "could not read today's usage, keeping the last reading");
            }
        }
    }

    fn in_flight(&self, at: i64) -> u32 {
        let trail = self
            .last_write_at
            .map_or(0, |written| at.saturating_sub(written));
        u32::try_from(trail.clamp(0, MAX_IN_FLIGHT_SECONDS)).unwrap_or_default()
    }
}

fn credit(usage: &mut [(LimitTarget, u32)], activity: &Activity, seconds: u32) {
    if seconds == 0 || !activity.engaged || activity.private {
        return;
    }
    for (_, used) in usage
        .iter_mut()
        .filter(|(target, _)| counts_for(target, activity))
    {
        *used = used.saturating_add(seconds);
    }
}

fn counts_for(target: &LimitTarget, activity: &Activity) -> bool {
    match target {
        LimitTarget::App { app } => app.bundle_id() == activity.app.bundle_id(),
        LimitTarget::Site { domain } => activity
            .domain
            .as_ref()
            .is_some_and(|visited| domain.covers(visited)),
    }
}

#[cfg(test)]
mod tests;
