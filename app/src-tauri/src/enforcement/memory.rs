//! What enforcement already did today, so a blocked app that stays in front or a limit that
//! stays reached produces one notification and one report instead of one per sample.

use std::collections::{BTreeMap, BTreeSet};

use chrono::NaiveDate;
use focus_core::{Domain, Limit, LimitTarget};

use super::action::{Action, NoticeKind};

/// A quit or tab close is retried after this long, a little over one 2 s sample, so a
/// slow-to-quit app is not asked twice for the same tick but is asked again if it stays.
pub const ACTION_RETRY_SECONDS: i64 = 3;
pub const NOTICE_INTERVAL_SECONDS: i64 = 60;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Subject {
    App(String),
    Site(Domain),
}

/// How an action ended, which decides what is worth remembering about it.
#[derive(Debug, Clone, Copy)]
pub struct Outcome {
    pub now: i64,
    pub succeeded: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EnforcementMemory {
    day: NaiveDate,
    acted: BTreeMap<Subject, i64>,
    noticed: BTreeMap<Subject, i64>,
    reported: BTreeMap<LimitTarget, u16>,
    warned: BTreeSet<LimitTarget>,
}

impl EnforcementMemory {
    pub fn new(day: NaiveDate) -> Self {
        Self {
            day,
            acted: BTreeMap::new(),
            noticed: BTreeMap::new(),
            reported: BTreeMap::new(),
            warned: BTreeSet::new(),
        }
    }

    /// Forgets everything when the local date changes. A date that moved backwards, as after
    /// a clock change, resets too, because yesterday's memory would only suppress today.
    pub fn roll_to(&mut self, day: NaiveDate) -> bool {
        if self.day == day {
            return false;
        }
        *self = Self::new(day);
        true
    }

    /// Whether the rate limits let `action` happen now. Reports are decided by
    /// [`Self::limit_reported`], since they depend on the allowance and not on time.
    pub fn allows(&self, action: &Action, now: i64) -> bool {
        match action {
            Action::QuitApp(_) | Action::CloseTab { .. } => {
                due(&self.acted, subject_of(action), now, ACTION_RETRY_SECONDS)
            }
            Action::Notify(NoticeKind::LimitWarning { target, .. }) => {
                !self.warned.contains(target)
            }
            Action::Notify(kind) => due(
                &self.noticed,
                notice_subject(kind),
                now,
                NOTICE_INTERVAL_SECONDS,
            ),
            Action::ReportLimitReached { .. } => true,
        }
    }

    /// True when this limit's exhaustion was already reported. Raising the limit changes
    /// the allowance, so the same target can be reported again against the new one.
    pub fn limit_reported(&self, limit: &Limit) -> bool {
        self.reported.get(&limit.target) == Some(&limit.daily_minutes)
    }

    pub fn note(&mut self, action: &Action, outcome: Outcome) {
        match action {
            Action::QuitApp(_) | Action::CloseTab { .. } => {
                insert(&mut self.acted, subject_of(action), outcome.now);
            }
            Action::Notify(NoticeKind::LimitWarning { target, .. }) => {
                self.warned.insert(target.clone());
            }
            Action::Notify(kind) => insert(&mut self.noticed, notice_subject(kind), outcome.now),
            Action::ReportLimitReached {
                target,
                daily_minutes,
            } if outcome.succeeded => {
                self.reported.insert(target.clone(), *daily_minutes);
            }
            Action::ReportLimitReached { .. } => {}
        }
    }
}

fn insert(times: &mut BTreeMap<Subject, i64>, subject: Option<Subject>, now: i64) {
    if let Some(subject) = subject {
        times.insert(subject, now);
    }
}

fn due(times: &BTreeMap<Subject, i64>, subject: Option<Subject>, now: i64, every: i64) -> bool {
    let Some(last) = subject.and_then(|subject| times.get(&subject).copied()) else {
        return true;
    };
    !(0..every).contains(&now.saturating_sub(last))
}

fn subject_of(action: &Action) -> Option<Subject> {
    match action {
        Action::QuitApp(app) => Some(Subject::App(app.bundle_id().to_owned())),
        Action::CloseTab { domain, .. } => Some(Subject::Site(domain.clone())),
        Action::ReportLimitReached { .. } | Action::Notify(_) => None,
    }
}

fn notice_subject(kind: &NoticeKind) -> Option<Subject> {
    match kind {
        NoticeKind::AppBlocked { app, .. } => Some(Subject::App(app.bundle_id().to_owned())),
        NoticeKind::SiteBlocked { domain, .. } => Some(Subject::Site(domain.clone())),
        NoticeKind::LimitWarning { .. } => None,
    }
}

#[cfg(test)]
mod tests;
