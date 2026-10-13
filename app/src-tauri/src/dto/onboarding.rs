use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct OnboardingState {
    /// Whether the person has finished the welcome flow, so the app opens straight to Today.
    pub completed: bool,
}
