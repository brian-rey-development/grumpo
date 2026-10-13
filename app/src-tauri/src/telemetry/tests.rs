use super::test_support::*;
use super::*;

#[test]
fn a_missing_log_folder_is_created_before_the_appender_lists_it() {
    let root = tempfile::tempdir().unwrap();
    let dir = root.path().join("Library").join("Logs").join("Focus");

    let writer = file_writer_in(&dir);

    assert!(writer.is_ok(), "{:?}", writer.err());
    assert!(dir.is_dir());
}

#[test]
fn an_existing_log_folder_is_fine() {
    let root = tempfile::tempdir().unwrap();
    assert!(file_writer_in(root.path()).is_ok());
}

#[test]
fn a_panic_is_logged_as_an_error_with_only_where() {
    let text = logged_while(|| log_panic(Some("src/sampler.rs:10:5"), "index out of bounds"));

    let error_line = text
        .lines()
        .find(|line| line.contains("ERROR"))
        .unwrap_or_default();
    assert!(error_line.contains("the app panicked"), "{text}");
    assert!(error_line.contains("src/sampler.rs:10:5"), "{text}");
    assert!(!error_line.contains("index out of bounds"), "{text}");
}

#[test]
fn the_panic_payload_is_only_logged_at_debug() {
    let text = logged_while(|| log_panic(None, "blocked example.com"));

    let payload_line = text
        .lines()
        .find(|line| line.contains("blocked example.com"))
        .unwrap_or_default();
    assert!(payload_line.contains("DEBUG"), "{text}");
    assert!(!text
        .lines()
        .any(|line| line.contains("ERROR") && line.contains("example.com")));
}

#[test]
fn both_kinds_of_panic_payload_are_read() {
    assert_eq!(payload_text(&"static text"), "static text");
    assert_eq!(
        payload_text(&String::from("formatted text")),
        "formatted text"
    );
    assert_eq!(payload_text(&42_u32), "a panic with no text");
}
