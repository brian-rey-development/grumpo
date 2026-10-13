use serde_json::json;

use super::*;

#[test]
fn a_site_block_is_sent_with_a_kind_tag_and_camel_case_fields() {
    let screen = BlockScreen {
        id: 7,
        target: BlockTarget::Site {
            domain: "youtube.com".to_owned(),
        },
        reason: BlockReason::Schedule,
        until: Some(1_800_000_000),
        strict: true,
    };

    assert_eq!(
        serde_json::to_value(screen).unwrap(),
        json!({
            "id": 7,
            "target": { "kind": "site", "domain": "youtube.com" },
            "reason": "schedule",
            "until": 1_800_000_000,
            "strict": true,
        })
    );
}

#[test]
fn an_app_block_without_an_end_sends_null() {
    let screen = BlockScreen {
        id: 7,
        target: BlockTarget::App {
            name: "Steam".to_owned(),
        },
        reason: BlockReason::Always,
        until: None,
        strict: false,
    };

    assert_eq!(
        serde_json::to_value(screen).unwrap(),
        json!({
            "id": 7,
            "target": { "kind": "app", "name": "Steam" },
            "reason": "always",
            "until": null,
            "strict": false,
        })
    );
}

#[test]
fn the_reasons_are_sent_in_lower_case() {
    for (reason, wire) in [
        (BlockReason::Schedule, "\"schedule\""),
        (BlockReason::Limit, "\"limit\""),
        (BlockReason::Always, "\"always\""),
    ] {
        assert_eq!(serde_json::to_string(&reason).unwrap(), wire);
    }
}
