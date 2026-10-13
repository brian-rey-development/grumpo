use chrono::{DateTime, Duration, NaiveDateTime, Utc};

use super::*;
use crate::test_support::{self, local, now_from};

fn start() -> NaiveDateTime {
    local(2026, 10, 12, 9, 0)
}

fn after(minutes: i64) -> Now {
    test_support::after(&now_from(start()), minutes)
}

fn after_seconds(minutes: i64, seconds: i64) -> Now {
    now_from(start() + Duration::minutes(minutes) + Duration::seconds(seconds))
}

fn at_utc(minutes: i64) -> DateTime<Utc> {
    after(minutes).utc()
}

fn locked() -> UnlockState {
    UnlockState::new(&after(0))
}

fn ready_at_minute(minutes: i64) -> UnlockState {
    let requested = locked().request(&after(0)).unwrap();
    requested.tick(&after(minutes))
}

fn unlocked_for(minutes: u16) -> UnlockState {
    let ready = ready_at_minute(5);
    ready.confirm(UNLOCK_SENTENCE, minutes, &after(5)).unwrap()
}

#[test]
fn new_state_is_locked_with_nothing_used() {
    let state = locked();
    assert_eq!(state.phase, UnlockPhase::Locked);
    assert_eq!(state.unlocks_today, 0);
    assert_eq!(state.day, start().date());
    assert_eq!(state.next_cooldown_minutes(), 5);
}

#[test]
fn the_phase_getter_exposes_the_state() {
    let state = locked().request(&after(0)).unwrap();
    assert_eq!(
        state.phase(),
        UnlockPhase::CoolingDown {
            ready_at: at_utc(5),
            minutes: 5
        }
    );
}

#[test]
fn request_starts_the_first_cooldown() {
    let state = locked().request(&after(0)).unwrap();
    assert_eq!(
        state.phase,
        UnlockPhase::CoolingDown {
            ready_at: at_utc(5),
            minutes: 5
        }
    );
    assert_eq!(state.unlocks_today, 0);
}

#[test]
fn request_is_rejected_while_pending_or_unlocked() {
    let cooling = locked().request(&after(0)).unwrap();
    assert_eq!(cooling.request(&after(1)), Err(UnlockError::AlreadyPending));
    assert_eq!(
        ready_at_minute(5).request(&after(5)),
        Err(UnlockError::AlreadyPending)
    );
    assert_eq!(
        unlocked_for(15).request(&after(6)),
        Err(UnlockError::AlreadyUnlocked)
    );
}

#[test]
fn request_after_an_expired_request_starts_over() {
    let cooling = locked().request(&after(0)).unwrap();
    let state = cooling.request(&after(10)).unwrap();
    assert_eq!(
        state.phase,
        UnlockPhase::CoolingDown {
            ready_at: at_utc(15),
            minutes: 5
        }
    );
}

#[test]
fn tick_walks_cooling_ready_and_lapsed() {
    let cooling = locked().request(&after(0)).unwrap();
    let cases = [
        (
            4,
            UnlockPhase::CoolingDown {
                ready_at: at_utc(5),
                minutes: 5,
            },
        ),
        (
            5,
            UnlockPhase::Ready {
                ready_at: at_utc(5),
                expires_at: at_utc(10),
            },
        ),
        (
            9,
            UnlockPhase::Ready {
                ready_at: at_utc(5),
                expires_at: at_utc(10),
            },
        ),
        (10, UnlockPhase::Locked),
        (600, UnlockPhase::Locked),
    ];
    for (minute, expected) in cases {
        assert_eq!(
            cooling.tick(&after(minute)).phase,
            expected,
            "minute {minute}"
        );
    }
}

#[test]
fn tick_expires_an_unlock_exactly_at_its_end() {
    let state = unlocked_for(15);
    assert_eq!(
        state.tick(&after(19)).phase,
        UnlockPhase::Unlocked {
            until: at_utc(20),
            minutes: 15
        }
    );
    assert_eq!(state.tick(&after(20)).phase, UnlockPhase::Locked);
}

