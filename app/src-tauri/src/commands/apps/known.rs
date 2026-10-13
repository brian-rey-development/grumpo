use std::collections::BTreeMap;

use focus_core::{is_protected, AppRef};

use crate::dto::KnownApp;

/// Running apps and apps from recent history as one list, one entry per bundle id, sorted
/// by name. A running app keeps its live name and is marked running.
pub fn merge(running: Vec<AppRef>, history: Vec<AppRef>) -> Vec<KnownApp> {
    let mut by_id: BTreeMap<String, KnownApp> = BTreeMap::new();
    for app in history {
        by_id.insert(
            app.bundle_id().to_owned(),
            KnownApp {
                app,
                running: false,
            },
        );
    }
    for app in running {
        by_id.insert(app.bundle_id().to_owned(), KnownApp { app, running: true });
    }
    let mut apps: Vec<KnownApp> = by_id.into_values().collect();
    apps.sort_by_cached_key(|known| {
        (
            known.app.name().to_lowercase(),
            known.app.bundle_id().to_owned(),
        )
    });
    apps
}

/// Drops the apps that must never be offered for blocking: the ones Grumpo protects so a rule
/// cannot lock the person out of their Mac, and Grumpo itself.
pub fn without_unblockable(apps: Vec<KnownApp>, own_bundle_id: &str) -> Vec<KnownApp> {
    apps.into_iter()
        .filter(|known| {
            let id = known.app.bundle_id();
            !is_protected(id) && !id.eq_ignore_ascii_case(own_bundle_id)
        })
        .collect()
}

#[cfg(test)]
mod tests;
