use std::collections::HashMap;

use crate::platform::{Browser, PlatformError, PlatformErrorKind};

/// Which probe call failed. Browsers are carried so each tab read tracks its own state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Source {
    Foreground,
    Idle,
    ScreenLock,
    Media,
    BrowserTab(Browser),
}

impl Source {
    fn label(self) -> &'static str {
        match self {
            Self::Foreground => "foreground",
            Self::Idle => "idle",
            Self::ScreenLock => "screen_lock",
            Self::Media => "media",
            Self::BrowserTab(_) => "browser_tab",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Transition {
    Unchanged,
    Started(PlatformErrorKind),
    Recovered,
}

/// Remembers the last error kind per source so a probe that fails every 2 s is logged once.
#[derive(Debug, Default)]
pub struct ErrorLog {
    last: HashMap<Source, PlatformErrorKind>,
}

impl ErrorLog {
    pub fn record(&mut self, source: Source, error: Option<PlatformErrorKind>) -> Transition {
        let previous = match error {
            Some(kind) => self.last.insert(source, kind),
            None => self.last.remove(&source),
        };
        match (previous, error) {
            (before, Some(kind)) if before != Some(kind) => Transition::Started(kind),
            (Some(_), None) => Transition::Recovered,
            _ => Transition::Unchanged,
        }
    }

    /// Logs on transitions only. The browser and the platform detail can name an app or a
    /// site, so they are kept at debug.
    pub fn log(&mut self, source: Source, error: Option<&PlatformError>) {
        match self.record(source, error.map(PlatformError::kind)) {
            Transition::Started(kind) => {
                tracing::warn!(
                    source = source.label(),
                    kind = %kind,
                    "probe failing"
                );
                tracing::debug!(?source, detail = ?error, "probe failure detail");
            }
            Transition::Recovered => {
                tracing::info!(source = source.label(), "probe recovered");
            }
            Transition::Unchanged => {}
        }
    }
}

#[cfg(test)]
mod tests;
