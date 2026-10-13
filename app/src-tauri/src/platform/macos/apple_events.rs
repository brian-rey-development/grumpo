#![expect(
    unsafe_code,
    reason = "AEDeterminePermissionToAutomateTarget and the AEDesc lifecycle are raw C calls"
)]

//! Asks the system whether this process may script an app. Reading the state never prompts;
//! requesting it may.

use std::ffi::c_void;
use std::ptr;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use objc2_core_services::{
    typeApplicationBundleID, typeNull, typeWildCard, AECreateDesc, AEDesc,
    AEDeterminePermissionToAutomateTarget, AEDisposeDesc,
};

use super::os_status::{OsStatus, ERR_WOULD_REQUIRE_CONSENT};

/// Long enough for a person to read the consent dialog and answer it.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(120);
const REQUEST_THREAD: &str = "focus-automation-request";
const BAD_ARGUMENT: OsStatus = -1;

/// An address descriptor naming an app by bundle id, disposed on drop.
struct TargetDescriptor(AEDesc);

impl TargetDescriptor {
    fn for_bundle_id(bundle_id: &str) -> Result<Self, OsStatus> {
        let length = i64::try_from(bundle_id.len()).map_err(|_| BAD_ARGUMENT)?;
        let mut desc = AEDesc {
            descriptorType: typeNull,
            dataHandle: ptr::null_mut(),
        };
        // SAFETY: the pointer and length describe `bundle_id`, which AECreateDesc copies,
        // and `desc` is a valid out parameter.
        let status = unsafe {
            AECreateDesc(
                typeApplicationBundleID,
                bundle_id.as_ptr().cast::<c_void>(),
                length,
                &raw mut desc,
            )
        };
        if status == 0 {
            Ok(Self(desc))
        } else {
            Err(OsStatus::from(status))
        }
    }
}

impl Drop for TargetDescriptor {
    fn drop(&mut self) {
        // SAFETY: the descriptor was created by AECreateDesc and is disposed exactly once.
        unsafe { AEDisposeDesc(&raw mut self.0) };
    }
}

/// Reads the Automation state for `bundle_id` without ever showing a prompt.
pub(super) fn automation_status(bundle_id: &str) -> OsStatus {
    determine(bundle_id, false)
}

/// Shows the consent prompt when the user has not decided yet and waits for the answer on a
/// fresh thread, because the call blocks until the dialog is dismissed. If nobody answers
/// within [`REQUEST_TIMEOUT`] the state is reported as undecided and the dialog stays up.
pub(super) fn request_automation(bundle_id: &str) -> OsStatus {
    let (sender, receiver) = mpsc::channel();
    let target = bundle_id.to_owned();
    let spawned = thread::Builder::new()
        .name(REQUEST_THREAD.into())
        .spawn(move || {
            if sender.send(determine(&target, true)).is_err() {
                tracing::debug!("automation request answered after the caller stopped waiting");
            }
        });
    if let Err(error) = spawned {
        tracing::warn!(%error, "could not start the automation request thread");
        return ERR_WOULD_REQUIRE_CONSENT;
    }
    receiver.recv_timeout(REQUEST_TIMEOUT).unwrap_or_else(|_| {
        tracing::warn!("automation consent request timed out");
        ERR_WOULD_REQUIRE_CONSENT
    })
}

fn determine(bundle_id: &str, ask_user: bool) -> OsStatus {
    let target = match TargetDescriptor::for_bundle_id(bundle_id) {
        Ok(target) => target,
        Err(status) => return status,
    };
    // SAFETY: `target.0` is a live descriptor and wildcards ask about every event.
    unsafe {
        AEDeterminePermissionToAutomateTarget(
            &raw const target.0,
            typeWildCard,
            typeWildCard,
            ask_user,
        )
    }
}
