//! Listing, quitting and probing applications through `NSWorkspace` / `NSRunningApplication`.

use focus_core::{AppRef, Domain};
use objc2::rc::{autoreleasepool, Retained};
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
use objc2_foundation::{NSArray, NSString};

use super::{browsers, frontmost::app_ref_of};
use crate::platform::{AppControl, Browser, PlatformError};

#[derive(Debug)]
pub(super) struct MacControl;

impl AppControl for MacControl {
    fn running_apps(&self) -> Result<Vec<AppRef>, PlatformError> {
        Ok(autoreleasepool(|_| collect_regular_apps()))
    }

    fn is_running(&self, bundle_id: &str) -> Result<bool, PlatformError> {
        Ok(is_running(bundle_id))
    }

    fn quit(&self, app: &AppRef) -> Result<(), PlatformError> {
        autoreleasepool(|_| terminate_all(app.bundle_id()))
    }

    fn close_active_tab_if(
        &self,
        browser: Browser,
        domain: &Domain,
    ) -> Result<bool, PlatformError> {
        browsers::close_active_tab_if(browser, domain)
    }
}

pub(super) fn is_running(bundle_id: &str) -> bool {
    autoreleasepool(|_| !instances(bundle_id).is_empty())
}

#[cfg(test)]
pub(super) fn running_pid(bundle_id: &str) -> Option<i32> {
    autoreleasepool(|_| {
        instances(bundle_id)
            .iter()
            .find(|app| !app.isTerminated())
            .map(|app| app.processIdentifier())
    })
}

fn instances(bundle_id: &str) -> Retained<NSArray<NSRunningApplication>> {
    NSRunningApplication::runningApplicationsWithBundleIdentifier(&NSString::from_str(bundle_id))
}

fn collect_regular_apps() -> Vec<AppRef> {
    let mut apps: Vec<AppRef> = NSWorkspace::sharedWorkspace()
        .runningApplications()
        .iter()
        .filter(|app| app.activationPolicy() == NSApplicationActivationPolicy::Regular)
        .filter_map(|app| app_ref_of(&app))
        .collect();
    apps.sort_by(|a, b| a.bundle_id().cmp(b.bundle_id()));
    apps.dedup();
    apps.sort_by_cached_key(|app| (app.name().to_lowercase(), app.bundle_id().to_owned()));
    apps
}

fn terminate_all(bundle_id: &str) -> Result<(), PlatformError> {
    let refused = instances(bundle_id)
        .iter()
        .filter(|app| !app.isTerminated() && !app.terminate())
        .count();
    if refused == 0 {
        return Ok(());
    }
    Err(super::failed(
        "quit app",
        format!("{refused} instance(s) did not accept the quit request"),
    ))
}
