//! Storage structs to command DTOs.

use chrono::NaiveDate;
use focus_core::{Limit, LimitTarget};

use super::dates::format_date;
use crate::dto::{
    DayReport, DayTotal, FocusWindow, LimitUsage, TimelineSegment, UsageItem, UsageKind, WeekReport,
};
use crate::storage::reports::{DayUsage, Segment, UsageEntry, WeekUsage};
use crate::time::units::{SECONDS_PER_HOUR, SECONDS_PER_MINUTE};
use crate::time::{LocalDay, HOURS_PER_DAY};

pub fn day_report(
    date: NaiveDate,
    day: &LocalDay,
    usage: DayUsage,
    focus_windows: Vec<FocusWindow>,
) -> DayReport {
    DayReport {
        date: format_date(date),
        total_seconds: usage.total_seconds,
        hourly: usage.hourly.to_vec(),
        timeline: usage
            .timeline
            .iter()
            .map(|segment| timeline_segment(day, segment))
            .collect(),
        focus_windows,
        top_apps: items(usage.top_apps, UsageKind::App),
        top_sites: items(usage.top_sites, UsageKind::Site),
    }
}

pub fn week_report(usage: WeekUsage) -> WeekReport {
    WeekReport {
        days: usage
            .days
            .iter()
            .map(|day| DayTotal {
                date: format_date(day.date),
                total_seconds: day.total_seconds,
            })
            .collect(),
        total_seconds: usage.total_seconds,
        top_apps: items(usage.top_apps, UsageKind::App),
        top_sites: items(usage.top_sites, UsageKind::Site),
    }
}

/// Pairs each limit with the time spent on its target. `used` comes back in the order of the
/// limits it was asked for, and a limit without a row counts as unused. A limit counts as
/// reached when the daemon lists it as exhausted, which also covers time spent in private
/// windows that the store never sees, or when the stored time alone has used it up.
pub fn limit_usage(
    limits: &[Limit],
    used: &[(LimitTarget, u32)],
    exhausted: &[LimitTarget],
) -> Vec<LimitUsage> {
    limits
        .iter()
        .map(|limit| {
            let used_seconds = used
                .iter()
                .find(|(target, _)| *target == limit.target)
                .map_or(0, |(_, seconds)| *seconds);
            let limit_seconds = u32::from(limit.daily_minutes) * SECONDS_PER_MINUTE;
            LimitUsage {
                target: limit.target.clone(),
                used_seconds,
                limit_seconds,
                reached: exhausted.contains(&limit.target) || used_seconds >= limit_seconds,
            }
        })
        .collect()
}

fn timeline_segment(day: &LocalDay, segment: &Segment) -> TimelineSegment {
    let (key, label, kind) = match &segment.domain {
        Some(domain) => (domain.clone(), domain.clone(), UsageKind::Site),
        None => (
            segment.app_id.clone(),
            segment.app_name.clone(),
            UsageKind::App,
        ),
    };
    TimelineSegment {
        start: seconds_of_day(day, segment.start),
        end: seconds_of_day(day, segment.end),
        key,
        label,
        kind,
    }
}

fn items(entries: Vec<UsageEntry>, kind: UsageKind) -> Vec<UsageItem> {
    entries
        .into_iter()
        .map(|entry| UsageItem {
            key: entry.key,
            label: entry.label,
            kind,
            seconds: entry.seconds,
        })
        .collect()
}

/// Seconds on the wall clock since the day began, so a segment lines up with the hourly
/// buckets and the focus windows even on a daylight-saving day with 23 or 25 hours.
fn seconds_of_day(day: &LocalDay, instant: i64) -> u32 {
    let bounds = day.hour_bounds();
    let hour = bounds[..HOURS_PER_DAY]
        .partition_point(|&start| start <= instant)
        .saturating_sub(1);
    let into_hour = (instant - bounds[hour]).clamp(0, SECONDS_PER_HOUR);
    let hour = i64::try_from(hour).unwrap_or_default();
    u32::try_from(hour * SECONDS_PER_HOUR + into_hour).unwrap_or_default()
}

#[cfg(test)]
mod tests;
