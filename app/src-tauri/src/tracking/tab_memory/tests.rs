use focus_core::Domain;

use super::*;

const PID: i32 = 100;

fn reading(host: &str) -> TabReading {
    TabReading {
        domain: Domain::parse(host).ok(),
        private: false,
    }
}

#[test]
fn nothing_known_and_disengaged_reports_no_tab() {
    let memory = TabMemory::default();
    assert_eq!(
        memory.decide(Browser::Chrome, PID, false, 0),
        Decision::Reuse(TabReading::default())
    );
    assert_eq!(memory.decide(Browser::Chrome, PID, true, 0), Decision::Read);
}

#[test]
fn the_same_browser_rereads_only_after_the_interval() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 100, Some(reading("a.com")));
    let carried = Decision::Reuse(reading("a.com"));
    assert_eq!(memory.decide(Browser::Chrome, PID, true, 103), carried);
    assert_eq!(
        memory.decide(Browser::Chrome, PID, true, 104),
        Decision::Read
    );
}

#[test]
fn a_new_browser_or_pid_reads_immediately() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 100, Some(reading("a.com")));
    assert_eq!(
        memory.decide(Browser::Safari, PID, true, 101),
        Decision::Read
    );
    assert_eq!(
        memory.decide(Browser::Chrome, PID + 1, true, 101),
        Decision::Read
    );
}

#[test]
fn disengaged_samples_reuse_the_last_known_tab_however_old() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 100, Some(reading("a.com")));
    assert_eq!(
        memory.decide(Browser::Chrome, PID, false, 10_000),
        Decision::Reuse(reading("a.com"))
    );
    assert_eq!(
        memory.decide(Browser::Safari, PID, false, 101),
        Decision::Reuse(TabReading::default())
    );
}

#[test]
fn a_clock_that_moved_back_forces_a_read() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 100, Some(reading("a.com")));
    assert_eq!(
        memory.decide(Browser::Chrome, PID, true, 90),
        Decision::Read
    );
}

#[test]
fn failures_carry_the_previous_tab_up_to_the_limit() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 0, Some(reading("a.com")));
    for attempt in 1..=MAX_CARRIED_FAILURES {
        let now = i64::from(attempt) * 4;
        assert_eq!(
            memory.record(Browser::Chrome, PID, now, None),
            reading("a.com"),
            "failure {attempt}"
        );
    }
    assert_eq!(
        memory.record(Browser::Chrome, PID, 20, None),
        TabReading::default()
    );
}

#[test]
fn a_success_resets_the_failure_count() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 0, Some(reading("a.com")));
    memory.record(Browser::Chrome, PID, 4, None);
    memory.record(Browser::Chrome, PID, 8, Some(reading("b.com")));
    for now in [12, 16, 20] {
        assert_eq!(
            memory.record(Browser::Chrome, PID, now, None),
            reading("b.com")
        );
    }
}

#[test]
fn a_failure_for_a_different_browser_carries_nothing() {
    let mut memory = TabMemory::default();
    memory.record(Browser::Chrome, PID, 0, Some(reading("a.com")));
    assert_eq!(
        memory.record(Browser::Safari, PID + 5, 1, None),
        TabReading::default()
    );
}
