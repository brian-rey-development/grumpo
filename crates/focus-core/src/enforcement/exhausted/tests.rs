use super::*;
use crate::test_support::{app_target, now_at, rules_with, site_target};
use crate::Schedule;

fn rules() -> ValidatedRules {
    rules_with(Schedule::default())
}

fn targets<'a>(
    exhausted: &'a Exhausted,
    rules: &'a ValidatedRules,
    now: &Now,
) -> Vec<&'a LimitTarget> {
    exhausted.effective(rules, now).collect()
}

#[test]
fn starts_empty_on_the_current_day() {
    let now = now_at(2026, 10, 12, 9, 0);
    let exhausted = Exhausted::new(&now);
    assert_eq!(exhausted.day(), now.local().date());
    assert_eq!(targets(&exhausted, &rules(), &now).len(), 0);
}

#[test]
fn record_adds_targets_without_mutating_the_original() {
    let now = now_at(2026, 10, 12, 9, 0);
    let rules = rules();
    let empty = Exhausted::new(&now);
    let one = empty
        .record(&rules, site_target("x.com"), 30, now.local().date(), &now)
        .unwrap();
    let two = one
        .record(
            &rules,
            app_target("com.discord"),
            30,
            now.local().date(),
            &now,
        )
        .unwrap()
        .record(&rules, site_target("x.com"), 30, now.local().date(), &now)
        .unwrap();
    assert_eq!(targets(&empty, &rules, &now).len(), 0);
    assert_eq!(targets(&one, &rules, &now).len(), 1);
    assert_eq!(targets(&two, &rules, &now).len(), 2);
}

#[test]
fn record_rejects_targets_without_a_limit() {
    let now = now_at(2026, 10, 12, 9, 0);
    let unknown = site_target("nolimit.com");
    assert_eq!(
        Exhausted::new(&now).record(&rules(), unknown.clone(), 30, now.local().date(), &now),
        Err(ExhaustedError::LimitUnknown(unknown))
    );
}

#[test]
fn record_remembers_the_minutes_that_ran_out() {
    let now = now_at(2026, 10, 12, 9, 0);
    let recorded = Exhausted::new(&now)
        .record(&rules(), site_target("x.com"), 30, now.local().date(), &now)
        .unwrap();
    assert_eq!(recorded.targets.get(&site_target("x.com")), Some(&30));
}

#[test]
fn record_refuses_a_report_the_current_limit_has_outgrown() {
    let now = now_at(2026, 10, 12, 9, 0);
    let target = site_target("x.com");
    assert_eq!(
        Exhausted::new(&now).record(&rules(), target.clone(), 20, now.local().date(), &now),
        Err(ExhaustedError::LimitRaised {
            target,
            current: 30,
            reported: 20
        })
    );
}

#[test]
fn record_accepts_a_report_above_a_lowered_limit_and_keeps_the_larger_one() {
    let now = now_at(2026, 10, 12, 9, 0);
    let target = site_target("x.com");
    let recorded = Exhausted::new(&now)
        .record(&rules(), target.clone(), 45, now.local().date(), &now)
        .unwrap();
    assert_eq!(recorded.targets.get(&target), Some(&45));
    let again = recorded
        .record(&rules(), target.clone(), 30, now.local().date(), &now)
        .unwrap();
    assert_eq!(again.targets.get(&target), Some(&45));
}

#[test]
fn record_starts_a_fresh_day_when_the_old_one_is_over() {
    let rules = rules();
    let yesterday = now_at(2026, 10, 11, 23, 0);
    let today = now_at(2026, 10, 12, 8, 0);
    let old = Exhausted::new(&yesterday)
        .record(
            &rules,
            site_target("x.com"),
            30,
            yesterday.local().date(),
            &yesterday,
        )
        .unwrap();
    let fresh = old
        .record(
            &rules,
            app_target("com.discord"),
            30,
            today.local().date(),
            &today,
        )
        .unwrap();
    assert_eq!(fresh.day(), today.local().date());
    assert_eq!(
        targets(&fresh, &rules, &today),
        [&app_target("com.discord")]
    );
}

