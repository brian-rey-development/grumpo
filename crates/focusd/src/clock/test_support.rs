use std::sync::{Arc, Mutex};

use focus_core::test_support::{after, now_at};
use focus_core::Now;

use super::Clock;

/// A clock whose local time equals UTC, shared between clones so a test can move it.
#[derive(Debug, Clone)]
pub struct FakeClock(Arc<Mutex<Now>>);

impl FakeClock {
    pub fn at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> Self {
        Self(Arc::new(Mutex::new(now_at(year, month, day, hour, minute))))
    }

    pub fn advance_minutes(&self, minutes: i64) {
        let mut now = self.0.lock().unwrap();
        *now = after(&now, minutes);
    }
}

impl Clock for FakeClock {
    fn now(&self) -> Now {
        *self.0.lock().unwrap()
    }
}
