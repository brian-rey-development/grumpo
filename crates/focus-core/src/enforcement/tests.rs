use chrono::TimeZone;

use super::*;
use crate::test_support::{
    after, app, app_target, exhausted_by, hm, limit, local, now_at, now_from, rules_with, schedule,
    site, site_target, valid, window,
};
use crate::unlock::UNLOCK_SENTENCE;
use crate::{Day, Loosening, Rules, Schedule};

fn monday_nine_to_five() -> Schedule {
    schedule(vec![window(&[Day::Mon], hm(9, 0), hm(17, 0))])
}

fn plan(rules: &ValidatedRules, spent: &[LimitTarget], now: &Now) -> EnforcementPlan {
    let record = exhausted_by(rules, spent, now);
    EnforcementPlan::compute(rules, &UnlockState::new(now), &record, now)
}

fn utc(day: u32, hour: u32, minute: u32) -> Option<DateTime<Utc>> {
    Utc.with_ymd_and_hms(2026, 10, day, hour, minute, 0)
        .single()
}

fn until_of(plan: &EnforcementPlan) -> Option<DateTime<Utc>> {
    match plan {
        EnforcementPlan::Active { until, .. } => *until,
        EnforcementPlan::Inactive => panic!("expected an active plan, got {plan:?}"),
    }
}

fn reason_of(plan: &EnforcementPlan) -> ActiveReason {
    match plan {
        EnforcementPlan::Active { reason, .. } => *reason,
        EnforcementPlan::Inactive => panic!("expected an active plan, got {plan:?}"),
    }
}

fn targets_of(plan: &EnforcementPlan) -> (BTreeSet<Domain>, BTreeSet<AppRef>) {
    match plan {
        EnforcementPlan::Active { sites, apps, .. } => (sites.clone(), apps.clone()),
        EnforcementPlan::Inactive => (BTreeSet::new(), BTreeSet::new()),
    }
}

fn unlocked_at(start: &Now, minutes: u16) -> UnlockState {
    let ready = after(start, 5);
    UnlockState::new(start)
        .request(start)
        .unwrap()
        .confirm(UNLOCK_SENTENCE, minutes, &ready)
        .unwrap()
}

