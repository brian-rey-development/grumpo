use serde::Serialize;

/// The id of a block that the hub has not presented yet.
pub const UNASSIGNED_ID: u64 = 0;

/// What the block screen window shows. `id` counts every show, starting at 1, so the page can
/// tell a new block from a repeat and hand the same id back when it dismisses. Whoever builds
/// a block leaves it at `UNASSIGNED_ID`, and the hub stamps it. `until` is when the block ends,
/// in unix seconds, and null when it never does. `strict` is true while loosening is locked,
/// so leaving the block early needs the cooldown unlock.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub struct BlockScreen {
    #[cfg_attr(feature = "ts", ts(type = "number"))]
    pub id: u64,
    pub target: BlockTarget,
    pub reason: BlockReason,
    pub until: Option<i64>,
    pub strict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(
    tag = "kind",
    rename_all = "camelCase",
    rename_all_fields = "camelCase"
)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum BlockTarget {
    Site { domain: String },
    App { name: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export))]
pub enum BlockReason {
    /// A focus window is in force and ends at `until`.
    Schedule,
    /// The daily allowance is spent and the block lifts at `until`, the coming midnight.
    Limit,
    /// A schedule that never ends, so there is no `until`.
    Always,
}

#[cfg(test)]
mod tests;
