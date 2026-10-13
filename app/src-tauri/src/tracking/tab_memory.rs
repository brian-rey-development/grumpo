use std::time::Duration;

use crate::platform::{Browser, TabReading};

/// How often the same frontmost browser is asked for its tab. The host is carried between
/// reads, so a tab switch can show up this much late, and a tab read costs a process spawn.
pub const TAB_REFRESH_INTERVAL: Duration = Duration::from_secs(4);
/// Consecutive failed reads that keep reporting the last known tab before it is dropped.
pub const MAX_CARRIED_FAILURES: u8 = 3;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// Use this reading without touching the browser.
    Reuse(TabReading),
    Read,
}

#[derive(Debug)]
struct Remembered {
    browser: Browser,
    pid: i32,
    reading: TabReading,
    read_at: i64,
    failures: u8,
}

/// The last tab read per frontmost browser process, which decides when a new read is due.
#[derive(Debug, Default)]
pub struct TabMemory {
    last: Option<Remembered>,
}

impl TabMemory {
    /// A disengaged sample never reads. A changed browser or pid reads at once, and the same
    /// browser reads again once the refresh interval has passed.
    pub fn decide(&self, browser: Browser, pid: i32, engaged: bool, now: i64) -> Decision {
        let known = self.matching(browser, pid);
        if !engaged {
            return Decision::Reuse(known.map(|r| r.reading.clone()).unwrap_or_default());
        }
        match known {
            Some(r) if (0..refresh_seconds()).contains(&(now - r.read_at)) => {
                Decision::Reuse(r.reading.clone())
            }
            _ => Decision::Read,
        }
    }

    /// Stores the outcome of a read and returns the reading to report. A failure carries the
    /// previous reading forward for up to [`MAX_CARRIED_FAILURES`] reads in a row.
    pub fn record(
        &mut self,
        browser: Browser,
        pid: i32,
        now: i64,
        outcome: Option<TabReading>,
    ) -> TabReading {
        let previous = self.matching(browser, pid);
        let failures = if outcome.is_some() {
            0
        } else {
            previous.map_or(1, |r| r.failures.saturating_add(1))
        };
        let reading = outcome.unwrap_or_else(|| carried(previous, failures));
        self.last = Some(Remembered {
            browser,
            pid,
            reading: reading.clone(),
            read_at: now,
            failures,
        });
        reading
    }

    fn matching(&self, browser: Browser, pid: i32) -> Option<&Remembered> {
        self.last
            .as_ref()
            .filter(|r| r.browser == browser && r.pid == pid)
    }
}

fn carried(previous: Option<&Remembered>, failures: u8) -> TabReading {
    previous
        .filter(|_| failures <= MAX_CARRIED_FAILURES)
        .map(|r| r.reading.clone())
        .unwrap_or_default()
}

fn refresh_seconds() -> i64 {
    i64::try_from(TAB_REFRESH_INTERVAL.as_secs()).unwrap_or(i64::MAX)
}

#[cfg(test)]
mod tests;
