use chrono::{DateTime, Duration, Utc};
use cortex_domain::graph::{Edge, EdgeKind, Graph, Node};
use cortex_domain::integrity::{
    CommonsEnforcementMode, CommonsEnforcementOutcome, CommonsRuleset, SuggestedEnrichment,
    evaluate_commons_ruleset_with_suggested_enrichments, extract_suggested_enrichments,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

pub const DEFAULT_STEWARD_GATE_TOKEN_TTL_SECS: i64 = 600;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StewardGateStatus {
    Pass,
    ActionRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct StewardGateEvaluation {
    pub status: StewardGateStatus,
    pub outcome: CommonsEnforcementOutcome,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct StewardGateTokenClaims {
    pub artifact_id: String,
    pub head_revision_id: String,
    pub actor_id: String,
    pub outcome_hash: String,
    pub issued_at: String,
    pub expires_at: String,
}

pub fn evaluate_heap_steward_gate(
    workspace_root: &Path,
    space_id: &str,
    block_id: &str,
    text: &str,
    tags: &[String],
    page_links: &[String],
    already_applied_enrichment_ids: &HashSet<String>,
) -> Result<StewardGateEvaluation, String> {
    let ruleset = load_or_bootstrap_space_ruleset(workspace_root, space_id)?;
    let graph = build_heap_context_graph(block_id, space_id, tags, page_links);
    let suggested_enrichments = extract_suggested_enrichments(text)
        .into_iter()
        .filter(|entry| !already_applied_enrichment_ids.contains(&entry.enrichment_id))
        .collect::<Vec<_>>();

    let outcome = evaluate_commons_ruleset_with_suggested_enrichments(
        &graph,
        &ruleset,
        CommonsEnforcementMode::WarnOrBlock,
        suggested_enrichments,
    );

    let status = if outcome.should_block || !outcome.suggested_enrichments.is_empty() {
        StewardGateStatus::ActionRequired
    } else {
        StewardGateStatus::Pass
    };

    Ok(StewardGateEvaluation { status, outcome })
}

pub fn build_steward_gate_surface(artifact_id: &str, evaluation: &StewardGateEvaluation) -> Value {
    let outcome = &evaluation.outcome;
    let severity = if outcome.should_block {
        "error"
    } else {
        "warning"
    };
    let violation_count = outcome.violations.len();
    let suggestion_count = outcome.suggested_enrichments.len();
    let summary = if outcome.should_block {
        format!(
            "Publish is blocked until {} critical integrity issue(s) are resolved.",
            violation_count
        )
    } else if suggestion_count > 0 {
        format!(
            "{} optional structural enrichment(s) are available before publish.",
            suggestion_count
        )
    } else {
        "No steward intervention required.".to_string()
    };

    let action_buttons = outcome
        .suggested_enrichments
        .iter()
        .map(|entry| {
            json!({
                "id": format!("enrichment:{}", entry.enrichment_id),
                "type": "Button",
                "props": {
                    "label": entry.display_label,
                    "action": format!("stewardGateApply?artifactId={artifact_id}&enrichmentId={}", entry.enrichment_id),
                    "enrichmentId": entry.enrichment_id,
                },
                "children": []
            })
        })
        .collect::<Vec<_>>();

    json!({
        "surfaceId": format!("steward_gate:{artifact_id}"),
        "title": "Steward Gate",
        "root": "steward_gate_root",
        "components": [
            {
                "id": "steward_gate_root",
                "type": "Card",
                "props": {
                    "title": "Steward Gate",
                    "description": summary,
                    "severity": severity,
                },
                "children": []
            }
        ],
        "meta": {
            "kind": "steward_gate",
            "status": evaluation.status,
            "severity": severity,
            "violations": outcome.violations,
            "suggestedEnrichments": outcome.suggested_enrichments,
            "actions": action_buttons,
        }
    })
}

pub fn load_or_bootstrap_space_ruleset(
    workspace_root: &Path,
    space_id: &str,
) -> Result<CommonsRuleset, String> {
    let path = space_ruleset_path(workspace_root, space_id);
    if path.exists() {
        let raw = fs::read_to_string(&path).map_err(|err| {
            format!(
                "Failed to read Commons ruleset at {}: {}",
                path.display(),
                err
            )
        })?;
        let decoded = serde_json::from_str::<CommonsRuleset>(&raw).map_err(|err| {
            format!(
                "Failed to parse Commons ruleset at {}: {}",
                path.display(),
                err
            )
        })?;
        return Ok(decoded);
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| {
            format!(
                "Failed to create Commons ruleset directory {}: {}",
                parent.display(),
                err
            )
        })?;
    }

    let fallback = CommonsRuleset {
        commons_id: space_id.to_string(),
        commons_version: "1.0.0".to_string(),
        rules: Vec::new(),
    };
    let encoded = serde_json::to_string_pretty(&fallback)
        .map_err(|err| format!("Failed to encode fallback Commons ruleset: {}", err))?;
    fs::write(&path, encoded).map_err(|err| {
        format!(
            "Failed to persist fallback Commons ruleset at {}: {}",
            path.display(),
            err
        )
    })?;

    Ok(fallback)
}

pub fn issue_publish_token(
    artifact_id: &str,
    head_revision_id: &str,
    actor_id: &str,
    outcome: &CommonsEnforcementOutcome,
) -> Result<String, String> {
    let issued_at = Utc::now();
    let expires_at = issued_at + Duration::seconds(DEFAULT_STEWARD_GATE_TOKEN_TTL_SECS);
    let claims = StewardGateTokenClaims {
        artifact_id: artifact_id.to_string(),
        head_revision_id: head_revision_id.to_string(),
        actor_id: actor_id.to_string(),
        outcome_hash: outcome_hash(outcome)?,
        issued_at: issued_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    };

    encode_token_claims(&claims)
}

pub fn validate_publish_token(
    token: &str,
    artifact_id: &str,
    head_revision_id: &str,
    actor_id: &str,
    outcome: &CommonsEnforcementOutcome,
) -> Result<StewardGateTokenClaims, String> {
    let claims = decode_token_claims(token)?;
    if claims.artifact_id != artifact_id {
        return Err("Steward Gate token artifact mismatch.".to_string());
    }
    if claims.head_revision_id != head_revision_id {
        return Err("Steward Gate token revision mismatch.".to_string());
    }
    if claims.actor_id != actor_id {
        return Err("Steward Gate token actor mismatch.".to_string());
    }
    if claims.outcome_hash != outcome_hash(outcome)? {
        return Err("Steward Gate token outcome hash mismatch.".to_string());
    }

    let expires_at = DateTime::parse_from_rfc3339(&claims.expires_at)
        .map_err(|err| format!("Invalid Steward Gate token expiry timestamp: {}", err))?
        .with_timezone(&Utc);
    if Utc::now() > expires_at {
        return Err("Steward Gate token expired.".to_string());
    }

    Ok(claims)
}

pub fn build_heap_context_graph(
    block_id: &str,
    space_id: &str,
    tags: &[String],
    page_links: &[String],
) -> Graph {
    let mut graph = Graph::default();
    let mut attributes = BTreeMap::new();
    attributes.insert("space_id".to_string(), space_id.to_string());
    attributes.insert(
        "tags".to_string(),
        tags.iter()
            .map(String::as_str)
            .collect::<Vec<_>>()
            .join(","),
    );

    graph.add_node(Node {
        id: block_id.to_string(),
        node_type: "heap_block".to_string(),
        attributes,
    });

    let mut linked_ids = BTreeSet::new();
    linked_ids.extend(tags.iter().cloned());
    linked_ids.extend(page_links.iter().cloned());

    for linked in linked_ids {
        graph.add_node(Node {
            id: linked.clone(),
            node_type: "heap_block_ref".to_string(),
            attributes: BTreeMap::new(),
        });
        graph.add_edge(Edge {
            from: block_id.to_string(),
            to: linked,
            kind: EdgeKind::References,
        });
    }

    graph
}

pub fn outcome_hash(outcome: &CommonsEnforcementOutcome) -> Result<String, String> {
    let bytes = serde_json::to_vec(outcome)
        .map_err(|err| format!("Failed to encode Commons outcome for token hash: {}", err))?;
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    Ok(hex::encode(hasher.finalize()))
}

fn encode_token_claims(claims: &StewardGateTokenClaims) -> Result<String, String> {
    let encoded_claims = serde_json::to_vec(claims)
        .map_err(|err| format!("Failed to encode Steward Gate token claims: {}", err))?;
    let payload_hex = hex::encode(encoded_claims);
    let signature = token_signature(&payload_hex);
    Ok(format!("sgt1.{payload_hex}.{signature}"))
}

fn decode_token_claims(token: &str) -> Result<StewardGateTokenClaims, String> {
    let mut parts = token.split('.');
    let Some(version) = parts.next() else {
        return Err("Steward Gate token is malformed.".to_string());
    };
    let Some(payload_hex) = parts.next() else {
        return Err("Steward Gate token is malformed.".to_string());
    };
    let Some(signature) = parts.next() else {
        return Err("Steward Gate token is malformed.".to_string());
    };
    if parts.next().is_some() {
        return Err("Steward Gate token is malformed.".to_string());
    }
    if version != "sgt1" {
        return Err("Steward Gate token version is unsupported.".to_string());
    }

    let expected = token_signature(payload_hex);
    if signature != expected {
        return Err("Steward Gate token signature mismatch.".to_string());
    }

    let payload = hex::decode(payload_hex)
        .map_err(|err| format!("Steward Gate token payload is invalid hex: {}", err))?;
    serde_json::from_slice::<StewardGateTokenClaims>(&payload)
        .map_err(|err| format!("Steward Gate token payload is invalid JSON: {}", err))
}

fn token_signature(payload_hex: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(steward_gate_token_secret().as_bytes());
    hasher.update(b".");
    hasher.update(payload_hex.as_bytes());
    hex::encode(hasher.finalize())
}

fn steward_gate_token_secret() -> String {
    let candidate = std::env::var("NOSTRA_STEWARD_GATE_TOKEN_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());
    candidate.unwrap_or_else(|| "cortex-local-steward-gate".to_string())
}

fn space_ruleset_path(workspace_root: &Path, space_id: &str) -> PathBuf {
    workspace_root
        .join("_spaces")
        .join(space_id)
        .join("commons_ruleset.json")
}

pub fn resolve_applied_enrichment_ids(children_surface_json: &[Value]) -> HashSet<String> {
    children_surface_json
        .iter()
        .filter_map(|surface| {
            surface
                .get("meta")
                .and_then(|meta| meta.get("steward_gate"))
                .and_then(|gate| gate.get("source_enrichment_id"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .collect()
}

pub fn find_enrichment_by_id<'a>(
    outcome: &'a CommonsEnforcementOutcome,
    enrichment_id: &str,
) -> Option<&'a SuggestedEnrichment> {
    outcome
        .suggested_enrichments
        .iter()
        .find(|entry| entry.enrichment_id == enrichment_id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::integrity::{
        CommonsEnforcementMode, CommonsRuleset, evaluate_commons_ruleset,
    };

    #[test]
    fn publish_token_roundtrip_verifies_claims() {
        let graph = Graph::default();
        let ruleset = CommonsRuleset {
            commons_id: "space".to_string(),
            commons_version: "1.0.0".to_string(),
            rules: vec![],
        };
        let outcome =
            evaluate_commons_ruleset(&graph, &ruleset, CommonsEnforcementMode::WarnOrBlock);
        let token = issue_publish_token("artifact-1", "rev-1", "actor-1", &outcome)
            .expect("token should be issued");

        let claims = validate_publish_token(&token, "artifact-1", "rev-1", "actor-1", &outcome)
            .expect("token should validate");
        assert_eq!(claims.artifact_id, "artifact-1");
    }
}
