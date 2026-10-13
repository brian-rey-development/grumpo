/// A session, or the part of it that falls inside a range.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Piece {
    pub start: i64,
    pub end: i64,
    pub app_id: String,
    pub app_name: String,
    pub domain: Option<String>,
}

impl Piece {
    pub fn seconds(&self) -> i64 {
        self.end - self.start
    }

    /// The overlap with `[start, end)`, or `None` when it is empty.
    pub fn clipped(&self, start: i64, end: i64) -> Option<Self> {
        let clipped_start = self.start.max(start);
        let clipped_end = self.end.min(end);
        (clipped_end > clipped_start).then(|| Self {
            start: clipped_start,
            end: clipped_end,
            ..self.clone()
        })
    }
}

pub fn clip_all(pieces: &[Piece], start: i64, end: i64) -> Vec<Piece> {
    pieces
        .iter()
        .filter_map(|piece| piece.clipped(start, end))
        .collect()
}

pub fn total_seconds(pieces: &[Piece]) -> u32 {
    saturating_u32(pieces.iter().map(Piece::seconds).sum())
}

pub fn saturating_u32(seconds: i64) -> u32 {
    u32::try_from(seconds.max(0)).unwrap_or(u32::MAX)
}
