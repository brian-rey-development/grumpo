use focus_core::{AppRef, Domain};

use super::activity::Activity;
pub use super::timing::MAX_SAMPLE_GAP;
use crate::storage::sessions::NewSession;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionEvent {
    Opened(NewSession),
    Extended {
        ended_at: i64,
    },
    /// Moves the end of the open session back, which `Extended` never does.
    Trimmed {
        ended_at: i64,
    },
    Closed,
}

#[derive(Debug)]
struct OpenSession {
    app: AppRef,
    domain: Option<Domain>,
    started_at: i64,
    last_seen: i64,
    /// Latest sample that kept the session alive with media, so time up to it is credited
    /// even though the user gave no input.
    media_until: i64,
}

impl OpenSession {
    fn is_same_target(&self, activity: &Activity) -> bool {
        self.app == activity.app && self.domain == activity.domain
    }

    fn is_continuous_until(&self, at: i64) -> bool {
        (0..=MAX_SAMPLE_GAP).contains(&(at - self.last_seen))
    }

    /// Where a session ends when the user went idle at sample time `at`: the last input,
    /// but never before the session started or its last media-credited sample, and never
    /// after the last time it was seen.
    fn end_after_idle(&self, activity: &Activity, at: i64) -> i64 {
        let idle = i64::try_from(activity.idle.as_secs()).unwrap_or(i64::MAX);
        let last_input = at.saturating_sub(idle);
        last_input
            .max(self.media_until)
            .max(self.started_at)
            .min(self.last_seen)
    }
}

/// Turns a stream of samples into session events. Pure: it never reads a clock or touches
/// storage, so the caller supplies the timestamp of every sample.
///
/// A session runs from the first engaged sample of an app and domain to the last one. When
/// the user switches target while staying engaged, the old session extends to the switching
/// sample so no time falls between the two. A missing app, a private window, a locked screen
/// or a gap close the session at the last sample that saw it, because the time after that is
/// unknown. Going idle trims the session back to the last input, so reading without
/// touching anything is credited for at most the idle threshold, and for as long as media
/// plays.
///
/// Time never runs backwards here: a sample earlier than the last session's last seen time
/// is treated as taken at that time, so sessions never overlap after a clock step.
#[derive(Debug, Default)]
pub struct SessionBuilder {
    open: Option<OpenSession>,
    floor: i64,
}

impl SessionBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    pub fn feed(&mut self, sample: Option<&Activity>, at: i64) -> Vec<SessionEvent> {
        let at = at.max(self.floor);
        let credited = sample.filter(|activity| activity.engaged && !activity.private);
        let Some(open) = self.open.take() else {
            return self.open_for(credited, at);
        };
        if !open.is_continuous_until(at) {
            return self.close_then_open(&open, credited, at);
        }
        match credited {
            Some(activity) if open.is_same_target(activity) => {
                self.continue_with(open, activity, at)
            }
            Some(activity) => self.switch_to(&open, activity, at),
            None => self.end(&open, sample, at),
        }
    }

    fn open_for(&mut self, credited: Option<&Activity>, at: i64) -> Vec<SessionEvent> {
        let Some(activity) = credited else {
            return Vec::new();
        };
        self.floor = at;
        self.open = Some(OpenSession {
            app: activity.app.clone(),
            domain: activity.domain.clone(),
            started_at: at,
            last_seen: at,
            media_until: at,
        });
        vec![SessionEvent::Opened(NewSession {
            app: activity.app.clone(),
            domain: activity.domain.clone(),
            started_at: at,
        })]
    }

    fn close_then_open(
        &mut self,
        open: &OpenSession,
        credited: Option<&Activity>,
        at: i64,
    ) -> Vec<SessionEvent> {
        self.floor = open.last_seen;
        let mut events = vec![SessionEvent::Closed];
        events.extend(self.open_for(credited, at));
        events
    }

    fn continue_with(
        &mut self,
        open: OpenSession,
        activity: &Activity,
        at: i64,
    ) -> Vec<SessionEvent> {
        let moved = at > open.last_seen;
        let media_until = if activity.credited_by_media() {
            at
        } else {
            open.media_until
        };
        self.floor = at;
        self.open = Some(OpenSession {
            last_seen: at,
            media_until,
            ..open
        });
        moved
            .then_some(SessionEvent::Extended { ended_at: at })
            .into_iter()
            .collect()
    }

    fn switch_to(&mut self, open: &OpenSession, next: &Activity, at: i64) -> Vec<SessionEvent> {
        let mut events = Vec::new();
        if at > open.last_seen {
            events.push(SessionEvent::Extended { ended_at: at });
        }
        events.push(SessionEvent::Closed);
        events.extend(self.open_for(Some(next), at));
        events
    }

    fn end(&mut self, open: &OpenSession, sample: Option<&Activity>, at: i64) -> Vec<SessionEvent> {
        if let Some(activity) = sample.filter(|activity| activity.went_idle()) {
            let ended_at = open.end_after_idle(activity, at);
            self.floor = ended_at;
            return vec![SessionEvent::Trimmed { ended_at }, SessionEvent::Closed];
        }
        self.floor = open.last_seen;
        vec![SessionEvent::Closed]
    }
}

#[cfg(test)]
mod clock_tests;
#[cfg(test)]
mod idle_tests;
#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
