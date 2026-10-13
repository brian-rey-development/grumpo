use std::collections::BTreeSet;

use super::*;
use crate::rules::{Rules, Schedule, Window};
use crate::test_support::{app, app_target, limit, site, site_target, valid, window};
use crate::time::Day;

fn scheduled(windows: Vec<Window>) -> ValidatedRules {
    valid(Rules {
        schedule: Schedule { windows },
        ..Rules::default()
    })
}

fn blocking_sites(hosts: &[&str]) -> ValidatedRules {
    valid(Rules {
        blocked_sites: hosts.iter().map(|host| site(host)).collect(),
        ..Rules::default()
    })
}

fn blocking_apps(ids: &[&str]) -> ValidatedRules {
    valid(Rules {
        blocked_apps: ids.iter().map(|id| app(id)).collect(),
        ..Rules::default()
    })
}

fn limited(entries: &[(LimitTarget, u16)]) -> ValidatedRules {
    valid(Rules {
        limits: entries
            .iter()
            .map(|(target, minutes)| limit(target.clone(), *minutes))
            .collect(),
        ..Rules::default()
    })
}

/// Hostnames that `hosts::render_bytes` would block for `rules`, read back from its output.
fn rendered_hosts(rules: &ValidatedRules) -> BTreeSet<String> {
    String::from_utf8_lossy(&hosts::render_bytes(b"", &rules.blocked_sites))
        .lines()
        .filter_map(|line| line.strip_prefix("0.0.0.0 "))
        .map(str::to_owned)
        .collect()
}

#[test]
fn identical_rules_have_no_loosenings() {
    let rules = valid(Rules {
        limits: vec![limit(site_target("x.com"), 30)],
        ..blocking_sites(&["youtube.com"]).into_inner()
    });
    assert_eq!(loosenings(&rules, &rules).len(), 0);
}

#[test]
fn removing_a_site_is_a_loosening() {
    let found = loosenings(
        &blocking_sites(&["youtube.com", "x.com"]),
        &blocking_sites(&["x.com"]),
    );
    assert_eq!(
        found,
        [Loosening::SiteRemoved {
            domain: site("youtube.com")
        }]
    );
}

#[test]
fn replacing_m_youtube_with_youtube_is_not_a_loosening() {
    let old = blocking_sites(&["m.youtube.com"]);
    let new = blocking_sites(&["youtube.com"]);
    assert_eq!(loosenings(&old, &new).len(), 0);
    assert!(rendered_hosts(&new).contains("m.youtube.com"));
}

#[test]
fn dropping_a_subdomain_while_keeping_its_parent_is_a_loosening() {
    let old = blocking_sites(&["reddit.com", "api.reddit.com"]);
    let new = blocking_sites(&["reddit.com"]);
    assert_eq!(
        loosenings(&old, &new),
        [Loosening::SiteRemoved {
            domain: site("api.reddit.com")
        }]
    );
    assert!(!rendered_hosts(&new).contains("api.reddit.com"));
}

#[test]
fn dropping_a_listed_alias_while_keeping_its_parent_is_not_a_loosening() {
    let old = blocking_sites(&["reddit.com", "old.reddit.com"]);
    let new = blocking_sites(&["reddit.com"]);
    assert_eq!(loosenings(&old, &new).len(), 0);
    assert!(rendered_hosts(&new).contains("old.reddit.com"));
}

#[test]
fn replacing_a_subdomain_with_its_parent_is_a_loosening() {
    let found = loosenings(
        &blocking_sites(&["api.reddit.com"]),
        &blocking_sites(&["reddit.com"]),
    );
    assert_eq!(
        found,
        [Loosening::SiteRemoved {
            domain: site("api.reddit.com")
        }]
    );
}

#[test]
fn loosenings_agree_with_what_the_hosts_file_blocks() {
    let sets: [&[&str]; 8] = [
        &[],
        &["reddit.com"],
        &["old.reddit.com"],
        &["reddit.com", "old.reddit.com"],
        &["m.reddit.com"],
        &["youtube.com", "m.youtube.com"],
        &["m.youtube.com"],
        &["youtube.com"],
    ];
    for old in sets.map(blocking_sites) {
        for new in sets.map(blocking_sites) {
            let unblocked = old
                .blocked_sites
                .iter()
                .any(|site| !rendered_hosts(&new).contains(site.as_str()));
            let reported = loosenings(&old, &new)
                .iter()
                .any(|found| matches!(found, Loosening::SiteRemoved { .. }));
            assert_eq!(reported, unblocked, "old {old:?} new {new:?}");
        }
    }
}

#[test]
fn replacing_a_site_with_a_narrower_domain_is_a_loosening() {
    let found = loosenings(
        &blocking_sites(&["reddit.com"]),
        &blocking_sites(&["old.reddit.com"]),
    );
    assert_eq!(
        found,
        [Loosening::SiteRemoved {
            domain: site("reddit.com")
        }]
    );
}

#[test]
fn removing_an_app_is_a_loosening_even_if_the_name_changes() {
    let renamed = valid(Rules {
        blocked_apps: [AppRef::new("com.a", "Renamed").unwrap()].into(),
        ..Rules::default()
    });
    assert_eq!(loosenings(&blocking_apps(&["com.a"]), &renamed).len(), 0);
    assert_eq!(
        loosenings(
            &blocking_apps(&["com.a", "com.b"]),
            &blocking_apps(&["com.b"])
        ),
        [Loosening::AppRemoved { app: app("com.a") }]
    );
}

