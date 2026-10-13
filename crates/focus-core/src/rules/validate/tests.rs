use super::*;
use crate::rules::protected::PROTECTED_APPS;
use crate::rules::Schedule;
use crate::test_support::{app, app_target, limit, site, site_target, window};
use crate::time::Day;

fn check(rules: Rules) -> Result<(), RulesError> {
    rules.validate().map(drop)
}

fn with_windows(windows: Vec<Window>) -> Rules {
    Rules {
        schedule: Schedule { windows },
        ..Rules::default()
    }
}

fn with_limits(limits: Vec<Limit>) -> Rules {
    Rules {
        limits,
        ..Rules::default()
    }
}

#[test]
fn empty_rules_are_valid() {
    assert_eq!(check(Rules::default()), Ok(()));
}

#[test]
fn a_realistic_ruleset_is_valid() {
    let rules = Rules {
        blocked_sites: [site("youtube.com"), site("reddit.com")].into(),
        blocked_apps: [app("com.valvesoftware.steam")].into(),
        schedule: Schedule {
            windows: vec![window(&[Day::Mon, Day::Tue], 0, 1140)],
        },
        limits: vec![
            limit(site_target("x.com"), 30),
            limit(app_target("com.x"), 30),
        ],
    };
    assert_eq!(check(rules), Ok(()));
}

#[test]
fn window_rules() {
    let cases = [
        (window(&[Day::Mon], 540, 541), None),
        (window(&[Day::Mon], 0, 1440), None),
        (
            window(&[Day::Mon], 540, 540),
            Some(RulesError::WindowNotForward { number: 1 }),
        ),
        (
            window(&[Day::Mon], 600, 540),
            Some(RulesError::WindowNotForward { number: 1 }),
        ),
        (
            window(&[], 540, 600),
            Some(RulesError::WindowWithoutDays { number: 1 }),
        ),
    ];
    for (window, expected) in cases {
        let result = check(with_windows(vec![window.clone()]));
        assert_eq!(result.err(), expected, "window {window:?}");
    }
}

#[test]
fn window_errors_name_the_offending_window_by_position() {
    let rules = with_windows(vec![
        window(&[Day::Mon], 0, 60),
        window(&[Day::Tue], 0, 60),
        window(&[], 0, 60),
    ]);
    assert_eq!(
        check(rules),
        Err(RulesError::WindowWithoutDays { number: 3 })
    );
}

#[test]
fn limit_minutes_bounds() {
    for (minutes, valid) in [
        (0, false),
        (1, true),
        (60, true),
        (1440, true),
        (1441, false),
    ] {
        let result = check(with_limits(vec![limit(site_target("x.com"), minutes)]));
        assert_eq!(result.is_ok(), valid, "minutes {minutes}");
    }
}

#[test]
fn out_of_range_limits_report_target_and_minutes() {
    let result = check(with_limits(vec![limit(site_target("x.com"), 0)]));
    assert_eq!(
        result,
        Err(RulesError::LimitOutOfRange {
            target: site_target("x.com"),
            minutes: 0
        })
    );
}

#[test]
fn duplicate_limit_targets_are_rejected() {
    let rules = with_limits(vec![
        limit(site_target("x.com"), 10),
        limit(app_target("com.x"), 10),
        limit(site_target("x.com"), 20),
    ]);
    assert_eq!(
        check(rules),
        Err(RulesError::DuplicateLimit(site_target("x.com")))
    );
}

#[test]
fn a_site_and_an_app_with_the_same_name_are_distinct_targets() {
    let rules = with_limits(vec![
        limit(site_target("x.com"), 10),
        limit(app_target("x.com"), 10),
    ]);
    assert_eq!(check(rules), Ok(()));
}

