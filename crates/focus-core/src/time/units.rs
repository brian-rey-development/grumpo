//! The unit conversions every layer shares, so no module redefines them.

pub const SECONDS_PER_MINUTE: u32 = 60;
pub const MINUTES_PER_HOUR: u32 = 60;
pub const HOURS_PER_DAY: u32 = 24;
pub const MINUTES_PER_DAY: u32 = MINUTES_PER_HOUR * HOURS_PER_DAY;
pub const DAYS_PER_WEEK: usize = 7;
pub const SECONDS_PER_HOUR: i64 = 3_600;
pub const SECONDS_PER_DAY: i64 = 86_400;

/// Narrows one of the constants above to the `u16` that minutes of the day are stored in,
/// which fails the build when it does not fit.
#[expect(
    clippy::cast_possible_truncation,
    reason = "the assertion on the line above rules out truncation"
)]
pub(crate) const fn as_u16(value: u32) -> u16 {
    assert!(value >> u16::BITS == 0, "value does not fit in a u16");
    value as u16
}

#[cfg(test)]
mod tests;
