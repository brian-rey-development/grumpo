use std::collections::{BTreeSet, HashSet};

use super::*;
use crate::test_support::named_app as app;

#[test]
fn rejects_blank_bundle_ids() {
    for bundle_id in ["", "   ", "\t\n"] {
        assert_eq!(
            AppRef::new(bundle_id, "Name"),
            Err(AppRefError::EmptyBundleId)
        );
    }
}

#[test]
fn bundle_ids_allow_only_letters_digits_dots_and_hyphens() {
    for valid in ["com.apple.Safari", "a", "x-y.z9", &"a".repeat(255)] {
        assert!(AppRef::new(valid, "n").is_ok(), "bundle id {valid:?}");
    }
    let cases = [
        ("com apple", AppRefError::InvalidBundleIdCharacter(' ')),
        ("com_apple", AppRefError::InvalidBundleIdCharacter('_')),
        ("com/apple", AppRefError::InvalidBundleIdCharacter('/')),
        (
            "com.\u{e9}",
            AppRefError::InvalidBundleIdCharacter('\u{e9}'),
        ),
        (&"a".repeat(256), AppRefError::BundleIdTooLong),
    ];
    for (bundle_id, expected) in cases {
        assert_eq!(
            AppRef::new(bundle_id, "n"),
            Err(expected),
            "bundle id {bundle_id:?}"
        );
    }
}

#[test]
fn names_are_trimmed_bounded_and_free_of_control_characters() {
    assert_eq!(app("com.a", "  Slack \n").name(), "Slack");
    assert_eq!(app("com.a", &"x".repeat(255)).name().len(), 255);
    assert_eq!(
        AppRef::new("com.a", "x".repeat(256)),
        Err(AppRefError::NameTooLong)
    );
    assert_eq!(
        AppRef::new("com.a", "bad\u{0}name"),
        Err(AppRefError::ControlCharacterInName('\u{0}'))
    );
    assert_eq!(
        AppRef::new("com.a", "two\nlines"),
        Err(AppRefError::ControlCharacterInName('\n'))
    );
}

#[test]
fn name_length_counts_characters_not_bytes() {
    assert!(AppRef::new("com.a", "\u{1f600}".repeat(255)).is_ok());
    assert_eq!(
        AppRef::new("com.a", "\u{1f600}".repeat(256)),
        Err(AppRefError::NameTooLong)
    );
}

#[test]
fn a_blank_name_falls_back_to_the_bundle_id() {
    for name in ["", "   ", "\t"] {
        let app = app("com.apple.Safari", name);
        assert_eq!(app.name(), "com.apple.Safari");
        assert_eq!(app.to_string(), "com.apple.Safari");
    }
}

#[test]
fn trims_the_bundle_id_and_keeps_the_name() {
    let app = app("  com.apple.Safari ", "Safari");
    assert_eq!(app.bundle_id(), "com.apple.Safari");
    assert_eq!(app.name(), "Safari");
    assert_eq!(app.to_string(), "Safari");
}

#[test]
fn identity_is_the_bundle_id() {
    let old_name = app("com.tinyspeck.slackmacgap", "Slack");
    let new_name = app("com.tinyspeck.slackmacgap", "Slack Technologies");
    assert_eq!(old_name, new_name);
    assert_eq!(old_name.cmp(&new_name), Ordering::Equal);

    let hashed: HashSet<_> = [old_name.clone(), new_name].into_iter().collect();
    assert_eq!(hashed.len(), 1);
    assert_ne!(old_name, app("com.apple.Safari", "Slack"));
}

#[test]
fn sorts_by_bundle_id() {
    let set: BTreeSet<_> = [
        app("com.b", "Alpha"),
        app("com.a", "Zulu"),
        app("com.c", "Mike"),
    ]
    .into_iter()
    .collect();
    let ids: Vec<_> = set.iter().map(AppRef::bundle_id).collect();
    assert_eq!(ids, ["com.a", "com.b", "com.c"]);
}

#[test]
fn serializes_in_camel_case_and_validates_on_the_way_in() {
    let app = app("com.apple.Safari", "Safari");
    let json = serde_json::to_string(&app).unwrap();
    assert_eq!(json, r#"{"bundleId":"com.apple.Safari","name":"Safari"}"#);
    assert_eq!(
        serde_json::from_str::<AppRef>(&json).unwrap().name(),
        "Safari"
    );

    let blank = r#"{"bundleId":" ","name":"X"}"#;
    assert!(serde_json::from_str::<AppRef>(blank).is_err());
    assert!(serde_json::from_str::<AppRef>(r#"{"name":"X"}"#).is_err());
    let invalid = r#"{"bundleId":"not valid","name":"X"}"#;
    assert!(serde_json::from_str::<AppRef>(invalid).is_err());
}

#[test]
fn deserializing_rejects_unknown_fields() {
    let json = r#"{"bundleId":"com.a","name":"A","extra":true}"#;
    let error = serde_json::from_str::<AppRef>(json).unwrap_err();
    assert!(error.to_string().contains("unknown field"));
}