#[test]
fn caps_allow_the_maximum_and_reject_one_more() {
    let sites = |count: usize| -> Rules {
        Rules {
            blocked_sites: (0..count).map(|i| site(&format!("site{i}.com"))).collect(),
            ..Rules::default()
        }
    };
    let apps = |count: usize| -> Rules {
        Rules {
            blocked_apps: (0..count).map(|i| app(&format!("com.app{i}"))).collect(),
            ..Rules::default()
        }
    };
    let windows = |count: usize| with_windows(vec![window(&[Day::Mon], 0, 60); count]);
    let limits = |count: usize| {
        with_limits(
            (0..count)
                .map(|i| limit(site_target(&format!("site{i}.com")), 5))
                .collect(),
        )
    };

    assert_eq!(check(sites(MAX_SITES)), Ok(()));
    assert_eq!(check(apps(MAX_APPS)), Ok(()));
    assert_eq!(check(windows(MAX_WINDOWS)), Ok(()));
    assert_eq!(check(limits(MAX_LIMITS)), Ok(()));

    let too_many = |collection, count, max| {
        Err(RulesError::TooMany {
            collection,
            count,
            max,
        })
    };
    assert_eq!(
        check(sites(MAX_SITES + 1)),
        too_many(RuleCollection::Sites, MAX_SITES + 1, MAX_SITES)
    );
    assert_eq!(
        check(apps(MAX_APPS + 1)),
        too_many(RuleCollection::Apps, MAX_APPS + 1, MAX_APPS)
    );
    assert_eq!(
        check(windows(MAX_WINDOWS + 1)),
        too_many(RuleCollection::Windows, MAX_WINDOWS + 1, MAX_WINDOWS)
    );
    assert_eq!(
        check(limits(MAX_LIMITS + 1)),
        too_many(RuleCollection::Limits, MAX_LIMITS + 1, MAX_LIMITS)
    );
}

#[test]
fn error_messages_are_readable() {
    let cases = [
        (
            RulesError::TooMany {
                collection: RuleCollection::Sites,
                count: 501,
                max: 500,
            },
            "too many blocked sites (501), the maximum is 500",
        ),
        (
            RulesError::WindowWithoutDays { number: 2 },
            "schedule window 2 has no days selected",
        ),
        (
            RulesError::WindowNotForward { number: 1 },
            "schedule window 1 must start before it ends",
        ),
        (
            RulesError::LimitOutOfRange {
                target: site_target("x.com"),
                minutes: 0,
            },
            "the limit for x.com must be between 1 and 1440 minutes a day, got 0",
        ),
        (
            RulesError::DuplicateLimit(site_target("x.com")),
            "there is more than one limit for x.com",
        ),
    ];
    for (error, message) in cases {
        assert_eq!(error.to_string(), message);
    }
}

#[test]
fn validation_hands_back_the_same_rules() {
    let rules = Rules {
        blocked_sites: [site("youtube.com")].into(),
        ..Rules::default()
    };
    let validated = rules.clone().validate().unwrap();
    assert_eq!(*validated, rules);
    assert_eq!(validated.blocked_sites.len(), 1);
    assert_eq!(validated.into_inner(), rules);
}

#[test]
fn the_default_validated_rules_are_empty() {
    assert_eq!(ValidatedRules::default().into_inner(), Rules::default());
}

#[test]
fn protected_apps_cannot_be_blocked_or_limited() {
    for (bundle_id, _) in PROTECTED_APPS {
        let blocked = Rules {
            blocked_apps: [app(bundle_id)].into(),
            ..Rules::default()
        };
        let limited = with_limits(vec![limit(app_target(bundle_id), 30)]);
        for rules in [blocked, limited] {
            assert_eq!(
                check(rules),
                Err(RulesError::ProtectedApp {
                    bundle_id: bundle_id.to_owned()
                }),
                "{bundle_id}"
            );
        }
    }
}

#[test]
fn protected_apps_are_caught_regardless_of_case_and_alongside_valid_apps() {
    let rules = Rules {
        blocked_apps: [app("com.valvesoftware.steam"), app("COM.APPLE.FINDER")].into(),
        ..Rules::default()
    };
    assert!(matches!(
        check(rules),
        Err(RulesError::ProtectedApp { bundle_id }) if bundle_id == "COM.APPLE.FINDER"
    ));
}

#[test]
fn unprotected_apple_apps_and_protected_looking_sites_are_allowed() {
    let rules = Rules {
        blocked_apps: [app("com.apple.Terminal"), app("com.apple.Safari")].into(),
        blocked_sites: [site("finder.com")].into(),
        limits: vec![limit(app_target("com.apple.Terminal"), 30)],
        ..Rules::default()
    };
    assert_eq!(check(rules), Ok(()));
}

#[test]
fn the_protected_app_message_names_the_app() {
    let error = RulesError::ProtectedApp {
        bundle_id: "com.apple.finder".to_owned(),
    };
    assert_eq!(
        error.to_string(),
        "Finder can't be blocked because Grumpo needs it to keep working"
    );
}
