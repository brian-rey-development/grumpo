use super::*;

const BASE: &str =
    "127.0.0.1\tlocalhost\n255.255.255.255\tbroadcasthost\n::1             localhost\n";

fn sites(hosts: &[&str]) -> BTreeSet<Domain> {
    hosts
        .iter()
        .map(|host| Domain::parse(host).unwrap())
        .collect()
}

const HOSTS_VECTORS: &str = include_str!("../../tests/vectors/hosts.json");
const COMMON_PREFIXES: [&str; 3] = ["", "www.", "m."];

/// The block for `domains`, sites without aliases of their own, which lists each with the
/// common aliases in sorted order.
fn block_for(domains: &[&str]) -> String {
    let aliases: BTreeSet<String> = domains
        .iter()
        .flat_map(|domain| {
            COMMON_PREFIXES
                .iter()
                .map(move |prefix| format!("{prefix}{domain}"))
        })
        .collect();
    let entries: Vec<&str> = aliases.iter().map(String::as_str).collect();
    block(&entries)
}

fn render(existing: &str, sites: &BTreeSet<Domain>) -> String {
    String::from_utf8_lossy(&render_bytes(existing.as_bytes(), sites)).into_owned()
}

fn block(entries: &[&str]) -> String {
    let mut lines = vec![MARKER_START.to_owned()];
    for host in entries {
        lines.push(format!("0.0.0.0 {host}"));
        lines.push(format!(":: {host}"));
    }
    lines.push(MARKER_END.to_owned());
    lines.join("\n") + "\n"
}

#[test]
fn appends_a_block_with_both_address_families_for_every_alias() {
    let rendered = render(BASE, &sites(&["tiktok.com"]));
    let expected = format!("{BASE}{}", block_for(&["tiktok.com"]));
    assert_eq!(rendered, expected);
}

#[test]
fn entries_are_sorted_and_deduplicated_across_sites() {
    let rendered = render(
        "",
        &sites(&["reddit.com", "old.reddit.com", "a.io", "m.a.io"]),
    );
    let hosts: Vec<&str> = rendered
        .lines()
        .filter(|line| line.starts_with("0.0.0.0 "))
        .map(|line| &line["0.0.0.0 ".len()..])
        .collect();
    let mut sorted = hosts.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(hosts, sorted);
    assert!(hosts.contains(&"m.a.io"));
    assert_eq!(hosts.iter().filter(|host| **host == "m.a.io").count(), 1);
}

#[test]
fn no_sites_removes_the_block_and_adds_nothing() {
    let with_block = render(BASE, &sites(&["youtube.com"]));
    assert_eq!(render(&with_block, &BTreeSet::new()), BASE);
    assert_eq!(render(BASE, &BTreeSet::new()), BASE);
}

#[test]
fn replaces_an_existing_current_block() {
    let first = render(BASE, &sites(&["youtube.com"]));
    let second = render(&first, &sites(&["tiktok.com"]));
    assert_eq!(second, format!("{BASE}{}", block_for(&["tiktok.com"])));
}

#[test]
fn removes_a_legacy_block_and_keeps_surrounding_content() {
    let legacy = format!(
        "{BASE}# FOCUS-HOSTS START\n0.0.0.0 youtube.com\n0.0.0.0 www.youtube.com\n# FOCUS-HOSTS END\n10.0.0.1 intranet\n"
    );
    let expected = format!("{BASE}10.0.0.1 intranet\n");
    assert_eq!(render(&legacy, &BTreeSet::new()), expected);
}

#[test]
fn migrates_a_legacy_block_into_the_current_one() {
    let legacy = format!("{BASE}# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n");
    let rendered = render(&legacy, &sites(&["new.com"]));
    assert!(!rendered.contains("FOCUS-HOSTS"));
    assert!(!rendered.contains("old.com"));
    assert_eq!(rendered, format!("{BASE}{}", block_for(&["new.com"])));
}

#[test]
fn replaces_the_block_written_under_the_daemons_earlier_name() {
    let earlier = format!(
        "{BASE}# >>> focus: managed block, do not edit >>>\n0.0.0.0 old.com\n# <<< focus: managed block <<<\n10.0.0.1 intranet\n"
    );
    let rendered = render(&earlier, &sites(&["new.com"]));
    assert_eq!(
        rendered,
        format!("{BASE}10.0.0.1 intranet\n{}", block_for(&["new.com"]))
    );
}

