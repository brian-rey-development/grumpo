//! Domain logic for Grumpo: rules, schedule, unlock policy, hosts rendering and the IPC protocol.
//!
//! Nothing here reads the clock, the filesystem or the environment. The only I/O is the
//! blocking socket transport in [`ipc::transport`], which both ends of the protocol share.

mod app_ref;
#[cfg(feature = "ts")]
pub mod bindings;
mod domain;
mod enforcement;
pub mod hosts;
pub mod ipc;
pub mod legacy;
mod rules;
#[cfg(any(test, feature = "test-support"))]
pub mod test_support;
pub mod time;
mod unlock;

pub use app_ref::{AppRef, AppRefError};
pub use domain::{Domain, DomainError, MAX_DOMAIN_LEN, MAX_LABEL_LEN};
pub use enforcement::{
    locked_loosenings, ActiveReason, EnforcementPlan, Exhausted, ExhaustedError,
};
pub use rules::{
    is_protected, loosenings, Coverage, Limit, LimitTarget, Loosening, RuleCollection, Rules,
    RulesError, Schedule, ValidatedRules, Window, MAX_APPS, MAX_LIMITS, MAX_LIMIT_MINUTES,
    MAX_SITES, MAX_WINDOWS, MIN_LIMIT_MINUTES,
};
pub use time::{Day, Now, TimeOfDay, TimeOfDayError};
pub use unlock::{
    cooldown_minutes_after, UnlockError, UnlockPhase, UnlockState, CONFIRM_WINDOW_MINUTES,
    COOLDOWN_STEPS_MINUTES, UNLOCK_DURATIONS_MINUTES, UNLOCK_SENTENCE,
};
