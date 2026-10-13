use tauri::State;

use super::blocking;
use crate::dto::{ClockStyle, CommandError};
use crate::platform::HourCycle;
use crate::state::AppState;

/// Read each time, because the person can switch the system clock setting while Grumpo runs.
#[tauri::command]
pub async fn get_clock_style(state: State<'_, AppState>) -> Result<ClockStyle, CommandError> {
    let state = state.inner().clone();
    blocking(move || Ok(style_of(state.platform.hour_cycle.hour_cycle()))).await
}

fn style_of(cycle: HourCycle) -> ClockStyle {
    match cycle {
        HourCycle::Twelve => ClockStyle::H12,
        HourCycle::TwentyFour => ClockStyle::H23,
    }
}

#[cfg(test)]
mod tests;
