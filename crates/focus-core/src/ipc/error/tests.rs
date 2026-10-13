use serde_json::json;

use super::*;
use crate::test_support::{site, site_target};

#[test]
fn error_codes_serialize_as_snake_case() {
    let cases = [
        (ErrorCode::VersionMismatch, "version_mismatch"),
        (ErrorCode::RulesInvalid, "rules_invalid"),
        (ErrorCode::LooseningLocked, "loosening_locked"),
        (ErrorCode::UnlockNotReady, "unlock_not_ready"),
        (ErrorCode::UnlockNotRequested, "unlock_not_requested"),
        (ErrorCode::UnlockAlreadyPending, "unlock_already_pending"),
        (ErrorCode::UnlockAlreadyActive, "unlock_already_active"),
        (ErrorCode::PhraseMismatch, "phrase_mismatch"),
        (ErrorCode::DurationInvalid, "duration_invalid"),
        (ErrorCode::Protocol, "protocol"),
        (ErrorCode::Internal, "internal"),
    ];
    for (code, text) in cases {
        assert_eq!(serde_json::to_value(code).unwrap(), json!(text));
    }
}

#[test]
fn unlock_errors_map_to_their_codes() {
    let cases = [
        (UnlockError::NotRequested, ErrorCode::UnlockNotRequested),
        (UnlockError::AlreadyPending, ErrorCode::UnlockAlreadyPending),
        (UnlockError::AlreadyUnlocked, ErrorCode::UnlockAlreadyActive),
        (UnlockError::NotReady, ErrorCode::UnlockNotReady),
        (UnlockError::PhraseMismatch, ErrorCode::PhraseMismatch),
        (UnlockError::DurationInvalid, ErrorCode::DurationInvalid),
    ];
    for (error, code) in cases {
        let mapped = IpcError::from(error);
        assert_eq!(mapped.code, code);
        assert_eq!(mapped.message, error.to_string());
    }
}

#[test]
fn rules_errors_map_to_rules_invalid() {
    let error = RulesError::WindowWithoutDays { number: 2 };
    let mapped = IpcError::from(error.clone());
    assert_eq!(mapped.code, ErrorCode::RulesInvalid);
    assert_eq!(mapped.message, error.to_string());
}

#[test]
fn codec_errors_map_to_protocol_except_version_mismatches() {
    let too_large = IpcError::from(IpcCodecError::TooLarge { max: 10 });
    assert_eq!(too_large.code, ErrorCode::Protocol);
    assert!(too_large.message.contains("byte limit"));

    let mismatch = IpcError::from(IpcCodecError::VersionMismatch {
        expected: 1,
        found: 2,
    });
    assert_eq!(mismatch.code, ErrorCode::VersionMismatch);
    assert!(mismatch.message.contains("version 2"));
}

#[test]
fn loosening_locked_lists_every_change() {
    let error = IpcError::loosening_locked(&[
        Loosening::SiteRemoved {
            domain: site("youtube.com"),
        },
        Loosening::LimitRemoved {
            target: site_target("x.com"),
        },
    ]);
    assert_eq!(error.code, ErrorCode::LooseningLocked);
    assert_eq!(
        error.message,
        "This change needs an unlock first because it unblocks youtube.com, removes the limit for x.com"
    );
    assert_eq!(error.to_string(), error.message);
}
