use focus_core::{AppRef, Domain, LimitTarget};

use crate::dto::BlockReason;
use crate::platform::Browser;

/// What the policy wants done. The executor performs it, so deciding stays free of I/O.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    QuitApp(AppRef),
    CloseTab {
        browser: Browser,
        domain: Domain,
    },
    /// `daily_minutes` is the allowance in force when the decision was made, so the daemon
    /// can tell a report for a limit that has since been raised from one that still holds.
    ReportLimitReached {
        target: LimitTarget,
        daily_minutes: u16,
    },
    Notify(NoticeKind),
}

/// What a notification says, without its wording. Times are unix seconds, formatted by the
/// renderer in the user's local time. The two block kinds also carry what the block screen
/// shows, which is the window the person sees first and the notification is the fallback of.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NoticeKind {
    AppBlocked {
        app: AppRef,
        until: Option<i64>,
        reason: BlockReason,
        strict: bool,
    },
    SiteBlocked {
        domain: Domain,
        until: Option<i64>,
        reason: BlockReason,
        strict: bool,
    },
    LimitWarning {
        target: LimitTarget,
        seconds_left: u32,
    },
}