#[test]
fn removes_every_block_when_several_exist() {
    let doubled = format!(
        "{MARKER_START}\n0.0.0.0 a.com\n{MARKER_END}\n10.0.0.1 keep\n# FOCUS-HOSTS START\n0.0.0.0 b.com\n# FOCUS-HOSTS END\n{MARKER_START}\n0.0.0.0 c.com\n{MARKER_END}\n"
    );
    assert_eq!(render(&doubled, &BTreeSet::new()), "10.0.0.1 keep\n");
}

#[test]
fn an_unmatched_start_marker_is_removed_alone() {
    let broken = format!("{BASE}{MARKER_START}\n10.0.0.1 important\n");
    assert_eq!(
        render(&broken, &BTreeSet::new()),
        format!("{BASE}10.0.0.1 important\n")
    );

    let legacy = format!("{BASE}# FOCUS-HOSTS START\n10.0.0.1 important\n");
    assert_eq!(
        render(&legacy, &BTreeSet::new()),
        format!("{BASE}10.0.0.1 important\n")
    );
}

#[test]
fn an_unmatched_end_marker_is_removed_alone() {
    let broken = format!("{BASE}10.0.0.1 before\n{MARKER_END}\n10.0.0.1 after\n");
    assert_eq!(
        render(&broken, &BTreeSet::new()),
        format!("{BASE}10.0.0.1 before\n10.0.0.1 after\n")
    );
}

#[test]
fn a_start_marker_followed_by_another_start_only_removes_the_closed_block() {
    let nested = format!(
        "{MARKER_START}\n10.0.0.1 orphan content\n{MARKER_START}\n0.0.0.0 a.com\n{MARKER_END}\n10.0.0.1 tail\n"
    );
    assert_eq!(
        render(&nested, &BTreeSet::new()),
        "10.0.0.1 orphan content\n10.0.0.1 tail\n"
    );
}

#[test]
fn an_unmatched_marker_then_a_complete_block_is_handled() {
    let mixed =
        format!("{MARKER_END}\n10.0.0.1 keep\n{MARKER_START}\n0.0.0.0 a.com\n{MARKER_END}\n");
    assert_eq!(render(&mixed, &BTreeSet::new()), "10.0.0.1 keep\n");
}

#[test]
fn marker_lines_tolerate_surrounding_whitespace_and_crlf() {
    let padded = format!("{BASE}  {MARKER_START}  \r\n0.0.0.0 a.com\r\n{MARKER_END}\r\n");
    assert_eq!(render(&padded, &BTreeSet::new()), BASE);
}

#[test]
fn other_content_is_preserved_byte_for_byte() {
    let content = "# comment\n\n127.0.0.1 localhost   # trailing\n\t10.0.0.1\tweird spacing  \n\n192.168.0.5 nas";
    let rendered = render(content, &sites(&["a.com"]));
    assert!(rendered.starts_with(content));
    assert_eq!(
        rendered[content.len()..],
        format!("\n{}", block_for(&["a.com"]))
    );
}

#[test]
fn content_ends_with_exactly_one_newline() {
    for existing in ["", "\n", "\n\n\n", "a", "a\n", "a\n\n\n"] {
        let rendered = render(existing, &BTreeSet::new());
        assert!(!rendered.ends_with("\n\n"), "existing {existing:?}");
        assert_eq!(rendered.ends_with('\n'), !rendered.is_empty());
    }
    assert_eq!(render("a", &BTreeSet::new()), "a\n");
    assert_eq!(render("a\n\n\n", &BTreeSet::new()), "a\n");
}

#[test]
fn an_empty_file_with_sites_holds_only_the_block() {
    assert_eq!(render("", &sites(&["a.com"])), block_for(&["a.com"]));
    assert_eq!(render("", &BTreeSet::new()), "");
}

#[test]
fn rendering_is_idempotent() {
    let inputs = [
        String::new(),
        BASE.to_owned(),
        "no trailing newline".to_owned(),
        format!("{BASE}# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n\n\n"),
        format!("{BASE}{MARKER_START}\ndangling\n"),
        format!("{MARKER_END}\n{BASE}{MARKER_START}\n{MARKER_START}\n{MARKER_END}\n"),
        render(BASE, &sites(&["a.com", "b.com"])),
    ];
    let site_sets = [
        sites(&[]),
        sites(&["youtube.com"]),
        sites(&["a.com", "m.a.com", "b.org"]),
    ];
    for input in &inputs {
        for set in &site_sets {
            let once = render(input, set);
            assert_eq!(render(&once, set), once, "input {input:?}");
        }
    }
}