#[test]
fn tick_is_idempotent() {
    let state = locked().request(&after(0)).unwrap();
    let once = state.tick(&after(7));
    assert_eq!(once.tick(&after(7)), once);
}

#[test]
fn tick_resets_the_daily_count_on_a_new_local_day() {
    let busy = UnlockState {
        unlocks_today: 3,
        ..locked()
    };
    let same_day = busy.tick(&after(60));
    assert_eq!(same_day.unlocks_today, 3);

    let next_day = busy.tick(&now_from(local(2026, 10, 13, 0, 0)));
    assert_eq!(next_day.unlocks_today, 0);
    assert_eq!(next_day.day, local(2026, 10, 13, 0, 0).date());
    assert_eq!(next_day.next_cooldown_minutes(), 5);
}

#[test]
fn rollover_does_not_end_a_running_unlock() {
    let late = now_from(local(2026, 10, 12, 23, 50));
    let ready = UnlockState {
        phase: UnlockPhase::Ready {
            ready_at: late.utc(),
            expires_at: late.utc() + Duration::minutes(5),
        },
        ..UnlockState::new(&late)
    };
    let unlocked = ready.confirm(UNLOCK_SENTENCE, 60, &late).unwrap();
    let after_midnight = now_from(local(2026, 10, 13, 0, 10));
    let ticked = unlocked.tick(&after_midnight);
    assert!(matches!(ticked.phase, UnlockPhase::Unlocked { .. }));
    assert_eq!(ticked.unlocks_today, 0);
    assert!(ticked.is_unlocked(&after_midnight));
}

fn used_once(state: &UnlockState, minute: i64) -> UnlockState {
    let ready_minute = minute + i64::from(state.next_cooldown_minutes());
    let requested = state.request(&after(minute)).unwrap();
    let confirmed = requested
        .confirm(UNLOCK_SENTENCE, 15, &after(ready_minute))
        .unwrap();
    confirmed.relock(&after(ready_minute))
}

#[test]
fn cooldown_escalates_with_each_unlock_and_caps_at_forty() {
    let mut state = locked();
    for expected in [5, 10, 20, 40, 40, 40] {
        assert_eq!(state.next_cooldown_minutes(), expected);
        let requested = state.request(&after(0)).unwrap();
        assert_eq!(
            requested.phase,
            UnlockPhase::CoolingDown {
                ready_at: at_utc(i64::from(expected)),
                minutes: expected
            }
        );
        state = used_once(&state, 0);
    }
    assert_eq!(state.unlocks_today, 6);
}

#[test]
fn escalation_survives_until_local_midnight_then_resets() {
    let state = used_once(&used_once(&locked(), 0), 100);
    assert_eq!(state.next_cooldown_minutes(), 20);
    let tomorrow = state.tick(&now_from(local(2026, 10, 13, 0, 0)));
    assert_eq!(tomorrow.next_cooldown_minutes(), 5);
}

#[test]
fn a_cooldown_keeps_its_length_when_the_daily_count_resets_underneath_it() {
    let used = used_once(&used_once(&locked(), 0), 100);
    let late = now_from(local(2026, 10, 12, 23, 50));
    let cooling = used.request(&late).unwrap();
    let after_midnight = cooling.tick(&now_from(local(2026, 10, 13, 0, 5)));
    assert!(matches!(
        after_midnight.phase,
        UnlockPhase::CoolingDown { minutes: 20, .. }
    ));
    assert_eq!(after_midnight.unlocks_today, 0);
    assert_eq!(after_midnight.next_cooldown_minutes(), 5);
}

#[test]
fn cooldown_minutes_after_follows_the_steps_and_caps() {
    let minutes: Vec<u16> = (0..=6).map(cooldown_minutes_after).collect();
    assert_eq!(minutes, [5, 10, 20, 40, 40, 40, 40]);
    assert_eq!(cooldown_minutes_after(u8::MAX), 40);
}

