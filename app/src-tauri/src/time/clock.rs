/// Source of unix seconds, injected so tests control time.
pub trait Clock: Send + Sync {
    fn now_unix(&self) -> i64;
}

#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now_unix(&self) -> i64 {
        chrono::Utc::now().timestamp()
    }
}
