use focus_core::ipc::EnforcementReason;

use super::*;
use crate::enforcement::engine::sweep::BACKGROUND_SWEEP_INTERVAL_SECONDS as INTERVAL;

const NOTES: &str = "com.apple.Notes";
const DISCORD: &str = "com.discord";

fn rig_with_running(running: &[&str]) -> Rig {
    let apps = running.iter().map(|id| app(id)).collect();
    Rig::with_control(
        blocking(&[STEAM, DISCORD], &[]),
        FakeControl::with_running(apps),
    )
}

#[test]
fn a_blocked_app_behind_the_front_window_is_quit_when_blocking_starts() {
    let rig = rig_with_running(&[STEAM, NOTES]);
    let mut engine = rig.engine();

    engine.sample(Some(&in_app(NOTES)), T0);

    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(rig.surface.shown(), [first_shown(app_block(STEAM))]);
}

#[test]
fn every_running_blocked_app_is_quit_not_only_the_front_one() {
    let rig = rig_with_running(&[STEAM, DISCORD, NOTES]);

    rig.engine().sample(Some(&in_app(NOTES)), T0);

    assert_eq!(rig.control.quits(), [STEAM, DISCORD]);
}

#[test]
fn running_apps_are_listed_once_per_status_and_not_on_every_sample() {
    let rig = rig_with_running(&[NOTES]);
    let mut engine = rig.engine();

    for step in 0..5 {
        engine.sample(None, T0 + 2 * step);
    }

    assert_eq!(rig.control.listings(), 1);
}

#[test]
fn a_changed_status_while_blocking_is_swept_again() {
    let rig = rig_with_running(&[STEAM, NOTES]);
    let mut engine = rig.engine();
    engine.sample(None, T0);
    assert_eq!(rig.control.quits(), [STEAM]);

    rig.daemon.respond_with(blocking(&[STEAM, NOTES], &[]));
    rig.hub.refresh();
    engine.sample(None, T0 + 10);

    assert_eq!(rig.control.listings(), 2);
    assert_eq!(rig.control.quits(), [STEAM, STEAM, NOTES]);
}

#[test]
fn an_app_the_front_window_just_handled_is_not_quit_twice() {
    let rig = rig_with_running(&[STEAM]);

    rig.engine().sample(Some(&in_app(STEAM)), T0);

    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(rig.surface.shown().len(), 1);
}

#[test]
fn blocking_that_stops_and_starts_again_sweeps_again() {
    let rig = rig_with_running(&[NOTES]);
    let mut engine = rig.engine();
    engine.sample(None, T0);

    rig.daemon.respond_with(daemon_status());
    rig.hub.refresh();
    engine.sample(None, T0 + 10);
    assert_eq!(rig.control.listings(), 1, "nothing to look for while free");

    rig.daemon.respond_with(blocking(&[STEAM, DISCORD], &[]));
    rig.hub.refresh();
    engine.sample(None, T0 + 20);

    assert_eq!(rig.control.listings(), 2);
}

#[test]
fn a_protected_app_is_never_swept() {
    let rig = Rig::with_control(
        blocking(&["com.apple.finder", STEAM], &[]),
        FakeControl::with_running(vec![app("com.apple.finder"), app(STEAM)]),
    );

    rig.engine().sample(None, T0);

    assert_eq!(rig.control.quits(), [STEAM]);
}

#[test]
fn a_failed_listing_is_tried_again_on_the_next_sample() {
    let rig = Rig::with_control(blocking(&[STEAM], &[]), FakeControl::failing());
    let mut engine = rig.engine();

    engine.sample(None, T0);
    engine.sample(None, T0 + 2);

    assert_eq!(rig.control.listings(), 2);
    assert_eq!(rig.control.quits().len(), 0);
}

#[test]
fn nothing_is_swept_while_the_daemon_is_unreachable() {
    let rig = Rig::offline();

    rig.engine().sample(None, T0);

    assert_eq!(rig.control.listings(), 0);
}

fn spent_discord_limit() -> focus_core::ipc::DaemonStatus {
    let mut status = blocking(&[DISCORD], &[]);
    status.enforcement.reason = EnforcementReason::Limit;
    status.rules.limits.push(focus_core::Limit {
        target: app_target(DISCORD),
        daily_minutes: 30,
    });
    status.exhausted = vec![app_target(DISCORD)];
    status
}

