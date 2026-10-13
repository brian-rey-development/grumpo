use super::*;
use crate::test_support::{app, app_target, limit, site, site_target};

#[test]
fn default_rules_are_empty() {
    let rules = Rules::default();
    assert!(rules.blocked_sites.is_empty());
    assert!(rules.blocked_apps.is_empty());
    assert_eq!(rules.schedule.windows.len(), 0);
    assert_eq!(rules.limits.len(), 0);
}

#[test]
fn limit_targets_order_sites_before_apps_then_by_identity() {
    let mut targets = vec![
        app_target("com.b"),
        site_target("zeta.com"),
        app_target("com.a"),
        site_target("alpha.com"),
    ];
    targets.sort();
    assert_eq!(
        targets,
        [
            site_target("alpha.com"),
            site_target("zeta.com"),
            app_target("com.a"),
            app_target("com.b"),
        ]
    );
}

#[test]
fn limit_target_display_names_the_target() {
    assert_eq!(site_target("youtube.com").to_string(), "youtube.com");
    let app = AppRef::new("com.apple.Safari", "Safari").unwrap();
    assert_eq!(LimitTarget::App { app }.to_string(), "Safari");
}

#[test]
fn rules_round_trip_through_json() {
    let rules = Rules {
        blocked_sites: [site("youtube.com")].into(),
        blocked_apps: [app("com.valvesoftware.steam")].into(),
        schedule: Schedule::default(),
        limits: vec![
            limit(site_target("reddit.com"), 30),
            limit(app_target("com.x"), 45),
        ],
    };
    let json = serde_json::to_string(&rules).unwrap();
    assert!(json.contains(r#""blockedSites":["youtube.com"]"#));
    assert!(json.contains(r#""target":{"kind":"site","domain":"reddit.com"}"#));
    assert!(json.contains(r#""dailyMinutes":30"#));
    assert_eq!(serde_json::from_str::<Rules>(&json).unwrap(), rules);
}

#[test]
fn unknown_fields_are_rejected_at_every_level() {
    let cases = [
        r#"{"blockedSites":[],"blockedApps":[],"schedule":{"windows":[]},"limits":[],"extra":1}"#,
        r#"{"blockedSites":[],"blockedApps":[],"schedule":{"windows":[],"extra":1},"limits":[]}"#,
        r#"{"blockedSites":[],"blockedApps":[],"schedule":{"windows":[{"days":["mon"],"start":0,"end":60,"extra":1}]},"limits":[]}"#,
        r#"{"blockedSites":[],"blockedApps":[],"schedule":{"windows":[]},"limits":[{"target":{"kind":"site","domain":"x.com"},"dailyMinutes":5,"extra":1}]}"#,
        r#"{"blockedSites":[],"blockedApps":[{"bundleId":"com.a","name":"A","extra":1}],"schedule":{"windows":[]},"limits":[]}"#,
    ];
    for json in cases {
        let error = serde_json::from_str::<Rules>(json).unwrap_err();
        assert!(error.to_string().contains("unknown field"), "json {json}");
    }
}
