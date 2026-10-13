use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use super::{LaunchAtLogin, LaunchAtLoginError};

#[derive(Debug, Default)]
pub struct FakeLaunch {
    enabled: AtomicBool,
    fail: AtomicBool,
    writes: AtomicUsize,
}

impl FakeLaunch {
    /// An item already registered under the current label, as after an earlier start.
    pub fn already_enabled() -> Self {
        Self {
            enabled: AtomicBool::new(true),
            ..Self::default()
        }
    }

    pub fn failing() -> Self {
        Self {
            fail: AtomicBool::new(true),
            ..Self::default()
        }
    }

    /// How many times `set_enabled` was called, successful or not.
    pub fn writes(&self) -> usize {
        self.writes.load(Ordering::SeqCst)
    }
}

impl LaunchAtLogin for FakeLaunch {
    fn is_enabled(&self) -> Result<bool, LaunchAtLoginError> {
        Ok(self.enabled.load(Ordering::SeqCst))
    }

    fn set_enabled(&self, enabled: bool) -> Result<(), LaunchAtLoginError> {
        self.writes.fetch_add(1, Ordering::SeqCst);
        if self.fail.load(Ordering::SeqCst) {
            return Err(tauri_plugin_autostart::Error::Anyhow("refused".into()).into());
        }
        self.enabled.store(enabled, Ordering::SeqCst);
        Ok(())
    }
}