#[test]
fn a_limit_block_from_yesterday_is_left_alone_until_the_daemon_confirms_it_after_midnight() {
    let rig = Rig::with_control(
        spent_discord_limit(),
        FakeControl::with_running(vec![app(DISCORD)]),
    );
    let mut engine = rig.engine();

    engine.sample(Some(&in_app(DISCORD)), T0);
    assert_eq!(rig.control.quits(), [DISCORD], "still the same day");

    engine.sample(Some(&in_app(DISCORD)), T0 + DAY);

    assert_eq!(
        rig.control.quits().len(),
        1,
        "the status predates the new day"
    );
}

#[test]
fn once_the_daemon_confirms_the_status_after_midnight_the_block_is_enforced_again() {
    let rig = Rig::with_control(spent_discord_limit(), FakeControl::default());
    let mut engine = rig.engine();
    engine.sample(None, T0);

    rig.clock.set(T0 + DAY);
    rig.hub.refresh();
    engine.sample(Some(&in_app(DISCORD)), T0 + DAY + 2);

    assert_eq!(rig.control.quits(), [DISCORD]);
}

#[test]
fn an_app_launched_in_the_background_mid_window_is_quit_within_one_sweep_interval() {
    let rig = rig_with_running(&[NOTES]);
    let mut engine = rig.engine();
    engine.sample(None, T0);
    assert_eq!(rig.control.quits().len(), 0);

    rig.control.launch(app(STEAM));
    engine.sample(None, T0 + INTERVAL - 1);
    assert_eq!(rig.control.quits().len(), 0, "the listing is still fresh");

    engine.sample(None, T0 + INTERVAL);

    assert_eq!(rig.control.quits(), [STEAM]);
    assert_eq!(rig.surface.shown(), [first_shown(app_block(STEAM))]);
}

#[test]
fn the_running_apps_are_listed_again_once_per_interval_while_blocking() {
    let rig = rig_with_running(&[NOTES]);
    let mut engine = rig.engine();

    for step in 0..=3 {
        engine.sample(None, T0 + step * INTERVAL);
        engine.sample(None, T0 + step * INTERVAL + 2);
    }

    assert_eq!(rig.control.listings(), 4);
}

#[test]
fn nothing_is_listed_periodically_while_blocking_is_inactive() {
    let rig = rig_with_running(&[STEAM]);
    rig.daemon.respond_with(daemon_status());
    rig.hub.refresh();
    let mut engine = rig.engine();

    for step in 0..5 {
        engine.sample(None, T0 + step * INTERVAL);
    }

    assert_eq!(rig.control.listings(), 0);
    assert_eq!(rig.control.quits().len(), 0);
}

#[test]
fn a_protected_app_started_mid_window_is_never_quit_by_a_later_sweep() {
    let rig = Rig::with_control(
        blocking(&["com.apple.finder", STEAM], &[]),
        FakeControl::default(),
    );
    let mut engine = rig.engine();
    engine.sample(None, T0);

    rig.control.launch(app("com.apple.finder"));
    rig.control.launch(app(STEAM));
    engine.sample(None, T0 + INTERVAL);

    assert_eq!(rig.control.quits(), [STEAM]);
}

#[test]
fn a_sweep_right_after_the_front_window_handled_an_app_does_not_quit_it_again() {
    let rig = rig_with_running(&[]);
    let mut engine = rig.engine();
    engine.sample(None, T0);
    rig.control.launch(app(STEAM));

    engine.sample(Some(&in_app(STEAM)), T0 + INTERVAL - 2);
    assert_eq!(rig.control.quits(), [STEAM]);

    engine.sample(None, T0 + INTERVAL);

    assert_eq!(rig.control.quits(), [STEAM], "inside the retry window");
}

#[test]
fn an_app_that_survives_is_asked_again_each_sweep_but_announced_once_a_minute() {
    let rig = rig_with_running(&[STEAM]);
    let mut engine = rig.engine();

    engine.sample(None, T0);
    engine.sample(None, T0 + INTERVAL);
    assert_eq!(rig.control.quits().len(), 2);
    assert_eq!(rig.surface.shown().len(), 1);

    engine.sample(None, T0 + 2 * INTERVAL);
    assert_eq!(rig.control.quits().len(), 3);
    assert_eq!(rig.surface.shown().len(), 2);
}

#[test]
fn a_clock_that_moves_backwards_makes_the_next_sweep_due() {
    let rig = rig_with_running(&[NOTES]);
    let mut engine = rig.engine();
    engine.sample(None, T0 + 100);

    engine.sample(None, T0);

    assert_eq!(rig.control.listings(), 2);
}
