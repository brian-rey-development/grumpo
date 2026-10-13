#![expect(
    unsafe_code,
    reason = "The Accessibility API is raw C with out pointers and unchecked CF casts"
)]

//! Reads attributes of Accessibility elements, every call charged against one deadline so a
//! hung application can stall a read for at most the budget.

use std::ptr::NonNull;
use std::time::{Duration, Instant};

use objc2_application_services::{AXError, AXUIElement};
use objc2_core_foundation::{CFArray, CFRetained, CFString, CFType};

use crate::platform::{Permission, PlatformError};

const MESSAGING_TIMEOUT_SECS: f32 = 0.25;
const OPERATION: &str = "firefox accessibility";

/// An element handle that may be kept between samples.
pub(super) struct Element(pub(super) CFRetained<AXUIElement>);

// SAFETY: AXUIElement is an immutable CF handle to a remote object, and the Accessibility
// client API may be called from any thread. Handles are only ever used behind a mutex.
unsafe impl Send for Element {}

impl Element {
    pub(super) fn is_same_as(&self, other: &AXUIElement) -> bool {
        let this: &CFType = &self.0;
        this == <AXUIElement as AsRef<CFType>>::as_ref(other)
    }
}

impl std::fmt::Debug for Element {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Element").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct Ax {
    deadline: Instant,
}

impl Ax {
    pub(super) fn with_budget(budget: Duration) -> Self {
        Self {
            deadline: Instant::now() + budget,
        }
    }

    pub(super) fn check_deadline(&self) -> Result<(), PlatformError> {
        if Instant::now() >= self.deadline {
            return Err(PlatformError::Timeout {
                operation: OPERATION,
            });
        }
        Ok(())
    }

    pub(super) fn application(pid: i32) -> CFRetained<AXUIElement> {
        // SAFETY: any pid is valid input, a dead process just fails later reads.
        let app = unsafe { AXUIElement::new_application(pid) };
        // SAFETY: `app` is a live element and the value is a plain float.
        unsafe { app.set_messaging_timeout(MESSAGING_TIMEOUT_SECS) };
        app
    }

    fn copy(
        &self,
        element: &AXUIElement,
        name: &str,
    ) -> Result<Option<CFRetained<CFType>>, PlatformError> {
        self.check_deadline()?;
        let attribute = CFString::from_str(name);
        let mut raw: *const CFType = std::ptr::null();
        // SAFETY: `raw` is a valid out pointer, and on success it receives an owned CF object.
        let error = unsafe { element.copy_attribute_value(&attribute, NonNull::from(&mut raw)) };
        match error {
            AXError::Success => Ok(NonNull::new(raw.cast_mut())
                // SAFETY: success means the copy rule handed us one retain count.
                .map(|owned| unsafe { CFRetained::from_raw(owned) })),
            AXError::APIDisabled => Err(PlatformError::PermissionDenied(Permission::Accessibility)),
            AXError::CannotComplete => Err(PlatformError::Timeout {
                operation: OPERATION,
            }),
            _ => Ok(None),
        }
    }

    pub(super) fn element(
        &self,
        element: &AXUIElement,
        name: &str,
    ) -> Result<Option<CFRetained<AXUIElement>>, PlatformError> {
        Ok(self
            .copy(element, name)?
            .and_then(|value| value.downcast::<AXUIElement>().ok()))
    }

    pub(super) fn string(
        &self,
        element: &AXUIElement,
        name: &str,
    ) -> Result<Option<String>, PlatformError> {
        Ok(self
            .copy(element, name)?
            .and_then(|value| value.downcast_ref::<CFString>().map(ToString::to_string)))
    }

    pub(super) fn children(
        &self,
        element: &AXUIElement,
        name: &str,
    ) -> Result<Vec<CFRetained<AXUIElement>>, PlatformError> {
        let Some(Ok(array)) = self
            .copy(element, name)?
            .map(CFRetained::downcast::<CFArray>)
        else {
            return Ok(Vec::new());
        };
        // SAFETY: AXChildren is documented as an array of AXUIElementRef.
        let typed = unsafe { CFRetained::cast_unchecked::<CFArray<AXUIElement>>(array) };
        let mut children = Vec::with_capacity(typed.len());
        for child in typed.iter() {
            self.check_deadline()?;
            children.push(child);
        }
        Ok(children)
    }
}
