//! Performs the actions the policy chose, and tells the memory how each one went.

use std::sync::Arc;

use focus_core::ipc::Request;
use focus_core::{AppRef, Domain, LimitTarget};

use super::action::{Action, NoticeKind};
use super::memory::{EnforcementMemory, Outcome};
use super::notice::{block_screen, render};
use crate::block_screen::BlockScreenHub;
use crate::daemon_client::DaemonError;
use crate::notify::Notifier;
use crate::platform::{AppControl, Browser};
use crate::status::StatusHub;
use crate::time::{Calendar, ClockFormatter};

pub struct Executor {
    control: Arc<dyn AppControl>,
    status: Arc<StatusHub>,
    notifier: Arc<dyn Notifier>,
    block_screens: Arc<BlockScreenHub>,
    calendar: Arc<dyn Calendar>,
    clock_text: ClockFormatter,
}

impl Executor {
    pub fn new(
        control: Arc<dyn AppControl>,
        status: Arc<StatusHub>,
        notifier: Arc<dyn Notifier>,
        block_screens: Arc<BlockScreenHub>,
        calendar: Arc<dyn Calendar>,
        clock_text: ClockFormatter,
    ) -> Self {
        Self {
            control,
            status,
            notifier,
            block_screens,
            calendar,
            clock_text,
        }
    }

    pub fn run(&self, actions: &[Action], memory: &mut EnforcementMemory, now: i64) {
        for action in actions {
            let succeeded = self.perform(action, now);
            memory.note(action, Outcome { now, succeeded });
        }
    }

    fn perform(&self, action: &Action, now: i64) -> bool {
        match action {
            Action::QuitApp(app) => self.quit(app),
            Action::CloseTab { browser, domain } => self.close_tab(*browser, domain),
            Action::ReportLimitReached {
                target,
                daily_minutes,
            } => self.report(target, *daily_minutes, now),
            Action::Notify(kind) => {
                self.notify(kind);
                true
            }
        }
    }

    fn quit(&self, app: &AppRef) -> bool {
        match self.control.quit(app) {
            Ok(()) => {
                tracing::info!("asked a blocked app to quit");
                tracing::debug!(app = app.name(), "quit requested");
                true
            }
            Err(error) => {
                tracing::warn!(kind = %error.kind(), "could not quit a blocked app");
                tracing::debug!(%error, app = app.name(), "quit failed");
                false
            }
        }
    }

    fn close_tab(&self, browser: Browser, domain: &Domain) -> bool {
        match self.control.close_active_tab_if(browser, domain) {
            Ok(closed) => {
                tracing::info!(closed, "handled a blocked tab");
                true
            }
            Err(error) => {
                tracing::warn!(kind = %error.kind(), "could not close a blocked tab");
                tracing::debug!(%error, ?browser, domain = domain.as_str(), "tab close failed");
                false
            }
        }
    }

    fn report(&self, target: &LimitTarget, daily_minutes: u16, now: i64) -> bool {
        let Some(day) = self.calendar.instant(now).map(|at| at.local().date()) else {
            tracing::warn!("could not read the local day, so the limit report waits");
            return false;
        };
        let request = Request::ReportLimitReached {
            target: target.clone(),
            daily_minutes,
            day,
        };
        match self.status.send(&request) {
            Ok(_) => {
                tracing::info!("reported a reached limit to the daemon");
                true
            }
            Err(error) => {
                tracing::warn!(
                    kind = daemon_kind(&error),
                    "could not report a reached limit"
                );
                tracing::debug!(%error, %target, "limit report failed");
                false
            }
        }
    }

    /// A block goes to the block screen, and the notification is what the person gets when
    /// the window cannot be shown. A warning has no screen and is only a notification.
    fn notify(&self, kind: &NoticeKind) {
        let notice = render(kind, &self.clock_text);
        let Some(screen) = block_screen(kind) else {
            self.notifier.notify(notice);
            return;
        };
        let notifier = Arc::clone(&self.notifier);
        self.block_screens
            .present(screen, Box::new(move || notifier.notify(notice)));
    }
}

fn daemon_kind(error: &DaemonError) -> &'static str {
    match error {
        DaemonError::NotInstalled => "not_installed",
        DaemonError::NotResponding(_) => "not_responding",
        DaemonError::VersionMismatch { .. } => "version_mismatch",
        DaemonError::Renamed => "renamed",
        DaemonError::OtherUser => "other_user",
        DaemonError::Untrusted(_) => "untrusted",
        DaemonError::Rejected(_) => "rejected",
        DaemonError::Protocol(_) => "protocol",
    }
}