#[test]
fn confirm_unlocks_for_each_allowed_duration() {
    for minutes in UNLOCK_DURATIONS_MINUTES {
        let ready = ready_at_minute(5);
        let state = ready.confirm(UNLOCK_SENTENCE, minutes, &after(6)).unwrap();
        assert_eq!(
            state.phase,
            UnlockPhase::Unlocked {
                until: at_utc(6 + i64::from(minutes)),
                minutes
            }
        );
        assert_eq!(state.unlocks_today, 1);
        assert!(state.is_unlocked(&after(6)));
    }
}

#[test]
fn confirm_requires_the_right_phase() {
    let cooling = locked().request(&after(0)).unwrap();
    let lapsed = locked().request(&after(0)).unwrap();
    let cases = [
        (locked(), after(0), UnlockError::NotRequested),
        (cooling, after(4), UnlockError::NotReady),
        (lapsed, after(10), UnlockError::NotRequested),
        (unlocked_for(15), after(6), UnlockError::AlreadyUnlocked),
    ];
    for (state, now, expected) in cases {
        assert_eq!(state.confirm(UNLOCK_SENTENCE, 15, &now), Err(expected));
    }
}

#[test]
fn confirm_accepts_whitespace_variations_of_the_sentence() {
    let spaced = UNLOCK_SENTENCE.replace(' ', "  ");
    let phrases = [
        UNLOCK_SENTENCE.to_owned(),
        format!("  {UNLOCK_SENTENCE}  "),
        format!("\n{UNLOCK_SENTENCE}\n"),
        spaced,
        UNLOCK_SENTENCE.replace(' ', "\t"),
    ];
    for phrase in phrases {
        let result = ready_at_minute(5).confirm(&phrase, 15, &after(5));
        assert!(result.is_ok(), "phrase {phrase:?}");
    }
}

#[test]
fn confirm_rejects_any_other_text() {
    let phrases = [
        String::new(),
        "   ".to_owned(),
        UNLOCK_SENTENCE.to_lowercase(),
        UNLOCK_SENTENCE.to_uppercase(),
        UNLOCK_SENTENCE.trim_end_matches('.').to_owned(),
        format!("{UNLOCK_SENTENCE} extra"),
        UNLOCK_SENTENCE.replace("focus", "work"),
    ];
    for phrase in phrases {
        let result = ready_at_minute(5).confirm(&phrase, 15, &after(5));
        assert_eq!(
            result,
            Err(UnlockError::PhraseMismatch),
            "phrase {phrase:?}"
        );
    }
}

#[test]
fn confirm_rejects_durations_outside_the_menu() {
    for minutes in [0, 1, 14, 16, 45, 61, 120, u16::MAX] {
        let result = ready_at_minute(5).confirm(UNLOCK_SENTENCE, minutes, &after(5));
        assert_eq!(
            result,
            Err(UnlockError::DurationInvalid),
            "minutes {minutes}"
        );
    }
}

#[test]
fn a_rejected_confirm_leaves_the_request_usable() {
    let ready = ready_at_minute(5);
    assert!(ready.confirm("wrong", 15, &after(6)).is_err());
    assert!(ready.confirm(UNLOCK_SENTENCE, 15, &after(6)).is_ok());
}

#[test]
fn the_daily_count_saturates() {
    let maxed = UnlockState {
        unlocks_today: u8::MAX,
        ..locked()
    };
    assert_eq!(maxed.next_cooldown_minutes(), 40);
    let requested = maxed.request(&after(0)).unwrap();
    let ready = requested.tick(&after(40));
    let state = ready.confirm(UNLOCK_SENTENCE, 15, &after(40)).unwrap();
    assert_eq!(state.unlocks_today, u8::MAX);
}

#[test]
fn cancel_returns_pending_requests_to_locked_without_escalating() {
    let cooling = locked().request(&after(0)).unwrap();
    for (state, minute) in [(cooling, 3), (ready_at_minute(5), 6)] {
        let cancelled = state.cancel(&after(minute)).unwrap();
        assert_eq!(cancelled.phase, UnlockPhase::Locked);
        assert_eq!(cancelled.unlocks_today, 0);
    }
}

