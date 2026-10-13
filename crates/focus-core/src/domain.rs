use std::fmt;
use std::str::FromStr;

use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;

mod address;

/// 253 minus the 4 characters of `www.`, so a domain this long can still be reached through
/// that alias. Longer aliases such as `mobile.` are skipped for domains near the cap, because
/// lowering it would make saved state fail validation.
pub const MAX_DOMAIN_LEN: usize = 249;
pub const MAX_LABEL_LEN: usize = 63;
const MIN_LABELS: usize = 2;
const SCHEME_SEPARATOR: &str = "://";
const WWW_PREFIX: &str = "www.";
const HOST_SUFFIX_TERMINATORS: [char; 3] = ['/', '?', '#'];
const SECOND_LEVEL_LABELS: [&str; 7] = ["co", "com", "net", "org", "gov", "edu", "ac"];
const COUNTRY_CODE_LEN: usize = 2;

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DomainError {
    #[error("the domain is empty")]
    Empty,
    #[error("the domain must be ASCII, write internationalized names as punycode (xn--...)")]
    NonAscii,
    #[error("the domain is longer than {max} characters", max = MAX_DOMAIN_LEN)]
    TooLong,
    #[error("the domain needs at least {min} labels, like example.com", min = MIN_LABELS)]
    TooFewLabels,
    #[error("the domain has an empty label, check for doubled dots")]
    EmptyLabel,
    #[error("the last label of a domain cannot be a number, so IP addresses are not accepted")]
    IpAddress,
    #[error(
        "\"{0}\" is a public suffix shared by many sites, block a specific site under it instead"
    )]
    PublicSuffix(String),
    #[error("the label \"{0}\" is longer than {max} characters", max = MAX_LABEL_LEN)]
    LabelTooLong(String),
    #[error("the label \"{0}\" starts or ends with a hyphen")]
    HyphenAtEdge(String),
    #[error("the character {0:?} is not allowed in a domain")]
    InvalidCharacter(char),
}

/// A normalized registrable-style hostname such as `youtube.com`.
///
/// One domain covers another when it is the same or a subdomain of it, so `youtube.com`
/// covers `m.youtube.com` while `notyoutube.com` is not covered.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "ts", derive(ts_rs::TS), ts(export, type = "string"))]
pub struct Domain(String);

impl Domain {
    /// Accepts a bare host or a full URL and reduces it to a lowercase host without
    /// scheme, credentials, port, path, query, fragment, trailing dot or leading `www.`.
    pub fn parse(input: &str) -> Result<Self, DomainError> {
        let trimmed = input.trim();
        if trimmed.is_empty() {
            return Err(DomainError::Empty);
        }
        if !trimmed.is_ascii() {
            return Err(DomainError::NonAscii);
        }
        let lowered = trimmed.to_ascii_lowercase();
        let host = extract_host(&lowered);
        let host = strip_www(host.strip_suffix('.').unwrap_or(host));
        validate_host(host)?;
        Ok(Self(host.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// True when `other` is this domain or one of its subdomains.
    pub fn covers(&self, other: &Self) -> bool {
        other.0 == self.0
            || other
                .0
                .strip_suffix(self.0.as_str())
                .is_some_and(|prefix| prefix.ends_with('.'))
    }
}

fn extract_host(input: &str) -> &str {
    let without_scheme = input
        .split_once(SCHEME_SEPARATOR)
        .map_or(input, |(_, rest)| rest);
    let authority = without_scheme
        .split(HOST_SUFFIX_TERMINATORS)
        .next()
        .unwrap_or_default();
    let host_and_port = authority
        .rsplit_once('@')
        .map_or(authority, |(_, host)| host);
    strip_port(host_and_port)
}

fn strip_port(host_and_port: &str) -> &str {
    match host_and_port.rsplit_once(':') {
        Some((host, port)) if port.bytes().all(|byte| byte.is_ascii_digit()) => host,
        _ => host_and_port,
    }
}

fn strip_www(host: &str) -> &str {
    host.strip_prefix(WWW_PREFIX)
        .filter(|rest| rest.contains('.') && !is_bare_public_suffix(rest))
        .unwrap_or(host)
}

/// Heuristic for suffixes like `co.uk` or `com.ar`, which no one owns, so blocking them
/// would cut off every site beneath them.
fn is_bare_public_suffix(host: &str) -> bool {
    host.split_once('.').is_some_and(|(second_level, country)| {
        SECOND_LEVEL_LABELS.contains(&second_level)
            && country.len() == COUNTRY_CODE_LEN
            && country.bytes().all(|byte| byte.is_ascii_lowercase())
    })
}

fn validate_host(host: &str) -> Result<(), DomainError> {
    if host.is_empty() {
        return Err(DomainError::Empty);
    }
    if host.len() > MAX_DOMAIN_LEN {
        return Err(DomainError::TooLong);
    }
    host.split('.').try_for_each(validate_label)?;
    if host.split('.').count() < MIN_LABELS {
        return Err(DomainError::TooFewLabels);
    }
    if host.rsplit('.').next().is_some_and(is_numeric) {
        return Err(DomainError::IpAddress);
    }
    if is_bare_public_suffix(host) {
        return Err(DomainError::PublicSuffix(host.to_owned()));
    }
    Ok(())
}

fn is_numeric(label: &str) -> bool {
    label.bytes().all(|byte| byte.is_ascii_digit())
}

fn validate_label(label: &str) -> Result<(), DomainError> {
    if label.is_empty() {
        return Err(DomainError::EmptyLabel);
    }
    if label.len() > MAX_LABEL_LEN {
        return Err(DomainError::LabelTooLong(label.to_owned()));
    }
    let is_allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-';
    if let Some(invalid) = label.chars().find(|&c| !is_allowed(c)) {
        return Err(DomainError::InvalidCharacter(invalid));
    }
    if label.starts_with('-') || label.ends_with('-') {
        return Err(DomainError::HyphenAtEdge(label.to_owned()));
    }
    Ok(())
}

impl Serialize for Domain {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for Domain {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        Self::parse(&String::deserialize(deserializer)?).map_err(D::Error::custom)
    }
}

impl From<Domain> for String {
    fn from(domain: Domain) -> Self {
        domain.0
    }
}

impl FromStr for Domain {
    type Err = DomainError;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        Self::parse(input)
    }
}

impl fmt::Display for Domain {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

#[cfg(test)]
mod tests;
