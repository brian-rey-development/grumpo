use serde::Serialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum ServiceInstallOutcome {
    Installed,
    /// The person dismissed the password prompt, which is not a failure.
    Cancelled,
}
