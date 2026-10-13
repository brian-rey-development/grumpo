use std::collections::BTreeSet;
use std::fmt;
use std::ops::Deref;

use thiserror::Error;

use super::protected::{display_name, is_protected};
use super::{Limit, LimitTarget, Rules, Window};

// Sized so a worst-case ruleset still fits the IPC frame limits, which the codec tests pin.
pub const MAX_SITES: usize = 500;
pub const MAX_APPS: usize = 50;
pub const MAX_WINDOWS: usize = 50;
pub const MAX_LIMITS: usize = 30;
pub const MIN_LIMIT_MINUTES: u16 = 1;
pub const MAX_LIMIT_MINUTES: u16 = 1440;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuleCollection {
    Sites,
    Apps,
    Windows,
    Limits,
}

impl fmt::Display for RuleCollection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::Sites => "blocked sites",
            Self::Apps => "blocked apps",
            Self::Windows => "schedule windows",
            Self::Limits => "limits",
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum RulesError {
    #[error("too many {collection} ({count}), the maximum is {max}")]
    TooMany {
        collection: RuleCollection,
        count: usize,
        max: usize,
    },
    #[error("schedule window {number} has no days selected")]
    WindowWithoutDays { number: usize },
    #[error("schedule window {number} must start before it ends")]
    WindowNotForward { number: usize },
    #[error(
        "the limit for {target} must be between {min} and {max} minutes a day, got {minutes}",
        min = MIN_LIMIT_MINUTES,
        max = MAX_LIMIT_MINUTES
    )]
    LimitOutOfRange { target: LimitTarget, minutes: u16 },
    #[error("there is more than one limit for {0}")]
    DuplicateLimit(LimitTarget),
    #[error(
        "{} can't be blocked because Grumpo needs it to keep working",
        display_name(bundle_id)
    )]
    ProtectedApp { bundle_id: String },
}

/// Rules that passed [`Rules::validate`], the only form the engine accepts.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ValidatedRules(Rules);

impl ValidatedRules {
    pub fn into_inner(self) -> Rules {
        self.0
    }
}

impl Deref for ValidatedRules {
    type Target = Rules;

    fn deref(&self) -> &Rules {
        &self.0
    }
}

impl Rules {
    /// Checks the bounds, window order and limit uniqueness the engine relies on.
    pub fn validate(self) -> Result<ValidatedRules, RulesError> {
        check_size(RuleCollection::Sites, self.blocked_sites.len(), MAX_SITES)?;
        check_size(RuleCollection::Apps, self.blocked_apps.len(), MAX_APPS)?;
        let windows = &self.schedule.windows;
        check_size(RuleCollection::Windows, windows.len(), MAX_WINDOWS)?;
        check_size(RuleCollection::Limits, self.limits.len(), MAX_LIMITS)?;
        for (index, window) in windows.iter().enumerate() {
            validate_window(index + 1, window)?;
        }
        validate_limits(&self.limits)?;
        validate_not_protected(&self)?;
        Ok(ValidatedRules(self))
    }
}

fn check_size(collection: RuleCollection, count: usize, max: usize) -> Result<(), RulesError> {
    if count > max {
        return Err(RulesError::TooMany {
            collection,
            count,
            max,
        });
    }
    Ok(())
}

fn validate_window(number: usize, window: &Window) -> Result<(), RulesError> {
    if window.days.is_empty() {
        return Err(RulesError::WindowWithoutDays { number });
    }
    if window.start >= window.end {
        return Err(RulesError::WindowNotForward { number });
    }
    Ok(())
}

fn validate_limits(limits: &[Limit]) -> Result<(), RulesError> {
    let mut seen = BTreeSet::new();
    for limit in limits {
        if !(MIN_LIMIT_MINUTES..=MAX_LIMIT_MINUTES).contains(&limit.daily_minutes) {
            return Err(RulesError::LimitOutOfRange {
                target: limit.target.clone(),
                minutes: limit.daily_minutes,
            });
        }
        if !seen.insert(&limit.target) {
            return Err(RulesError::DuplicateLimit(limit.target.clone()));
        }
    }
    Ok(())
}

fn validate_not_protected(rules: &Rules) -> Result<(), RulesError> {
    let limited = rules.limits.iter().filter_map(|limit| match &limit.target {
        LimitTarget::App { app } => Some(app),
        LimitTarget::Site { .. } => None,
    });
    let protected = rules
        .blocked_apps
        .iter()
        .chain(limited)
        .find(|app| is_protected(app.bundle_id()));
    match protected {
        Some(app) => Err(RulesError::ProtectedApp {
            bundle_id: app.bundle_id().to_owned(),
        }),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests;