#[test]
fn cancel_fails_when_nothing_is_pending() {
    assert_eq!(locked().cancel(&after(0)), Err(UnlockError::NotRequested));
    let lapsed = locked().request(&after(0)).unwrap();
    assert_eq!(lapsed.cancel(&after(30)), Err(UnlockError::NotRequested));
    assert_eq!(
        unlocked_for(15).cancel(&after(6)),
        Err(UnlockError::AlreadyUnlocked)
    );
}

#[test]
fn relock_ends_every_phase_and_keeps_the_count() {
    let states = [
        locked(),
        locked().request(&after(0)).unwrap(),
        ready_at_minute(5),
        unlocked_for(60),
    ];
    for state in states {
        let relocked = state.relock(&after(6));
        assert_eq!(relocked.phase, UnlockPhase::Locked);
        assert_eq!(relocked.unlocks_today, state.unlocks_today);
        assert!(!relocked.is_unlocked(&after(6)));
    }
}

#[test]
fn is_unlocked_is_exclusive_of_the_end_and_needs_no_tick() {
    let state = unlocked_for(15);
    assert!(state.is_unlocked(&after(19)));
    assert!(!state.is_unlocked(&after(20)));
    assert!(!locked().is_unlocked(&after(0)));
    assert!(!ready_at_minute(5).is_unlocked(&after(5)));
}

#[test]
fn state_round_trips_through_json_with_a_tagged_phase() {
    let state = locked().request(&after(0)).unwrap();
    let json = serde_json::to_value(state).unwrap();
    assert_eq!(json["phase"]["kind"], "coolingDown");
    assert!(json["phase"]["readyAt"].is_string());
    assert_eq!(json["phase"]["minutes"], 5);
    assert_eq!(json["unlocksToday"], 0);
    assert_eq!(json["day"], "2026-10-12");
    let restored: UnlockState = serde_json::from_value(json).unwrap();
    assert_eq!(restored, state);
}

#[test]
fn error_messages_are_readable() {
    assert_eq!(
        UnlockError::DurationInvalid.to_string(),
        "the unlock duration must be one of [15, 30, 60] minutes"
    );
    assert_eq!(
        UnlockError::NotReady.to_string(),
        "the cooldown has not finished yet"
    );
}

#[test]
fn a_clock_set_back_relocks_an_unlock_that_would_outlast_its_grant() {
    let state = unlocked_for(15);
    assert!(state.is_unlocked(&after(6)));
    assert!(!state.is_unlocked(&after(3)));
    assert_eq!(state.tick(&after(3)).phase, UnlockPhase::Locked);
    assert_eq!(state.tick(&after(-600)).phase, UnlockPhase::Locked);
}

#[test]
fn an_unlock_keeps_running_while_the_clock_stays_within_its_grant() {
    let state = unlocked_for(15);
    assert!(state.is_unlocked(&after(5)));
    assert_eq!(state.tick(&after(5)).phase, state.phase);
}

#[test]
fn a_relocked_unlock_still_counts_toward_escalation() {
    let state = unlocked_for(15);
    let tampered = state.tick(&after(-60));
    assert_eq!(tampered.unlocks_today, 1);
    assert_eq!(tampered.next_cooldown_minutes(), 10);
}

#[test]
fn a_clock_set_back_restarts_a_cooldown_that_would_outlast_its_step() {
    let requested = locked().request(&after(0)).unwrap();
    let set_back = requested.tick(&after(-30));
    assert_eq!(
        set_back.phase,
        UnlockPhase::CoolingDown {
            ready_at: at_utc(-25),
            minutes: 5
        }
    );
    assert!(matches!(
        set_back.tick(&after(-25)).phase,
        UnlockPhase::Ready { .. }
    ));
}

#[test]
fn a_cooldown_within_its_step_is_left_alone() {
    let requested = locked().request(&after(0)).unwrap();
    assert_eq!(requested.tick(&after(0)).phase, requested.phase);
    assert_eq!(requested.tick(&after(3)).phase, requested.phase);
}

#[test]
fn a_clock_set_back_lapses_a_ready_request() {
    let ready = ready_at_minute(5);
    assert!(matches!(ready.phase, UnlockPhase::Ready { .. }));
    assert_eq!(ready.tick(&after(5)).phase, ready.phase);
    assert_eq!(ready.tick(&after(0)).phase, UnlockPhase::Locked);
}

