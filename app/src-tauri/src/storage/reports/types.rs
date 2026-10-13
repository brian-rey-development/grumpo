use chrono::NaiveDate;

use crate::time::HOURS_PER_DAY;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Segment {
    pub start: i64,
    pub end: i64,
    pub app_id: String,
    pub app_name: String,
    pub domain: Option<String>,
}

/// `key` is stable across renames (bundle id or domain), `label` is what the UI shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UsageEntry {
    pub key: String,
    pub label: String,
    pub seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayUsage {
    pub total_seconds: u32,
    pub hourly: [u32; HOURS_PER_DAY],
    pub timeline: Vec<Segment>,
    pub top_apps: Vec<UsageEntry>,
    pub top_sites: Vec<UsageEntry>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DayTotal {
    pub date: NaiveDate,
    pub total_seconds: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WeekUsage {
    pub days: Vec<DayTotal>,
    pub total_seconds: u32,
    pub top_apps: Vec<UsageEntry>,
    pub top_sites: Vec<UsageEntry>,
}
