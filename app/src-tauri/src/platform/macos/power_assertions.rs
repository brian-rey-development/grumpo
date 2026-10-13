#![expect(
    unsafe_code,
    reason = "IOPMCopyAssertionsByProcess is a raw IOKit call with an out pointer"
)]

//! A plain-Rust snapshot of the system's power assertions, so the media rules stay pure.

use std::ptr::{self, NonNull};

use objc2_core_foundation::{CFArray, CFDictionary, CFNumber, CFRetained, CFString, CFType};
use objc2_io_kit::{kIOReturnSuccess, IOPMCopyAssertionsByProcess};

use crate::platform::PlatformError;

const OPERATION: &str = "power assertions";
const KEY_TYPE: &str = "AssertType";
const KEY_LEVEL: &str = "AssertLevel";
const KEY_ON_BEHALF_OF_PID: &str = "AssertionOnBehalfOfPID";

type AssertionDict = CFDictionary<CFString, CFType>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct Assertion {
    pub holder_pid: i32,
    pub kind: String,
    pub level: i32,
    pub on_behalf_of_pid: Option<i32>,
}

pub(super) fn snapshot() -> Result<Vec<Assertion>, PlatformError> {
    let (pids, lists) = copy_by_process()?.to_vecs();
    let mut assertions = Vec::new();
    for (pid, list) in pids.iter().zip(lists) {
        let Some(holder_pid) = pid.downcast_ref::<CFNumber>().and_then(CFNumber::as_i32) else {
            continue;
        };
        if let Ok(list) = list.downcast::<CFArray>() {
            assertions.extend(parse_list(holder_pid, list));
        }
    }
    Ok(assertions)
}

fn copy_by_process() -> Result<CFRetained<CFDictionary<CFType, CFType>>, PlatformError> {
    let mut raw: *const CFDictionary = ptr::null();
    // SAFETY: `raw` is a valid, writable out pointer for the duration of the call.
    let status = unsafe { IOPMCopyAssertionsByProcess(&raw mut raw) };
    if status != kIOReturnSuccess {
        return Err(super::failed(OPERATION, format!("IOReturn {status:#x}")));
    }
    let raw = NonNull::new(raw.cast_mut())
        .ok_or_else(|| super::failed(OPERATION, "no dictionary returned"))?;
    // SAFETY: the Copy rule hands us one retain count, which `CFRetained` releases on drop.
    let dictionary = unsafe { CFRetained::from_raw(raw) };
    // SAFETY: every CF object is a CFType, so typing keys and values that way is always
    // valid. Each one is still checked with `downcast_ref` before it is used as anything
    // more specific.
    Ok(unsafe { CFRetained::cast_unchecked(dictionary) })
}

fn parse_list(holder_pid: i32, list: CFRetained<CFArray>) -> Vec<Assertion> {
    // SAFETY: every CF object is a CFType, and each element is downcast before use.
    let list = unsafe { CFRetained::cast_unchecked::<CFArray<CFType>>(list) };
    list.iter()
        .filter_map(|entry| {
            let dict = entry.downcast::<CFDictionary>().ok()?;
            // SAFETY: assertion dictionaries are keyed by CFString, and a key of another
            // type would only fail to match. Values are read as CFType and downcast.
            let dict = unsafe { CFRetained::cast_unchecked::<AssertionDict>(dict) };
            parse_entry(holder_pid, &dict)
        })
        .collect()
}

fn parse_entry(holder_pid: i32, dict: &AssertionDict) -> Option<Assertion> {
    let text = |key: &'static str| dict.get(&CFString::from_static_str(key));
    let number = |key: &'static str| {
        text(key).and_then(|value| value.downcast_ref::<CFNumber>().and_then(CFNumber::as_i32))
    };
    let kind = text(KEY_TYPE)?.downcast_ref::<CFString>()?.to_string();
    Some(Assertion {
        holder_pid,
        kind,
        level: number(KEY_LEVEL).unwrap_or(0),
        on_behalf_of_pid: number(KEY_ON_BEHALF_OF_PID),
    })
}
