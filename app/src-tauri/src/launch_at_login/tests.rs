use std::path::PathBuf;

use super::test_support::FakeLaunch;
use super::*;

fn no_agents() -> PathBuf {
    PathBuf::from("/nonexistent/LaunchAgents")
}

fn installed() -> PathBuf {
    PathBuf::from("/Applications/Grumpo.app/Contents/MacOS/Grumpo")
}

#[test]
fn first_run_enables_once_and_respects_a_later_opt_out() {
    let store = Store::open_in_memory().unwrap();
    let launch = FakeLaunch::default();

    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    assert!(launch.is_enabled().unwrap());

    launch.set_enabled(false).unwrap();
    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    assert!(!launch.is_enabled().unwrap());
}

#[test]
fn a_refused_first_run_is_retried_next_time() {
    let store = Store::open_in_memory().unwrap();
    assert!(sync_on_start(&store, &FakeLaunch::failing(), &installed(), &no_agents()).is_err());

    let launch = FakeLaunch::default();
    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    assert!(launch.is_enabled().unwrap());
}

#[test]
fn an_existing_login_item_is_rewritten_on_every_start() {
    let store = Store::open_in_memory().unwrap();
    let launch = FakeLaunch::default();
    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    assert_eq!(launch.writes(), 1);

    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();

    assert_eq!(launch.writes(), 3);
    assert!(launch.is_enabled().unwrap());
}

fn initialized_store() -> Store {
    let store = Store::open_in_memory().unwrap();
    store
        .set_setting(INITIALIZED_KEY, INITIALIZED_VALUE)
        .unwrap();
    store
}

fn plist_launching(program: &str) -> String {
    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict>
<key>Label</key><string>Focus</string>
<key>ProgramArguments</key><array><string>{program}</string><string>--autostarted</string></array>
</dict></plist>"#
    )
}

fn leave_item_launching(agents: &Path, label: &str, program: &str) -> PathBuf {
    let plist = agents.join(format!("{label}.plist"));
    std::fs::write(&plist, plist_launching(program)).unwrap();
    plist
}

fn leave_item(agents: &Path, label: &str) -> PathBuf {
    leave_item_launching(
        agents,
        label,
        "/Applications/Focus.app/Contents/MacOS/Focus",
    )
}

#[test]
fn a_login_item_left_by_the_old_app_is_replaced_on_the_first_start_of_the_renamed_app() {
    let agents = tempfile::tempdir().unwrap();
    let old = leave_item(agents.path(), "io.github.brian-rey-development.focus");
    let launch = FakeLaunch::default();

    sync_on_start(&initialized_store(), &launch, &installed(), agents.path()).unwrap();

    assert!(launch.is_enabled().unwrap(), "the new item is on");
    assert!(!old.exists(), "the old plist is gone");
}

#[test]
fn the_oldest_login_item_name_is_removed_too() {
    let agents = tempfile::tempdir().unwrap();
    let oldest = leave_item(agents.path(), "Focus");
    let launch = FakeLaunch::default();

    sync_on_start(&initialized_store(), &launch, &installed(), agents.path()).unwrap();

    assert!(launch.is_enabled().unwrap());
    assert!(!oldest.exists());
}

#[test]
fn the_old_item_is_kept_when_the_new_one_cannot_be_turned_on() {
    let agents = tempfile::tempdir().unwrap();
    let old = leave_item(agents.path(), "io.github.brian-rey-development.focus");

    let result = sync_on_start(
        &initialized_store(),
        &FakeLaunch::failing(),
        &installed(),
        agents.path(),
    );

    assert!(result.is_err());
    assert!(old.exists(), "the next start finishes the replacement");
}

#[test]
fn replacing_the_old_item_marks_the_first_run_as_done() {
    let agents = tempfile::tempdir().unwrap();
    leave_item(agents.path(), "io.github.brian-rey-development.focus");
    let store = Store::open_in_memory().unwrap();
    let launch = FakeLaunch::default();
    sync_on_start(&store, &launch, &installed(), agents.path()).unwrap();
    launch.set_enabled(false).unwrap();

    sync_on_start(&store, &launch, &installed(), agents.path()).unwrap();

    assert!(!launch.is_enabled().unwrap(), "the opt-out sticks");
}

