use super::*;

#[test]
fn nothing_is_enforced_while_the_daemon_is_unavailable() {
    let activity = in_app(STEAM);

    assert_eq!(
        decide_with(Some(&activity), None, &[], &memory(), T0).len(),
        0
    );
}

#[test]
fn a_blocked_app_in_front_is_quit_and_announced_with_the_end_time() {
    let activity = in_app(STEAM);

    let actions = decide_now(Some(&activity), &blocking(&[STEAM], &[]));

    assert_eq!(actions, [Action::QuitApp(app(STEAM)), app_blocked_notice()]);
}

#[test]
fn an_app_that_is_not_blocked_is_left_alone() {
    let activity = in_app("com.apple.Notes");

    assert_eq!(
        decide_now(Some(&activity), &blocking(&[STEAM], &["youtube.com"])).len(),
        0
    );
}

#[test]
fn blocked_apps_are_ignored_while_enforcement_is_inactive() {
    let activity = in_app(STEAM);
    let mut status = blocking(&[STEAM], &[]);
    status.enforcement.active = false;

    assert_eq!(decide_now(Some(&activity), &status).len(), 0);
}

#[test]
fn no_activity_means_nothing_to_block() {
    assert_eq!(
        decide_now(None, &blocking(&[STEAM], &["youtube.com"])).len(),
        0
    );
}

#[test]
fn an_idle_blocked_app_is_still_quit() {
    let mut activity = in_app(STEAM);
    activity.engaged = false;

    let actions = decide_now(Some(&activity), &blocking(&[STEAM], &[]));

    assert_eq!(actions.len(), 2);
}

#[test]
fn a_blocked_site_in_a_browser_closes_the_tab_and_names_the_rule_domain() {
    let activity = browsing(Browser::Chrome, "m.youtube.com");

    let actions = decide_now(Some(&activity), &blocking(&[], &["youtube.com"]));

    assert_eq!(
        actions,
        [
            Action::CloseTab {
                browser: Browser::Chrome,
                domain: site("m.youtube.com"),
            },
            site_blocked_notice("youtube.com"),
        ]
    );
}

#[test]
fn a_domain_that_merely_ends_like_a_blocked_one_is_not_blocked() {
    let activity = browsing(Browser::Chrome, "notyoutube.com");

    assert_eq!(
        decide_now(Some(&activity), &blocking(&[], &["youtube.com"])).len(),
        0
    );
}

#[test]
fn private_browsing_is_enforced_like_any_other() {
    let mut activity = browsing(Browser::Safari, "youtube.com");
    activity.private = true;

    let actions = decide_now(Some(&activity), &blocking(&[], &["youtube.com"]));

    assert_eq!(actions.len(), 2);
}

#[test]
fn firefox_tabs_cannot_be_closed_so_nothing_is_done() {
    let activity = browsing(Browser::Firefox, "youtube.com");

    assert_eq!(
        decide_now(Some(&activity), &blocking(&[], &["youtube.com"])).len(),
        0
    );
}

#[test]
fn a_browser_on_a_page_without_a_domain_is_left_alone() {
    let mut activity = browsing(Browser::Chrome, "youtube.com");
    activity.domain = None;

    assert_eq!(
        decide_now(Some(&activity), &blocking(&[], &["youtube.com"])).len(),
        0
    );
}

#[test]
fn a_blocked_browser_is_quit_instead_of_having_its_tab_closed() {
    let activity = browsing(Browser::Chrome, "youtube.com");
    let status = blocking(&[Browser::Chrome.bundle_id()], &["youtube.com"]);

    let actions = decide_now(Some(&activity), &status);

    assert!(matches!(actions[0], Action::QuitApp(_)));
    assert!(!actions.iter().any(|a| matches!(a, Action::CloseTab { .. })));
}

#[test]
fn the_same_app_is_not_quit_again_within_the_retry_window() {
    let activity = in_app(STEAM);
    let status = blocking(&[STEAM], &[]);
    let mut memory = memory();
    let first = decide_with(Some(&activity), Some(&status), &[], &memory, T0);
    note_all(&mut memory, &first, T0);

    let soon = decide_with(Some(&activity), Some(&status), &[], &memory, T0 + 2);
    let later = decide_with(Some(&activity), Some(&status), &[], &memory, T0 + 3);

    assert!(soon.is_empty(), "{soon:?}");
    assert_eq!(
        later,
        [Action::QuitApp(app(STEAM))],
        "retried, but not re-announced"
    );
}

#[test]
fn the_notice_for_the_same_app_returns_after_a_minute() {
    let activity = in_app(STEAM);
    let status = blocking(&[STEAM], &[]);
    let mut memory = memory();
    note_all(
        &mut memory,
        &[Action::QuitApp(app(STEAM)), app_blocked_notice()],
        T0,
    );

    let before = decide_with(Some(&activity), Some(&status), &[], &memory, T0 + 59);
    let after = decide_with(Some(&activity), Some(&status), &[], &memory, T0 + 60);

    assert_eq!(before, [Action::QuitApp(app(STEAM))]);
    assert_eq!(after, [Action::QuitApp(app(STEAM)), app_blocked_notice()]);
}

#[test]
fn rate_limits_are_per_app_and_per_site() {
    let status = blocking(&[STEAM], &["youtube.com"]);
    let mut memory = memory();
    note_all(
        &mut memory,
        &[Action::QuitApp(app(STEAM)), app_blocked_notice()],
        T0,
    );

    let site = browsing(Browser::Chrome, "youtube.com");
    let actions = decide_with(Some(&site), Some(&status), &[], &memory, T0 + 1);

    assert_eq!(
        actions.len(),
        2,
        "a site is not throttled by an app: {actions:?}"
    );
}

#[test]
fn a_repeated_tab_close_is_throttled_like_a_quit() {
    let activity = browsing(Browser::Chrome, "youtube.com");
    let status = blocking(&[], &["youtube.com"]);
    let mut memory = memory();
    let first = decide_with(Some(&activity), Some(&status), &[], &memory, T0);
    note_all(&mut memory, &first, T0);

    assert_eq!(
        decide_with(Some(&activity), Some(&status), &[], &memory, T0 + 1).len(),
        0
    );
}
