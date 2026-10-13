mod dates;
mod mapping;
mod windows;

use chrono::NaiveDate;
use tauri::State;

use super::blocking;
use crate::dto::{CommandError, DayReport, LimitUsage, WeekReport};
use crate::error::AppError;
use crate::state::AppState;
use crate::time::{local_day_for, today};

#[tauri::command]
pub async fn get_day_report(
    state: State<'_, AppState>,
    date: String,
) -> Result<DayReport, CommandError> {
    let date = dates::parse_date(&date)?;
    let state = state.inner().clone();
    blocking(move || day_report(&state, date)).await
}

#[tauri::command]
pub async fn get_week_report(
    state: State<'_, AppState>,
    end_date: String,
) -> Result<WeekReport, CommandError> {
    let end = dates::parse_date(&end_date)?;
    let state = state.inner().clone();
    blocking(move || week_report(&state, end)).await
}

#[tauri::command]
pub async fn get_limit_usage(state: State<'_, AppState>) -> Result<Vec<LimitUsage>, CommandError> {
    let state = state.inner().clone();
    blocking(move || limit_usage(&state)).await
}

fn day_report(state: &AppState, date: NaiveDate) -> Result<DayReport, AppError> {
    let day = local_day_for(date);
    let usage = state.store.day_usage(day)?;
    let status = state.status.current();
    let windows = status
        .daemon()
        .map(|daemon| windows::focus_windows_on(&daemon.rules.schedule, date, today().date))
        .unwrap_or_default();
    Ok(mapping::day_report(date, &day, usage, windows))
}

fn week_report(state: &AppState, end: NaiveDate) -> Result<WeekReport, AppError> {
    let days = dates::week_ending(end)?.map(local_day_for);
    Ok(mapping::week_report(state.store.week_usage(&days)?))
}

fn limit_usage(state: &AppState) -> Result<Vec<LimitUsage>, AppError> {
    let status = state.status.current();
    let Some(daemon) = status.daemon() else {
        return Ok(Vec::new());
    };
    let limits = &daemon.rules.limits;
    let targets: Vec<_> = limits.iter().map(|l| l.target.clone()).collect();
    let used = state.store.usage_seconds(today(), &targets)?;
    Ok(mapping::limit_usage(limits, &used, &daemon.exhausted))
}
