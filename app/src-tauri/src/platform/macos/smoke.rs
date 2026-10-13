//! Live checks against the real system. Run with
//! `cargo test -p focus-app platform::macos -- --ignored --nocapture`.

use crate::platform::{Browser, Permission, Platform};

fn platform() -> Platform {
    super::platform()
}

const PNG_SIGNATURE: [u8; 8] = [0x89, b'P', b'N', b'G', 0x0d, 0x0a, 0x1a, 0x0a];

fn png_dimensions(png: &[u8]) -> (u32, u32) {
    let read = |range: std::ops::Range<usize>| u32::from_be_bytes(png[range].try_into().unwrap());
    (read(16..20), read(20..24))
}

#[test]
#[ignore = "touches the live system"]
fn foreground_app_is_reported() {
    let app = platform().probe.foreground_app();
    println!("foreground: {app:?}");
    assert!(matches!(app, Ok(Some(_))));
}

#[test]
#[ignore = "touches the live system"]
fn idle_time_is_small() {
    let idle = platform().probe.idle_time().unwrap();
    println!("idle: {idle:?}");
    assert!(idle < std::time::Duration::from_hours(24));
}

#[test]
#[ignore = "touches the live system"]
fn media_playing_is_decided_for_the_frontmost_app() {
    let platform = platform();
    let front = platform.probe.foreground_app().unwrap().unwrap();
    let playing = platform.probe.media_playing(front.pid);
    println!(
        "media playing for {} (pid {}): {playing:?}",
        front.app.name(),
        front.pid
    );
    assert!(playing.is_ok());
    assert!(!platform.probe.media_playing(1).unwrap());
}

#[test]
#[ignore = "touches the live system"]
fn power_assertions_expose_the_keys_the_media_rules_read() {
    let assertions = super::power_assertions::snapshot().unwrap();
    println!("{} power assertions", assertions.len());
    for assertion in &assertions {
        println!("  {assertion:?}");
    }
}

#[test]
#[ignore = "touches the live system"]
fn process_tree_resolves_this_process() {
    let own = i32::try_from(std::process::id()).unwrap();
    println!(
        "works_for(self, self): {}",
        super::process_tree::works_for(own, own)
    );
    println!(
        "works_for(self, launchd): {}",
        super::process_tree::works_for(own, 1)
    );
    assert!(super::process_tree::works_for(own, own));
}

#[test]
#[ignore = "touches the live system"]
fn screen_lock_state_is_readable() {
    let locked = platform().probe.screen_locked();
    println!("screen locked: {locked:?}");
    assert!(locked.is_ok());
}

#[test]
#[ignore = "touches the live system"]
fn running_apps_is_not_empty_and_sorted() {
    let apps = platform().control.running_apps().unwrap();
    println!("running apps: {}", apps.len());
    for app in apps.iter().take(5) {
        println!("  {} ({})", app.name(), app.bundle_id());
    }
    assert_ne!(apps.len(), 0);
    let names: Vec<_> = apps.iter().map(|a| a.name().to_lowercase()).collect();
    assert!(names.windows(2).all(|pair| pair[0] <= pair[1]));
}

#[test]
#[ignore = "touches the live system"]
fn finder_icon_is_a_png() {
    let png = platform()
        .icons
        .app_icon_png("com.apple.finder", 64)
        .unwrap()
        .unwrap();
    println!("finder icon: {} bytes", png.len());
    assert_eq!(png[..8], PNG_SIGNATURE);
    let (width, height) = png_dimensions(&png);
    println!("finder icon: {width}x{height}");
    assert_eq!((width, height), (64, 64));
    assert!(platform()
        .icons
        .app_icon_png("com.example.not-installed", 64)
        .unwrap()
        .is_none());
}

#[test]
#[ignore = "touches the live system"]
fn permission_states_print_without_prompting() {
    let permissions = platform().permissions;
    println!(
        "accessibility: {:?}",
        permissions.state(Permission::Accessibility)
    );
    for browser in Browser::ALL {
        let state = permissions.state(Permission::Automation(browser));
        println!("automation {browser:?}: {state:?}");
    }
}

#[test]
#[ignore = "touches the live system"]
fn active_tabs_print_for_running_browsers() {
    let platform = platform();
    for browser in Browser::ALL {
        let pid = super::control::running_pid(browser.bundle_id());
        let reading = pid.map(|pid| platform.probe.active_tab(browser, pid));
        println!("tab {browser:?} (pid {pid:?}): {reading:?}");
    }
}
