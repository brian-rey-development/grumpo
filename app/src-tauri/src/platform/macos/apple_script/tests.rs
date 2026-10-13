use super::*;

#[test]
fn extracts_the_trailing_error_number() {
    let stderr = "execution error: Not authorized to send Apple events to Google Chrome. (-1743)\n";
    assert_eq!(error_code(stderr), Some(-1743));
    assert_eq!(
        error_code("12:20: execution error: Application isn\u{2019}t running. (-600)"),
        Some(-600)
    );
}

#[test]
fn uses_the_last_marker_when_the_message_has_parentheses() {
    let stderr = "execution error: Can't get window (the first) of x. (-1728)";
    assert_eq!(error_code(stderr), Some(-1728));
}

#[test]
fn yields_no_code_for_unstructured_output() {
    assert_eq!(error_code(""), None);
    assert_eq!(error_code("syntax error (-"), None);
    assert_eq!(error_code("oops (-abc)"), None);
    assert_eq!(error_code("weird (-é"), None);
}

#[test]
fn maps_codes_to_platform_outcomes() {
    let denied = interpret(
        Browser::Chrome,
        Err(ScriptFailure::Rejected { code: Some(-1743) }),
    );
    assert!(matches!(
        denied,
        Err(PlatformError::PermissionDenied(Permission::Automation(
            Browser::Chrome
        )))
    ));
    for code in [-600, -1728, -1719] {
        let gone = interpret(
            Browser::Safari,
            Err(ScriptFailure::Rejected { code: Some(code) }),
        );
        assert!(matches!(gone, Ok(None)), "{code}");
    }
    let timeout = interpret(Browser::Brave, Err(ScriptFailure::Timeout));
    assert!(matches!(timeout, Err(PlatformError::Timeout { .. })));
    let other = interpret(
        Browser::Edge,
        Err(ScriptFailure::Rejected { code: Some(-2700) }),
    );
    assert!(matches!(other, Err(PlatformError::Failed { .. })));
    let unknown = interpret(Browser::Edge, Err(ScriptFailure::Rejected { code: None }));
    assert!(matches!(unknown, Err(PlatformError::Failed { .. })));
}

#[test]
fn passes_successful_output_through() {
    let ok = interpret(Browser::Chrome, Ok("https://a.com".to_owned()));
    assert_eq!(ok.unwrap().as_deref(), Some("https://a.com"));
}

#[test]
fn escapes_quotes_backslashes_and_whitespace_controls() {
    assert_eq!(escape_literal(r#"a"b\c"#).as_deref(), Some(r#"a\"b\\c"#));
    assert_eq!(
        escape_literal("a\nb\rc\td").as_deref(),
        Some("a\\nb\\rc\\td")
    );
    assert_eq!(
        escape_literal("https://x.com/?q=é").as_deref(),
        Some("https://x.com/?q=é")
    );
}

#[test]
fn refuses_other_control_characters() {
    assert_eq!(escape_literal("a\u{0}b"), None);
    assert_eq!(escape_literal("a\u{1b}b"), None);
}

#[test]
fn an_escaped_value_cannot_terminate_the_literal() {
    let hostile = r#"x" & (do shell script "id") & ""#;
    let escaped = escape_literal(hostile).unwrap();
    let mut previous_backslashes = 0;
    for c in escaped.chars() {
        if c == '"' {
            assert_eq!(previous_backslashes % 2, 1, "unescaped quote in {escaped}");
        }
        previous_backslashes = if c == '\\' {
            previous_backslashes + 1
        } else {
            0
        };
    }
}
