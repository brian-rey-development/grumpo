use super::*;
use crate::dto::BlockScreen;

#[test]
fn a_blocked_app_is_quit_and_announced_once_while_it_stays_in_front() {
    let rig = Rig::new(blocking(&[STEAM], &[]));
    let mut engine = rig.engine();
    let steam = in_app(STEAM);

    engine.sample(Some(&steam), T0);
    engine.sample(Some(&steam), T0 + 2);

    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(rig.surface.shown(), [first_shown(app_block(STEAM))]);
}

#[test]
fn a_blocked_app_that_survives_is_asked_again_but_not_announced_again() {
    let rig = Rig::new(blocking(&[STEAM], &[]));
    let mut engine = rig.engine();
    let steam = in_app(STEAM);

    engine.sample(Some(&steam), T0);
    engine.sample(Some(&steam), T0 + 4);
    assert_eq!(rig.control.quits().len(), 2);
    assert_eq!(rig.surface.shown().len(), 1);

    engine.sample(Some(&steam), T0 + 61);
    assert_eq!(rig.surface.shown().len(), 2);
}

#[test]
fn a_blocked_tab_is_closed_and_announced() {
    let rig = Rig::new(blocking(&[], &["youtube.com"]));
    let mut engine = rig.engine();

    engine.sample(Some(&browsing(Browser::Chrome, "m.youtube.com")), T0);

    assert_eq!(
        rig.control.closes(),
        [(Browser::Chrome, "m.youtube.com".to_owned())]
    );
    assert_eq!(
        rig.surface.shown(),
        [first_shown(site_block("youtube.com"))]
    );
}

#[test]
fn a_blocked_app_is_announced_by_notification_when_the_window_cannot_be_shown() {
    let rig = Rig::new(blocking(&[STEAM], &[])).without_window();

    rig.engine().sample(Some(&in_app(STEAM)), T0);

    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(
        rig.titles(),
        ["com.valvesoftware.steam is blocked until 09:00"]
    );
    assert_eq!(rig.surface.shown().len(), 0);
}

#[test]
fn a_blocked_tab_is_announced_by_notification_when_the_window_cannot_be_shown() {
    let rig = Rig::new(blocking(&[], &["youtube.com"])).without_window();

    rig.engine()
        .sample(Some(&browsing(Browser::Chrome, "youtube.com")), T0);

    assert_eq!(rig.titles(), ["youtube.com is blocked until 09:00"]);
}

#[test]
fn a_block_that_reaches_the_window_sends_no_notification() {
    let rig = Rig::new(blocking(&[STEAM], &[]));

    rig.engine().sample(Some(&in_app(STEAM)), T0);

    assert_eq!(rig.titles().len(), 0);
}

#[test]
fn the_block_screen_keeps_the_latest_block_when_another_follows() {
    let rig = Rig::new(blocking(&[STEAM], &["youtube.com"]));
    let mut engine = rig.engine();

    engine.sample(Some(&in_app(STEAM)), T0);
    engine.sample(Some(&browsing(Browser::Chrome, "youtube.com")), T0 + 4);

    let expected = BlockScreen {
        id: 2,
        ..site_block("youtube.com")
    };
    assert_eq!(rig.block_screens.current(), Some(expected));
}

#[test]
fn private_tabs_are_closed_too() {
    let rig = Rig::new(blocking(&[], &["youtube.com"]));
    let mut private = browsing(Browser::Safari, "youtube.com");
    private.private = true;

    rig.engine().sample(Some(&private), T0);

    assert_eq!(rig.control.closes().len(), 1);
}

#[test]
fn nothing_is_done_while_the_daemon_is_unreachable() {
    let rig = Rig::offline();

    rig.engine().sample(Some(&in_app(STEAM)), T0);

    assert_eq!(rig.control.quits().len(), 0);
    assert_eq!(rig.surface.shown().len(), 0);
}

#[test]
fn a_failing_quit_does_not_panic_and_is_still_rate_limited() {
    let rig = Rig::with_control(blocking(&[STEAM], &[]), FakeControl::failing());
    let mut engine = rig.engine();
    let steam = in_app(STEAM);

    engine.sample(Some(&steam), T0);
    engine.sample(Some(&steam), T0 + 1);

    assert_eq!(rig.control.quits().len(), 1);
}
