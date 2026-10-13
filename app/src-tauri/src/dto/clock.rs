use serde::Serialize;

/// How the system shows the time of day, so the UI formats clock times the way the menu bar
/// and the notifications do.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ClockStyle {
    /// A 12 hour clock with AM and PM.
    H12,
    /// A 24 hour clock.
    H23,
}