#[test]
fn limit_changes() {
    let x = site_target("x.com");
    let steam = app_target("com.steam");
    let cases = [
        (
            limited(&[(x.clone(), 30)]),
            limited(&[(x.clone(), 30)]),
            vec![],
        ),
        (
            limited(&[(x.clone(), 30)]),
            limited(&[(x.clone(), 10)]),
            vec![],
        ),
        (limited(&[]), limited(&[(x.clone(), 10)]), vec![]),
        (
            limited(&[(x.clone(), 30)]),
            limited(&[(x.clone(), 31)]),
            vec![Loosening::LimitRaised {
                target: x.clone(),
                from: 30,
                to: 31,
            }],
        ),
        (
            limited(&[(x.clone(), 30), (steam.clone(), 60)]),
            limited(&[(steam.clone(), 60)]),
            vec![Loosening::LimitRemoved { target: x.clone() }],
        ),
        (
            limited(&[(x.clone(), 30)]),
            limited(&[(steam.clone(), 30)]),
            vec![Loosening::LimitRemoved { target: x.clone() }],
        ),
    ];
    for (old, new, expected) in cases {
        assert_eq!(loosenings(&old, &new), expected, "old {old:?} new {new:?}");
    }
}

#[test]
fn schedule_changes() {
    let morning = window(&[Day::Mon], 480, 720);
    let cases = [
        (scheduled(vec![]), scheduled(vec![morning.clone()]), None),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![morning.clone()]),
            None,
        ),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![window(&[Day::Mon], 480, 1020)]),
            None,
        ),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![window(&[Day::Mon], 480, 700)]),
            Some(20),
        ),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![window(&[Day::Mon], 500, 720)]),
            Some(20),
        ),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![]),
            Some(240),
        ),
        (
            scheduled(vec![morning.clone()]),
            scheduled(vec![window(&[Day::Tue], 480, 720)]),
            Some(240),
        ),
    ];
    for (old, new, expected) in cases {
        let found = loosenings(&old, &new);
        let expected: Vec<_> = expected
            .map(|minutes_uncovered| Loosening::ScheduleReduced { minutes_uncovered })
            .into_iter()
            .collect();
        assert_eq!(found, expected, "old {old:?} new {new:?}");
    }
}

#[test]
fn splitting_a_window_into_equivalent_pieces_is_not_a_loosening() {
    let whole = scheduled(vec![window(&[Day::Mon], 480, 720)]);
    let split = scheduled(vec![
        window(&[Day::Mon], 480, 600),
        window(&[Day::Mon], 600, 720),
    ]);
    assert_eq!(loosenings(&whole, &split).len(), 0);
    assert_eq!(loosenings(&split, &whole).len(), 0);
}

#[test]
fn adding_everything_at_once_is_never_a_loosening() {
    let new = valid(Rules {
        blocked_sites: [site("a.com")].into(),
        blocked_apps: [app("com.a")].into(),
        schedule: Schedule {
            windows: vec![window(&[Day::Mon], 0, 60)],
        },
        limits: vec![limit(site_target("a.com"), 5)],
    });
    assert_eq!(loosenings(&ValidatedRules::default(), &new).len(), 0);
}

#[test]
fn multiple_loosenings_are_reported_in_a_stable_order() {
    let old = valid(Rules {
        blocked_sites: [site("a.com")].into(),
        blocked_apps: [app("com.a")].into(),
        schedule: Schedule {
            windows: vec![window(&[Day::Mon], 0, 60)],
        },
        limits: vec![
            limit(site_target("a.com"), 5),
            limit(app_target("com.a"), 5),
        ],
    });
    let new = limited(&[(app_target("com.a"), 9)]);
    assert_eq!(
        loosenings(&old, &new),
        [
            Loosening::SiteRemoved {
                domain: site("a.com")
            },
            Loosening::AppRemoved { app: app("com.a") },
            Loosening::LimitRemoved {
                target: site_target("a.com")
            },
            Loosening::LimitRaised {
                target: app_target("com.a"),
                from: 5,
                to: 9
            },
            Loosening::ScheduleReduced {
                minutes_uncovered: 60
            },
        ]
    );
}

#[test]
fn display_describes_each_variant() {
    let cases = [
        (
            Loosening::SiteRemoved {
                domain: site("a.com"),
            },
            "unblocks a.com",
        ),
        (
            Loosening::AppRemoved { app: app("com.a") },
            "unblocks com.a",
        ),
        (
            Loosening::LimitRemoved {
                target: site_target("a.com"),
            },
            "removes the limit for a.com",
        ),
        (
            Loosening::LimitRaised {
                target: site_target("a.com"),
                from: 5,
                to: 9,
            },
            "raises the limit for a.com from 5 to 9 minutes",
        ),
        (
            Loosening::ScheduleReduced {
                minutes_uncovered: 20,
            },
            "frees 20 minutes of scheduled blocking each week",
        ),
    ];
    for (loosening, text) in cases {
        assert_eq!(loosening.to_string(), text);
    }
}
