use super::{Domain, HOST_SUFFIX_TERMINATORS, SCHEME_SEPARATOR};

const WEB_SCHEMES: [&str; 2] = ["http", "https"];
const USERINFO_SEPARATOR: char = '@';

enum Address<'a> {
    Web(&'a str),
    Bare(&'a str),
    Other,
}

impl Domain {
    /// Reads the host of an address a browser shows or reports, which is an `http` or
    /// `https` URL or a scheme-less one such as `example.com/path`. Any other scheme
    /// (`view-source:`, `chrome:`, `about:`, `file:`, `mailto:`), search text, IP addresses
    /// and hosts with fewer than two labels yield `None`, so internal pages never become a
    /// site and a prefix can never smuggle a different host past a rule.
    pub fn from_address(address: &str) -> Option<Self> {
        let (rest, has_scheme) = match classify(address.trim()) {
            Address::Web(rest) => (rest, true),
            Address::Bare(rest) => (rest, false),
            Address::Other => return None,
        };
        let authority = rest.split(HOST_SUFFIX_TERMINATORS).next()?;
        if !has_scheme && authority.contains(USERINFO_SEPARATOR) {
            return None;
        }
        Self::parse(authority).ok()
    }
}

fn classify(address: &str) -> Address<'_> {
    match address.split_once(SCHEME_SEPARATOR) {
        Some((scheme, rest)) if is_scheme_token(scheme) => {
            if WEB_SCHEMES
                .iter()
                .any(|web| scheme.eq_ignore_ascii_case(web))
            {
                Address::Web(rest)
            } else {
                Address::Other
            }
        }
        _ => Address::Bare(address),
    }
}

fn is_scheme_token(candidate: &str) -> bool {
    let mut chars = candidate.chars();
    chars
        .next()
        .is_some_and(|first| first.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.'))
}

#[cfg(test)]
mod tests;
