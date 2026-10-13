use super::*;
use crate::storage::test_support::{record, store};

fn names(mut apps: Vec<AppRef>) -> Vec<(String, String)> {
    apps.sort();
    apps.iter()
        .map(|a| (a.bundle_id().to_owned(), a.name().to_owned()))
        .collect()
}

#[test]
fn lists_each_recent_app_once_with_its_latest_name() {
    let store = store();
    record(&store, "com.a", "Old Name", None, (100, 200));
    record(&store, "com.a", "New Name", None, (300, 400));
    record(&store, "com.b", "Beta", Some("github.com"), (310, 320));

    let apps = store.apps_since(0).unwrap();
    assert_eq!(
        names(apps),
        [
            ("com.a".to_owned(), "New Name".to_owned()),
            ("com.b".to_owned(), "Beta".to_owned())
        ]
    );
}

#[test]
fn leaves_out_apps_not_used_since_the_cutoff() {
    let store = store();
    record(&store, "com.old", "Old", None, (100, 200));
    record(&store, "com.new", "New", None, (900, 1_000));

    let apps = store.apps_since(500).unwrap();
    assert_eq!(names(apps), [("com.new".to_owned(), "New".to_owned())]);
}
