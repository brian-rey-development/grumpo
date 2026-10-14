use chrono::{Local, Utc};
use focus_core::Now;

pub trait Clock {
    fn now(&self) -> Now;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> Now {
        let utc = Utc::now();
        Now::new(utc, utc.with_timezone(&Local).naive_local())
    }
}

#[cfg(test)]
pub mod test_support;