const SMALL_STEP_BACK_SECONDS: i64 = -2;
const SLACK_EDGE_SECONDS: i64 = -60;
const LARGE_STEP_BACK_SECONDS: i64 = -61;

#[test]
fn a_small_step_back_keeps_a_cooldown_as_it_is() {
    let requested = locked().request(&after(0)).unwrap();
    for seconds in [SMALL_STEP_BACK_SECONDS, SLACK_EDGE_SECONDS] {
        assert_eq!(
            requested.tick(&after_seconds(0, seconds)).phase,
            requested.phase,
            "{seconds} s"
        );
    }
}

#[test]
fn a_large_step_back_restarts_a_cooldown() {
    let requested = locked().request(&after(0)).unwrap();
    let at = after_seconds(0, LARGE_STEP_BACK_SECONDS);
    assert_eq!(
        requested.tick(&at).phase,
        UnlockPhase::CoolingDown {
            ready_at: at.utc() + Duration::minutes(5),
            minutes: 5
        }
    );
}

#[test]
fn a_small_step_back_keeps_a_ready_request() {
    let ready = ready_at_minute(5);
    for seconds in [SMALL_STEP_BACK_SECONDS, SLACK_EDGE_SECONDS] {
        let at = after_seconds(5, seconds);
        assert_eq!(ready.tick(&at).phase, ready.phase, "{seconds} s");
    }
}

#[test]
fn a_large_step_back_lapses_a_ready_request() {
    let ready = ready_at_minute(5);
    let at = after_seconds(0, LARGE_STEP_BACK_SECONDS);
    assert_eq!(ready.tick(&at).phase, UnlockPhase::Locked);
}

#[test]
fn a_small_step_back_keeps_an_unlock() {
    let state = unlocked_for(15);
    for seconds in [SMALL_STEP_BACK_SECONDS, SLACK_EDGE_SECONDS] {
        let at = after_seconds(5, seconds);
        assert_eq!(state.tick(&at).phase, state.phase, "{seconds} s");
        assert!(state.is_unlocked(&at));
    }
}

#[test]
fn a_large_step_back_relocks_an_unlock() {
    let state = unlocked_for(15);
    let at = after_seconds(5, LARGE_STEP_BACK_SECONDS);
    assert_eq!(state.tick(&at).phase, UnlockPhase::Locked);
}

#[test]
fn confirming_after_the_clock_was_set_back_fails() {
    let ready = ready_at_minute(5);
    assert_eq!(
        ready.confirm(UNLOCK_SENTENCE, 15, &after(-60)),
        Err(UnlockError::NotRequested)
    );
}

#[test]
fn the_daily_count_resets_only_when_the_date_moves_forward() {
    let used = UnlockState {
        phase: UnlockPhase::Locked,
        unlocks_today: 3,
        day: start().date(),
    };
    let yesterday = used.tick(&after(-24 * 60));
    assert_eq!(yesterday.unlocks_today, 3);
    assert_eq!(yesterday.day, start().date());
    let week_ago = used.tick(&after(-7 * 24 * 60));
    assert_eq!(week_ago.unlocks_today, 3);
    let tomorrow = used.tick(&after(24 * 60));
    assert_eq!(tomorrow.unlocks_today, 0);
    assert_eq!(tomorrow.day, after(24 * 60).local().date());
}

#[test]
fn going_back_a_day_and_forward_again_does_not_reset_the_count() {
    let used = UnlockState {
        phase: UnlockPhase::Locked,
        unlocks_today: 3,
        day: start().date(),
    };
    let tampered = used.tick(&after(-24 * 60)).tick(&after(0));
    assert_eq!(tampered.unlocks_today, 3);
}

#[test]
fn a_request_is_possible_whatever_the_time_of_day() {
    for hour in [0, 7, 12, 19, 23] {
        let now = test_support::now_at(2026, 10, 12, hour, 30);
        assert!(UnlockState::new(&now).request(&now).is_ok(), "hour {hour}");
    }
}
