//! Data transfer objects for Tauri commands and events. This module is the
//! contract with the UI: TypeScript bindings are generated from it.

mod apps;
mod block_screen;
mod clock;
mod command_error;
mod onboarding;
mod permissions;
mod reports;
mod service;
mod settings;
mod status;

pub use apps::KnownApp;
pub use block_screen::{BlockReason, BlockScreen, BlockTarget, UNASSIGNED_ID};
pub use clock::ClockStyle;
pub use command_error::{CommandError, CommandErrorCode, RuleSection, RulesFault, SiteProblem};
pub use onboarding::OnboardingState;
pub use permissions::{BrowserAccess, BrowserId, PermissionKind, PermissionStatus, Permissions};
pub use reports::{
    DayReport, DayTotal, FocusWindow, LimitUsage, TimelineSegment, UsageItem, UsageKind, WeekReport,
};
pub use service::ServiceInstallOutcome;
pub use settings::Settings;
pub use status::{DaemonUnavailableReason, FocusStatus};
