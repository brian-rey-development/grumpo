#![expect(
    unsafe_code,
    reason = "AXIsProcessTrusted* are raw C calls and the prompt key is an extern static"
)]

//! Whether this process may use the Accessibility API, and the request that prompts for it.

use objc2_application_services::{
    kAXTrustedCheckOptionPrompt, AXIsProcessTrusted, AXIsProcessTrustedWithOptions,
};
use objc2_core_foundation::{CFBoolean, CFDictionary, CFString};

pub(super) fn is_trusted() -> bool {
    // SAFETY: AXIsProcessTrusted takes no arguments and has no preconditions.
    unsafe { AXIsProcessTrusted() }
}

/// Shows the system prompt that leads to the Accessibility pane when the process is not
/// trusted. The prompt is asynchronous, so the returned value is the state at call time.
pub(super) fn request_trust() -> bool {
    // SAFETY: reading an immutable extern static that the framework initializes at load.
    let prompt_key: &CFString = unsafe { kAXTrustedCheckOptionPrompt };
    let options =
        CFDictionary::<CFString, CFBoolean>::from_slices(&[prompt_key], &[CFBoolean::new(true)]);
    // SAFETY: the dictionary maps the documented key to a CFBoolean.
    unsafe { AXIsProcessTrustedWithOptions(Some(options.as_opaque())) }
}
