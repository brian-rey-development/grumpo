use super::*;
use crate::daemon_client::test_support::daemon_status;

const INTERVAL: Duration = Duration::from_secs(5);

fn in_phase(phase: UnlockPhaseView) -> FocusStatus {
    let mut status = daemon_status();
    status.unlock.phase = phase;
    FocusStatus::Connected {
        status: Box::new(status),
    }
}

#[test]
fn a_locked_or_unreachable_service_is_polled_on_the_regular_interval() {
    let down = FocusStatus::Unavailable {
        reason: crate::dto::DaemonUnavailableReason::NotResponding,
        message: String::new(),
    };
    assert_eq!(next_wake(&down, 100, INTERVAL), INTERVAL);
    assert_eq!(
        next_wake(&in_phase(UnlockPhaseView::Locked), 100, INTERVAL),
        INTERVAL
    );
}

#[test]
fn a_near_deadline_cuts_the_wait_short() {
    let cooling = in_phase(UnlockPhaseView::CoolingDown {
        ready_at: 102,
        minutes: 5,
    });
    assert_eq!(
        next_wake(&cooling, 100, INTERVAL),
        Duration::from_secs(2) + DEADLINE_SLACK
    );
}

#[test]
fn every_phase_wakes_at_its_own_deadline() {
    let ready = in_phase(UnlockPhaseView::Ready {
        ready_at: 90,
        expires_at: 101,
    });
    let unlocked = in_phase(UnlockPhaseView::Unlocked { until: 103 });
    assert_eq!(
        next_wake(&ready, 100, INTERVAL),
        Duration::from_secs(1) + DEADLINE_SLACK
    );
    assert_eq!(
        next_wake(&unlocked, 100, INTERVAL),
        Duration::from_secs(3) + DEADLINE_SLACK
    );
}

#[test]
fn a_far_deadline_never_stretches_the_wait_past_the_interval() {
    let unlocked = in_phase(UnlockPhaseView::Unlocked { until: 10_000 });
    assert_eq!(next_wake(&unlocked, 100, INTERVAL), INTERVAL);
}

#[test]
fn a_deadline_already_passed_waits_the_minimum_instead_of_spinning() {
    let cooling = in_phase(UnlockPhaseView::CoolingDown {
        ready_at: 50,
        minutes: 5,
    });
    assert_eq!(next_wake(&cooling, 100, INTERVAL), MIN_WAKE);
}
