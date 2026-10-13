use std::collections::BTreeSet;

use chrono::NaiveDate;
use focus_core::ipc::{
    DaemonStatus, EnforcementReason, EnforcementView, UnlockPhaseView, UnlockView,
};
use focus_core::{Domain, Limit, LimitTarget, Rules};

use super::*;
use crate::dto::DaemonUnavailableReason;

fn connected(rules: Rules) -> FocusStatus {
    FocusStatus::Connected {
        status: Box::new(DaemonStatus {
            rules,
            enforcement: EnforcementView {
                active: false,
                reason: EnforcementReason::None,
                active_until: None,
                blocked_sites: BTreeSet::new(),
                blocked_apps: BTreeSet::new(),
            },
            exhausted: Vec::new(),
            unlock: UnlockView {
                phase: UnlockPhaseView::Locked,
                next_cooldown_minutes: 5,
                sentence: String::new(),
                durations: Vec::new(),
            },
        }),
    }
}

#[test]
fn export_files_are_named_by_local_second() {
    let when = NaiveDate::from_ymd_opt(2026, 10, 12)
        .unwrap()
        .and_hms_opt(9, 5, 7)
        .unwrap();
    assert_eq!(
        export_file_name(when),
        "Grumpo export 2026-10-12 090507.json"
    );
}

#[test]
fn two_exports_in_the_same_minute_do_not_share_a_name() {
    let day = NaiveDate::from_ymd_opt(2026, 10, 12).unwrap();
    let first = export_file_name(day.and_hms_opt(9, 5, 7).unwrap());
    let second = export_file_name(day.and_hms_opt(9, 5, 8).unwrap());
    assert_ne!(first, second);
}

fn io_failure(kind: std::io::ErrorKind) -> StorageError {
    StorageError::Io {
        context: "create export file",
        source: std::io::Error::from(kind),
    }
}

#[test]
fn a_refused_write_is_reported_as_a_missing_permission() {
    let mapped = export_error(io_failure(std::io::ErrorKind::PermissionDenied));
    assert!(matches!(mapped, AppError::ExportDenied));
}

#[test]
fn other_write_failures_stay_storage_errors() {
    let mapped = export_error(io_failure(std::io::ErrorKind::NotFound));
    assert!(matches!(mapped, AppError::Storage(_)));
}

#[test]
fn today_is_kept_only_while_limits_exist_or_are_unknown() {
    assert!(!keeps_today(&connected(Rules::default())));

    let limited = Rules {
        limits: vec![Limit {
            target: LimitTarget::Site {
                domain: Domain::parse("reddit.com").unwrap(),
            },
            daily_minutes: 30,
        }],
        ..Rules::default()
    };
    assert!(keeps_today(&connected(limited)));

    let down = FocusStatus::Unavailable {
        reason: DaemonUnavailableReason::NotResponding,
        message: String::new(),
    };
    assert!(keeps_today(&down));
}
