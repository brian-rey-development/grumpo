mod strip;

use std::collections::BTreeSet;

use crate::Domain;

pub(crate) const MARKER_START: &str = "# >>> grumpo: managed block, do not edit >>>";
pub(crate) const MARKER_END: &str = "# <<< grumpo: managed block <<<";
const IPV4_SINK: &str = "0.0.0.0";
const IPV6_SINK: &str = "::";
const MAX_HOSTNAME_LEN: usize = 253;
/// The hostnames written for each blocked domain, as prefixes of it, because a hosts file has
/// no wildcards. Each prefix is listed for IPv4 and for IPv6. They are the subdomains sites
/// serve the same content from, which a browser would otherwise reach around the block.
pub const ALIAS_PREFIXES: [&str; 9] = [
    "", "www.", "m.", "mobile.", "old.", "new.", "music.", "i.", "amp.",
];

/// Rewrites a hosts file so it holds exactly one managed block for `sites`, or none when empty.
///
/// Works on bytes because a hosts file may hold a legacy encoding in comments, and refusing
/// to touch it would leave blocking off for good. The markers and generated lines are ASCII,
/// so foreign lines are kept verbatim whatever their encoding.
///
/// Current and legacy blocks are removed when properly closed by a marker of the same kind,
/// a lone marker line is removed by itself, and all other content is kept as is apart from
/// ending in one newline.
pub fn render_bytes(existing: &[u8], sites: &BTreeSet<Domain>) -> Vec<u8> {
    let lines: Vec<&[u8]> = existing.split(|byte| *byte == b'\n').collect();
    let mut content = strip::managed_blocks(&lines).join(&b'\n');
    let kept = content.iter().rposition(|byte| *byte != b'\n');
    content.truncate(kept.map_or(0, |last| last + 1));
    if !content.is_empty() {
        content.push(b'\n');
    }
    if !sites.is_empty() {
        content.extend_from_slice(managed_block(sites).as_bytes());
    }
    content
}

/// Every hostname the managed block lists for `sites`, which a hosts file needs spelled out
/// because it has no wildcards. An alias too long to be a hostname is left out, which only
/// happens to sites near the length cap.
pub(crate) fn blocked_hosts(sites: &BTreeSet<Domain>) -> BTreeSet<String> {
    sites
        .iter()
        .flat_map(|site| {
            ALIAS_PREFIXES
                .iter()
                .map(move |prefix| format!("{prefix}{site}"))
        })
        .filter(|host| host.len() <= MAX_HOSTNAME_LEN)
        .collect()
}

fn managed_block(sites: &BTreeSet<Domain>) -> String {
    let mut lines = vec![MARKER_START.to_owned()];
    for host in blocked_hosts(sites) {
        lines.push(format!("{IPV4_SINK} {host}"));
        lines.push(format!("{IPV6_SINK} {host}"));
    }
    lines.push(MARKER_END.to_owned());
    lines.join("\n") + "\n"
}

#[cfg(test)]
mod tests;
