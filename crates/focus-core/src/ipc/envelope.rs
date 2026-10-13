use serde::{Deserialize, Serialize};

use super::PROTOCOL_VERSION;

/// Wraps every message in both directions so the peer can check the version before the body.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct Envelope<T> {
    pub protocol_version: u32,
    pub body: T,
}

impl<T> Envelope<T> {
    pub(super) fn new(body: T) -> Self {
        Self {
            protocol_version: PROTOCOL_VERSION,
            body,
        }
    }
}
