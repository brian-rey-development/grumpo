use serde_json::json;

use super::*;
use crate::test_support::{local, sample_rules, site_target};
use crate::Rules;

#[test]
fn requests_use_a_kind_tag_in_camel_case() {
    let cases = [
        (Request::Status, json!({"kind": "status"})),
        (Request::RequestUnlock, json!({"kind": "requestUnlock"})),
        (Request::CancelUnlock, json!({"kind": "cancelUnlock"})),
        (Request::Relock, json!({"kind": "relock"})),
        (
            Request::ConfirmUnlock {
                phrase: "p".to_owned(),
                minutes: 15,
            },
            json!({"kind": "confirmUnlock", "phrase": "p", "minutes": 15}),
        ),
        (
            Request::ReportLimitReached {
                target: site_target("x.com"),
                daily_minutes: 30,
                day: local(2026, 10, 12, 0, 0).date(),
            },
            json!({
                "kind": "reportLimitReached",
                "target": {"kind": "site", "domain": "x.com"},
                "dailyMinutes": 30,
                "day": "2026-10-12"
            }),
        ),
    ];
    for (request, expected) in cases {
        assert_eq!(serde_json::to_value(&request).unwrap(), expected);
        assert_eq!(
            serde_json::from_value::<Request>(expected).unwrap(),
            request
        );
    }
}

#[test]
fn request_variants_with_fields_reject_unknown_fields() {
    let rules = serde_json::to_value(Rules::clone(&sample_rules())).unwrap();
    let cases = [
        json!({"kind": "updateRules", "rules": rules, "extra": 1}),
        json!({"kind": "confirmUnlock", "phrase": "p", "minutes": 15, "extra": 1}),
        json!({"kind": "reportLimitReached", "target": {"kind": "site", "domain": "x.com"}, "dailyMinutes": 30, "day": "2026-10-12", "extra": 1}),
    ];
    for value in cases {
        let error = serde_json::from_value::<Request>(value.clone()).unwrap_err();
        assert!(error.to_string().contains("unknown field"), "value {value}");
    }
}

#[test]
fn request_variants_with_fields_reject_unknown_fields_inside_rules() {
    let mut rules = serde_json::to_value(Rules::clone(&sample_rules())).unwrap();
    rules["fromTheFuture"] = json!(true);
    let value = json!({"kind": "updateRules", "rules": rules});
    assert!(serde_json::from_value::<Request>(value).is_err());
}

#[test]
fn unit_request_variants_ignore_unknown_fields_because_serde_cannot_deny_them_when_tagged() {
    for kind in ["status", "requestUnlock", "cancelUnlock", "relock"] {
        let value = json!({"kind": kind, "extra": 1});
        assert!(
            serde_json::from_value::<Request>(value).is_ok(),
            "kind {kind}"
        );
    }
}

#[test]
fn error_responses_have_the_documented_shape() {
    let response = Response::Err {
        error: IpcError::new(crate::ipc::ErrorCode::PhraseMismatch, "nope"),
    };
    assert_eq!(
        serde_json::to_value(&response).unwrap(),
        json!({"kind": "err", "error": {"code": "phrase_mismatch", "message": "nope"}})
    );
}
