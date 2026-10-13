use super::*;
use crate::test_support::local;

const REAL_SCRIPT: &str = include_str!("../../tests/fixtures/focus-hosts.sh");
const REAL_SCRIPT_DOMAIN_COUNT: usize = 56;

fn hosts(script: &str) -> Vec<String> {
    parse_domains(script)
        .iter()
        .map(ToString::to_string)
        .collect()
}

#[test]
fn parses_every_unique_domain_from_the_real_script() {
    let domains = hosts(REAL_SCRIPT);
    assert_eq!(domains.len(), REAL_SCRIPT_DOMAIN_COUNT);
    assert_eq!(domains.first().map(String::as_str), Some("youtube.com"));
    for expected in [
        "m.youtube.com",
        "old.reddit.com",
        "news.ycombinator.com",
        "lanacion.com.ar",
        "articulo.mercadolibre.com.ar",
        "store.steampowered.com",
        "store.playstation.com",
    ] {
        assert!(
            domains.iter().any(|domain| domain == expected),
            "missing {expected}"
        );
    }
}

#[test]
fn normalization_drops_www_and_duplicates_but_keeps_other_subdomains() {
    let domains = hosts(REAL_SCRIPT);
    assert!(domains.iter().all(|domain| !domain.starts_with("www.")));
    let unique: BTreeSet<_> = domains.iter().collect();
    assert_eq!(unique.len(), domains.len());
    assert_eq!(
        domains
            .iter()
            .filter(|domain| *domain == "espn.com")
            .count(),
        1
    );
}

#[test]
fn ignores_comments_and_text_outside_the_array() {
    let domains = hosts(REAL_SCRIPT);
    assert!(!domains
        .iter()
        .any(|domain| domain == "not-in-the-array.com"));
    assert!(!domains.iter().any(|domain| domain.contains("redes")));
}

#[test]
fn parser_edge_cases() {
    let cases: [(&str, &[&str]); 12] = [
        ("", &[]),
        ("echo no array here", &[]),
        ("DOMAINS=()", &[]),
        ("DOMAINS=( \"a.com\" )", &["a.com"]),
        ("DOMAINS=(\n  'a.com' \"b.com\"\n)", &["a.com", "b.com"]),
        (
            "DOMAINS=(\n \"a.com\" # \"b.com\" )\n \"c.com\"\n)",
            &["a.com", "c.com"],
        ),
        ("DOMAINS=(\n \"a)b.com\" \"d.com\"\n)", &["d.com"]),
        (
            "DOMAINS=(\n \"a.com\" \"www.a.com\" \"A.COM\"\n)",
            &["a.com"],
        ),
        (
            "DOMAINS=(\n \"not a domain\" \"localhost\" \"\" \"ok.com\"\n)",
            &["ok.com"],
        ),
        (
            "DOMAINS=(\n \"a.com\"\n)\nOTHER=(\n \"b.com\"\n)",
            &["a.com"],
        ),
        ("  DOMAINS=(\n\t\"a.com\"\n)", &["a.com"]),
        ("# DOMAINS=( \"a.com\" )\nDOMAINS=( \"b.com\" )", &["b.com"]),
    ];
    for (script, expected) in cases {
        assert_eq!(hosts(script), expected, "script {script:?}");
    }
}

#[test]
fn an_unterminated_array_yields_what_was_read() {
    assert_eq!(hosts("DOMAINS=(\n \"a.com\"\n \"b.com"), ["a.com"]);
}

#[test]
fn legacy_schedule_matches_the_old_script() {
    let legacy = schedule();
    let cases = [
        (local(2026, 10, 12, 0, 0), true),
        (local(2026, 10, 14, 18, 59), true),
        (local(2026, 10, 14, 19, 0), false),
        (local(2026, 10, 16, 18, 59), true),
        (local(2026, 10, 16, 23, 59), false),
        (local(2026, 10, 17, 0, 0), true),
        (local(2026, 10, 17, 12, 59), true),
        (local(2026, 10, 17, 13, 0), false),
        (local(2026, 10, 18, 0, 0), false),
        (local(2026, 10, 18, 12, 0), false),
    ];
    for (at, expected) in cases {
        assert_eq!(legacy.active_at(at), expected, "at {at}");
    }
}

#[test]
fn legacy_schedule_covers_the_expected_minutes_each_week() {
    let covered = schedule().mask().uncovered_by(&Schedule::default().mask());
    assert_eq!(covered, 5 * 19 * 60 + 13 * 60);
}

#[test]
fn legacy_rules_combine_sites_and_schedule_and_are_valid() {
    let rules = rules(REAL_SCRIPT);
    assert_eq!(rules.blocked_sites.len(), REAL_SCRIPT_DOMAIN_COUNT);
    assert!(rules.blocked_apps.is_empty());
    assert_eq!(rules.limits.len(), 0);
    assert_eq!(rules.schedule, schedule());
    assert!(rules.validate().is_ok());
}

#[test]
fn legacy_rules_from_an_empty_script_keep_the_schedule() {
    let rules = rules("");
    assert!(rules.blocked_sites.is_empty());
    assert_eq!(rules.schedule, schedule());
}
