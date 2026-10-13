use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize};
use thiserror::Error;

const MAX_BUNDLE_ID_LEN: usize = 255;
const MAX_NAME_LEN: usize = 255;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AppRefError {
    #[error("the app bundle id is empty")]
    EmptyBundleId,
    #[error("the app bundle id is longer than {max} characters", max = MAX_BUNDLE_ID_LEN)]
    BundleIdTooLong,
    #[error("the character {0:?} is not allowed in an app bundle id")]
    InvalidBundleIdCharacter(char),
    #[error("the app name is longer than {max} characters", max = MAX_NAME_LEN)]
    NameTooLong,
    #[error("the app name contains the control character {0:?}")]
    ControlCharacterInName(char),
}

/// An application identified by its bundle id, with a display name for the UI.
///
/// Equality, ordering and hashing use the bundle id alone, so a renamed app is the same app.
/// A blank name falls back to the bundle id so the UI always has something to show.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct AppRef {
    bundle_id: String,
    name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct RawAppRef {
    bundle_id: String,
    name: String,
}

impl AppRef {
    pub fn new(bundle_id: impl Into<String>, name: impl Into<String>) -> Result<Self, AppRefError> {
        let bundle_id = bundle_id.into().trim().to_owned();
        validate_bundle_id(&bundle_id)?;
        let name = name.into().trim().to_owned();
        validate_name(&name)?;
        let name = if name.is_empty() {
            bundle_id.clone()
        } else {
            name
        };
        Ok(Self { bundle_id, name })
    }

    pub fn bundle_id(&self) -> &str {
        &self.bundle_id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}

fn validate_bundle_id(bundle_id: &str) -> Result<(), AppRefError> {
    if bundle_id.is_empty() {
        return Err(AppRefError::EmptyBundleId);
    }
    if bundle_id.chars().count() > MAX_BUNDLE_ID_LEN {
        return Err(AppRefError::BundleIdTooLong);
    }
    let is_allowed = |c: char| c.is_ascii_alphanumeric() || c == '.' || c == '-';
    match bundle_id.chars().find(|&c| !is_allowed(c)) {
        Some(invalid) => Err(AppRefError::InvalidBundleIdCharacter(invalid)),
        None => Ok(()),
    }
}

fn validate_name(name: &str) -> Result<(), AppRefError> {
    if name.chars().count() > MAX_NAME_LEN {
        return Err(AppRefError::NameTooLong);
    }
    match name.chars().find(|c| c.is_control()) {
        Some(control) => Err(AppRefError::ControlCharacterInName(control)),
        None => Ok(()),
    }
}

impl<'de> Deserialize<'de> for AppRef {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RawAppRef::deserialize(deserializer)?;
        Self::new(raw.bundle_id, raw.name).map_err(D::Error::custom)
    }
}

impl PartialEq for AppRef {
    fn eq(&self, other: &Self) -> bool {
        self.bundle_id == other.bundle_id
    }
}

impl Eq for AppRef {}

impl Hash for AppRef {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.bundle_id.hash(state);
    }
}

impl PartialOrd for AppRef {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AppRef {
    fn cmp(&self, other: &Self) -> Ordering {
        self.bundle_id.cmp(&other.bundle_id)
    }
}

impl fmt::Display for AppRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[cfg(test)]
mod tests;
