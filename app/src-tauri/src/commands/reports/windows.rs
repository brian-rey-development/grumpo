use chrono::{Datelike, NaiveDate};
use focus_core::{Day, Schedule};

use crate::dto::FocusWindow;
use crate::time::units::SECONDS_PER_MINUTE;

/// The blocking windows of the current schedule, but only for today. The schedule can be
/// edited at any time, so drawing it over a past day would show blocks that were never in
/// force then, and a day that has not come yet has no usage to set them against.
pub fn focus_windows_on(
    schedule: &Schedule,
    date: NaiveDate,
    today: NaiveDate,
) -> Vec<FocusWindow> {
    if date != today {
        return Vec::new();
    }
    focus_windows(schedule, date)
}

/// The blocking windows that apply on `date`'s weekday, sorted, with overlapping and
/// touching ones joined so the UI draws one band for each stretch.
fn focus_windows(schedule: &Schedule, date: NaiveDate) -> Vec<FocusWindow> {
    let day = Day::from(date.weekday());
    let mut spans: Vec<(u32, u32)> = schedule
        .windows
        .iter()
        .filter(|window| window.days.contains(&day))
        .map(|window| {
            (
                u32::from(window.start.minutes()) * SECONDS_PER_MINUTE,
                u32::from(window.end.minutes()) * SECONDS_PER_MINUTE,
            )
        })
        .collect();
    spans.sort_unstable();
    merge(spans)
}

fn merge(spans: Vec<(u32, u32)>) -> Vec<FocusWindow> {
    let mut merged: Vec<FocusWindow> = Vec::new();
    for (start, end) in spans {
        match merged.last_mut() {
            Some(last) if start <= last.end => last.end = last.end.max(end),
            _ => merged.push(FocusWindow { start, end }),
        }
    }
    merged
}

#[cfg(test)]
mod tests;
