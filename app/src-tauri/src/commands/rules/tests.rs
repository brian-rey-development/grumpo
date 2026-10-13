use serde_json::json;

use super::*;
use crate::dto::{CommandError, CommandErrorCode, RulesFault, SiteProblem};

fn rules_with_site(site: &str) -> Value {
    json!({
        "blockedSites": [site],
        "blockedApps": [],
        "schedule": { "windows": [] },
        "limits": []
    })
}

fn rules_with_app(bundle_id: &str) -> Value {
    json!({
        "blockedSites": [],
        "blockedApps": [{ "bundleId": bundle_id, "name": "Thing" }],
        "schedule": { "windows": [] },
        "limits": []
    })
}

fn failure(result: Result<Rules, AppError>) -> CommandError {
    CommandError::from(result.unwrap_err())
}

#[test]
fn well_formed_rules_are_read() {
    let rules = parse_rules(&rules_with_site("youtube.com")).unwrap();
    assert_eq!(rules.blocked_sites.len(), 1);
}

#[test]
fn a_domain_the_core_rejects_is_a_rules_error_that_names_the_site_and_the_problem() {
    let error = failure(parse_rules(&rules_with_site("co.uk")));

    assert_eq!(error.code, CommandErrorCode::RulesInvalid);
    assert_eq!(
        error.rules_fault,
        Some(RulesFault::InvalidSite {
            site: "co.uk".into(),
            problem: SiteProblem::PublicSuffix
        })
    );
}

#[test]
fn rules_of_the_wrong_shape_are_a_rules_error_too() {
    let error = failure(parse_rules(&json!({ "blockedSites": "youtube.com" })));

    assert_eq!(error.code, CommandErrorCode::RulesInvalid);
    assert_eq!(error.rules_fault, Some(RulesFault::Unreadable));
}

#[test]
fn a_protected_app_is_refused_with_its_bundle_id() {
    let rules = parse_rules(&rules_with_app("com.apple.finder")).unwrap();

    let error = CommandError::from(AppError::from(rules.validate().unwrap_err()));

    assert_eq!(error.code, CommandErrorCode::RulesInvalid);
    assert_eq!(
        error.rules_fault,
        Some(RulesFault::ProtectedApp {
            bundle_id: "com.apple.finder".into()
        })
    );
}
