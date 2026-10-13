use super::*;

/// The inputs and problem names are in `tests/vectors/domain.json`, shared with the interface.
#[test]
fn rejections_carry_what_they_found() {
    let cases = [
        ("exa mple.com", DomainError::InvalidCharacter(' ')),
        ("[::1]", DomainError::InvalidCharacter('[')),
        ("co.uk", DomainError::PublicSuffix("co.uk".to_owned())),
        (
            "https://org.au/",
            DomainError::PublicSuffix("org.au".to_owned()),
        ),
        ("-bad.com", DomainError::HyphenAtEdge("-bad".to_owned())),
        ("bad-.com", DomainError::HyphenAtEdge("bad-".to_owned())),
    ];
    for (input, expected) in cases {
        assert_eq!(Domain::parse(input), Err(expected), "input {input:?}");
    }
}

#[test]
fn enforces_label_and_total_length_limits() {
    let longest_label = "a".repeat(MAX_LABEL_LEN);
    assert!(Domain::parse(&format!("{longest_label}.com")).is_ok());

    let too_long_label = "a".repeat(MAX_LABEL_LEN + 1);
    assert_eq!(
        Domain::parse(&format!("{too_long_label}.com")),
        Err(DomainError::LabelTooLong(too_long_label))
    );

    let filler = "a".repeat(MAX_LABEL_LEN);
    let at_limit = format!("{filler}.{filler}.{filler}.{}.com", "b".repeat(53));
    assert_eq!(at_limit.len(), MAX_DOMAIN_LEN);
    assert!(Domain::parse(&at_limit).is_ok());

    let over_limit = format!("c{at_limit}");
    assert_eq!(Domain::parse(&over_limit), Err(DomainError::TooLong));
}

#[test]
fn the_longest_domain_leaves_room_for_the_longest_alias_prefix() {
    let filler = "a".repeat(MAX_LABEL_LEN);
    let longest = format!("{filler}.{filler}.{filler}.{}.com", "b".repeat(53));
    let with_www = format!("www.{longest}");
    assert_eq!(Domain::parse(&with_www).unwrap().as_str(), longest);
    assert!(with_www.len() <= 253);
}

#[test]
fn covers_is_exact_or_subdomain_between_parsed_domains() {
    let reddit = Domain::parse("reddit.com").unwrap();
    let old_reddit = Domain::parse("old.reddit.com").unwrap();
    assert!(reddit.covers(&reddit));
    assert!(reddit.covers(&old_reddit));
    assert!(!old_reddit.covers(&reddit));
    assert!(!reddit.covers(&Domain::parse("notreddit.com").unwrap()));
    assert!(!reddit.covers(&Domain::parse("reddit.com.evil.io").unwrap()));
}

#[test]
fn serializes_as_a_plain_string_and_validates_on_the_way_in() {
    let domain = Domain::parse("https://www.Example.com/").unwrap();
    assert_eq!(serde_json::to_string(&domain).unwrap(), "\"example.com\"");
    assert_eq!(
        serde_json::from_str::<Domain>("\"WWW.Example.com\"").unwrap(),
        domain
    );
    assert!(serde_json::from_str::<Domain>("\"not a domain\"").is_err());
    assert!(serde_json::from_str::<Domain>("\"\"").is_err());
}

#[test]
fn from_str_and_as_str_agree_with_parse() {
    let domain: Domain = "Example.com".parse().unwrap();
    assert_eq!(domain.as_str(), "example.com");
    assert_eq!(String::from(domain), "example.com");
}

#[test]
fn error_messages_are_readable() {
    assert_eq!(
        DomainError::NonAscii.to_string(),
        "the domain must be ASCII, write internationalized names as punycode (xn--...)"
    );
    assert_eq!(
        DomainError::TooFewLabels.to_string(),
        "the domain needs at least 2 labels, like example.com"
    );
}
