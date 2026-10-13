use super::piece::{saturating_u32, Piece};
use crate::time::{LocalDay, HOURS_PER_DAY};

/// Buckets time by the local clock hour it fell in, using the hour starts the day carries.
/// On a 23 hour day the skipped hour stays empty, and on a 25 hour day the repeated hour
/// lands in one bucket, so the buckets sum to the day's total on every day of the year.
pub fn hourly_buckets(pieces: &[Piece], day: &LocalDay) -> [u32; HOURS_PER_DAY] {
    let bounds = day.hour_bounds();
    let mut buckets = [0_i64; HOURS_PER_DAY];
    for piece in pieces {
        spread(&mut buckets, piece, &bounds);
    }
    buckets.map(saturating_u32)
}

fn spread(buckets: &mut [i64; HOURS_PER_DAY], piece: &Piece, bounds: &[i64; HOURS_PER_DAY + 1]) {
    let mut cursor = piece.start;
    while cursor < piece.end {
        let index = bucket_index(bounds, cursor);
        let chunk_end = piece.end.min(bounds[index + 1]);
        if chunk_end <= cursor {
            return;
        }
        buckets[index] += chunk_end - cursor;
        cursor = chunk_end;
    }
}

/// The last bucket that has started by `at`, which skips zero-length buckets because the
/// bucket after them starts at the same instant.
fn bucket_index(bounds: &[i64; HOURS_PER_DAY + 1], at: i64) -> usize {
    bounds[..HOURS_PER_DAY]
        .partition_point(|&hour_start| hour_start <= at)
        .saturating_sub(1)
}
