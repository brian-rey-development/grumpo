//! Runs `AppleScript` through `/usr/bin/osascript` under a hard deadline and maps its failures
//! onto the platform error model.

use std::process::Command;
use std::time::Duration;

use super::consent;
use super::os_status::{ERR_CANT_GET, ERR_INVALID_INDEX, ERR_NOT_PERMITTED, PROC_NOT_FOUND};
use super::process::{run_bounded, RunError};
use crate::platform::{Browser, Permission, PlatformError};

pub(super) const SCRIPT_TIMEOUT: Duration = Duration::from_millis(1500);
pub(super) const OSASCRIPT: &str = "/usr/bin/osascript";
const OPERATION: &str = "osascript";

const ERR_CODE_MARKER: &str = "(-";

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ScriptFailure {
    Timeout,
    Rejected { code: Option<i32> },
    Io(String),
}

/// Runs `script`, one `-e` argument per line, and returns trimmed stdout.
pub(super) fn run(script: &str) -> Result<String, ScriptFailure> {
    let mut command = Command::new(OSASCRIPT);
    for line in script.lines() {
        command.arg("-e").arg(line);
    }
    let output = run_bounded(&mut command, SCRIPT_TIMEOUT).map_err(|error| match error {
        RunError::Timeout => ScriptFailure::Timeout,
        RunError::Spawn(error) | RunError::Wait(error) => ScriptFailure::Io(error.to_string()),
    })?;
    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned());
    }
    let stderr = String::from_utf8_lossy(&output.stderr);
    Err(ScriptFailure::Rejected {
        code: error_code(&stderr),
    })
}

/// Runs a script aimed at `browser`, but only once Automation consent is granted, because
/// otherwise the script itself would make macOS show the consent dialog. `Ok(None)` means
/// there was nothing to read: the browser is not running or has no usable window or tab.
pub(super) fn run_for_browser(
    browser: Browser,
    script: &str,
) -> Result<Option<String>, PlatformError> {
    consent::require(browser)?;
    interpret(browser, run(script))
}

fn interpret(
    browser: Browser,
    result: Result<String, ScriptFailure>,
) -> Result<Option<String>, PlatformError> {
    match result {
        Ok(output) => Ok(Some(output)),
        Err(ScriptFailure::Timeout) => Err(PlatformError::Timeout {
            operation: OPERATION,
        }),
        Err(ScriptFailure::Io(detail)) => Err(super::failed(OPERATION, detail)),
        Err(ScriptFailure::Rejected { code }) => match code {
            Some(ERR_NOT_PERMITTED) => Err(PlatformError::PermissionDenied(
                Permission::Automation(browser),
            )),
            Some(PROC_NOT_FOUND | ERR_CANT_GET | ERR_INVALID_INDEX) => Ok(None),
            Some(code) => Err(super::failed(OPERATION, format!("script error {code}"))),
            None => Err(super::failed(
                OPERATION,
                "script failed without an error code",
            )),
        },
    }
}

/// osascript ends its diagnostics with the `AppleScript` error number, as in
/// `... not authorized to send Apple events to Google Chrome. (-1743)`.
fn error_code(stderr: &str) -> Option<i32> {
    let tail = &stderr[stderr.rfind(ERR_CODE_MARKER)? + 1..];
    let digits_end = tail.find(')')?;
    tail[..digits_end].parse().ok()
}

/// Escapes `value` for use inside an `AppleScript` string literal. Control characters other
/// than the three with escape sequences cannot be represented on a `-e` line, so they are
/// refused instead of altered.
pub(super) fn escape_literal(value: &str) -> Option<String> {
    let mut escaped = String::with_capacity(value.len());
    for c in value.chars() {
        match c {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => return None,
            c => escaped.push(c),
        }
    }
    Some(escaped)
}

#[cfg(test)]
mod tests;
