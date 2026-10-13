//! Names the fault in rules that were refused, so the UI can word it without reading Rust's
//! sentences.

use focus_core::{AppRef, Domain, DomainError, RuleCollection, RulesError};
use serde_json::Value;

use crate::dto::{RuleSection, RulesFault, SiteProblem};

impl From<&RulesError> for RulesFault {
    fn from(error: &RulesError) -> Self {
        match error {
            RulesError::TooMany { collection, .. } => Self::TooMany {
                section: section_of(*collection),
            },
            RulesError::WindowWithoutDays { number } => Self::WindowWithoutDays {
                number: window_number(*number),
            },
            RulesError::WindowNotForward { number } => Self::WindowNotForward {
                number: window_number(*number),
            },
            RulesError::LimitOutOfRange { target, minutes } => Self::LimitOutOfRange {
                target: target.clone(),
                minutes: *minutes,
            },
            RulesError::DuplicateLimit(target) => Self::DuplicateLimit {
                target: target.clone(),
            },
            RulesError::ProtectedApp { bundle_id } => Self::ProtectedApp {
                bundle_id: bundle_id.clone(),
            },
        }
    }
}

fn section_of(collection: RuleCollection) -> RuleSection {
    match collection {
        RuleCollection::Sites => RuleSection::Sites,
        RuleCollection::Apps => RuleSection::Apps,
        RuleCollection::Windows => RuleSection::Windows,
        RuleCollection::Limits => RuleSection::Limits,
    }
}

fn window_number(number: usize) -> u32 {
    u32::try_from(number).unwrap_or(u32::MAX)
}

impl From<&DomainError> for SiteProblem {
    fn from(error: &DomainError) -> Self {
        match error {
            DomainError::Empty => Self::Empty,
            DomainError::NonAscii => Self::NonAscii,
            DomainError::TooLong => Self::TooLong,
            DomainError::TooFewLabels => Self::TooFewLabels,
            DomainError::EmptyLabel => Self::EmptyLabel,
            DomainError::IpAddress => Self::IpAddress,
            DomainError::PublicSuffix(_) => Self::PublicSuffix,
            DomainError::LabelTooLong(_) => Self::LabelTooLong,
            DomainError::HyphenAtEdge(_) => Self::HyphenAtEdge,
            DomainError::InvalidCharacter(_) => Self::InvalidCharacter,
        }
    }
}

/// Finds why `raw` could not be read as rules. Reading stops at the first bad value without
/// saying which one it was, so the values are checked again one by one, sites first because
/// they are what people type.
pub fn diagnose_unreadable_rules(raw: &Value) -> RulesFault {
    first_bad_site(raw)
        .or_else(|| first_bad_app(raw))
        .unwrap_or(RulesFault::Unreadable)
}

fn first_bad_site(raw: &Value) -> Option<RulesFault> {
    site_texts(raw).find_map(|site| {
        let error = Domain::parse(site).err()?;
        Some(RulesFault::InvalidSite {
            site: site.to_owned(),
            problem: SiteProblem::from(&error),
        })
    })
}

fn first_bad_app(raw: &Value) -> Option<RulesFault> {
    app_values(raw)
        .any(|app| !is_valid_app(app))
        .then_some(RulesFault::InvalidApp)
}

fn site_texts(raw: &Value) -> impl Iterator<Item = &str> {
    let blocked = array_at(raw, "blockedSites")
        .filter_map(Value::as_str)
        .map(Some);
    let limited = limit_targets(raw, "site").map(|target| target.get("domain")?.as_str());
    blocked.chain(limited).flatten()
}

fn app_values(raw: &Value) -> impl Iterator<Item = &Value> {
    let blocked = array_at(raw, "blockedApps");
    let limited = limit_targets(raw, "app").filter_map(|target| target.get("app"));
    blocked.chain(limited)
}

fn limit_targets<'a>(raw: &'a Value, kind: &'a str) -> impl Iterator<Item = &'a Value> {
    array_at(raw, "limits")
        .filter_map(|limit| limit.get("target"))
        .filter(move |target| target.get("kind").and_then(Value::as_str) == Some(kind))
}

fn array_at<'a>(raw: &'a Value, key: &str) -> impl Iterator<Item = &'a Value> {
    raw.get(key).and_then(Value::as_array).into_iter().flatten()
}

fn is_valid_app(app: &Value) -> bool {
    let text = |key| app.get(key).and_then(Value::as_str);
    match (text("bundleId"), text("name")) {
        (Some(bundle_id), Some(name)) => AppRef::new(bundle_id, name).is_ok(),
        _ => true,
    }
}

#[cfg(test)]
mod tests;
