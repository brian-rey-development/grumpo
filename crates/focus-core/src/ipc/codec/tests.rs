mod worst_case;

use std::io::{BufReader, Cursor};

use serde_json::json;

use super::*;
use crate::ipc::IpcError;
use crate::test_support::{app_target, local, now_at, sample_rules, sample_status, site_target};
use crate::Rules;

fn all_requests() -> Vec<Request> {
    vec![
        Request::Status,
        Request::UpdateRules {
            rules: Rules::clone(&sample_rules()),
        },
        Request::RequestUnlock,
        Request::ConfirmUnlock {
            phrase: "typed".to_owned(),
            minutes: 30,
        },
        Request::CancelUnlock,
        Request::Relock,
        Request::ReportLimitReached {
            target: app_target("com.x"),
            daily_minutes: 45,
            day: local(2026, 10, 12, 0, 0).date(),
        },
    ]
}

fn all_responses() -> Vec<Response> {
    vec![
        Response::Ok {
            status: sample_status(&now_at(2026, 10, 12, 10, 0)),
        },
        Response::Err {
            error: IpcError::internal("boom"),
        },
    ]
}

fn frame_with_version(version: u32, body: &serde_json::Value) -> Vec<u8> {
    let mut bytes = serde_json::to_vec(&json!({"protocolVersion": version, "body": body})).unwrap();
    bytes.push(b'\n');
    bytes
}

#[test]
fn every_request_round_trips_as_one_line() {
    for request in all_requests() {
        let bytes = encode(&request).unwrap();
        assert_eq!(
            bytes.iter().position(|&byte| byte == b'\n'),
            Some(bytes.len() - 1)
        );
        assert_eq!(decode::<Request>(&bytes).unwrap(), request);
        assert_eq!(
            decode::<Request>(&bytes[..bytes.len() - 1]).unwrap(),
            request
        );
    }
}

#[test]
fn every_response_round_trips() {
    for response in all_responses() {
        let bytes = encode(&response).unwrap();
        assert_eq!(decode::<Response>(&bytes).unwrap(), response);
    }
}

#[test]
fn both_directions_travel_in_a_versioned_envelope() {
    let request: serde_json::Value =
        serde_json::from_slice(&encode(&Request::Status).unwrap()).unwrap();
    assert_eq!(
        request,
        json!({"protocolVersion": PROTOCOL_VERSION, "body": {"kind": "status"}})
    );
    for response in all_responses() {
        let value: serde_json::Value = serde_json::from_slice(&encode(&response).unwrap()).unwrap();
        assert_eq!(value["protocolVersion"], json!(PROTOCOL_VERSION));
        assert_eq!(
            value["body"]["kind"],
            json!(match response {
                Response::Ok { .. } => "ok",
                Response::Err { .. } => "err",
            })
        );
    }
}

#[test]
fn a_request_from_another_version_is_a_version_mismatch_even_with_an_unknown_body() {
    let body = json!({"kind": "somethingFromTheFuture", "shape": [1, 2]});
    let frame = frame_with_version(PROTOCOL_VERSION + 1, &body);
    assert!(matches!(
        decode::<Request>(&frame),
        Err(IpcCodecError::VersionMismatch { expected, found })
            if expected == PROTOCOL_VERSION && found == PROTOCOL_VERSION + 1
    ));
}

#[test]
fn a_response_from_another_version_is_a_version_mismatch_even_with_an_unknown_body() {
    let frame = frame_with_version(0, &json!("not a response"));
    assert!(matches!(
        decode::<Response>(&frame),
        Err(IpcCodecError::VersionMismatch { expected, found: 0 }) if expected == PROTOCOL_VERSION
    ));
}

#[test]
fn a_matching_version_with_a_bad_body_is_a_json_error() {
    let frame = frame_with_version(PROTOCOL_VERSION, &json!({"kind": "somethingFromTheFuture"}));
    assert!(matches!(
        decode::<Request>(&frame),
        Err(IpcCodecError::Json(_))
    ));
}

#[test]
fn the_envelope_rejects_unknown_fields_and_a_missing_version() {
    let extra = format!(
        r#"{{"protocolVersion":{PROTOCOL_VERSION},"body":{{"kind":"status"}},"extra":true}}"#
    );
    assert!(matches!(
        decode::<Request>(extra.as_bytes()),
        Err(IpcCodecError::Json(_))
    ));
    let unversioned = br#"{"body":{"kind":"status"}}"#;
    assert!(matches!(
        decode::<Request>(unversioned),
        Err(IpcCodecError::Json(_))
    ));
}

#[test]
fn encoding_stops_at_the_size_limit_of_each_direction() {
    let overhead = encode(&Request::ConfirmUnlock {
        phrase: String::new(),
        minutes: 15,
    })
    .unwrap()
    .len();
    let message = |phrase_len: usize| Request::ConfirmUnlock {
        phrase: "a".repeat(phrase_len),
        minutes: 15,
    };
    let at_limit = encode(&message(MAX_REQUEST_BYTES - overhead)).unwrap();
    assert_eq!(at_limit.len(), MAX_REQUEST_BYTES);
    assert!(matches!(
        encode(&message(MAX_REQUEST_BYTES - overhead + 1)),
        Err(IpcCodecError::TooLarge { max }) if max == MAX_REQUEST_BYTES
    ));
}

#[test]
fn responses_may_be_larger_than_requests() {
    let message = IpcError::internal("x".repeat(MAX_REQUEST_BYTES));
    let response = Response::Err { error: message };
    let bytes = encode(&response).unwrap();
    assert!(bytes.len() > MAX_REQUEST_BYTES);
    assert!(bytes.len() <= MAX_RESPONSE_BYTES);
    assert!(matches!(
        decode::<Request>(&bytes),
        Err(IpcCodecError::TooLarge { .. })
    ));
    assert_eq!(decode::<Response>(&bytes).unwrap(), response);
}