#[test]
fn nothing_scheduled_and_nothing_exhausted_is_inactive() {
    let now = now_at(2026, 10, 12, 10, 0);
    let result = plan(&rules_with(Schedule::default()), &[], &now);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn an_active_window_blocks_every_listed_site_and_app() {
    let now = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(monday_nine_to_five());
    let result = plan(&rules, &[], &now);
    assert_eq!(
        result,
        EnforcementPlan::Active {
            reason: ActiveReason::Schedule,
            until: utc(12, 17, 0),
            sites: rules.blocked_sites.clone(),
            apps: rules.blocked_apps.clone(),
        }
    );
}

#[test]
fn outside_the_window_the_schedule_does_not_block() {
    let now = now_at(2026, 10, 12, 17, 0);
    let result = plan(&rules_with(monday_nine_to_five()), &[], &now);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn an_exhausted_limit_blocks_only_its_target_until_midnight() {
    let now = now_at(2026, 10, 12, 20, 0);
    let result = plan(
        &rules_with(monday_nine_to_five()),
        &[site_target("x.com")],
        &now,
    );
    assert_eq!(
        result,
        EnforcementPlan::Active {
            reason: ActiveReason::Limit,
            until: utc(13, 0, 0),
            sites: [site("x.com")].into(),
            apps: BTreeSet::new(),
        }
    );
}

#[test]
fn an_exhausted_app_limit_blocks_the_app() {
    let now = now_at(2026, 10, 12, 20, 0);
    let result = plan(
        &rules_with(Schedule::default()),
        &[app_target("com.discord")],
        &now,
    );
    assert_eq!(
        targets_of(&result),
        (BTreeSet::new(), [app("com.discord")].into())
    );
}

#[test]
fn schedule_and_limits_together_union_their_targets_and_last_until_the_later_end() {
    let now = now_at(2026, 10, 12, 10, 0);
    let spent = [site_target("x.com"), app_target("com.discord")];
    let result = plan(&rules_with(monday_nine_to_five()), &spent, &now);
    assert_eq!(reason_of(&result), ActiveReason::Both);
    assert_eq!(until_of(&result), utc(13, 0, 0));
    let (sites, apps) = targets_of(&result);
    assert_eq!(
        sites,
        [site("youtube.com"), site("reddit.com"), site("x.com")].into()
    );
    assert_eq!(
        apps,
        [app("com.valvesoftware.steam"), app("com.discord")].into()
    );
}

#[test]
fn a_schedule_running_past_midnight_outlasts_the_limit() {
    let now = now_at(2026, 10, 12, 22, 0);
    let rules = rules_with(schedule(vec![
        window(&[Day::Mon], hm(21, 0), hm(24, 0)),
        window(&[Day::Tue], 0, hm(7, 0)),
    ]));
    let result = plan(&rules, &[site_target("x.com")], &now);
    assert_eq!(reason_of(&result), ActiveReason::Both);
    assert_eq!(until_of(&result), utc(13, 7, 0));
}

#[test]
fn a_limit_before_a_window_that_spans_midnight_runs_to_the_window_end() {
    let now = now_at(2026, 10, 12, 20, 0);
    let rules = rules_with(schedule(vec![
        window(&[Day::Mon], hm(23, 0), hm(24, 0)),
        window(&[Day::Tue], 0, hm(2, 0)),
    ]));
    let result = plan(&rules, &[site_target("x.com")], &now);
    assert_eq!(reason_of(&result), ActiveReason::Limit);
    assert_eq!(until_of(&result), utc(13, 2, 0));
    assert_eq!(targets_of(&result).0, [site("x.com")].into());
}

#[test]
fn an_active_window_ending_before_midnight_chains_into_a_later_window_through_the_limit() {
    let now = now_at(2026, 10, 12, 20, 0);
    let rules = rules_with(schedule(vec![
        window(&[Day::Mon], hm(18, 0), hm(22, 0)),
        window(&[Day::Mon], hm(23, 0), hm(24, 0)),
        window(&[Day::Tue], 0, hm(1, 0)),
    ]));
    let result = plan(&rules, &[site_target("x.com")], &now);
    assert_eq!(reason_of(&result), ActiveReason::Both);
    assert_eq!(until_of(&result), utc(13, 1, 0));
}

#[test]
fn without_limits_a_window_ending_before_midnight_does_not_chain() {
    let now = now_at(2026, 10, 12, 20, 0);
    let rules = rules_with(schedule(vec![
        window(&[Day::Mon], hm(18, 0), hm(22, 0)),
        window(&[Day::Mon], hm(23, 0), hm(24, 0)),
    ]));
    assert_eq!(until_of(&plan(&rules, &[], &now)), utc(12, 22, 0));
}

#[test]
fn an_always_on_schedule_has_no_end() {
    let now = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(schedule(vec![window(&Day::ALL, 0, hm(24, 0))]));
    let result = plan(&rules, &[site_target("x.com")], &now);
    assert_eq!(reason_of(&result), ActiveReason::Both);
    assert_eq!(until_of(&result), None);
}

#[test]
fn an_active_unlock_suspends_everything() {
    let start = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(monday_nine_to_five());
    let unlock = unlocked_at(&start, 15);
    let ready = after(&start, 5);
    let spent = exhausted_by(&rules, &[site_target("x.com")], &ready);
    let result = EnforcementPlan::compute(&rules, &unlock, &spent, &ready);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn an_unlock_stops_suspending_exactly_when_it_ends() {
    let start = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(monday_nine_to_five());
    let unlock = unlocked_at(&start, 15);
    let spent = Exhausted::new(&start);
    let last_minute = after(&start, 19);
    let ended = after(&start, 20);
    assert_eq!(
        EnforcementPlan::compute(&rules, &unlock, &spent, &last_minute),
        EnforcementPlan::Inactive
    );
    assert_eq!(
        reason_of(&EnforcementPlan::compute(&rules, &unlock, &spent, &ended)),
        ActiveReason::Schedule
    );
}

#[test]
fn a_pending_unlock_request_does_not_suspend_enforcement() {
    let start = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(monday_nine_to_five());
    let requested = UnlockState::new(&start).request(&start).unwrap();
    let ready = after(&start, 5);
    let unlock = requested.tick(&ready);
    let result = EnforcementPlan::compute(&rules, &unlock, &Exhausted::new(&ready), &ready);
    assert_eq!(reason_of(&result), ActiveReason::Schedule);
}

#[test]
fn exhausted_records_from_a_previous_day_are_ignored() {
    let yesterday = now_at(2026, 10, 11, 20, 0);
    let rules = rules_with(Schedule::default());
    let stale = exhausted_by(&rules, &[site_target("x.com")], &yesterday);
    let now = now_at(2026, 10, 12, 20, 0);
    let result = EnforcementPlan::compute(&rules, &UnlockState::new(&now), &stale, &now);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn a_recorded_target_whose_limit_was_removed_is_ignored() {
    let now = now_at(2026, 10, 12, 20, 0);
    let before = rules_with(Schedule::default());
    let spent = exhausted_by(&before, &[site_target("x.com")], &now);
    let after_removal = valid(Rules {
        limits: vec![limit(app_target("com.discord"), 30)],
        ..Rules::clone(&before)
    });
    let result = EnforcementPlan::compute(&after_removal, &UnlockState::new(&now), &spent, &now);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn raising_a_limit_frees_the_target_but_keeping_or_lowering_it_does_not() {
    let now = now_at(2026, 10, 12, 20, 0);
    let original = rules_with(Schedule::default());
    let spent = exhausted_by(&original, &[site_target("x.com")], &now);
    let with_limit = |minutes: u16| {
        valid(Rules {
            limits: vec![limit(site_target("x.com"), minutes)],
            ..Rules::clone(&original)
        })
    };
    let blocked = |rules: &ValidatedRules| {
        EnforcementPlan::compute(rules, &UnlockState::new(&now), &spent, &now)
            != EnforcementPlan::Inactive
    };
    assert!(blocked(&with_limit(30)));
    assert!(blocked(&with_limit(10)));
    assert!(!blocked(&with_limit(31)));
}

#[test]
fn rules_without_limits_never_block_on_exhaustion() {
    let now = now_at(2026, 10, 12, 20, 0);
    let with_limits = rules_with(Schedule::default());
    let spent = exhausted_by(&with_limits, &[site_target("x.com")], &now);
    let none = ValidatedRules::default();
    let result = EnforcementPlan::compute(&none, &UnlockState::new(&now), &spent, &now);
    assert_eq!(result, EnforcementPlan::Inactive);
}

#[test]
fn active_until_uses_the_offset_between_utc_and_local() {
    let utc_now = Utc.with_ymd_and_hms(2026, 10, 12, 13, 0, 0).unwrap();
    let now = Now::new(utc_now, local(2026, 10, 12, 10, 0));
    let scheduled = plan(&rules_with(monday_nine_to_five()), &[], &now);
    assert_eq!(until_of(&scheduled), utc(12, 20, 0));

    let limit_only = plan(
        &rules_with(Schedule::default()),
        &[site_target("x.com")],
        &now,
    );
    assert_eq!(until_of(&limit_only), utc(13, 3, 0));
}

#[test]
fn the_last_minute_of_the_day_ends_at_the_next_midnight() {
    let now = now_from(local(2026, 10, 12, 23, 59));
    let result = plan(
        &rules_with(Schedule::default()),
        &[site_target("x.com")],
        &now,
    );
    assert_eq!(until_of(&result), utc(13, 0, 0));
}

fn removed_site() -> Loosening {
    Loosening::SiteRemoved {
        domain: site("youtube.com"),
    }
}

fn raised_limit() -> Loosening {
    Loosening::LimitRaised {
        target: site_target("x.com"),
        from: 30,
        to: 60,
    }
}

fn removed_limit() -> Loosening {
    Loosening::LimitRemoved {
        target: site_target("x.com"),
    }
}

fn all_loosenings() -> Vec<Loosening> {
    vec![
        removed_site(),
        Loosening::AppRemoved {
            app: app("com.discord"),
        },
        raised_limit(),
        removed_limit(),
        Loosening::ScheduleReduced {
            minutes_uncovered: 60,
        },
    ]
}

#[test]
fn outside_every_window_only_limit_changes_need_an_unlock() {
    let now = now_at(2026, 10, 12, 20, 0);
    let idle = plan(&rules_with(monday_nine_to_five()), &[], &now);
    assert_eq!(idle, EnforcementPlan::Inactive);
    let locked = locked_loosenings(&all_loosenings(), &idle, false);
    assert_eq!(locked, [raised_limit(), removed_limit()]);
}

#[test]
fn inside_a_window_every_loosening_needs_an_unlock() {
    let now = now_at(2026, 10, 12, 10, 0);
    let busy = plan(&rules_with(monday_nine_to_five()), &[], &now);
    assert_eq!(
        locked_loosenings(&all_loosenings(), &busy, false),
        all_loosenings()
    );
}

#[test]
fn an_active_unlock_lets_every_loosening_through() {
    let start = now_at(2026, 10, 12, 10, 0);
    let rules = rules_with(monday_nine_to_five());
    let unlocked = unlocked_at(&start, 15);
    let ready = after(&start, 5);
    let during = EnforcementPlan::compute(&rules, &unlocked, &Exhausted::new(&ready), &ready);
    assert!(unlocked.is_unlocked(&ready));
    assert_eq!(locked_loosenings(&all_loosenings(), &during, true), []);
}

#[test]
fn adding_rules_and_lowering_limits_are_not_loosenings_to_lock() {
    assert_eq!(
        locked_loosenings(&[], &EnforcementPlan::Inactive, false),
        []
    );
}

#[test]
fn only_limit_changes_are_marked_as_needing_an_unlock_regardless_of_the_schedule() {
    let limit_changes: Vec<bool> = all_loosenings()
        .iter()
        .map(Loosening::is_limit_change)
        .collect();
    assert_eq!(limit_changes, [false, false, true, true, false]);
}
