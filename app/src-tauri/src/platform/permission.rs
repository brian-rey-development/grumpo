use std::fmt;

use super::Browser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Permission {
    Accessibility,
    Automation(Browser),
}

impl fmt::Display for Permission {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Accessibility => f.write_str("accessibility"),
            Self::Automation(browser) => write!(f, "automation for {browser:?}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionState {
    Granted,
    Denied,
    NotDetermined,
    /// Automation cannot be checked because the browser is not running.
    NotRunning,
    Unsupported,
}
