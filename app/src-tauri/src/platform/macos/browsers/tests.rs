use focus_core::test_support::site;

use super::*;

#[test]
fn covers_the_domain_and_its_subdomains_only() {
    let youtube = site("youtube.com");
    assert!(is_covered(&youtube, "https://www.youtube.com/watch?v=1"));
    assert!(is_covered(&youtube, "https://m.youtube.com/"));
    assert!(!is_covered(&youtube, "https://notyoutube.com/"));
    assert!(!is_covered(&youtube, "https://youtube.com.evil.io/"));
    assert!(!is_covered(&youtube, "https://evil.io/youtube.com"));
    assert!(!is_covered(&youtube, "about:blank"));
}

#[test]
fn a_view_source_tab_is_never_covered() {
    let youtube = site("youtube.com");
    assert!(!is_covered(
        &youtube,
        "view-source:https://www.youtube.com/"
    ));
    assert!(!is_covered(&youtube, "chrome://youtube.com/"));
}
