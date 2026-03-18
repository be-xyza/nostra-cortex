use crate::services::viewspec::{
    ConstraintRule, ViewSpecScope, ViewSpecV1, ViewSpecValidationResult,
    compile_viewspec_to_render_surface, generate_candidate_viewspecs, now_iso, scope_key,
    validate_viewspec,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;

pub const VIEWSPEC_CANDIDATE_SET_INDEX_KEY: &str = "/cortex/ux/viewspecs/candidates/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ViewSpecGenerationMode {
    DeterministicScaffold,
    TemplateHybrid,
}

impl Default for ViewSpecGenerationMode {
    fn default() -> Self {
        Self::DeterministicScaffold
    }
}

impl ViewSpecGenerationMode {
    pub fn parse(value: Option<&str>) -> Self {
        match value
            .map(|v| v.trim().to_ascii_lowercase())
            .unwrap_or_else(|| "deterministic_scaffold".to_string())
            .as_str()
        {
            "template_hybrid" => Self::TemplateHybrid,
            _ => Self::DeterministicScaffold,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeterministicScaffold => "deterministic_scaffold",
            Self::TemplateHybrid => "template_hybrid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct GenerationTrace {
    pub strategy: String,
    #[serde(default)]
    pub seed_refs: Vec<String>,
    #[serde(default)]
    pub policy_flags: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecCandidateEnvelope {
    pub candidate_id: String,
    pub view_spec: ViewSpecV1,
    pub validation: ViewSpecValidationResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview_surface: Option<Value>,
    pub generation_trace: GenerationTrace,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecCandidateSet {
    pub candidate_set_id: String,
    pub scope_key: String,
    pub intent: String,
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
    pub mode: ViewSpecGenerationMode,
    pub created_by: String,
    pub created_at: String,
    #[serde(default)]
    pub candidates: Vec<ViewSpecCandidateEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecCandidateSetIndexEntry {
    pub candidate_set_id: String,
    pub scope_key: String,
    pub updated_at: String,
}

pub fn candidate_set_store_key(scope: &ViewSpecScope, candidate_set_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/candidates/{}/{}.json",
        scope_key(scope),
        sanitize_token(candidate_set_id)
    )
}

fn sanitize_token(value: &str) -> String {
    value
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '_'
            }
        })
        .collect()
}

fn build_trace(mode: &ViewSpecGenerationMode, idx: usize) -> GenerationTrace {
    let mut policy_flags = BTreeMap::new();
    policy_flags.insert("autonomous_promotion_disabled".to_string(), true);
    policy_flags.insert("learning_writes_disabled".to_string(), true);
    policy_flags.insert("preview_only".to_string(), true);

    match mode {
        ViewSpecGenerationMode::DeterministicScaffold => GenerationTrace {
            strategy: "deterministic_scaffold".to_string(),
            seed_refs: vec![format!("viewspec.seed.scaffold:{}", idx + 1)],
            policy_flags,
        },
        ViewSpecGenerationMode::TemplateHybrid => GenerationTrace {
            strategy: "template_hybrid".to_string(),
            seed_refs: vec![
                format!("viewspec.seed.scaffold:{}", idx + 1),
                "viewspec.template.core.safe".to_string(),
            ],
            policy_flags,
        },
    }
}

pub fn compute_candidate_input_hash(
    view_spec: &ViewSpecV1,
    trace: &GenerationTrace,
    mode: &ViewSpecGenerationMode,
    intent: &str,
    constraints: &[ConstraintRule],
    scope: &ViewSpecScope,
) -> String {
    let canonical = serde_json::json!({
        "scope": scope,
        "intent": intent,
        "constraints": constraints,
        "mode": mode.as_str(),
        "viewSpec": view_spec,
        "generationTrace": trace,
    });

    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

pub fn blocked_count(candidates: &[ViewSpecCandidateEnvelope]) -> u32 {
    candidates
        .iter()
        .filter(|candidate| !candidate.validation.valid)
        .count() as u32
}

pub fn generate_candidate_set(
    scope: ViewSpecScope,
    intent: &str,
    constraints: &[ConstraintRule],
    count: usize,
    created_by: &str,
    source_mode: &str,
    mode: ViewSpecGenerationMode,
    candidate_set_id: Option<String>,
) -> ViewSpecCandidateSet {
    let mut generated = generate_candidate_viewspecs(
        scope.clone(),
        intent,
        constraints,
        count,
        created_by,
        source_mode,
    );

    if mode == ViewSpecGenerationMode::TemplateHybrid {
        for (idx, spec) in generated.iter_mut().enumerate() {
            spec.style_tokens.insert(
                "template_profile".to_string(),
                if idx % 2 == 0 {
                    "dense_research".to_string()
                } else {
                    "operator_focus".to_string()
                },
            );
            spec.style_tokens.insert(
                "density".to_string(),
                if idx % 2 == 0 {
                    "compact".to_string()
                } else {
                    "regular".to_string()
                },
            );
        }
    }

    let mut envelopes = Vec::with_capacity(generated.len());
    for (idx, spec) in generated.into_iter().enumerate() {
        let validation = validate_viewspec(&spec);
        let preview_surface = if validation.valid {
            compile_viewspec_to_render_surface(&spec).ok()
        } else {
            None
        };
        let trace = build_trace(&mode, idx);
        let input_hash =
            compute_candidate_input_hash(&spec, &trace, &mode, intent, constraints, &scope);
        envelopes.push(ViewSpecCandidateEnvelope {
            candidate_id: spec.view_spec_id.clone(),
            view_spec: spec,
            validation,
            preview_surface,
            generation_trace: trace,
            input_hash,
        });
    }

    ViewSpecCandidateSet {
        candidate_set_id: candidate_set_id
            .unwrap_or_else(|| format!("viewspec_set_{}", Utc::now().timestamp_millis())),
        scope_key: scope_key(&scope),
        intent: intent.to_string(),
        constraints: constraints.to_vec(),
        mode,
        created_by: created_by.to_string(),
        created_at: now_iso(),
        candidates: envelopes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_scope() -> ViewSpecScope {
        ViewSpecScope {
            space_id: Some("space-alpha".to_string()),
            route_id: Some("/studio".to_string()),
            role: Some("operator".to_string()),
        }
    }

    #[test]
    fn candidate_set_contains_validation_and_preview() {
        let set = generate_candidate_set(
            test_scope(),
            "Show project progress clearly",
            &[],
            2,
            "tester",
            "human",
            ViewSpecGenerationMode::DeterministicScaffold,
            Some("set_a".to_string()),
        );

        assert_eq!(set.candidate_set_id, "set_a");
        assert_eq!(set.candidates.len(), 2);
        assert!(set.candidates.iter().all(|c| c.validation.valid));
        assert!(set.candidates.iter().all(|c| c.preview_surface.is_some()));
        assert_eq!(blocked_count(&set.candidates), 0);
    }

    #[test]
    fn blocked_count_marks_invalid_source_mode_generation() {
        let set = generate_candidate_set(
            test_scope(),
            "Show project progress clearly",
            &[],
            1,
            "tester",
            "robot",
            ViewSpecGenerationMode::DeterministicScaffold,
            Some("set_invalid".to_string()),
        );

        assert_eq!(set.candidates.len(), 1);
        assert!(!set.candidates[0].validation.valid);
        assert_eq!(blocked_count(&set.candidates), 1);
    }

    #[test]
    fn deterministic_mode_keeps_component_topology_stable() {
        let set_a = generate_candidate_set(
            test_scope(),
            "Show project progress clearly",
            &[],
            1,
            "tester",
            "human",
            ViewSpecGenerationMode::DeterministicScaffold,
            Some("set_a".to_string()),
        );
        let set_b = generate_candidate_set(
            test_scope(),
            "Show project progress clearly",
            &[],
            1,
            "tester",
            "human",
            ViewSpecGenerationMode::DeterministicScaffold,
            Some("set_b".to_string()),
        );

        let a_types = set_a.candidates[0]
            .view_spec
            .component_refs
            .iter()
            .map(|component| component.component_type.clone())
            .collect::<Vec<_>>();
        let b_types = set_b.candidates[0]
            .view_spec
            .component_refs
            .iter()
            .map(|component| component.component_type.clone())
            .collect::<Vec<_>>();
        assert_eq!(a_types, b_types);

        let a_roles = set_a.candidates[0]
            .view_spec
            .layout_graph
            .nodes
            .iter()
            .map(|node| node.role.clone())
            .collect::<Vec<_>>();
        let b_roles = set_b.candidates[0]
            .view_spec
            .layout_graph
            .nodes
            .iter()
            .map(|node| node.role.clone())
            .collect::<Vec<_>>();
        assert_eq!(a_roles, b_roles);
    }
}
