use std::collections::HashMap;

use super::piece::{saturating_u32, Piece};
use super::types::UsageEntry;

pub const TOP_LIST_LIMIT: usize = 10;

struct Tally {
    label: String,
    seconds: i64,
}

/// Apps ranked by time, keyed by bundle id. The label follows the most recent name seen.
pub fn top_apps(pieces: &[Piece]) -> Vec<UsageEntry> {
    rank(
        pieces
            .iter()
            .map(|p| (p.app_id.as_str(), p.app_name.as_str(), p)),
    )
}

/// Sites ranked by time, keyed and labelled by domain. Pieces without a domain are skipped.
pub fn top_sites(pieces: &[Piece]) -> Vec<UsageEntry> {
    rank(pieces.iter().filter_map(|p| {
        let domain = p.domain.as_deref()?;
        Some((domain, domain, p))
    }))
}

fn rank<'a>(items: impl Iterator<Item = (&'a str, &'a str, &'a Piece)>) -> Vec<UsageEntry> {
    let mut tallies: HashMap<&str, Tally> = HashMap::new();
    for (key, label, piece) in items {
        let tally = tallies.entry(key).or_insert_with(|| Tally {
            label: label.to_owned(),
            seconds: 0,
        });
        tally.seconds += piece.seconds();
        label.clone_into(&mut tally.label);
    }
    let mut entries: Vec<UsageEntry> = tallies
        .into_iter()
        .map(|(key, tally)| UsageEntry {
            key: key.to_owned(),
            label: tally.label,
            seconds: saturating_u32(tally.seconds),
        })
        .collect();
    entries.sort_by(|a, b| {
        b.seconds
            .cmp(&a.seconds)
            .then_with(|| a.label.cmp(&b.label))
            .then_with(|| a.key.cmp(&b.key))
    });
    entries.truncate(TOP_LIST_LIMIT);
    entries
}
