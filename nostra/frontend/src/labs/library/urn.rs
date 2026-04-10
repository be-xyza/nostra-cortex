use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum VersionSpec {
    #[serde(rename = "latest")]
    Latest,
    #[serde(rename = "exact")]
    ExactSemver(String),
    #[serde(rename = "hash")]
    Hash(String),
    #[serde(rename = "range")]
    Range(String),
    #[serde(rename = "edition")]
    EditionId(String),
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct VersionedRef {
    pub contribution_id: String,
    pub version: VersionSpec,
    #[serde(default)]
    pub path: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ParsedUrn {
    pub resource_type: String,
    pub id: String,
    pub version: Option<VersionSpec>,
    pub path: Option<String>,
}

pub fn parse_nostra_urn(input: &str) -> Result<ParsedUrn, String> {
    let input = input.trim();
    let prefix = "urn:nostra:";
    if !input.starts_with(prefix) {
        return Err("URN must start with 'urn:nostra:'".to_string());
    }

    let after_prefix = &input[prefix.len()..];
    let (before_path, path) = match after_prefix.split_once('#') {
        Some((a, b)) => (a, Some(format!("#{}", b))),
        None => (after_prefix, None),
    };

    let (before_version, version_part) = match before_path.split_once('@') {
        Some((a, b)) => (a, Some(b)),
        None => (before_path, None),
    };

    let mut segments = before_version.splitn(2, ':');
    let resource_type = segments
        .next()
        .ok_or_else(|| "Missing resource type".to_string())?
        .to_string();
    let id = segments
        .next()
        .ok_or_else(|| "Missing id".to_string())?
        .to_string();

    let version = version_part.map(parse_version_spec).transpose()?;

    Ok(ParsedUrn {
        resource_type,
        id,
        version,
        path,
    })
}

pub fn parse_version_spec(input: &str) -> Result<VersionSpec, String> {
    let input = input.trim();
    if input == "latest" {
        return Ok(VersionSpec::Latest);
    }

    if let Some(rest) = input.strip_prefix("v") {
        if rest.is_empty() {
            return Err("Invalid semver".to_string());
        }
        return Ok(VersionSpec::ExactSemver(rest.to_string()));
    }

    if let Some(rest) = input.strip_prefix("^") {
        if rest.is_empty() {
            return Err("Invalid range".to_string());
        }
        return Ok(VersionSpec::Range(format!("^{}", rest)));
    }

    if let Some(rest) = input.strip_prefix("edition:") {
        if rest.is_empty() {
            return Err("Invalid edition id".to_string());
        }
        return Ok(VersionSpec::EditionId(rest.to_string()));
    }

    let is_hex = input
        .chars()
        .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase());
    if input.len() >= 32 && is_hex {
        return Ok(VersionSpec::Hash(input.to_string()));
    }

    Err("Unrecognized version spec".to_string())
}

pub fn format_version_spec(spec: &VersionSpec) -> String {
    match spec {
        VersionSpec::Latest => "latest".to_string(),
        VersionSpec::ExactSemver(v) => format!("v{}", v),
        VersionSpec::Hash(h) => h.clone(),
        VersionSpec::Range(r) => r.clone(),
        VersionSpec::EditionId(id) => format!("edition:{}", id),
    }
}

pub fn format_versioned_ref_urn(resource_type: &str, id: &str, spec: &VersionSpec) -> String {
    format!(
        "urn:nostra:{}:{}@{}",
        resource_type,
        id,
        format_version_spec(spec)
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_latest() {
        let u = parse_nostra_urn("urn:nostra:contribution:abc@latest").unwrap();
        assert_eq!(u.resource_type, "contribution");
        assert_eq!(u.id, "abc");
        assert_eq!(u.version, Some(VersionSpec::Latest));
    }

    #[test]
    fn parses_semver() {
        let u = parse_nostra_urn("urn:nostra:dpub:abc@v1.2.3#ch-1").unwrap();
        assert_eq!(u.resource_type, "dpub");
        assert_eq!(u.id, "abc");
        assert_eq!(u.version, Some(VersionSpec::ExactSemver("1.2.3".to_string())));
        assert_eq!(u.path, Some("#ch-1".to_string()));
    }
}

