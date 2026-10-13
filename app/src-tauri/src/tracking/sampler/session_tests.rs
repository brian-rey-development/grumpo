use super::test_support::{rig, T0};
use crate::platform::Browser;
use crate::tracking::test_support::CHROME;

#[test]
fn private_browsing_is_seen_by_the_sink_but_never_stored() {
    let mut rig = rig();
    rig.probe.set_foreground(Some(CHROME));
    rig.probe
        .set_tab(Browser::Chrome, Some("youtube.com"), true);
    rig.tick_every_two_seconds(3);
    rig.sampler.finish();

    assert_eq!(rig.rows().len(), 0);
    let samples = rig.sink.samples.lock().unwrap();
    assert!(samples
        .iter()
        .all(|(a, _)| a.as_ref().is_some_and(|a| a.private)));
    let domain = samples[0].0.as_ref().and_then(|a| a.domain.as_ref());
    assert_eq!(domain.map(focus_core::Domain::as_str), Some("youtube.com"));
}

#[test]
fn leaving_a_private_window_closes_the_stored_session() {
    let mut rig = rig();
    rig.probe.set_foreground(Some(CHROME));
    rig.probe.set_host(Browser::Chrome, Some("docs.rs"));
    rig.tick_every_two_seconds(3);
    rig.probe.set_tab(Browser::Chrome, Some("docs.rs"), true);
    rig.tick_every_two_seconds(8);
    rig.sampler.finish();
    assert_eq!(
        rig.rows(),
        [(T0, T0 + 6, CHROME.to_owned(), Some("docs.rs".to_owned()))]
    );
}

#[test]
fn going_idle_ends_the_session_at_the_last_input() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(3);
    let threshold = crate::tracking::activity::IDLE_THRESHOLD.as_secs();
    for idle in (2..).step_by(2).take_while(|idle| *idle <= threshold) {
        rig.probe.set_idle(idle);
        rig.tick_every_two_seconds(1);
    }
    rig.tick_every_two_seconds(5);
    rig.sampler.finish();
    assert_eq!(rig.rows(), [(T0, T0 + 4, "com.a".to_owned(), None)]);
}

#[test]
fn media_keeps_an_idle_session_alive_until_it_stops() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(1);
    rig.probe.set_idle(500);
    rig.probe.set_media(true);
    rig.tick_every_two_seconds(5);
    rig.probe.set_media(false);
    rig.tick_every_two_seconds(2);
    rig.sampler.finish();
    assert_eq!(rig.rows(), [(T0, T0 + 10, "com.a".to_owned(), None)]);
}

#[test]
fn deleting_all_data_mid_session_does_not_stop_recording() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(3);
    rig.store.delete_all(0, false).unwrap();
    rig.tick_every_two_seconds(8);
    rig.sampler.finish();
    let rows = rig.rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(rows[0].2, "com.a");
    assert!(rows[0].1 > rows[0].0);
}

#[test]
fn a_locked_screen_ends_the_session() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(3);
    rig.probe.set_locked(true);
    rig.tick_every_two_seconds(5);
    rig.sampler.finish();
    assert_eq!(rig.rows(), [(T0, T0 + 4, "com.a".to_owned(), None)]);
}

#[test]
fn a_clock_step_back_never_produces_overlapping_sessions() {
    let mut rig = rig();
    rig.probe.set_foreground(Some("com.a"));
    rig.tick_every_two_seconds(4);
    rig.clock.set(T0 + 2);
    rig.probe.set_foreground(Some("com.b"));
    rig.tick_every_two_seconds(3);
    rig.sampler.finish();
    let rows = rig.rows();
    assert_eq!(rows.len(), 2);
    assert!(rows[0].1 <= rows[1].0);
}
