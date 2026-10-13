use focus_core::{Exhausted, Rules, UnlockState};
use serde::{Deserialize, Serialize};

use super::migrate::SCHEMA_VERSION;
use super::{Snapshot, StateError};

/// The on-disk shape of [`Snapshot`], whose rules are plain until they are validated on load.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct PersistedState {
    pub schema_version: u32,
    pub rules: Rules,
    pub unlock: UnlockState,
    pub exhausted: Exhausted,
}

impl From<&Snapshot> for PersistedState {
    fn from(snapshot: &Snapshot) -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            rules: Rules::clone(&snapshot.rules),
            unlock: snapshot.unlock,
            exhausted: snapshot.exhausted.clone(),
        }
    }
}

impl TryFrom<PersistedState> for Snapshot {
    type Error = StateError;

    fn try_from(state: PersistedState) -> Result<Self, StateError> {
        Ok(Self {
            rules: state.rules.validate()?,
            unlock: state.unlock,
            exhausted: state.exhausted,
        })
    }
}
