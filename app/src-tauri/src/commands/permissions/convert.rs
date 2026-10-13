use crate::dto::{BrowserId, PermissionKind, PermissionStatus};
use crate::platform::{Browser, Permission, PermissionState};

pub fn browser_id(browser: Browser) -> BrowserId {
    match browser {
        Browser::Chrome => BrowserId::Chrome,
        Browser::Brave => BrowserId::Brave,
        Browser::Edge => BrowserId::Edge,
        Browser::Safari => BrowserId::Safari,
        Browser::Firefox => BrowserId::Firefox,
    }
}

fn browser(id: BrowserId) -> Browser {
    match id {
        BrowserId::Chrome => Browser::Chrome,
        BrowserId::Brave => Browser::Brave,
        BrowserId::Edge => Browser::Edge,
        BrowserId::Safari => Browser::Safari,
        BrowserId::Firefox => Browser::Firefox,
    }
}

pub fn permission_status(state: PermissionState) -> PermissionStatus {
    match state {
        PermissionState::Granted => PermissionStatus::Granted,
        PermissionState::Denied => PermissionStatus::Denied,
        PermissionState::NotDetermined => PermissionStatus::NotDetermined,
        PermissionState::NotRunning => PermissionStatus::NotRunning,
        PermissionState::Unsupported => PermissionStatus::Unsupported,
    }
}

impl From<PermissionKind> for Permission {
    fn from(kind: PermissionKind) -> Self {
        match kind {
            PermissionKind::Accessibility => Self::Accessibility,
            PermissionKind::Automation { browser: id } => Self::Automation(browser(id)),
        }
    }
}

/// What the system reports now wins, so a grant or a refusal is never overridden by an older
/// reading of the tracker. Only "not determined" is open to refinement, because the probe
/// cannot tell "never asked" from "asked and refused" for a browser that is not frontmost,
/// while the tracker saw the refusal itself, which `tracker_saw_denial` tells.
pub fn merge_with_health(probed: PermissionState, tracker_saw_denial: bool) -> PermissionStatus {
    match probed {
        PermissionState::NotDetermined if tracker_saw_denial => PermissionStatus::Denied,
        state => permission_status(state),
    }
}

#[cfg(test)]
mod tests;
