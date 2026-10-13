use focus_core::LimitTarget;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum UsageKind {
    App,
    Site,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct UsageItem {
    pub key: String,
    pub label: String,
    pub kind: UsageKind,
    pub seconds: u32,
}

/// Times are seconds since the local midnight that starts the reported day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct TimelineSegment {
    pub start: u32,
    pub end: u32,
    pub key: String,
    pub label: String,
    pub kind: UsageKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct FocusWindow {
    pub start: u32,
    pub end: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DayReport {
    /// Local calendar date, `YYYY-MM-DD`.
    pub date: String,
    pub total_seconds: u32,
    pub hourly: Vec<u32>,
    pub timeline: Vec<TimelineSegment>,
    pub focus_windows: Vec<FocusWindow>,
    pub top_apps: Vec<UsageItem>,
    pub top_sites: Vec<UsageItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct DayTotal {
    pub date: String,
    pub total_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct WeekReport {
    pub days: Vec<DayTotal>,
    pub total_seconds: u32,
    pub top_apps: Vec<UsageItem>,
    pub top_sites: Vec<UsageItem>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct LimitUsage {
    pub target: LimitTarget,
    pub used_seconds: u32,
    pub limit_seconds: u32,
    // Also true when the daemon lists the target as exhausted, which covers private windows.
    pub reached: bool,
}
