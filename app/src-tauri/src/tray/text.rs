//! The words in the menu bar, kept free of Tauri so they are easy to test.

use std::time::Duration;

use focus_core::ipc::{DaemonStatus, UnixSeconds, UnlockPhaseView};

use crate::dto::{DaemonUnavailableReason, FocusStatus};
use crate::time::{ceil_minutes, ClockFormatter};

const NO_BLOCK: &str = "No block active";
const QUIT: &str = "Quit Grumpo";
const QUIT_WHILE_BLOCKING: &str = "Quit Grumpo (stops app and tab blocking)";
/// A cooldown line counts minutes, so it only needs to be redrawn a few times a minute.
const COUNTDOWN_REDRAW: Duration = Duration::from_secs(15);

/// The status line. `now` is unix seconds and only matters for a cooldown, whose line counts
/// down and is therefore the one line that needs redrawing as time passes.
pub fn status_line(status: &FocusStatus, now: UnixSeconds, clock_text: &ClockFormatter) -> String {
    match status {
        FocusStatus::Unavailable { reason, .. } => unavailable_line(*reason).to_owned(),
        FocusStatus::Connected { status } => connected_line(status, now, clock_text),
    }
}

/// The label of the quit item, which warns that quitting ends enforcement while it is on.
pub fn quit_label(status: &FocusStatus) -> &'static str {
    if status
        .daemon()
        .is_some_and(|daemon| daemon.enforcement.active)
    {
        QUIT_WHILE_BLOCKING
    } else {
        QUIT
    }
}

/// How long until the line changes by itself, or `None` when it only changes with the status.
pub fn redraw_after(status: &FocusStatus) -> Option<Duration> {
    let cooling = status
        .daemon()
        .is_some_and(|daemon| matches!(daemon.unlock.phase, UnlockPhaseView::CoolingDown { .. }));
    cooling.then_some(COUNTDOWN_REDRAW)
}

pub fn is_unlocked(status: &FocusStatus) -> bool {
    status
        .daemon()
        .is_some_and(|daemon| matches!(daemon.unlock.phase, UnlockPhaseView::Unlocked { .. }))
}

fn connected_line(daemon: &DaemonStatus, now: UnixSeconds, clock_text: &ClockFormatter) -> String {
    match daemon.unlock.phase {
        UnlockPhaseView::Unlocked { until } => {
            with_time("Unlocked until", until, "Unlocked", clock_text)
        }
        UnlockPhaseView::CoolingDown { ready_at, .. } => {
            format!("Unlock in {} min", minutes_until(ready_at, now))
        }
        UnlockPhaseView::Ready { expires_at, .. } => with_time(
            "Unlock ready, confirm by",
            expires_at,
            "Unlock ready",
            clock_text,
        ),
        UnlockPhaseView::Locked => enforcement_line(daemon, clock_text),
    }
}

fn enforcement_line(daemon: &DaemonStatus, clock_text: &ClockFormatter) -> String {
    match (daemon.enforcement.active, daemon.enforcement.active_until) {
        (true, Some(until)) => with_time("Focus on until", until, "Focus on", clock_text),
        (true, None) => "Focus on".to_owned(),
        (false, _) => NO_BLOCK.to_owned(),
    }
}

fn unavailable_line(reason: DaemonUnavailableReason) -> &'static str {
    match reason {
        DaemonUnavailableReason::NotInstalled => "Grumpo service not installed",
        DaemonUnavailableReason::NotResponding => "Grumpo service not responding",
        DaemonUnavailableReason::ServiceOutdated => "Grumpo service needs updating",
        DaemonUnavailableReason::AppOutdated => "Grumpo needs updating",
        DaemonUnavailableReason::OtherUser => "Grumpo service belongs to another user",
    }
}

/// "{lead} {time}", or `without_time` when the instant has no local time to show.
fn with_time(
    lead: &str,
    at: UnixSeconds,
    without_time: &str,
    clock_text: &ClockFormatter,
) -> String {
    clock_text
        .clock(at)
        .map_or_else(|| without_time.to_owned(), |time| format!("{lead} {time}"))
}

/// Whole minutes left, rounded up so the line never promises a shorter wait than there is,
/// and never below one while the cooldown has not ended.
fn minutes_until(ready_at: UnixSeconds, now: UnixSeconds) -> u32 {
    let seconds = u32::try_from(ready_at.saturating_sub(now).max(0)).unwrap_or(u32::MAX);
    ceil_minutes(seconds)
}

#[cfg(test)]
mod tests;