#[test]
fn effective_ignores_a_record_from_another_day() {
    let rules = rules();
    let yesterday = now_at(2026, 10, 11, 23, 0);
    let old = Exhausted::new(&yesterday)
        .record(
            &rules,
            site_target("x.com"),
            30,
            yesterday.local().date(),
            &yesterday,
        )
        .unwrap();
    assert_eq!(targets(&old, &rules, &now_at(2026, 10, 12, 0, 0)).len(), 0);
    assert_eq!(targets(&old, &rules, &yesterday).len(), 1);
}

#[test]
fn roll_over_keeps_the_same_day_and_resets_a_new_one() {
    let rules = rules();
    let morning = now_at(2026, 10, 12, 0, 0);
    let exhausted = Exhausted::new(&morning)
        .record(
            &rules,
            site_target("x.com"),
            30,
            morning.local().date(),
            &morning,
        )
        .unwrap();
    let night = now_at(2026, 10, 12, 23, 59);
    assert_eq!(exhausted.clone().roll_over(&night), exhausted);
    let tomorrow = now_at(2026, 10, 13, 0, 0);
    let reset = exhausted.roll_over(&tomorrow);
    assert_eq!(reset, Exhausted::new(&tomorrow));
}

#[test]
fn round_trips_through_json_as_a_list_of_entries() {
    let now = now_at(2026, 10, 12, 0, 0);
    let exhausted = Exhausted::new(&now)
        .record(
            &rules(),
            app_target("com.discord"),
            30,
            now.local().date(),
            &now,
        )
        .unwrap();
    let json = serde_json::to_value(&exhausted).unwrap();
    assert_eq!(
        json["targets"][0],
        serde_json::json!({
            "target": {"kind": "app", "app": {"bundleId": "com.discord", "name": "com.discord"}},
            "dailyMinutes": 30
        })
    );
    assert_eq!(
        serde_json::from_value::<Exhausted>(json).unwrap(),
        exhausted
    );
}

#[test]
fn record_refuses_a_report_from_another_day() {
    let rules = rules();
    let late = now_at(2026, 10, 12, 0, 0);
    let yesterday = now_at(2026, 10, 11, 23, 59).local().date();
    assert_eq!(
        Exhausted::new(&late).record(&rules, site_target("x.com"), 30, yesterday, &late),
        Err(ExhaustedError::WrongDay {
            reported: yesterday,
            today: late.local().date()
        })
    );
    let tomorrow = now_at(2026, 10, 13, 0, 0).local().date();
    assert!(Exhausted::new(&late)
        .record(&rules, site_target("x.com"), 30, tomorrow, &late)
        .is_err());
}

#[test]
fn a_clock_set_back_does_not_free_what_was_blocked() {
    let rules = rules();
    let today = now_at(2026, 10, 12, 9, 0);
    let exhausted = Exhausted::new(&today)
        .record(
            &rules,
            site_target("x.com"),
            30,
            today.local().date(),
            &today,
        )
        .unwrap();
    let yesterday = now_at(2026, 10, 11, 9, 0);
    assert_eq!(targets(&exhausted, &rules, &yesterday).len(), 1);
    assert_eq!(exhausted.clone().roll_over(&yesterday), exhausted);
}

#[test]
fn rolling_forward_again_after_going_back_keeps_the_record_until_the_date_passes() {
    let rules = rules();
    let today = now_at(2026, 10, 12, 9, 0);
    let exhausted = Exhausted::new(&today)
        .record(
            &rules,
            site_target("x.com"),
            30,
            today.local().date(),
            &today,
        )
        .unwrap();
    let back = exhausted.roll_over(&now_at(2026, 10, 10, 9, 0));
    let same_day_again = back.roll_over(&now_at(2026, 10, 12, 20, 0));
    assert_eq!(targets(&same_day_again, &rules, &today).len(), 1);
    let next_day = same_day_again.roll_over(&now_at(2026, 10, 13, 0, 0));
    assert_eq!(next_day.day(), now_at(2026, 10, 13, 0, 0).local().date());
}
