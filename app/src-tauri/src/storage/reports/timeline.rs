use std::collections::BTreeMap;

use super::piece::Piece;
use super::types::Segment;

/// The ribbon is drawn on a fixed grid, one cell per slot.
pub const SLOT_SECONDS: i64 = 60;
/// A slot whose busiest target was used less than this stays empty, so a glance at an app
/// does not paint a cell, while steady switching between two apps still does.
pub const MIN_SLOT_USAGE_SECONDS: i64 = 10;

struct Tally {
    app_id: String,
    app_name: String,
    domain: Option<String>,
    seconds: i64,
    first_start: i64,
}

impl Tally {
    fn is_for(&self, piece: &Piece) -> bool {
        self.app_id == piece.app_id && self.domain == piece.domain
    }
}

/// Picks the dominant target of every slot of the day, leaves quiet slots empty and joins
/// neighbouring slots that picked the same target into segments. Choosing per slot instead
/// of merging pieces first means rapid alternation between two targets still yields a
/// ribbon, with each minute shown as whichever target held most of it.
pub fn build_timeline(pieces: &[Piece], day_start: i64, day_end: i64) -> Vec<Segment> {
    let mut segments: Vec<Segment> = Vec::new();
    let mut previous_slot = None;
    for (slot, tallies) in slot_tallies(pieces, day_start) {
        let Some(winner) = dominant(&tallies) else {
            continue;
        };
        let start = day_start + slot * SLOT_SECONDS;
        let end = (start + SLOT_SECONDS).min(day_end);
        match segments.last_mut() {
            Some(last) if previous_slot == Some(slot - 1) && is_same_target(last, winner) => {
                last.end = end;
            }
            _ => segments.push(segment_of(winner, start, end)),
        }
        previous_slot = Some(slot);
    }
    segments
}

fn slot_tallies(pieces: &[Piece], day_start: i64) -> BTreeMap<i64, Vec<Tally>> {
    let mut slots: BTreeMap<i64, Vec<Tally>> = BTreeMap::new();
    for piece in pieces.iter().filter(|piece| piece.end > piece.start) {
        let first = (piece.start - day_start).div_euclid(SLOT_SECONDS);
        let last = (piece.end - 1 - day_start).div_euclid(SLOT_SECONDS);
        for slot in first..=last {
            let slot_start = day_start + slot * SLOT_SECONDS;
            let overlap = piece.end.min(slot_start + SLOT_SECONDS) - piece.start.max(slot_start);
            add(slots.entry(slot).or_default(), piece, overlap);
        }
    }
    slots
}

fn add(tallies: &mut Vec<Tally>, piece: &Piece, seconds: i64) {
    match tallies.iter_mut().find(|tally| tally.is_for(piece)) {
        Some(tally) => {
            tally.seconds += seconds;
            tally.first_start = tally.first_start.min(piece.start);
        }
        None => tallies.push(Tally {
            app_id: piece.app_id.clone(),
            app_name: piece.app_name.clone(),
            domain: piece.domain.clone(),
            seconds,
            first_start: piece.start,
        }),
    }
}

/// Most seconds wins, and a tie goes to the target that started first.
fn dominant(tallies: &[Tally]) -> Option<&Tally> {
    tallies
        .iter()
        .max_by_key(|tally| (tally.seconds, std::cmp::Reverse(tally.first_start)))
        .filter(|tally| tally.seconds >= MIN_SLOT_USAGE_SECONDS)
}

fn is_same_target(segment: &Segment, tally: &Tally) -> bool {
    segment.app_id == tally.app_id && segment.domain == tally.domain
}

fn segment_of(tally: &Tally, start: i64, end: i64) -> Segment {
    Segment {
        start,
        end,
        app_id: tally.app_id.clone(),
        app_name: tally.app_name.clone(),
        domain: tally.domain.clone(),
    }
}
