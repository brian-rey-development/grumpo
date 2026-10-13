use super::*;

#[test]
fn recognizes_protected_apps_in_any_case() {
    for (id, _) in PROTECTED_APPS {
        assert!(is_protected(id), "{id}");
        assert!(is_protected(&id.to_uppercase()), "{id}");
    }
}

#[test]
fn leaves_other_apps_and_lookalikes_alone() {
    for id in [
        "com.apple.Terminal",
        "com.apple.finder.helper",
        "com.apple",
        "",
    ] {
        assert!(!is_protected(id), "{id}");
    }
}

#[test]
fn every_protected_app_has_a_friendly_name() {
    for (id, _) in PROTECTED_APPS {
        assert_ne!(display_name(id), id);
    }
    assert_eq!(display_name("com.example.unknown"), "com.example.unknown");
}
