use super::*;

const FINDER: &str = "com.apple.finder";

#[test]
fn a_protected_app_in_front_is_never_quit_or_announced() {
    let activity = in_app(FINDER);

    let actions = decide_now(Some(&activity), &blocking(&[FINDER], &[]));

    assert_eq!(actions, []);
}

#[test]
fn the_protection_ignores_case_like_the_system_does() {
    let activity = in_app("com.Apple.Finder");
    let mut status = blocking(&[], &[]);
    status.enforcement.blocked_apps.insert(activity.app.clone());

    assert_eq!(decide_now(Some(&activity), &status), []);
}

#[test]
fn a_protected_app_never_falls_through_to_a_tab_close() {
    let activity = browsing(Browser::Chrome, "youtube.com");
    let mut status = blocking(&[], &["youtube.com"]);
    status.enforcement.blocked_apps.insert(app(FINDER));

    assert_eq!(decide_now(Some(&in_app(FINDER)), &status), []);
    assert_eq!(decide_now(Some(&activity), &status).len(), 2);
}

#[test]
fn the_background_sweep_skips_protected_apps_too() {
    let status = blocking(&[FINDER, STEAM], &[]);

    let actions = sweep_now(&[app(FINDER), app(STEAM)], &status, &memory());

    assert_eq!(actions, [Action::QuitApp(app(STEAM)), app_blocked_notice()]);
}
