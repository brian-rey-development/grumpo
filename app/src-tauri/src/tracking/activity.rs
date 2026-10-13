use std::time::Duration;

use focus_core::{AppRef, Domain};

use crate::platform::Browser;

/// Input-free time after which reading no longer counts as engagement unless media plays.
pub const IDLE_THRESHOLD: Duration = Duration::from_secs(120);

/// What the user is doing at one sample.
///
/// `engaged` is false after the idle threshold unless media is playing, and always false
/// while the screen is locked. A disengaged activity is still reported so enforcement can
/// see it. `private` marks an incognito window, which enforcement sees but storage never
/// does. `idle` is the time since the last input at the sample.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub app: AppRef,
    pub domain: Option<Domain>,
    pub browser: Option<Browser>,
    pub engaged: bool,
    pub private: bool,
    pub idle: Duration,
}

impl Activity {
    /// Engaged only because media plays, so the stretch since the last input is credited.
    pub fn credited_by_media(&self) -> bool {
        self.engaged && self.idle >= IDLE_THRESHOLD
    }

    /// The user stopped providing input long enough to be dropped from the session.
    pub fn went_idle(&self) -> bool {
        !self.engaged && self.idle >= IDLE_THRESHOLD
    }
}
