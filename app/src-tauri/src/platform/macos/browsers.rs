//! Reading and closing the active tab of the scriptable browsers (Chromium family, Safari).

use focus_core::Domain;

use super::apple_script::run_for_browser;
use super::control::is_running;
use super::tab_scripts::{
    close_script, parse_tab, read_script, TabUrl, CLOSED, HOST_READ_LIMIT, MAX_CLOSE_URL_LEN,
};
use crate::platform::{Browser, PlatformError, TabReading};

pub(super) fn active_tab(browser: Browser) -> Result<TabReading, PlatformError> {
    let Some(tab) = read_tab(browser, HOST_READ_LIMIT)? else {
        return Ok(TabReading::default());
    };
    Ok(TabReading {
        domain: Domain::from_address(&tab.url),
        private: tab.private,
    })
}

/// Closes the active tab when its host is `domain` or a subdomain. The script repeats the
/// comparison against the exact URL just read, so a tab switch in between closes nothing.
pub(super) fn close_active_tab_if(
    browser: Browser,
    domain: &Domain,
) -> Result<bool, PlatformError> {
    let Some(tab) = read_tab(browser, MAX_CLOSE_URL_LEN)? else {
        return Ok(false);
    };
    if !is_covered(domain, &tab.url) {
        return Ok(false);
    }
    let Some(script) = close_script(browser, &tab.url) else {
        tracing::debug!("active tab address cannot be embedded safely, not closing");
        return Ok(false);
    };
    Ok(run_for_browser(browser, &script)?.as_deref() == Some(CLOSED))
}

fn read_tab(browser: Browser, max_len: usize) -> Result<Option<TabUrl>, PlatformError> {
    let Some(script) = read_script(browser, max_len) else {
        return Ok(None);
    };
    if !is_running(browser.bundle_id()) {
        return Ok(None);
    }
    let output = run_for_browser(browser, &script)?;
    Ok(output.and_then(|output| parse_tab(browser, &output)))
}

fn is_covered(domain: &Domain, url: &str) -> bool {
    Domain::from_address(url).is_some_and(|tab_domain| domain.covers(&tab_domain))
}

#[cfg(test)]
mod tests;
