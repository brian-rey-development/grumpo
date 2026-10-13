use super::*;

const FRONT: i32 = 500;

fn assertion(holder: i32, kind: &str, level: i32, on_behalf_of: Option<i32>) -> Assertion {
    Assertion {
        holder_pid: holder,
        kind: kind.to_owned(),
        level,
        on_behalf_of_pid: on_behalf_of,
    }
}

fn display(holder: i32) -> Assertion {
    assertion(holder, "PreventUserIdleDisplaySleep", 255, None)
}

fn check(assertions: &[Assertion], helpers: &[i32]) -> bool {
    frontmost_holds_display_assertion(assertions, FRONT, |pid, app| {
        app == FRONT && helpers.contains(&pid)
    })
}

#[test]
fn the_frontmost_app_holding_the_assertion_counts() {
    assert!(check(&[display(FRONT)], &[]));
}

#[test]
fn the_legacy_no_display_sleep_type_counts() {
    assert!(check(
        &[assertion(FRONT, "NoDisplaySleepAssertion", 255, None)],
        &[]
    ));
}

#[test]
fn media_in_another_app_does_not_count() {
    assert!(!check(&[display(777)], &[]));
    assert!(!check(&[], &[]));
}

#[test]
fn an_assertion_made_on_behalf_of_the_frontmost_app_counts() {
    let held_for_us = assertion(40, "PreventUserIdleDisplaySleep", 255, Some(FRONT));
    assert!(check(&[held_for_us], &[]));
    let held_for_other = assertion(40, "PreventUserIdleDisplaySleep", 255, Some(777));
    assert!(!check(&[held_for_other], &[]));
}

#[test]
fn a_helper_of_the_frontmost_app_counts() {
    assert!(check(&[display(900)], &[900]));
    assert!(!check(&[display(901)], &[900]));
}

#[test]
fn a_helper_named_in_on_behalf_of_counts() {
    let delegated = assertion(40, "PreventUserIdleDisplaySleep", 255, Some(900));
    assert!(check(&[delegated], &[900]));
}

#[test]
fn released_and_unrelated_assertion_types_are_ignored() {
    let off = assertion(FRONT, "PreventUserIdleDisplaySleep", 0, None);
    let system_sleep = assertion(FRONT, "PreventUserIdleSystemSleep", 255, None);
    assert!(!check(&[off, system_sleep], &[]));
}

#[test]
fn the_helper_lookup_is_skipped_for_assertions_that_cannot_count() {
    let system_sleep = assertion(900, "PreventUserIdleSystemSleep", 255, None);
    let lookups = std::cell::Cell::new(0);
    let counted = frontmost_holds_display_assertion(&[system_sleep], FRONT, |_, _| {
        lookups.set(lookups.get() + 1);
        true
    });
    assert!(!counted);
    assert_eq!(lookups.get(), 0);
}
