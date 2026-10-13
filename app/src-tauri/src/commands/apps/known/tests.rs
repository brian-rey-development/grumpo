use focus_core::test_support::named_app;

use super::*;

fn summary(apps: &[KnownApp]) -> Vec<(&str, &str, bool)> {
    apps.iter()
        .map(|k| (k.app.bundle_id(), k.app.name(), k.running))
        .collect()
}

#[test]
fn dedupes_by_bundle_id_and_marks_running() {
    let running = vec![named_app("com.b", "Beta Live")];
    let history = vec![named_app("com.b", "Beta Old"), named_app("com.a", "Alpha")];

    let merged = merge(running, history);
    assert_eq!(
        summary(&merged),
        [("com.a", "Alpha", false), ("com.b", "Beta Live", true)]
    );
}

#[test]
fn sorts_by_name_ignoring_case() {
    let history = vec![
        named_app("com.z", "zed"),
        named_app("com.a", "Zebra"),
        named_app("com.m", "Mail"),
    ];
    let names: Vec<_> = merge(Vec::new(), history)
        .iter()
        .map(|k| k.app.name().to_owned())
        .collect();
    assert_eq!(names, ["Mail", "Zebra", "zed"]);
}

#[test]
fn equal_names_fall_back_to_the_bundle_id() {
    let history = vec![named_app("com.b", "Same"), named_app("com.a", "Same")];
    let ids: Vec<_> = merge(Vec::new(), history)
        .iter()
        .map(|k| k.app.bundle_id().to_owned())
        .collect();
    assert_eq!(ids, ["com.a", "com.b"]);
}

#[test]
fn protected_apps_and_this_app_are_never_offered() {
    let all = merge(
        vec![
            named_app("com.apple.finder", "Finder"),
            named_app("com.apple.systempreferences", "System Settings"),
            named_app("com.example.me", "Me"),
            named_app("com.example.other", "Other"),
        ],
        Vec::new(),
    );

    let offered = without_unblockable(all, "COM.EXAMPLE.ME");

    assert_eq!(summary(&offered), [("com.example.other", "Other", true)]);
}

#[test]
fn nothing_known_is_an_empty_list() {
    assert_eq!(merge(Vec::new(), Vec::new()), Vec::<KnownApp>::new());
}
