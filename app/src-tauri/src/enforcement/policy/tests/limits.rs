use super::*;

#[test]
fn reaching_a_limit_is_reported_once_it_is_met() {
    let status = status_with_limit(30);

    let at_limit = decide_with(None, Some(&status), &used(1_800), &memory(), T0);
    let past_limit = decide_with(None, Some(&status), &used(2_400), &memory(), T0);

    let report = reached(30);
    assert_eq!(at_limit, std::slice::from_ref(&report));
    assert_eq!(past_limit, [report]);
}

#[test]
fn a_limit_the_daemon_already_knows_is_exhausted_is_not_reported() {
    let mut status = status_with_limit(30);
    status.exhausted.push(app_target(STEAM));

    assert_eq!(
        decide_with(None, Some(&status), &used(1_800), &memory(), T0).len(),
        0
    );
}

#[test]
fn a_report_that_succeeded_is_not_repeated() {
    let status = status_with_limit(30);
    let mut memory = memory();
    let report = reached(30);
    memory.note(
        &report,
        Outcome {
            now: T0,
            succeeded: true,
        },
    );

    assert_eq!(
        decide_with(None, Some(&status), &used(1_900), &memory, T0 + 2).len(),
        0
    );
}

#[test]
fn a_report_that_failed_is_retried() {
    let status = status_with_limit(30);
    let mut memory = memory();
    let report = reached(30);
    memory.note(
        &report,
        Outcome {
            now: T0,
            succeeded: false,
        },
    );

    let retry = decide_with(None, Some(&status), &used(1_900), &memory, T0 + 2);

    assert_eq!(retry, [report]);
}

#[test]
fn a_raised_limit_that_is_reached_again_is_reported_again() {
    let mut memory = memory();
    let report = reached(30);
    memory.note(
        &report,
        Outcome {
            now: T0,
            succeeded: true,
        },
    );

    let raised = status_with_limit(45);

    assert_eq!(
        decide_with(None, Some(&raised), &used(2_000), &memory, T0).len(),
        0
    );
    assert_eq!(
        decide_with(None, Some(&raised), &used(2_700), &memory, T0),
        [reached(45)]
    );
}

#[test]
fn a_limit_with_no_usage_entry_is_skipped() {
    let status = status_with_limit(1);

    assert_eq!(
        decide_with(None, Some(&status), &[], &memory(), T0).len(),
        0
    );
}

#[test]
fn the_warning_starts_five_minutes_before_the_limit() {
    let status = status_with_limit(30);

    assert_eq!(
        decide_with(None, Some(&status), &used(1_499), &memory(), T0).len(),
        0
    );
    let warned = decide_with(None, Some(&status), &used(1_500), &memory(), T0);

    assert_eq!(
        warned,
        [Action::Notify(NoticeKind::LimitWarning {
            target: app_target(STEAM),
            seconds_left: 300,
        })]
    );
}

#[test]
fn the_warning_is_given_once_per_target_per_day() {
    let status = status_with_limit(30);
    let mut memory = memory();
    let first = decide_with(None, Some(&status), &used(1_600), &memory, T0);
    note_all(&mut memory, &first, T0);

    assert_eq!(
        decide_with(None, Some(&status), &used(1_700), &memory, T0 + 100).len(),
        0
    );
}

#[test]
fn a_limit_already_passed_is_reported_without_a_late_warning() {
    let status = status_with_limit(30);

    let actions = decide_with(None, Some(&status), &used(1_900), &memory(), T0);

    assert!(matches!(
        actions.as_slice(),
        [Action::ReportLimitReached { .. }]
    ));
}

#[test]
fn each_limit_is_judged_on_its_own_usage() {
    let status = with_limit(status_with_limit(30), site_target("reddit.com"), 60);
    let usage = vec![(app_target(STEAM), 1_800), (site_target("reddit.com"), 600)];

    let actions = decide_with(None, Some(&status), &usage, &memory(), T0);

    assert_eq!(actions, [reached(30)]);
}

#[test]
fn limits_are_tracked_even_when_blocking_is_inactive() {
    let activity = in_app(STEAM);
    let status = status_with_limit(30);

    let actions = decide_with(Some(&activity), Some(&status), &used(1_800), &memory(), T0);

    assert_eq!(actions, [reached(30)]);
}
