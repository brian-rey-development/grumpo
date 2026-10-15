mod strip;

use std::collections::BTreeSet;

use crate::Domain;

pub(crate) const MARKER_START: &str = "# >>> grumpo: managed block, do not edit >>>";
pub(crate) const MARKER_END: &str = "# <<< grumpo: managed block <<<";
const IPV4_SINK: &str = "0.0.0.0";
const IPV6_SINK: &str = "::";
/// The aliases every site is also written for, because a hosts file has no wildcards and most
/// sites serve the same pages from these subdomains, which a browser would otherwise reach
/// around the block.
const COMMON_ALIASES: [&str; 2] = ["www.", "m."];
/// Aliases that only these sites serve their pages from.
const SITE_ALIASES: [(&str, &[&str]); 4] = [
    ("facebook.com", &["mobile."]),
    ("reddit.com", &["old.", "new.", "i."]),
    ("twitter.com", &["mobile."]),
    ("x.com", &["mobile."]),
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

/// The sites a closed managed block in `existing` blocks, so a daemon that lost its rules can
/// keep them in force. Aliases are folded back into the site that lists them, which renders
/// the same block again. Legacy blocks and a block that never closes are not read, because
/// rendering does not keep them either.
pub fn managed_sites(existing: &[u8]) -> BTreeSet<Domain> {
    let hosts: BTreeSet<Domain> = managed_lines(existing)
        .into_iter()
        .filter_map(sink_host)
        .collect();
    let is_alias = |host: &Domain| hosts.iter().any(|site| site != host && blocks(site, host));
    hosts
        .iter()
        .filter(|host| !is_alias(host))
        .cloned()
        .collect()
}

/// The lines inside closed current-marker blocks. A start marker met while a block is open
/// abandons it, the way [`render_bytes`] treats the earlier marker as a lone line.
fn managed_lines(existing: &[u8]) -> Vec<&[u8]> {
    let mut closed = Vec::new();
    let mut open: Option<Vec<&[u8]>> = None;
    for line in existing
        .split(|byte| *byte == b'\n')
        .map(<[u8]>::trim_ascii)
    {
        match (line, open.as_mut()) {
            (start, _) if start == MARKER_START.as_bytes() => open = Some(Vec::new()),
            (end, Some(_)) if end == MARKER_END.as_bytes() => {
                closed.extend(open.take().unwrap_or_default());
            }
            (content, Some(lines)) => lines.push(content),
            (_, None) => {}
        }
    }
    closed
}

fn sink_host(line: &[u8]) -> Option<Domain> {
    let rest = line.strip_prefix(IPV4_SINK.as_bytes())?;
    if !rest.first()?.is_ascii_whitespace() {
        return None;
    }
    Domain::parse(std::str::from_utf8(rest).ok()?).ok()
}

/// Every hostname the managed block lists for `sites`, which a hosts file needs spelled out
/// because it has no wildcards.
pub(crate) fn blocked_hosts(sites: &BTreeSet<Domain>) -> BTreeSet<String> {
    sites.iter().flat_map(hosts_of).collect()
}

/// True when blocking `site` blocks `host`, which is the site itself or one of its aliases. Other
/// subdomains stay reachable, so anything that acts on a visited host has to agree with this.
pub fn blocks(site: &Domain, host: &Domain) -> bool {
    hosts_of(site)
        .iter()
        .any(|blocked| blocked == host.as_str())
}

/// A site and its aliases. Only a registrable domain gets aliases, because a subdomain such as
/// `m.youtube.com` or `store.steampowered.com` has none of its own.
pub fn hosts_of(site: &Domain) -> Vec<String> {
    let mut hosts = vec![site.to_string()];
    if site.is_registrable() {
        hosts.extend(alias_prefixes(site.as_str()).map(|prefix| format!("{prefix}{site}")));
    }
    hosts
}

fn alias_prefixes(site: &str) -> impl Iterator<Item = &'static str> + '_ {
    let own = SITE_ALIASES
        .iter()
        .filter(move |(domain, _)| *domain == site)
        .flat_map(|(_, prefixes)| prefixes.iter().copied());
    COMMON_ALIASES.into_iter().chain(own)
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
