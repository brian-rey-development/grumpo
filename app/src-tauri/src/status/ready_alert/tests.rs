use super::*;
use crate::daemon_client::test_support::daemon_status;
use crate::dto::DaemonUnavailableReason;
use crate::notify::test_support::RecordingNotifier;
use crate::platform::HourCycle;
use crate::time::test_support::{FakeHourCycle, FixedCalendar};

const COOLING: UnlockPhaseView = UnlockPhaseView::CoolingDown {
    ready_at: 100,
    minutes: 5,
};
const READY: UnlockPhaseView = UnlockPhaseView::Ready {
    ready_at: 100,
    expires_at: 400,
};

fn in_phase(phase: UnlockPhaseView) -> FocusStatus {
    let mut status = daemon_status();
    status.unlock.phase = phase;
    FocusStatus::Connected {
        status: Box::new(status),
    }
}

fn down() -> FocusStatus {
    FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotResponding,
        message: String::new(),
    }
}

fn alert() -> (UnlockReadyAlert, Arc<RecordingNotifier>) {
    let notifier = Arc::new(RecordingNotifier::default());
    (
        UnlockReadyAlert::new(
            notifier.clone(),
            ClockFormatter::new(
                Arc::new(FixedCalendar::utc()),
                Arc::new(FakeHourCycle::new(HourCycle::TwentyFour)),
            ),
        ),
        notifier,
    )
}

#[test]
fn the_end_of_a_cooldown_posts_one_notice() {
    let (alert, notifier) = alert();

    alert.observe(&in_phase(COOLING));
    alert.observe(&in_phase(COOLING));
    alert.observe(&in_phase(READY));
    alert.observe(&in_phase(READY));

    assert_eq!(
        notifier.notices(),
        [Notice {
            title: "Unlock ready".to_owned(),
            body: "Confirm by 00:06 in Grumpo.".to_owned(),
        }]
    );
}

#[test]
fn a_ready_state_the_app_starts_with_is_not_announced() {
    let (alert, notifier) = alert();

    alert.observe(&in_phase(READY));

    assert_eq!(notifier.notices().len(), 0);
}

#[test]
fn a_cooldown_that_ends_while_the_service_is_out_of_reach_still_alerts() {
    let (alert, notifier) = alert();

    alert.observe(&in_phase(COOLING));
    alert.observe(&down());
    alert.observe(&in_phase(READY));

    assert_eq!(notifier.notices().len(), 1);
}

#[test]
fn other_transitions_stay_quiet() {
    let (alert, notifier) = alert();

    alert.observe(&in_phase(UnlockPhaseView::Locked));
    alert.observe(&in_phase(COOLING));
    alert.observe(&in_phase(UnlockPhaseView::Locked));
    alert.observe(&in_phase(READY));
    alert.observe(&in_phase(UnlockPhaseView::Unlocked { until: 900 }));

    assert_eq!(notifier.notices().len(), 0);
}
