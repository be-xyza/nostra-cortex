use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::sync::LazyLock;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct MicroSyntaxMatch {
    pub extractor_id: String,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub captures: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum SuggestedEnrichmentKind {
    Mention,
    Tag,
    Duration,
    PullRequest,
}

impl SuggestedEnrichmentKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Mention => "mention",
            Self::Tag => "tag",
            Self::Duration => "duration",
            Self::PullRequest => "pull_request",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SuggestedEnrichment {
    pub enrichment_id: String,
    pub kind: SuggestedEnrichmentKind,
    pub display_label: String,
    pub matched_text: String,
    pub start: usize,
    pub end: usize,
    #[serde(default)]
    pub metadata: Value,
}

pub trait MicroSyntaxExtractor: Send + Sync {
    fn extractor_id(&self) -> &'static str;
    fn enrichment_kind(&self) -> SuggestedEnrichmentKind;
    fn extract(&self, text: &str) -> Vec<MicroSyntaxMatch>;
    fn display_label(&self, syntax: &MicroSyntaxMatch) -> String;
    fn metadata(&self, _syntax: &MicroSyntaxMatch) -> Value {
        json!({})
    }
}

pub fn extract_micro_syntax_matches(text: &str) -> Vec<MicroSyntaxMatch> {
    let mut out = Vec::new();
    for extractor in default_extractors() {
        out.extend(extractor.extract(text));
    }
    out.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
            .then_with(|| left.extractor_id.cmp(&right.extractor_id))
            .then_with(|| left.matched_text.cmp(&right.matched_text))
    });
    out
}

pub fn extract_suggested_enrichments(text: &str) -> Vec<SuggestedEnrichment> {
    let mut out = Vec::new();
    for extractor in default_extractors() {
        for syntax in extractor.extract(text) {
            let kind = extractor.enrichment_kind();
            out.push(SuggestedEnrichment {
                enrichment_id: stable_enrichment_id(&kind, &syntax),
                kind,
                display_label: extractor.display_label(&syntax),
                matched_text: syntax.matched_text.clone(),
                start: syntax.start,
                end: syntax.end,
                metadata: extractor.metadata(&syntax),
            });
        }
    }

    out.sort_by(|left, right| {
        left.start
            .cmp(&right.start)
            .then_with(|| left.end.cmp(&right.end))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.matched_text.cmp(&right.matched_text))
    });
    out
}

fn stable_enrichment_id(kind: &SuggestedEnrichmentKind, syntax: &MicroSyntaxMatch) -> String {
    let mut hasher = Sha256::new();
    hasher.update(kind.as_str().as_bytes());
    hasher.update(b":");
    hasher.update(syntax.extractor_id.as_bytes());
    hasher.update(b":");
    hasher.update(syntax.matched_text.to_ascii_lowercase().as_bytes());
    hasher.update(b":");
    hasher.update(syntax.start.to_string().as_bytes());
    hasher.update(b":");
    hasher.update(syntax.end.to_string().as_bytes());
    let digest = hex::encode(hasher.finalize());
    let suffix = digest.get(0..20).unwrap_or(digest.as_str());
    format!("enrichment_{suffix}")
}

fn default_extractors() -> Vec<Box<dyn MicroSyntaxExtractor>> {
    vec![
        Box::new(MentionExtractor),
        Box::new(TagExtractor),
        Box::new(DurationExtractor),
        Box::new(PullRequestExtractor),
    ]
}

struct MentionExtractor;
struct TagExtractor;
struct DurationExtractor;
struct PullRequestExtractor;

static MENTION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?P<raw>@[A-Za-z0-9_][A-Za-z0-9_\-]{1,63})").expect("mention regex is valid")
});

static TAG_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?m)(?P<raw>#[A-Za-z0-9_][A-Za-z0-9_\-]{1,63})").expect("tag regex is valid")
});

static DURATION_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)\b(?P<raw>\d{1,3}(?:d|h|m|w))\b").expect("duration regex is valid")
});

static PULL_REQUEST_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)\b(?P<raw>PR-\d{1,6})\b").expect("pr regex is valid"));

impl MicroSyntaxExtractor for MentionExtractor {
    fn extractor_id(&self) -> &'static str {
        "mention"
    }

    fn enrichment_kind(&self) -> SuggestedEnrichmentKind {
        SuggestedEnrichmentKind::Mention
    }

    fn extract(&self, text: &str) -> Vec<MicroSyntaxMatch> {
        MENTION_REGEX
            .captures_iter(text)
            .filter_map(|caps| {
                let raw = caps.name("raw")?;
                let mut captures = BTreeMap::new();
                captures.insert(
                    "handle".to_string(),
                    raw.as_str().trim_start_matches('@').to_ascii_lowercase(),
                );
                Some(MicroSyntaxMatch {
                    extractor_id: self.extractor_id().to_string(),
                    matched_text: raw.as_str().to_string(),
                    start: raw.start(),
                    end: raw.end(),
                    captures,
                })
            })
            .collect()
    }

    fn display_label(&self, syntax: &MicroSyntaxMatch) -> String {
        format!("Convert {} to Mention Edge", syntax.matched_text)
    }

    fn metadata(&self, syntax: &MicroSyntaxMatch) -> Value {
        json!({
            "handle": syntax
                .captures
                .get("handle")
                .cloned()
                .unwrap_or_else(|| syntax.matched_text.trim_start_matches('@').to_ascii_lowercase()),
        })
    }
}

