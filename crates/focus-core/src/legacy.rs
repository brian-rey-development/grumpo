use std::collections::BTreeSet;
use std::iter;

use crate::{Day, Domain, Rules, Schedule, TimeOfDay, Window};

const ARRAY_OPENER: &str = "DOMAINS=(";
const WEEKDAYS: [Day; 5] = [Day::Mon, Day::Tue, Day::Wed, Day::Thu, Day::Fri];
const WEEKDAY_BLOCK_END: TimeOfDay = TimeOfDay::from_hour(19);
const SATURDAY_BLOCK_END: TimeOfDay = TimeOfDay::from_hour(13);

struct LineScan {
    entries: Vec<String>,
    closed: bool,
}

/// Reads the quoted entries of the old `focus-hosts` script's `DOMAINS=( ... )` array.
///
/// Entries are normalized and deduplicated keeping first-seen order, and invalid ones are skipped.
fn parse_domains(script: &str) -> Vec<Domain> {
    let mut seen = BTreeSet::new();
    array_entries(script)
        .iter()
        .filter_map(|entry| Domain::parse(entry).ok())
        .filter(|domain| seen.insert(domain.clone()))
        .collect()
}

/// The schedule the old script enforced: weekdays until 19:00 and Saturday until 13:00.
fn schedule() -> Schedule {
    let window = |days: &[Day], end: TimeOfDay| Window {
        days: days.iter().copied().collect(),
        start: TimeOfDay::MIDNIGHT,
        end,
    };
    Schedule {
        windows: vec![
            window(&WEEKDAYS, WEEKDAY_BLOCK_END),
            window(&[Day::Sat], SATURDAY_BLOCK_END),
        ],
    }
}

/// The old script's domains and schedule as unvalidated rules, ready for migration.
pub fn rules(script: &str) -> Rules {
    Rules {
        blocked_sites: parse_domains(script).into_iter().collect(),
        schedule: schedule(),
        ..Rules::default()
    }
}

fn array_entries(script: &str) -> Vec<String> {
    let mut lines = script
        .lines()
        .skip_while(|line| !line.trim_start().starts_with(ARRAY_OPENER));
    let Some(opener) = lines.next() else {
        return Vec::new();
    };
    let after_opener = opener
        .trim_start()
        .strip_prefix(ARRAY_OPENER)
        .unwrap_or_default();
    let mut entries = Vec::new();
    for line in iter::once(after_opener).chain(lines) {
        let scan = scan_line(line);
        entries.extend(scan.entries);
        if scan.closed {
            break;
        }
    }
    entries
}

fn scan_line(line: &str) -> LineScan {
    let mut entries = Vec::new();
    let mut quote: Option<char> = None;
    let mut text = String::new();
    for ch in line.chars() {
        match (quote, ch) {
            (Some(open), _) if ch == open => {
                entries.push(std::mem::take(&mut text));
                quote = None;
            }
            (Some(_), _) => text.push(ch),
            (None, '"' | '\'') => quote = Some(ch),
            (None, '#') => break,
            (None, ')') => {
                return LineScan {
                    entries,
                    closed: true,
                }
            }
            (None, _) => {}
        }
    }
    LineScan {
        entries,
        closed: false,
    }
}

#[cfg(test)]
mod tests;
