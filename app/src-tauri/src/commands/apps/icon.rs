use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use focus_core::AppRef;

use crate::error::AppError;
use crate::icon_cache::Icon;
use crate::state::AppState;

const ICON_SIZE_PX: u32 = 128;
const DATA_URL_PREFIX: &str = "data:image/png;base64,";

/// The icon as a PNG data URL, or `None` when the app is not installed. Rendering happens
/// outside the cache lock, and only definite answers are cached, never a failure.
pub fn app_icon(state: &AppState, bundle_id: &str) -> Result<Option<String>, AppError> {
    let app = AppRef::new(bundle_id, "")
        .map_err(|_| AppError::invalid_input("That is not a valid app identifier."))?;
    let key = app.bundle_id();
    if let Some(icon) = state.icons.get(key) {
        return Ok(icon.into_data_url());
    }
    let icon = match state.platform.icons.app_icon_png(key, ICON_SIZE_PX)? {
        Some(png) => Icon::Found(data_url(&png)),
        None => Icon::Missing,
    };
    state.icons.insert(key, icon.clone());
    Ok(icon.into_data_url())
}

pub fn data_url(png: &[u8]) -> String {
    format!("{DATA_URL_PREFIX}{}", STANDARD.encode(png))
}

#[cfg(test)]
mod tests;