impl MicroSyntaxExtractor for TagExtractor {
    fn extractor_id(&self) -> &'static str {
        "tag"
    }

    fn enrichment_kind(&self) -> SuggestedEnrichmentKind {
        SuggestedEnrichmentKind::Tag
    }

    fn extract(&self, text: &str) -> Vec<MicroSyntaxMatch> {
        TAG_REGEX
            .captures_iter(text)
            .filter_map(|caps| {
                let raw = caps.name("raw")?;
                let mut captures = BTreeMap::new();
                captures.insert(
                    "tag".to_string(),
                    raw.as_str().trim_start_matches('#').to_ascii_lowercase(),
                );
                Some(MicroSyntaxMatch {
                    extractor_id: self.extractor_id().to_string(),
                    matched_text: raw.as_str().to_string(),
                    start: raw.start(),
                    end: raw.end(),
                    captures,
                })
            })
            .collect()
    }

    fn display_label(&self, syntax: &MicroSyntaxMatch) -> String {
        format!("Convert {} to Tag Edge", syntax.matched_text)
    }

    fn metadata(&self, syntax: &MicroSyntaxMatch) -> Value {
        json!({
            "tag": syntax
                .captures
                .get("tag")
                .cloned()
                .unwrap_or_else(|| syntax.matched_text.trim_start_matches('#').to_ascii_lowercase()),
        })
    }
}

impl MicroSyntaxExtractor for DurationExtractor {
    fn extractor_id(&self) -> &'static str {
        "duration"
    }

    fn enrichment_kind(&self) -> SuggestedEnrichmentKind {
        SuggestedEnrichmentKind::Duration
    }

    fn extract(&self, text: &str) -> Vec<MicroSyntaxMatch> {
        DURATION_REGEX
            .captures_iter(text)
            .filter_map(|caps| {
                let raw = caps.name("raw")?;
                Some(MicroSyntaxMatch {
                    extractor_id: self.extractor_id().to_string(),
                    matched_text: raw.as_str().to_string(),
                    start: raw.start(),
                    end: raw.end(),
                    captures: BTreeMap::from([(
                        "duration".to_string(),
                        raw.as_str().to_ascii_lowercase(),
                    )]),
                })
            })
            .collect()
    }

    fn display_label(&self, syntax: &MicroSyntaxMatch) -> String {
        format!("Convert {} to Duration Widget", syntax.matched_text)
    }

    fn metadata(&self, syntax: &MicroSyntaxMatch) -> Value {
        json!({
            "duration": syntax
                .captures
                .get("duration")
                .cloned()
                .unwrap_or_else(|| syntax.matched_text.to_ascii_lowercase()),
        })
    }
}

impl MicroSyntaxExtractor for PullRequestExtractor {
    fn extractor_id(&self) -> &'static str {
        "pull_request"
    }

    fn enrichment_kind(&self) -> SuggestedEnrichmentKind {
        SuggestedEnrichmentKind::PullRequest
    }

    fn extract(&self, text: &str) -> Vec<MicroSyntaxMatch> {
        PULL_REQUEST_REGEX
            .captures_iter(text)
            .filter_map(|caps| {
                let raw = caps.name("raw")?;
                Some(MicroSyntaxMatch {
                    extractor_id: self.extractor_id().to_string(),
                    matched_text: raw.as_str().to_string(),
                    start: raw.start(),
                    end: raw.end(),
                    captures: BTreeMap::from([(
                        "ticket".to_string(),
                        raw.as_str().to_ascii_uppercase(),
                    )]),
                })
            })
            .collect()
    }

    fn display_label(&self, syntax: &MicroSyntaxMatch) -> String {
        format!("Convert {} to Pull Request Widget", syntax.matched_text)
    }

    fn metadata(&self, syntax: &MicroSyntaxMatch) -> Value {
        json!({
            "ticket": syntax
                .captures
                .get("ticket")
                .cloned()
                .unwrap_or_else(|| syntax.matched_text.to_ascii_uppercase()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn micro_syntax_extracts_constitution_tag() {
        let text = "We must abide by #constitution";
        let enrichments = extract_suggested_enrichments(text);
        assert!(enrichments
            .iter()
            .any(|entry| entry.kind == SuggestedEnrichmentKind::Tag
                && entry.matched_text == "#constitution"));
    }

    #[test]
    fn enrichment_generation_is_deterministic() {
        let text = "Review PR-102 in 5d with @alice and #governance";
        let first = extract_suggested_enrichments(text);
        let second = extract_suggested_enrichments(text);
        assert_eq!(first, second);
    }
}