#[derive(serde::Deserialize)]
struct HostsVectors {
    hosts: Vec<(String, Vec<String>)>,
}

/// The cases are in `tests/vectors/hosts.json`, shared with the interface.
#[test]
fn each_site_is_written_with_the_aliases_it_serves_its_pages_from() {
    let vectors: HostsVectors = serde_json::from_str(HOSTS_VECTORS).unwrap();
    for (site, expected) in vectors.hosts {
        let expected: BTreeSet<String> = expected.into_iter().collect();
        assert_eq!(blocked_hosts(&sites(&[&site])), expected, "{site}");
    }
}

#[test]
fn every_site_with_aliases_of_its_own_has_a_vector() {
    let vectors: HostsVectors = serde_json::from_str(HOSTS_VECTORS).unwrap();
    for (site, _) in SITE_ALIASES {
        assert!(
            vectors.hosts.iter().any(|(listed, _)| listed == site),
            "{site}"
        );
    }
}

#[test]
fn a_site_blocks_exactly_the_hosts_it_is_written_with() {
    let vectors: HostsVectors = serde_json::from_str(HOSTS_VECTORS).unwrap();
    for (site, expected) in vectors.hosts {
        let site = Domain::parse(&site).unwrap();
        for host in &expected {
            assert!(
                blocks(&site, &Domain::parse(host).unwrap()),
                "{site} {host}"
            );
        }
    }
}

#[test]
fn a_site_leaves_its_other_subdomains_reachable() {
    let youtube = Domain::parse("youtube.com").unwrap();
    for host in ["music.youtube.com", "studio.youtube.com", "notyoutube.com"] {
        assert!(!blocks(&youtube, &Domain::parse(host).unwrap()), "{host}");
    }
}

#[test]
fn blocked_hosts_are_the_union_over_all_sites() {
    let hosts = blocked_hosts(&sites(&["a.com", "m.a.com", "shop.a.com"]));
    let expected: BTreeSet<String> = ["a.com", "www.a.com", "m.a.com", "shop.a.com"]
        .map(str::to_owned)
        .into();
    assert_eq!(hosts, expected);
}

#[test]
fn a_start_marker_is_not_closed_by_an_end_marker_of_the_other_kind() {
    let existing =
        "127.0.0.1 localhost\n# FOCUS-HOSTS START\n10.0.0.5 nas\n# <<< grumpo: managed block <<<\n";
    assert_eq!(
        render(existing, &BTreeSet::new()),
        "127.0.0.1 localhost\n10.0.0.5 nas\n"
    );

    let reversed = format!("{MARKER_START}\n10.0.0.5 nas\n# FOCUS-HOSTS END\n");
    assert_eq!(render(&reversed, &BTreeSet::new()), "10.0.0.5 nas\n");
}

#[test]
fn markers_of_the_other_kind_inside_a_block_go_with_the_block() {
    let nested = format!(
        "{MARKER_START}\n0.0.0.0 a.com\n# FOCUS-HOSTS START\n0.0.0.0 b.com\n{MARKER_END}\n10.0.0.1 tail\n"
    );
    assert_eq!(render(&nested, &BTreeSet::new()), "10.0.0.1 tail\n");
}

#[test]
fn a_legacy_start_glued_to_the_previous_line_opens_a_block_and_keeps_that_text() {
    let glued = "127.0.0.1 localhost\n10.0.0.5 nas# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n10.0.0.1 tail\n";
    assert_eq!(
        render(glued, &BTreeSet::new()),
        "127.0.0.1 localhost\n10.0.0.5 nas\n10.0.0.1 tail\n"
    );
}

#[test]
fn a_glued_start_without_a_matching_end_is_left_alone() {
    let unclosed = "10.0.0.5 nas# FOCUS-HOSTS START\n0.0.0.0 old.com\n";
    assert_eq!(render(unclosed, &BTreeSet::new()), unclosed);

    let wrong_kind = format!("10.0.0.5 nas# FOCUS-HOSTS START\n{MARKER_END}\n");
    assert_eq!(
        render(&wrong_kind, &BTreeSet::new()),
        "10.0.0.5 nas# FOCUS-HOSTS START\n"
    );
}

