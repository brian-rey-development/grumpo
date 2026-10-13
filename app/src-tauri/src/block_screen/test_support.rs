use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Mutex;

use super::{BlockSurface, Fallback};
use crate::concurrency::LockExt;
use crate::dto::BlockScreen;

#[derive(Debug, Default)]
pub struct FakeSurface {
    shown: Mutex<Vec<BlockScreen>>,
    hides: AtomicUsize,
    failing: AtomicBool,
}

impl FakeSurface {
    /// A window that cannot be shown, which runs the fallback at once.
    pub fn failing() -> Self {
        let surface = Self::default();
        surface.failing.store(true, Ordering::SeqCst);
        surface
    }

    pub fn shown(&self) -> Vec<BlockScreen> {
        self.shown.lock_or_recover().clone()
    }

    pub fn hides(&self) -> usize {
        self.hides.load(Ordering::SeqCst)
    }
}

impl BlockSurface for FakeSurface {
    fn show(&self, screen: BlockScreen, on_failure: Fallback) {
        if self.failing.load(Ordering::SeqCst) {
            on_failure();
            return;
        }
        self.shown.lock_or_recover().push(screen);
    }

    fn hide(&self) {
        self.hides.fetch_add(1, Ordering::SeqCst);
    }
}
