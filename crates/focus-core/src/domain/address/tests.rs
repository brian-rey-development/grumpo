use super::*;

fn host(address: &str) -> Option<String> {
    Domain::from_address(address).map(String::from)
}

#[test]
fn reads_the_host_of_web_urls() {
    let cases = [
        ("https://www.YouTube.com/watch?v=abc#t=1", "youtube.com"),
        ("http://example.com", "example.com"),
        ("HTTPS://Example.com:8443/x", "example.com"),
        ("https://user:pw@example.com/a", "example.com"),
        ("https://example.com./", "example.com"),
        ("https://example.com?q=1", "example.com"),
        ("  https://docs.rs/x  ", "docs.rs"),
        ("https://example.com/my page", "example.com"),
    ];
    for (input, expected) in cases {
        assert_eq!(host(input).as_deref(), Some(expected), "{input}");
    }
}

#[test]
fn reads_scheme_less_addresses() {
    let cases = [
        ("example.com", "example.com"),
        ("www.example.com/a/b?c", "example.com"),
        ("sub.example.co.uk:81", "sub.example.co.uk"),
        ("example.com/redirect?u=http://other.org", "example.com"),
    ];
    for (input, expected) in cases {
        assert_eq!(host(input).as_deref(), Some(expected), "{input}");
    }
}

#[test]
fn rejects_other_schemes_and_internal_pages() {
    let cases = [
        "view-source:https://example.com",
        "chrome://settings",
        "chrome://flags.example.com",
        "about:blank",
        "file:///Users/me/a.html",
        "file://host.example.com/x",
        "ftp://files.example.com",
        "ws://example.com",
        "blob:https://example.com/uuid",
        "javascript:alert(1)",
        "data:text/html,hello",
        "mailto:me@example.com",
    ];
    for input in cases {
        assert_eq!(host(input), None, "{input:?}");
    }
}

#[test]
fn rejects_search_text_and_malformed_hosts() {
    let cases = [
        "",
        "   ",
        "localhost:3000",
        "http://localhost:3000/",
        "hello world",
        "rust lifetimes",
        "example.com is great",
        "https://[::1]/",
        "http://192.168.0.1/admin",
        "https://",
        "https:///path",
        "https://exa mple.com",
        "https://exa\nmple.com",
        "https://日本.jp/",
        "user@example.com",
    ];
    for input in cases {
        assert_eq!(host(input), None, "{input:?}");
    }
}

#[test]
fn userinfo_cannot_spoof_the_host() {
    assert_eq!(
        host("https://good.com@evil.com/").as_deref(),
        Some("evil.com")
    );
    assert_eq!(
        host("https://evil.com/@good.com").as_deref(),
        Some("evil.com")
    );
}
