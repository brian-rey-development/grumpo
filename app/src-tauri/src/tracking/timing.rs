//! Cadence of the sampler and the longest silence between two samples that still counts as
//! one continuous run.

use std::time::Duration;

const SAMPLE_INTERVAL_SECONDS: i64 = 2;
/// Worst case time the probes of one tick take: a tab read is bounded by the 1.5 s script
/// timeout (or the 1.2 s Firefox budget) and the foreground, idle, lock and media reads add
/// well under a second. Sinks run on the sampler thread too, so they must not block and
/// consume none of this budget.
const PROBE_BUDGET_SECONDS: i64 = 4;

pub const SAMPLE_INTERVAL: Duration = Duration::from_secs(SAMPLE_INTERVAL_SECONDS.unsigned_abs());

/// Sleep and wake produce a larger gap, which closes the session at its last seen time.
pub const MAX_SAMPLE_GAP: i64 = SAMPLE_INTERVAL_SECONDS + PROBE_BUDGET_SECONDS;
