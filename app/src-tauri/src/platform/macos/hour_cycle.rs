//! The 12 or 24 hour clock setting, as System Settings applies it to the current locale.

use objc2::rc::autoreleasepool;
use objc2_foundation::{ns_string, NSDateFormatter, NSLocale};

use crate::platform::{HourCycle, HourCycleSource};

/// The template for "the hour in the locale's own cycle", which expands to a pattern whose hour
/// field letter tells the cycle.
const HOUR_TEMPLATE: &str = "j";
const TWELVE_HOUR_FIELDS: [char; 2] = ['h', 'K'];
const TWENTY_FOUR_HOUR_FIELDS: [char; 2] = ['H', 'k'];
const LITERAL_QUOTE: char = '\'';

#[derive(Debug)]
pub(super) struct MacHourCycle;

impl HourCycleSource for MacHourCycle {
    fn hour_cycle(&self) -> HourCycle {
        autoreleasepool(|_| {
            let locale = NSLocale::autoupdatingCurrentLocale();
            let pattern = NSDateFormatter::dateFormatFromTemplate_options_locale(
                ns_string!(HOUR_TEMPLATE),
                0,
                Some(&locale),
            );
            pattern.map_or(HourCycle::TwentyFour, |pattern| {
                cycle_of(&pattern.to_string())
            })
        })
    }
}

fn cycle_of(pattern: &str) -> HourCycle {
    unquoted(pattern)
        .find_map(|symbol| {
            if TWELVE_HOUR_FIELDS.contains(&symbol) {
                Some(HourCycle::Twelve)
            } else if TWENTY_FOUR_HOUR_FIELDS.contains(&symbol) {
                Some(HourCycle::TwentyFour)
            } else {
                None
            }
        })
        .unwrap_or(HourCycle::TwentyFour)
}

/// The symbols of a date pattern outside quoted literals, where `''` is a literal quote.
fn unquoted(pattern: &str) -> impl Iterator<Item = char> + '_ {
    pattern
        .chars()
        .scan(false, |quoted, symbol| {
            if symbol == LITERAL_QUOTE {
                *quoted = !*quoted;
                return Some(None);
            }
            Some((!*quoted).then_some(symbol))
        })
        .flatten()
}

#[cfg(test)]
mod tests;
