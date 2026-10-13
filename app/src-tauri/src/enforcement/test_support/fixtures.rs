//! Fakes and builders shared by the enforcement tests.

use std::collections::BTreeSet;
use std::time::Duration;

use focus_core::ipc::{DaemonStatus, EnforcementReason};
use focus_core::test_support::limit;
use focus_core::LimitTarget;

use crate::dto::{BlockReason, BlockScreen, BlockTarget, UNASSIGNED_ID};
use crate::platform::Browser;
use crate::tracking::activity::Activity;

pub use crate::daemon_client::test_support::daemon_status;
pub use focus_core::test_support::{app, app_target, site, site_target};

pub const T0: i64 = 1_800_000_000;
pub const END: i64 = T0 + 3_600;

pub fn in_app(bundle_id: &str) -> Activity {
    Activity {
        app: app(bundle_id),
        domain: None,
        browser: None,
        engaged: true,
        private: false,
        idle: Duration::ZERO,
    }
}

pub fn browsing(browser: Browser, host: &str) -> Activity {
    Activity {
        app: app(browser.bundle_id()),
        domain: Some(site(host)),
        browser: Some(browser),
        engaged: true,
        private: false,
        idle: Duration::ZERO,
    }
}

/// A daemon status with blocking active until `END` for the given apps and sites.
pub fn blocking(apps: &[&str], sites: &[&str]) -> DaemonStatus {
    let mut status = daemon_status();
    status.enforcement.active = true;
    status.enforcement.reason = EnforcementReason::Schedule;
    status.enforcement.active_until = Some(END);
    status.enforcement.blocked_apps = apps.iter().map(|id| app(id)).collect();
    status.enforcement.blocked_sites = sites.iter().map(|host| site(host)).collect::<BTreeSet<_>>();
    status
}

pub fn with_limit(mut status: DaemonStatus, target: LimitTarget, minutes: u16) -> DaemonStatus {
    status.rules.limits.push(limit(target, minutes));
    status
}

/// The block screen for a target the schedule blocks until `END`.
pub fn schedule_block(target: BlockTarget) -> BlockScreen {
    BlockScreen {
        id: UNASSIGNED_ID,
        target,
        reason: BlockReason::Schedule,
        until: Some(END),
        strict: true,
    }
}

pub fn app_block(name: &str) -> BlockScreen {
    schedule_block(BlockTarget::App {
        name: name.to_owned(),
    })
}

pub fn site_block(domain: &str) -> BlockScreen {
    schedule_block(BlockTarget::Site {
        domain: domain.to_owned(),
    })
}

/// What the surface receives for the first block the hub presents, which stamps it with id 1.
pub fn first_shown(screen: BlockScreen) -> BlockScreen {
    BlockScreen { id: 1, ..screen }
}