#[test]
fn a_glued_start_is_replaced_by_the_new_block() {
    let glued = "10.0.0.5 nas# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n";
    assert_eq!(
        render(glued, &sites(&["new.com"])),
        format!("10.0.0.5 nas\n{}", block_for(&["new.com"]))
    );
}

#[test]
fn rendering_is_idempotent_for_the_marker_edge_cases() {
    let inputs = [
        "127.0.0.1 localhost\n# FOCUS-HOSTS START\n10.0.0.5 nas\n# <<< grumpo: managed block <<<\n",
        "10.0.0.5 nas# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n",
        "10.0.0.5 nas# FOCUS-HOSTS START\n",
    ];
    for input in inputs {
        for set in [sites(&[]), sites(&["a.com"])] {
            let once = render(input, &set);
            assert_eq!(render(&once, &set), once, "input {input:?}");
        }
    }
}

mod non_utf8 {
    use super::*;

    const LATIN1_COMMENT: &[u8] = b"# caf\xE9 notes\n";

    fn base_with_foreign_bytes() -> Vec<u8> {
        let mut bytes = b"127.0.0.1 localhost\n".to_vec();
        bytes.extend_from_slice(LATIN1_COMMENT);
        bytes.extend_from_slice(b"10.0.0.5 nas\xFF\n");
        bytes
    }

    #[test]
    fn foreign_lines_with_invalid_utf8_are_kept_verbatim_next_to_the_block() {
        let base = base_with_foreign_bytes();
        let rendered = render_bytes(&base, &sites(&["a.com"]));
        let mut expected = base;
        expected.extend_from_slice(block_for(&["a.com"]).as_bytes());
        assert_eq!(rendered, expected);
    }

    #[test]
    fn a_block_is_removed_and_replaced_around_invalid_utf8() {
        let base = base_with_foreign_bytes();
        let with_block = render_bytes(&base, &sites(&["a.com"]));
        assert_eq!(render_bytes(&with_block, &BTreeSet::new()), base);
        let replaced = render_bytes(&with_block, &sites(&["b.com"]));
        assert!(replaced.starts_with(&base));
        assert!(!String::from_utf8_lossy(&replaced).contains("a.com"));
    }

    #[test]
    fn invalid_bytes_inside_a_managed_block_go_with_the_block() {
        let mut existing = b"127.0.0.1 localhost\n".to_vec();
        existing.extend_from_slice(MARKER_START.as_bytes());
        existing.extend_from_slice(b"\n0.0.0.0 \xC3\x28.com\n");
        existing.extend_from_slice(MARKER_END.as_bytes());
        existing.extend_from_slice(b"\n# FOCUS-HOSTS START\n\xFF\n# FOCUS-HOSTS END\n");
        assert_eq!(
            render_bytes(&existing, &BTreeSet::new()),
            b"127.0.0.1 localhost\n"
        );
    }

    #[test]
    fn a_glued_legacy_start_keeps_a_non_utf8_prefix() {
        let existing = b"10.0.0.5 nas\xFF# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n10.0.0.1 tail\n";
        assert_eq!(
            render_bytes(existing, &BTreeSet::new()),
            b"10.0.0.5 nas\xFF\n10.0.0.1 tail\n"
        );
    }

    #[test]
    fn rendering_non_utf8_input_is_idempotent_and_ends_with_one_newline() {
        let inputs: [&[u8]; 3] = [b"\xFF", b"a\xE9\n\n\n", b"\xC3\x28\n# FOCUS-HOSTS START\n"];
        for input in inputs {
            for set in [sites(&[]), sites(&["a.com"])] {
                let once = render_bytes(input, &set);
                assert_eq!(render_bytes(&once, &set), once, "input {input:?}");
                assert!(once.ends_with(b"\n") && !once.ends_with(b"\n\n"));
            }
        }
    }

    #[test]
    fn the_text_wrapper_matches_the_byte_renderer() {
        let text = format!("{BASE}# FOCUS-HOSTS START\n0.0.0.0 old.com\n# FOCUS-HOSTS END\n");
        let set = sites(&["a.com"]);
        assert_eq!(
            render(&text, &set).into_bytes(),
            render_bytes(text.as_bytes(), &set)
        );
    }
}
