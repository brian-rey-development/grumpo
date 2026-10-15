use std::collections::BTreeSet;

use focus_core::ipc::{
    DaemonStatus, EnforcementReason, EnforcementView, UnlockPhaseView, UnlockView,
};
use focus_core::Rules;

use super::*;
use crate::dto::DaemonUnavailableReason;
use crate::platform::HourCycle;
use crate::time::test_support::{FakeHourCycle, FixedCalendar};

fn clock(cycle: HourCycle) -> ClockFormatter {
    ClockFormatter::new(
        std::sync::Arc::new(FixedCalendar::utc()),
        std::sync::Arc::new(FakeHourCycle::new(cycle)),
    )
}

const SEVEN_PM: i64 = 19 * 3600;

fn connected(active: bool, until: Option<i64>, phase: UnlockPhaseView) -> FocusStatus {
    FocusStatus::Connected {
        status: Box::new(DaemonStatus {
            rules: Rules::default(),
            enforcement: EnforcementView {
                active,
                reason: if active {
                    EnforcementReason::Schedule
                } else {
                    EnforcementReason::None
                },
                active_until: until,
                blocked_sites: BTreeSet::new(),
                blocked_apps: BTreeSet::new(),
            },
            exhausted: Vec::new(),
            unlock: UnlockView {
                phase,
                next_cooldown_minutes: 5,
                sentence: String::new(),
                durations: Vec::new(),
            },
            held_sites: BTreeSet::new(),
        }),
    }
}

#[test]
fn an_unreachable_service_is_named() {
    let down = FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotInstalled,
        message: String::new(),
    };
    assert_eq!(
        status_line(&down, 0, &clock(HourCycle::TwentyFour)),
        "Grumpo service not installed"
    );
    assert!(!is_unlocked(&down));
}

#[test]
fn an_active_focus_names_when_it_ends() {
    let status = connected(true, Some(SEVEN_PM), UnlockPhaseView::Locked);
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::TwentyFour)),
        "Focus on until 19:00"
    );
    assert!(!is_unlocked(&status));
}

#[test]
fn an_endless_focus_has_no_time() {
    let status = connected(true, None, UnlockPhaseView::Locked);
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::TwentyFour)),
        "Focus on"
    );
}

#[test]
fn an_unlock_wins_over_the_schedule() {
    let until = 14 * 3600 + 30 * 60;
    let status = connected(true, Some(SEVEN_PM), UnlockPhaseView::Unlocked { until });
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::TwentyFour)),
        "Unlocked until 14:30"
    );
    assert!(is_unlocked(&status));
}

#[test]
fn no_enforcement_is_free() {
    let status = connected(false, None, UnlockPhaseView::Locked);
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::TwentyFour)),
        "No block active"
    );
}

#[test]
fn a_pending_unlock_does_not_enable_relock() {
    let status = connected(
        false,
        None,
        UnlockPhaseView::CoolingDown {
            ready_at: 100,
            minutes: 5,
        },
    );
    assert!(!is_unlocked(&status));
}

#[test]
fn times_follow_the_system_clock_setting() {
    let status = connected(true, Some(SEVEN_PM), UnlockPhaseView::Locked);
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::Twelve)),
        "Focus on until 7:00 PM"
    );
}

fn cooling_down(ready_at: i64) -> FocusStatus {
    connected(
        true,
        Some(SEVEN_PM),
        UnlockPhaseView::CoolingDown {
            ready_at,
            minutes: 5,
        },
    )
}

#[test]
fn a_cooldown_counts_the_minutes_left_rounded_up() {
    let cases = [
        (300, 300, "Unlock in 5 min"),
        (241, 300, "Unlock in 5 min"),
        (240, 300, "Unlock in 4 min"),
        (61, 300, "Unlock in 2 min"),
        (60, 300, "Unlock in 1 min"),
        (1, 300, "Unlock in 1 min"),
        (0, 300, "Unlock in 1 min"),
    ];
    for (left, ready_at, expected) in cases {
        let status = cooling_down(ready_at);
        assert_eq!(
            status_line(&status, ready_at - left, &clock(HourCycle::TwentyFour)),
            expected,
            "{left}"
        );
    }
}

#[test]
fn a_cooldown_wins_over_the_schedule_and_asks_for_a_redraw() {
    let status = cooling_down(400);
    assert_eq!(
        status_line(&status, 100, &clock(HourCycle::TwentyFour)),
        "Unlock in 5 min"
    );
    assert_eq!(redraw_after(&status), Some(COUNTDOWN_REDRAW));
}

#[test]
fn only_a_cooldown_needs_redrawing_by_itself() {
    let locked = connected(true, Some(SEVEN_PM), UnlockPhaseView::Locked);
    let ready = connected(
        true,
        None,
        UnlockPhaseView::Ready {
            ready_at: 0,
            expires_at: 300,
        },
    );
    let down = FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotResponding,
        message: String::new(),
    };
    assert_eq!(redraw_after(&locked), None);
    assert_eq!(redraw_after(&ready), None);
    assert_eq!(redraw_after(&down), None);
}

#[test]
fn a_ready_unlock_names_when_the_confirmation_window_closes() {
    let expires_at = 14 * 3600 + 35 * 60;
    let status = connected(
        true,
        Some(SEVEN_PM),
        UnlockPhaseView::Ready {
            ready_at: expires_at - 300,
            expires_at,
        },
    );
    assert_eq!(
        status_line(&status, 0, &clock(HourCycle::TwentyFour)),
        "Unlock ready, confirm by 14:35"
    );
    assert!(!is_unlocked(&status));
}

#[test]
fn every_unavailable_reason_has_its_own_line() {
    let line = |reason| {
        status_line(
            &FocusStatus::Unavailable {
                reason,
                message: String::new(),
            },
            0,
            &clock(HourCycle::TwentyFour),
        )
    };
    assert_eq!(
        line(DaemonUnavailableReason::NotInstalled),
        "Grumpo service not installed"
    );
    assert_eq!(
        line(DaemonUnavailableReason::NotResponding),
        "Grumpo service not responding"
    );
    assert_eq!(
        line(DaemonUnavailableReason::ServiceOutdated),
        "Grumpo service needs updating"
    );
    assert_eq!(
        line(DaemonUnavailableReason::AppOutdated),
        "Grumpo needs updating"
    );
    assert_eq!(
        line(DaemonUnavailableReason::OtherUser),
        "Grumpo service belongs to another user"
    );
}

#[test]
fn the_quit_item_warns_only_while_blocking_is_active() {
    let blocking = connected(true, None, UnlockPhaseView::Locked);
    let free = connected(false, None, UnlockPhaseView::Locked);
    let down = FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotInstalled,
        message: String::new(),
    };
    assert_eq!(
        quit_label(&blocking),
        "Quit Grumpo (stops app and tab blocking)"
    );
    assert_eq!(quit_label(&free), "Quit Grumpo");
    assert_eq!(quit_label(&down), "Quit Grumpo");
}
