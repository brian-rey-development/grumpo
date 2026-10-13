//! Wording of the notifications and the content of the block screen, kept apart from the
//! policy that decides to send them.

use chrono::NaiveTime;

use super::action::NoticeKind;
use crate::dto::{BlockScreen, BlockTarget, UNASSIGNED_ID};
use crate::notify::Notice;
use crate::time::{ceil_minutes, minutes_phrase, ClockFormatter};

const APP_BODY: &str = "Grumpo closed it. Back to work.";
const SITE_BODY: &str = "Grumpo closed the tab. Back to work.";

/// Renders `kind`, showing times on the person's clock.
pub fn render(kind: &NoticeKind, clock_text: &ClockFormatter) -> Notice {
    match kind {
        NoticeKind::AppBlocked { app, until, .. } => Notice {
            title: format!(
                "{} is blocked{}",
                app.name(),
                until_phrase(*until, clock_text)
            ),
            body: APP_BODY.to_owned(),
        },
        NoticeKind::SiteBlocked { domain, until, .. } => Notice {
            title: format!("{domain} is blocked{}", until_phrase(*until, clock_text)),
            body: SITE_BODY.to_owned(),
        },
        NoticeKind::LimitWarning {
            target,
            seconds_left,
        } => Notice {
            title: format!(
                "{} left on {target}",
                minutes_phrase(ceil_minutes(*seconds_left))
            ),
            body: format!("After that, Grumpo blocks {target} until midnight."),
        },
    }
}

/// What the block screen shows for `kind`, or `None` for a kind that is only ever a
/// notification.
pub fn block_screen(kind: &NoticeKind) -> Option<BlockScreen> {
    match kind {
        NoticeKind::AppBlocked {
            app,
            until,
            reason,
            strict,
        } => Some(BlockScreen {
            id: UNASSIGNED_ID,
            target: BlockTarget::App {
                name: app.name().to_owned(),
            },
            reason: *reason,
            until: *until,
            strict: *strict,
        }),
        NoticeKind::SiteBlocked {
            domain,
            until,
            reason,
            strict,
        } => Some(BlockScreen {
            id: UNASSIGNED_ID,
            target: BlockTarget::Site {
                domain: domain.as_str().to_owned(),
            },
            reason: *reason,
            until: *until,
            strict: *strict,
        }),
        NoticeKind::LimitWarning { .. } => None,
    }
}

fn until_phrase(until: Option<i64>, clock_text: &ClockFormatter) -> String {
    let Some(unix) = until else {
        return String::new();
    };
    if clock_text.time_of(unix) == Some(NaiveTime::MIN) {
        return " until midnight".to_owned();
    }
    clock_text
        .clock(unix)
        .map_or_else(String::new, |time| format!(" until {time}"))
}

#[cfg(test)]
mod test_support;
#[cfg(test)]
mod tests;
