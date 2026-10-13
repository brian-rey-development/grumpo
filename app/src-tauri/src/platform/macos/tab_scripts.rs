//! `AppleScript` source for reading and closing the active tab, and the parsing of what the
//! read script prints. Every script is guarded by `is running`, so it can never launch a
//! browser, and addresses the app by bundle id.

use super::apple_script::escape_literal;
use crate::platform::Browser;

/// The host sits at the start of an address, so a longer one is cut inside the script
/// before it is printed, which bounds the work for pages with enormous URLs.
pub(super) const HOST_READ_LIMIT: usize = 4096;
pub(super) const MAX_CLOSE_URL_LEN: usize = 16 * 1024;
pub(super) const CLOSED: &str = "closed";
const INCOGNITO_MODE: &str = "incognito";

#[derive(Debug, PartialEq, Eq)]
pub(super) struct TabUrl {
    pub url: String,
    pub private: bool,
}

fn tab_expression(browser: Browser) -> &'static str {
    match browser {
        Browser::Safari => "current tab",
        _ => "active tab",
    }
}

/// Chromium windows have a `mode` of `normal` or `incognito`. Safari exposes nothing like
/// it to scripts, so its private windows cannot be told apart.
fn reports_mode(browser: Browser) -> bool {
    matches!(browser, Browser::Chrome | Browser::Brave | Browser::Edge)
}

pub(super) fn read_script(browser: Browser, max_len: usize) -> Option<String> {
    if !browser.can_close_tabs() {
        return None;
    }
    let (id, tab) = (browser.bundle_id(), tab_expression(browser));
    let output = if reports_mode(browser) {
        "(mode of front window) & linefeed & theUrl"
    } else {
        "theUrl"
    };
    Some(format!(
        r#"if application id "{id}" is running then
tell application id "{id}"
if (count of windows) is 0 then return ""
set theUrl to URL of {tab} of front window
if theUrl is missing value then return ""
if (length of theUrl) > {max_len} then set theUrl to text 1 thru {max_len} of theUrl
return {output}
end tell
end if
return """#
    ))
}

pub(super) fn parse_tab(browser: Browser, output: &str) -> Option<TabUrl> {
    if !reports_mode(browser) {
        return (!output.is_empty()).then(|| TabUrl {
            url: output.to_owned(),
            private: false,
        });
    }
    let (mode, url) = output.split_once('\n')?;
    (!url.is_empty()).then(|| TabUrl {
        url: url.to_owned(),
        private: mode.trim() == INCOGNITO_MODE,
    })
}

pub(super) fn close_script(browser: Browser, url: &str) -> Option<String> {
    if !browser.can_close_tabs() || url.len() > MAX_CLOSE_URL_LEN {
        return None;
    }
    let literal = escape_literal(url)?;
    let (id, tab) = (browser.bundle_id(), tab_expression(browser));
    Some(format!(
        r#"if application id "{id}" is running then
tell application id "{id}"
if (count of windows) is 0 then return "none"
set theTab to {tab} of front window
set isSameTab to false
considering case
set isSameTab to ((URL of theTab) is "{literal}")
end considering
if not isSameTab then return "changed"
close theTab
return "{CLOSED}"
end tell
end if
return "none""#
    ))
}

#[cfg(test)]
mod tests;