#[test]
fn decoding_rejects_oversized_input_before_parsing() {
    let oversized = vec![b' '; MAX_REQUEST_BYTES + 1];
    assert!(matches!(
        decode::<Request>(&oversized),
        Err(IpcCodecError::TooLarge { .. })
    ));
}

#[test]
fn decoding_rejects_malformed_input() {
    let version = PROTOCOL_VERSION;
    let cases: [String; 7] = [
        String::new(),
        "\n".to_owned(),
        "not json\n".to_owned(),
        format!(r#"{{"protocolVersion":{version},"body":{{"kind":"status""#),
        format!("{{\"protocolVersion\":{version},\"body\":{{\"kind\":\"nope\"}}}}\n"),
        format!(
            "{{\"protocolVersion\":{version},\"body\":{{\"kind\":\"status\"}}}}\n{{\"protocolVersion\":{version}}}\n"
        ),
        format!(
            "{{\"protocolVersion\":{version},\"body\":{{\"kind\":\"confirmUnlock\",\"phrase\":\"p\"}}}}\n"
        ),
    ];
    for input in &cases {
        assert!(
            matches!(
                decode::<Request>(input.as_bytes()),
                Err(IpcCodecError::Json(_))
            ),
            "input {input:?}"
        );
    }
}

#[test]
fn decoding_validates_embedded_domains() {
    let frame = frame_with_version(
        PROTOCOL_VERSION,
        &json!({"kind": "reportLimitReached", "target": {"kind": "site", "domain": "not a domain"}, "dailyMinutes": 30}),
    );
    assert!(decode::<Request>(&frame).is_err());
    let valid = frame_with_version(
        PROTOCOL_VERSION,
        &json!({"kind": "reportLimitReached", "target": {"kind": "site", "domain": "x.com"}, "dailyMinutes": 30, "day": "2026-10-12"}),
    );
    assert_eq!(
        decode::<Request>(&valid).unwrap(),
        Request::ReportLimitReached {
            target: site_target("x.com"),
            daily_minutes: 30,
            day: local(2026, 10, 12, 0, 0).date(),
        }
    );
}

#[test]
fn codec_errors_are_readable() {
    assert_eq!(
        IpcCodecError::TooLarge { max: 262_144 }.to_string(),
        "message is larger than the 262144 byte limit"
    );
    assert_eq!(
        IpcCodecError::VersionMismatch {
            expected: 2,
            found: 3
        }
        .to_string(),
        "the peer speaks protocol version 3, this side speaks 2"
    );
    let malformed = decode::<Request>(b"x").unwrap_err();
    assert!(malformed.to_string().starts_with("invalid JSON message"));
}

#[test]
fn read_frame_returns_one_line_and_leaves_the_rest() {
    let mut reader = Cursor::new(b"first\nsecond\n".to_vec());
    assert_eq!(read_frame(&mut reader, 64).unwrap(), b"first\n");
    assert_eq!(read_frame(&mut reader, 64).unwrap(), b"second\n");
    assert!(matches!(
        read_frame(&mut reader, 64),
        Err(IpcCodecError::MissingTerminator)
    ));
}

#[test]
fn read_frame_limit_includes_the_newline() {
    let mut exact = Cursor::new(b"abcd\n".to_vec());
    assert_eq!(read_frame(&mut exact, 5).unwrap(), b"abcd\n");
    let mut over = Cursor::new(b"abcde\n".to_vec());
    assert!(matches!(
        read_frame(&mut over, 5),
        Err(IpcCodecError::TooLarge { max: 5 })
    ));
}

#[test]
fn read_frame_rejects_input_that_ends_without_a_newline() {
    for input in [&b""[..], b"abc", b"abcde"] {
        let mut reader = Cursor::new(input.to_vec());
        assert!(
            matches!(
                read_frame(&mut reader, 5),
                Err(IpcCodecError::MissingTerminator)
            ),
            "input {input:?}"
        );
    }
}

#[test]
fn read_frame_stops_reading_one_byte_past_the_limit() {
    let mut endless = BufReader::new(std::io::repeat(b'a'));
    assert!(matches!(
        read_frame(&mut endless, MAX_REQUEST_BYTES),
        Err(IpcCodecError::TooLarge { .. })
    ));

    let mut source = Cursor::new(vec![b'a'; 1000]);
    assert!(read_frame(&mut source, 10).is_err());
    assert_eq!(source.position(), 11);
}

#[test]
fn a_frame_from_read_frame_decodes() {
    let bytes = encode(&Request::Relock).unwrap();
    let frame = read_frame(&mut Cursor::new(bytes), MAX_REQUEST_BYTES).unwrap();
    assert_eq!(decode::<Request>(&frame).unwrap(), Request::Relock);
}

#[test]
fn a_limit_report_without_a_day_is_a_version_error_from_the_previous_protocol() {
    let old_report = json!({
        "kind": "reportLimitReached",
        "target": {"kind": "site", "domain": "x.com"},
        "dailyMinutes": 30
    });
    let frame = frame_with_version(2, &old_report);
    assert!(matches!(
        decode::<Request>(&frame),
        Err(IpcCodecError::VersionMismatch {
            expected: 3,
            found: 2
        })
    ));
    let current = frame_with_version(PROTOCOL_VERSION, &old_report);
    assert!(matches!(
        decode::<Request>(&current),
        Err(IpcCodecError::Json(_))
    ));
}