#[test]
fn nothing_is_replaced_from_a_disk_image() {
    let agents = tempfile::tempdir().unwrap();
    let old = leave_item(agents.path(), "io.github.brian-rey-development.focus");
    let launch = FakeLaunch::default();

    sync_on_start(
        &initialized_store(),
        &launch,
        Path::new("/Volumes/Grumpo/Grumpo.app/Contents/MacOS/Grumpo"),
        agents.path(),
    )
    .unwrap();

    assert_eq!(launch.writes(), 0);
    assert!(old.exists());
}

#[test]
fn an_enabled_login_item_under_the_current_label_is_rewritten_as_before() {
    let launch = FakeLaunch::already_enabled();

    sync_on_start(&initialized_store(), &launch, &installed(), &no_agents()).unwrap();

    assert_eq!(launch.writes(), 1);
}

#[test]
fn a_person_who_never_had_the_login_item_does_not_get_one_from_the_renamed_app() {
    let store = Store::open_in_memory().unwrap();
    store
        .set_setting(INITIALIZED_KEY, INITIALIZED_VALUE)
        .unwrap();
    let launch = FakeLaunch::default();

    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();

    assert_eq!(launch.writes(), 0);
    assert!(!launch.is_enabled().unwrap());
}

#[test]
fn a_disabled_login_item_is_left_disabled() {
    let store = Store::open_in_memory().unwrap();
    let launch = FakeLaunch::default();
    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    launch.set_enabled(false).unwrap();
    let before = launch.writes();

    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();

    assert_eq!(launch.writes(), before);
    assert!(!launch.is_enabled().unwrap());
}

#[test]
fn a_disk_image_or_translocated_run_writes_nothing_and_stays_pending() {
    let store = Store::open_in_memory().unwrap();
    let launch = FakeLaunch::default();
    for path in [
        "/Volumes/Grumpo/Grumpo.app/Contents/MacOS/Grumpo",
        "/private/var/folders/x/T/AppTranslocation/ABC/d/Grumpo.app/Contents/MacOS/Grumpo",
    ] {
        sync_on_start(&store, &launch, Path::new(path), &no_agents()).unwrap();
    }
    assert_eq!(launch.writes(), 0);

    sync_on_start(&store, &launch, &installed(), &no_agents()).unwrap();
    assert!(
        launch.is_enabled().unwrap(),
        "the first run was never used up"
    );
}

#[test]
fn a_volume_whose_name_merely_starts_like_the_prefix_is_not_special() {
    assert!(!is_transient_location(Path::new("/Volumesque/Grumpo.app")));
    assert!(!is_transient_location(Path::new(
        "/Users/me/Applications/Grumpo.app"
    )));
}

#[test]
fn the_login_item_argument_is_recognized() {
    let with = ["focus", "--autostarted"].map(str::to_owned);
    let without = ["focus", "--other"].map(str::to_owned);
    assert!(was_autostarted(with.into_iter()));
    assert!(!was_autostarted(without.into_iter()));
}

#[test]
fn a_login_item_of_another_app_under_a_legacy_label_is_left_alone() {
    let agents = tempfile::tempdir().unwrap();
    let foreign = leave_item_launching(
        agents.path(),
        "Focus",
        "/Applications/FocusFlow.app/Contents/MacOS/FocusFlow",
    );
    let launch = FakeLaunch::default();

    sync_on_start(&initialized_store(), &launch, &installed(), agents.path()).unwrap();

    assert!(foreign.exists(), "someone else's login item stays");
    assert_eq!(launch.writes(), 0, "a foreign item does not count as ours");
}

#[test]
fn an_unreadable_login_item_under_a_legacy_label_is_left_alone() {
    let agents = tempfile::tempdir().unwrap();
    let path = agents.path().join("Focus.plist");
    std::fs::write(&path, "not a property list").unwrap();

    sync_on_start(
        &initialized_store(),
        &FakeLaunch::default(),
        &installed(),
        agents.path(),
    )
    .unwrap();

    assert!(path.exists());
}

#[test]
fn a_login_item_that_launches_grumpo_under_a_legacy_label_is_replaced() {
    let agents = tempfile::tempdir().unwrap();
    let ours = leave_item_launching(
        agents.path(),
        "Focus",
        "/Applications/Grumpo.app/Contents/MacOS/Grumpo",
    );

    sync_on_start(
        &initialized_store(),
        &FakeLaunch::default(),
        &installed(),
        agents.path(),
    )
    .unwrap();

    assert!(!ours.exists());
}
