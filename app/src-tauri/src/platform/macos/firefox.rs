//! Firefox does not script its tabs, so the address is read from the URL field of the focused
//! window through the Accessibility API, best effort and strictly bounded.
//!
//! Finding the field walks the window's interface tree, which is slow, so the element is
//! cached per window. Steady state is two cheap calls: the focused window, to notice a
//! window change, and the field's value.

use std::sync::Mutex;
use std::time::Duration;

use focus_core::Domain;
use objc2_application_services::AXUIElement;
use objc2_core_foundation::CFRetained;

use super::accessibility;
use super::ax::{Ax, Element};
use super::firefox_search::find_url_field;
use crate::concurrency::LockExt;
use crate::platform::{Permission, PlatformError, TabReading};

const READ_BUDGET: Duration = Duration::from_millis(1200);
const ATTR_ROLE: &str = "AXRole";
const ATTR_FOCUSED_WINDOW: &str = "AXFocusedWindow";
const ATTR_VALUE: &str = "AXValue";

#[derive(Debug, Default)]
pub(super) struct FirefoxReader {
    cache: Mutex<Option<CachedField>>,
}

#[derive(Debug)]
struct CachedField {
    pid: i32,
    window: Element,
    field: Element,
}

impl FirefoxReader {
    pub(super) fn active_tab(&self, pid: i32) -> Result<TabReading, PlatformError> {
        let address = self.read_address(pid)?;
        Ok(TabReading {
            domain: address.and_then(|address| Domain::from_address(&address)),
            private: false,
        })
    }

    fn read_address(&self, pid: i32) -> Result<Option<String>, PlatformError> {
        if !accessibility::is_trusted() {
            return Err(PlatformError::PermissionDenied(Permission::Accessibility));
        }
        let ax = Ax::with_budget(READ_BUDGET);
        let mut cache = self.cache.lock_or_recover();
        let app = Ax::application(pid);
        let Some(window) = ax.element(&app, ATTR_FOCUSED_WINDOW)? else {
            *cache = None;
            return Ok(None);
        };
        if let Some(address) = cached_address(&ax, cache.as_ref(), pid, &window) {
            return Ok(Some(address));
        }
        *cache = None;
        // Firefox builds its accessibility tree only once a client asks the application for its
        // role, so until then every window is empty. The tree arrives by the next read.
        ax.string(&app, ATTR_ROLE)?;
        let (address, field) = search_and_read(&ax, &window)?;
        *cache = field.map(|field| CachedField {
            pid,
            window: Element(window),
            field: Element(field),
        });
        Ok(address)
    }
}

fn search_and_read(
    ax: &Ax,
    window: &AXUIElement,
) -> Result<(Option<String>, Option<CFRetained<AXUIElement>>), PlatformError> {
    let Some(field) = find_url_field(ax, window)? else {
        return Ok((None, None));
    };
    Ok((ax.string(&field, ATTR_VALUE)?, Some(field)))
}

/// `None` when the cache belongs to another window or process or its field no longer
/// answers, in which case the caller searches again.
fn cached_address(
    ax: &Ax,
    cached: Option<&CachedField>,
    pid: i32,
    window: &AXUIElement,
) -> Option<String> {
    let cached = cached.filter(|c| c.pid == pid && c.window.is_same_as(window))?;
    match ax.string(&cached.field.0, ATTR_VALUE) {
        Ok(address) => address,
        Err(error) => {
            tracing::debug!(%error, "cached firefox url field failed, searching again");
            None
        }
    }
}
