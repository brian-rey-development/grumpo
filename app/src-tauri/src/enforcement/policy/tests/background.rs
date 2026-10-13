use super::*;

const NOTES: &str = "com.apple.Notes";

#[test]
fn every_running_blocked_app_is_quit_and_announced() {
    let status = blocking(&[STEAM, "com.discord"], &[]);

    let actions = sweep_now(
        &[app(STEAM), app(NOTES), app("com.discord")],
        &status,
        &memory(),
    );

    let quits: Vec<_> = actions
        .iter()
        .filter(|action| matches!(action, Action::QuitApp(_)))
        .collect();
    assert_eq!(
        quits,
        [
            &Action::QuitApp(app(STEAM)),
            &Action::QuitApp(app("com.discord"))
        ]
    );
    assert_eq!(actions.len(), 4);
}

#[test]
fn nothing_is_swept_while_blocking_is_inactive_or_the_daemon_is_unreachable() {
    let mut inactive = blocking(&[STEAM], &[]);
    inactive.enforcement.active = false;

    assert_eq!(sweep_now(&[app(STEAM)], &inactive, &memory()), []);
    assert_eq!(
        sweep(&input_for(None, None, &[], &memory(), T0), &[app(STEAM)]),
        []
    );
}

#[test]
fn the_sweep_shares_the_per_app_rate_limit_with_the_front_window() {
    let status = blocking(&[STEAM], &[]);
    let mut memory = memory();
    note_all(&mut memory, &[Action::QuitApp(app(STEAM))], T0);

    let soon = sweep(
        &input_for(None, Some(&status), &[], &memory, T0 + 1),
        &[app(STEAM)],
    );
    let later = sweep(
        &input_for(None, Some(&status), &[], &memory, T0 + 3),
        &[app(STEAM)],
    );

    assert_eq!(
        soon,
        [app_blocked_notice()],
        "only the notice, which is on its own timer"
    );
    assert_eq!(later[0], Action::QuitApp(app(STEAM)));
}

#[test]
fn tabs_are_not_part_of_the_sweep() {
    let status = blocking(&[], &["youtube.com"]);

    assert_eq!(
        sweep_now(&[app(Browser::Chrome.bundle_id())], &status, &memory()),
        []
    );
}
