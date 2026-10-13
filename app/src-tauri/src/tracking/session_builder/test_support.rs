use super::*;
use crate::platform::Browser;
use crate::tracking::activity::IDLE_THRESHOLD;
use focus_core::test_support::app;
use std::time::Duration;

pub fn activity(bundle_id: &str, host: Option<&str>, engaged: bool) -> Activity {
    Activity {
        app: app(bundle_id),
        domain: host.map(|h| Domain::parse(h).unwrap()),
        browser: Browser::from_bundle_id(bundle_id),
        engaged,
        private: false,
        idle: Duration::ZERO,
    }
}

pub fn idle_for(mut activity: Activity, seconds: u64) -> Activity {
    activity.idle = Duration::from_secs(seconds);
    activity
}

pub fn private(mut activity: Activity) -> Activity {
    activity.private = true;
    activity
}

pub fn threshold() -> u64 {
    IDLE_THRESHOLD.as_secs()
}

pub fn opened(bundle_id: &str, host: Option<&str>, at: i64) -> SessionEvent {
    SessionEvent::Opened(NewSession {
        app: app(bundle_id),
        domain: host.map(|h| Domain::parse(h).unwrap()),
        started_at: at,
    })
}

pub fn extended(ended_at: i64) -> SessionEvent {
    SessionEvent::Extended { ended_at }
}
