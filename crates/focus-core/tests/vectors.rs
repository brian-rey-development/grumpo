//! Cases shared with the interface, which reads the same JSON files in its own tests, so the
//! two parsers cannot drift apart without one of the suites failing.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers outside #[test] functions are not covered by allow-unwrap-in-tests"
)]

use chrono::{DateTime, TimeDelta};
use focus_core::{Domain, DomainError, Now, UnlockState, UNLOCK_SENTENCE};
use serde::Deserialize;

const DOMAIN_VECTORS: &str = include_str!("vectors/domain.json");
const WHITESPACE_VECTORS: &str = include_str!("vectors/whitespace.json");
const CODE_POINT_PREFIX: &str = "U+";
const HEX_RADIX: u32 = 16;
const COOLDOWN_PASSED_MINUTES: i64 = 6;

#[derive(Deserialize)]
struct DomainVectors {
    accepted: Vec<(String, String)>,
    rejected: Vec<(String, String)>,
}

#[derive(Deserialize)]
struct WhitespaceVectors {
    whitespace: Vec<String>,
    #[serde(rename = "notWhitespace")]
    not_whitespace: Vec<String>,
}

fn problem(error: &DomainError) -> &'static str {
    match error {
        DomainError::Empty => "empty",
        DomainError::NonAscii => "nonAscii",
        DomainError::TooLong => "tooLong",
        DomainError::TooFewLabels => "tooFewLabels",
        DomainError::EmptyLabel => "emptyLabel",
        DomainError::IpAddress => "ipAddress",
        DomainError::PublicSuffix(_) => "publicSuffix",
        DomainError::LabelTooLong(_) => "labelTooLong",
        DomainError::HyphenAtEdge(_) => "hyphenAtEdge",
        DomainError::InvalidCharacter(_) => "invalidCharacter",
    }
}

fn character(label: &str) -> char {
    let digits = label.strip_prefix(CODE_POINT_PREFIX).unwrap();
    char::from_u32(u32::from_str_radix(digits, HEX_RADIX).unwrap()).unwrap()
}

fn characters(labels: &[String]) -> Vec<char> {
    labels.iter().map(|label| character(label)).collect()
}

#[test]
fn accepted_inputs_reduce_to_the_expected_domain() {
    let vectors: DomainVectors = serde_json::from_str(DOMAIN_VECTORS).unwrap();
    for (input, expected) in vectors.accepted {
        let parsed = Domain::parse(&input).unwrap_or_else(|error| panic!("{input:?}: {error}"));
        assert_eq!(parsed.as_str(), expected, "input {input:?}");
    }
}

#[test]
fn rejected_inputs_fail_with_the_expected_problem() {
    let vectors: DomainVectors = serde_json::from_str(DOMAIN_VECTORS).unwrap();
    for (input, expected) in vectors.rejected {
        let error = Domain::parse(&input).unwrap_err();
        assert_eq!(problem(&error), expected, "input {input:?}");
    }
}

#[test]
fn the_whitespace_list_is_exactly_what_rust_treats_as_whitespace() {
    let vectors: WhitespaceVectors = serde_json::from_str(WHITESPACE_VECTORS).unwrap();
    let listed = characters(&vectors.whitespace);
    let every_whitespace: Vec<char> = (0..=u32::from(char::MAX))
        .filter_map(char::from_u32)
        .filter(|candidate| candidate.is_whitespace())
        .collect();
    let mut sorted = listed.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, every_whitespace);
}

#[test]
fn lookalike_characters_are_not_whitespace() {
    let vectors: WhitespaceVectors = serde_json::from_str(WHITESPACE_VECTORS).unwrap();
    for candidate in characters(&vectors.not_whitespace) {
        assert!(!candidate.is_whitespace(), "{candidate:?}");
    }
}

#[test]
fn the_unlock_sentence_is_confirmed_with_any_listed_space_between_words() {
    let vectors: WhitespaceVectors = serde_json::from_str(WHITESPACE_VECTORS).unwrap();
    let start = Now::new(DateTime::UNIX_EPOCH, DateTime::UNIX_EPOCH.naive_utc());
    let requested = UnlockState::new(&start).request(&start).unwrap();
    let later = DateTime::UNIX_EPOCH + TimeDelta::minutes(COOLDOWN_PASSED_MINUTES);
    let ready = Now::new(later, later.naive_utc());
    for space in characters(&vectors.whitespace) {
        let phrase = UNLOCK_SENTENCE.replace(' ', &space.to_string());
        assert!(requested.confirm(&phrase, 15, &ready).is_ok(), "{space:?}");
    }
}
