use super::{MARKER_END, MARKER_START};

const CURRENT_START: &[u8] = MARKER_START.as_bytes();
const CURRENT_END: &[u8] = MARKER_END.as_bytes();
const LEGACY_START: &[u8] = b"# FOCUS-HOSTS START";
const LEGACY_END: &[u8] = b"# FOCUS-HOSTS END";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Current,
    Legacy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Line<'a> {
    Start(Kind),
    End(Kind),
    /// The legacy start marker appended with `tee -a` to a file lacking a final newline,
    /// so it follows the last line's text instead of starting its own line.
    GluedStart {
        prefix: &'a [u8],
    },
    Content,
}

impl Line<'_> {
    fn opens(self) -> Option<Kind> {
        match self {
            Self::Start(kind) => Some(kind),
            Self::GluedStart { .. } => Some(Kind::Legacy),
            Self::End(_) | Self::Content => None,
        }
    }
}

fn classify(line: &[u8]) -> Line<'_> {
    match line.trim_ascii() {
        CURRENT_START => Line::Start(Kind::Current),
        LEGACY_START => Line::Start(Kind::Legacy),
        CURRENT_END => Line::End(Kind::Current),
        LEGACY_END => Line::End(Kind::Legacy),
        _ => line
            .trim_ascii_end()
            .strip_suffix(LEGACY_START)
            .map_or(Line::Content, |prefix| Line::GluedStart { prefix }),
    }
}

/// Drops every closed marker block, and any marker line left without a partner, keeping the rest.
pub(super) fn managed_blocks<'a>(lines: &[&'a [u8]]) -> Vec<&'a [u8]> {
    let mut kept = Vec::with_capacity(lines.len());
    let mut index = 0;
    while let Some(&line) = lines.get(index) {
        let classified = classify(line);
        let end = classified
            .opens()
            .and_then(|kind| closing(lines, index, kind));
        match (classified, end) {
            (Line::GluedStart { prefix }, Some(end)) => {
                kept.push(prefix);
                index = end + 1;
            }
            (Line::GluedStart { .. } | Line::Content, _) => {
                kept.push(line);
                index += 1;
            }
            (_, Some(end)) => index = end + 1,
            (_, None) => index += 1,
        }
    }
    kept
}

/// Index of the end marker of `kind` closing the block opened at `start`, unless another
/// start of that kind intervenes.
fn closing(lines: &[&[u8]], start: usize, kind: Kind) -> Option<usize> {
    lines
        .iter()
        .enumerate()
        .skip(start + 1)
        .find_map(|(index, line)| match classify(line) {
            Line::End(found) if found == kind => Some(Some(index)),
            line if line.opens() == Some(kind) => Some(None),
            _ => None,
        })
        .flatten()
}
