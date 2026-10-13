//! The block screen window, shown when enforcement closes a blocked tab or quits a blocked
//! app. The hub remembers what the window currently shows, so the window can read it when it
//! opens and enforcement can replace it while it is already open.

mod fallback;
#[cfg(target_os = "macos")]
mod overlay;
#[cfg(target_os = "macos")]
mod screen_choice;
mod tauri_surface;

use std::sync::{Arc, Mutex};

use crate::concurrency::LockExt;
use crate::dto::BlockScreen;

pub use tauri_surface::TauriBlockSurface;

#[cfg(test)]
pub(crate) mod test_support;

/// Runs when the window could not be shown, so the person still hears about the block.
pub type Fallback = Box<dyn FnOnce() + Send>;

/// The window behind the hub. Neither method waits for the main thread, because enforcement
/// calls them from its worker while the main thread may be joining that worker on exit.
pub trait BlockSurface: Send + Sync {
    /// Shows the window with `screen`, creating it on first use. `on_failure` runs if that
    /// fails, wherever the attempt was made.
    fn show(&self, screen: BlockScreen, on_failure: Fallback);

    /// Hides the window and gives focus back to the app the person was in.
    fn hide(&self);
}

pub struct BlockScreenHub {
    surface: Arc<dyn BlockSurface>,
    state: Mutex<HubState>,
}

#[derive(Debug, Default)]
struct HubState {
    current: Option<BlockScreen>,
    last_id: u64,
}

impl std::fmt::Debug for BlockScreenHub {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BlockScreenHub").finish_non_exhaustive()
    }
}

impl BlockScreenHub {
    pub fn new(surface: Arc<dyn BlockSurface>) -> Self {
        Self {
            surface,
            state: Mutex::new(HubState::default()),
        }
    }

    /// Shows `screen` under a fresh id. A block that arrives while the window is up replaces
    /// its content, so the latest one always wins and windows never stack. The content is
    /// stored before the surface is asked to show it, so a page that reads `current` while the
    /// window opens never gets the previous block. The lock stays held while the surface
    /// queues its work, which keeps a show and a hide in the order they were decided.
    pub fn present(&self, mut screen: BlockScreen, on_failure: Fallback) {
        let mut state = self.state.lock_or_recover();
        state.last_id += 1;
        screen.id = state.last_id;
        state.current = Some(screen.clone());
        self.surface.show(screen, on_failure);
    }

    /// What the window shows now, or `None` once it was dismissed.
    pub fn current(&self) -> Option<BlockScreen> {
        self.state.lock_or_recover().current.clone()
    }

    /// Dismisses what the window shows. With `Some(id)` it only acts when that block is still
    /// the current one, so a dismissal that was meant for an earlier block cannot swallow a
    /// newer one. `None` dismisses whatever is up.
    pub fn dismiss(&self, id: Option<u64>) {
        let mut state = self.state.lock_or_recover();
        let current_id = state.current.as_ref().map(|current| current.id);
        if id.is_some() && id != current_id {
            return;
        }
        state.current = None;
        self.surface.hide();
    }
}

#[cfg(test)]
mod tests;
