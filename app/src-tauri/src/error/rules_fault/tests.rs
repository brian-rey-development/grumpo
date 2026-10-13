use focus_core::LimitTarget;
use serde_json::json;

use super::*;

fn rules(sites: &[&str]) -> Value {
    json!({
        "blockedSites": sites,
        "blockedApps": [],
        "schedule": { "windows": [] },
        "limits": []
    })
}

#[test]
fn a_site_the_core_refuses_is_named_with_its_problem() {
    let cases = [
        ("", SiteProblem::Empty),
        ("café.com", SiteProblem::NonAscii),
        ("localhost", SiteProblem::TooFewLabels),
        ("a..com", SiteProblem::EmptyLabel),
        ("10.0.0.1", SiteProblem::IpAddress),
        ("co.uk", SiteProblem::PublicSuffix),
        ("-a.com", SiteProblem::HyphenAtEdge),
        ("a b.com", SiteProblem::InvalidCharacter),
    ];
    for (site, problem) in cases {
        assert_eq!(
            diagnose_unreadable_rules(&rules(&[site])),
            RulesFault::InvalidSite {
                site: site.to_owned(),
                problem
            },
            "{site}"
        );
    }
}

#[test]
fn the_first_bad_site_is_the_one_reported() {
    let fault = diagnose_unreadable_rules(&rules(&["youtube.com", "co.uk", "10.0.0.1"]));
    assert_eq!(
        fault,
        RulesFault::InvalidSite {
            site: "co.uk".into(),
            problem: SiteProblem::PublicSuffix
        }
    );
}

#[test]
fn a_site_inside_a_limit_is_checked_too() {
    let raw = json!({
        "blockedSites": [],
        "blockedApps": [],
        "schedule": { "windows": [] },
        "limits": [{ "target": { "kind": "site", "domain": "co.uk" }, "dailyMinutes": 30 }]
    });
    assert!(matches!(
        diagnose_unreadable_rules(&raw),
        RulesFault::InvalidSite {
            problem: SiteProblem::PublicSuffix,
            ..
        }
    ));
}

#[test]
fn an_app_with_an_unusable_bundle_id_is_an_invalid_app() {
    let raw = json!({
        "blockedSites": [],
        "blockedApps": [{ "bundleId": "", "name": "Thing" }],
        "schedule": { "windows": [] },
        "limits": []
    });
    assert_eq!(diagnose_unreadable_rules(&raw), RulesFault::InvalidApp);
}

#[test]
fn rules_of_the_wrong_shape_are_unreadable() {
    assert_eq!(
        diagnose_unreadable_rules(&json!({ "blockedSites": "youtube.com" })),
        RulesFault::Unreadable
    );
}

#[test]
fn validation_errors_become_faults() {
    let target = LimitTarget::Site {
        domain: Domain::parse("x.com").unwrap(),
    };
    let cases = [
        (
            RulesError::TooMany {
                collection: RuleCollection::Limits,
                count: 31,
                max: 30,
            },
            RulesFault::TooMany {
                section: RuleSection::Limits,
            },
        ),
        (
            RulesError::WindowNotForward { number: 3 },
            RulesFault::WindowNotForward { number: 3 },
        ),
        (
            RulesError::LimitOutOfRange {
                target: target.clone(),
                minutes: 0,
            },
            RulesFault::LimitOutOfRange {
                target: target.clone(),
                minutes: 0,
            },
        ),
        (
            RulesError::DuplicateLimit(target.clone()),
            RulesFault::DuplicateLimit { target },
        ),
        (
            RulesError::ProtectedApp {
                bundle_id: "com.apple.finder".into(),
            },
            RulesFault::ProtectedApp {
                bundle_id: "com.apple.finder".into(),
            },
        ),
    ];
    for (error, fault) in cases {
        assert_eq!(RulesFault::from(&error), fault);
    }
}
