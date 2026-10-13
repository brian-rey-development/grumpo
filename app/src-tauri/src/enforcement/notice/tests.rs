use std::sync::Arc;

use focus_core::LimitTarget;

use super::test_support::*;
use super::*;
use crate::dto::{BlockReason, BlockScreen, BlockTarget, UNASSIGNED_ID};
use crate::platform::HourCycle;
use crate::time::test_support::FakeHourCycle;

#[test]
fn a_blocked_app_names_the_app_and_the_local_end_time() {
    let kind = app_blocked(Some(at(19, 0)));

    let notice = render(&kind, &plain());

    assert_eq!(notice.title, "Steam is blocked until 19:00");
    assert_eq!(notice.body, "Grumpo closed it. Back to work.");
}

#[test]
fn the_end_time_follows_the_system_clock_setting() {
    let kind = app_blocked(Some(at(19, 0)));

    let notice = render(&kind, &formatter(HourCycle::Twelve));

    assert_eq!(notice.title, "Steam is blocked until 7:00 PM");
}

#[test]
fn an_end_time_at_local_midnight_reads_as_midnight() {
    let kind = app_blocked(Some(0));

    assert_eq!(
        render(&kind, &plain()).title,
        "Steam is blocked until midnight"
    );
}

#[test]
fn blocking_without_an_end_has_no_until_clause() {
    let kind = site_blocked(None);

    assert_eq!(render(&kind, &plain()).title, "youtube.com is blocked");
}

#[test]
fn an_end_time_the_clock_cannot_convert_is_left_out() {
    let kind = site_blocked(Some(1));
    let lost = ClockFormatter::new(
        Arc::new(Lost),
        Arc::new(FakeHourCycle::new(HourCycle::TwentyFour)),
    );

    assert_eq!(render(&kind, &lost).title, "youtube.com is blocked");
}

#[test]
fn a_blocked_site_names_the_domain_and_end_time() {
    let kind = site_blocked(Some(at(7, 5)));

    let notice = render(&kind, &plain());

    assert_eq!(notice.title, "youtube.com is blocked until 07:05");
    assert_eq!(notice.body, "Grumpo closed the tab. Back to work.");
}

#[test]
fn the_limit_warning_rounds_seconds_up_to_whole_minutes() {
    let target = LimitTarget::App { app: steam() };
    let title = |seconds_left| {
        let kind = NoticeKind::LimitWarning {
            target: target.clone(),
            seconds_left,
        };
        render(&kind, &plain()).title
    };

    assert_eq!(title(300), "5 minutes left on Steam");
    assert_eq!(title(250), "5 minutes left on Steam");
    assert_eq!(title(61), "2 minutes left on Steam");
    assert_eq!(title(60), "1 minute left on Steam");
    assert_eq!(title(1), "1 minute left on Steam");
    assert_eq!(title(0), "1 minute left on Steam");
}

#[test]
fn the_limit_warning_body_says_what_happens_next() {
    let kind = NoticeKind::LimitWarning {
        target: LimitTarget::Site { domain: youtube() },
        seconds_left: 300,
    };

    assert_eq!(
        render(&kind, &plain()).body,
        "After that, Grumpo blocks youtube.com until midnight."
    );
}

#[test]
fn a_blocked_app_becomes_a_block_screen_for_that_app() {
    let kind = NoticeKind::AppBlocked {
        app: steam(),
        until: Some(at(19, 0)),
        reason: BlockReason::Limit,
        strict: false,
    };

    assert_eq!(
        block_screen(&kind),
        Some(BlockScreen {
            id: UNASSIGNED_ID,
            target: BlockTarget::App {
                name: "Steam".to_owned()
            },
            reason: BlockReason::Limit,
            until: Some(at(19, 0)),
            strict: false,
        })
    );
}

#[test]
fn a_blocked_site_becomes_a_block_screen_for_that_domain() {
    let kind = NoticeKind::SiteBlocked {
        domain: youtube(),
        until: None,
        reason: BlockReason::Always,
        strict: true,
    };

    assert_eq!(
        block_screen(&kind),
        Some(BlockScreen {
            id: UNASSIGNED_ID,
            target: BlockTarget::Site {
                domain: "youtube.com".to_owned()
            },
            reason: BlockReason::Always,
            until: None,
            strict: true,
        })
    );
}

#[test]
fn a_limit_warning_has_no_block_screen() {
    let kind = NoticeKind::LimitWarning {
        target: LimitTarget::App { app: steam() },
        seconds_left: 300,
    };

    assert_eq!(block_screen(&kind), None);
}
