use tauri::State;

use crate::dto::BlockScreen;
use crate::state::AppState;

/// What the block screen shows now. The window reads it when it opens, because a block can
/// arrive before the page has started listening for the event.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri hands a command its state by value"
)]
pub fn get_block_screen(state: State<'_, AppState>) -> Option<BlockScreen> {
    state.block_screens.current()
}

/// `id` is the id of the block the person is dismissing. A dismissal for a block that was
/// already replaced does nothing, and leaving `id` out dismisses whatever is up.
#[tauri::command]
#[expect(
    clippy::needless_pass_by_value,
    reason = "Tauri hands a command its state by value"
)]
pub fn dismiss_block_screen(state: State<'_, AppState>, id: Option<u64>) {
    state.block_screens.dismiss(id);
}
