use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use super::{DaemonStatus, IpcError};
use crate::rules::{LimitTarget, Rules};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase",
    deny_unknown_fields
)]
pub enum Request {
    Status,
    UpdateRules {
        rules: Rules,
    },
    RequestUnlock,
    ConfirmUnlock {
        phrase: String,
        minutes: u16,
    },
    CancelUnlock,
    Relock,
    /// `daily_minutes` is the allowance that ran out, so the daemon can tell a stale report
    /// from one that still matches the rules. `day` is the reporter's local day, so a report
    /// that reaches the daemon after midnight cannot block the new day.
    ReportLimitReached {
        target: LimitTarget,
        daily_minutes: u16,
        day: NaiveDate,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
#[expect(
    clippy::large_enum_variant,
    reason = "one short-lived value per connection, boxing would only burden callers"
)]
pub enum Response {
    Ok { status: DaemonStatus },
    Err { error: IpcError },
}

impl Response {
    pub fn into_result(self) -> Result<DaemonStatus, IpcError> {
        match self {
            Self::Ok { status } => Ok(status),
            Self::Err { error } => Err(error),
        }
    }
}

#[cfg(test)]
mod tests;
