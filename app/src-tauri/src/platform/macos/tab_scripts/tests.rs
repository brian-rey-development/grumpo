use super::*;

fn read(browser: Browser) -> String {
    read_script(browser, HOST_READ_LIMIT).unwrap()
}

#[test]
fn chromium_browsers_read_the_active_tab_and_window_mode() {
    for browser in [Browser::Chrome, Browser::Brave, Browser::Edge] {
        let script = read(browser);
        assert!(script.contains("URL of active tab of front window"));
        assert!(script.contains("mode of front window"));
        assert!(script.contains(&format!(r#"application id "{}""#, browser.bundle_id())));
    }
}

#[test]
fn safari_reads_the_current_tab_without_a_mode() {
    let script = read(Browser::Safari);
    assert!(script.contains("URL of current tab of front window"));
    assert!(!script.contains("active tab"));
    assert!(!script.contains("mode of"));
}

#[test]
fn long_addresses_are_cut_inside_the_script() {
    let script = read_script(Browser::Chrome, 4096).unwrap();
    assert!(script.contains("if (length of theUrl) > 4096 then set theUrl to text 1 thru 4096"));
}

#[test]
fn scripts_are_guarded_and_never_address_by_name() {
    for browser in Browser::ALL.into_iter().filter(|b| b.can_close_tabs()) {
        for script in [
            read(browser),
            close_script(browser, "https://a.com/").unwrap(),
        ] {
            assert!(script.starts_with("if application id"));
            assert!(script.contains("is running then"));
            assert!(script.contains("count of windows) is 0"));
            assert!(!script.contains("tell application \""));
        }
    }
}

#[test]
fn firefox_has_no_scripts() {
    assert_eq!(read_script(Browser::Firefox, HOST_READ_LIMIT), None);
    assert_eq!(close_script(Browser::Firefox, "https://a.com/"), None);
}

#[test]
fn chromium_output_carries_the_window_mode() {
    let tab = |mode: &str| TabUrl {
        url: "https://a.com/x".to_owned(),
        private: mode == "incognito",
    };
    for mode in ["normal", "incognito"] {
        let output = format!("{mode}\nhttps://a.com/x");
        assert_eq!(parse_tab(Browser::Chrome, &output), Some(tab(mode)));
    }
    assert_eq!(parse_tab(Browser::Brave, "normal"), None);
    assert_eq!(parse_tab(Browser::Edge, "incognito\n"), None);
    assert_eq!(parse_tab(Browser::Chrome, ""), None);
}

#[test]
fn safari_output_is_the_address_and_never_private() {
    assert_eq!(
        parse_tab(Browser::Safari, "https://a.com/"),
        Some(TabUrl {
            url: "https://a.com/".to_owned(),
            private: false
        })
    );
    assert_eq!(parse_tab(Browser::Safari, ""), None);
}

#[test]
fn close_script_compares_the_exact_url_before_closing() {
    let script = close_script(Browser::Chrome, r#"https://a.com/?q="x""#).unwrap();
    assert!(script.contains(r#"is "https://a.com/?q=\"x\"")"#));
    let compare = script.find("considering case").unwrap();
    let close = script.find("close theTab").unwrap();
    assert!(compare < close);
}

#[test]
fn close_script_refuses_unsafe_or_oversized_urls() {
    assert_eq!(close_script(Browser::Chrome, "https://a.com/\u{7}"), None);
    let long = format!("https://a.com/{}", "a".repeat(MAX_CLOSE_URL_LEN));
    assert_eq!(close_script(Browser::Chrome, &long), None);
}
