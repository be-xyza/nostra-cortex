use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ResourceRef(String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResourceRefError {
    Empty,
    ContainsWhitespace,
    MissingScheme,
    InvalidScheme,
    InvalidComponent(&'static str),
    UnknownGovernedPredicate(String),
}

impl fmt::Display for ResourceRefError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "resource ref is empty"),
            Self::ContainsWhitespace => write!(f, "resource ref contains whitespace"),
            Self::MissingScheme => write!(f, "resource ref is missing scheme"),
            Self::InvalidScheme => write!(f, "resource ref has invalid scheme"),
            Self::InvalidComponent(name) => write!(f, "resource ref has invalid {name}"),
            Self::UnknownGovernedPredicate(name) => write!(f, "unknown governed predicate '{name}'"),
        }
    }
}

impl std::error::Error for ResourceRefError {}

impl ResourceRef {
    pub fn parse(raw: &str) -> Result<Self, ResourceRefError> {
        let raw = raw.trim();
        if raw.is_empty() {
            return Err(ResourceRefError::Empty);
        }
        if raw.chars().any(|c| c.is_whitespace()) {
            return Err(ResourceRefError::ContainsWhitespace);
        }
        let scheme_end = raw.find("://").ok_or(ResourceRefError::MissingScheme)?;
        let scheme = &raw[..scheme_end];
        if !is_valid_scheme(scheme) {
            return Err(ResourceRefError::InvalidScheme);
        }
        Ok(Self(raw.to_string()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_nostra(&self) -> bool {
        self.0.starts_with("nostra://")
    }

    pub fn contribution(contribution_id: &str) -> Result<Self, ResourceRefError> {
        let id = contribution_id.trim();
        if id.is_empty() {
            return Err(ResourceRefError::InvalidComponent("contribution_id"));
        }
        if id.chars().any(|c| c.is_whitespace()) {
            return Err(ResourceRefError::InvalidComponent("contribution_id"));
        }
        Ok(Self(format!(
            "nostra://contribution?id={}",
            encode_query_component(id)
        )))
    }

    pub fn capability(capability_id: &str) -> Result<Self, ResourceRefError> {
        let id = capability_id.trim();
        if id.is_empty() {
            return Err(ResourceRefError::InvalidComponent("capability_id"));
        }
        if id.chars().any(|c| c.is_whitespace()) {
            return Err(ResourceRefError::InvalidComponent("capability_id"));
        }
        Ok(Self(format!(
            "nostra://capability?id={}",
            encode_query_component(id)
        )))
    }
}

impl fmt::Display for ResourceRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PredicateRef(String);

pub const GOVERNED_PREDICATES: &[&str] = &[
    "depends_on",
    "contradicts",
    "supersedes",
    "implements",
    "invalidates",
    "requires",
    "assumes",
    "constitutional_basis",
    "derives_from",
    "forked_into",
    "governs",
    "produces",
    "references",
];

impl PredicateRef {
    pub fn governed(name: &str) -> Result<Self, ResourceRefError> {
        let normalized = name.trim().to_ascii_lowercase();
        if !is_snake_case_identifier(normalized.as_str()) {
            return Err(ResourceRefError::InvalidComponent("predicate_name"));
        }
        if !GOVERNED_PREDICATES.contains(&normalized.as_str()) {
            return Err(ResourceRefError::UnknownGovernedPredicate(normalized));
        }
        Ok(Self(format!("nostra://predicate/{}", normalized)))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for PredicateRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

fn is_valid_scheme(scheme: &str) -> bool {
    if scheme.is_empty() {
        return false;
    }
    let mut chars = scheme.chars();
    let first = match chars.next() {
        Some(c) => c,
        None => return false,
    };
    if !first.is_ascii_alphabetic() {
        return false;
    }
    chars.all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '-' || c == '.')
}

fn is_snake_case_identifier(raw: &str) -> bool {
    !raw.is_empty() && raw.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn encode_query_component(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for b in raw.as_bytes() {
        let c = *b as char;
        if c.is_ascii_alphanumeric()
            || c == '-'
            || c == '.'
            || c == '_'
            || c == '~'
            || c == ':'
        {
            out.push(c);
        } else {
            out.push('%');
            out.push(nibble_hex((*b >> 4) & 0xF));
            out.push(nibble_hex(*b & 0xF));
        }
    }
    out
}

fn nibble_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'A' + (nibble - 10)) as char,
        _ => '0',
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_ref_parse_rejects_whitespace() {
        let err = ResourceRef::parse("nostra://profile/alice bob").unwrap_err();
        assert_eq!(err, ResourceRefError::ContainsWhitespace);
    }

    #[test]
    fn resource_ref_parse_requires_scheme() {
        let err = ResourceRef::parse("nostra:profile/alice").unwrap_err();
        assert_eq!(err, ResourceRefError::MissingScheme);
    }

    #[test]
    fn capability_ref_is_percent_encoded() {
        let rr = ResourceRef::capability("route:/system").unwrap();
        assert_eq!(rr.as_str(), "nostra://capability?id=route:%2Fsystem");
    }

    #[test]
    fn contribution_ref_is_stable() {
        let rr = ResourceRef::contribution("118").unwrap();
        assert_eq!(rr.as_str(), "nostra://contribution?id=118");
    }

    #[test]
    fn governed_predicate_requires_allowlist() {
        let err = PredicateRef::governed("made_up").unwrap_err();
        assert!(matches!(err, ResourceRefError::UnknownGovernedPredicate(_)));
    }

    #[test]
    fn governed_predicate_canonicalizes_to_uri() {
        let pred = PredicateRef::governed("depends_on").unwrap();
        assert_eq!(pred.as_str(), "nostra://predicate/depends_on");
    }
}
