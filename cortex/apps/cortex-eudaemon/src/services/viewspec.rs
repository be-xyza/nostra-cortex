use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, HashSet};

const VIEW_SPEC_SCHEMA_VERSION: &str = "1.0.0";

pub const VIEWSPEC_INDEX_KEY: &str = "/cortex/ux/viewspecs/current/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ConstraintRule {
    pub constraint_id: String,
    pub label: String,
    pub expression: String,
    #[serde(default)]
    pub hard: bool,
    #[serde(default)]
    pub locked: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutNode {
    pub node_id: String,
    pub role: String,
    pub component_ref_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LayoutEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct LayoutGraph {
    #[serde(default)]
    pub nodes: Vec<LayoutNode>,
    #[serde(default)]
    pub edges: Vec<LayoutEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecA11y {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub live: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub invalid: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ComponentRef {
    pub component_id: String,
    pub component_type: String,
    #[serde(default)]
    pub props: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub a11y: Option<ViewSpecA11y>,
    #[serde(default)]
    pub children: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecConfidence {
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecLineage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_view_spec_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_reason: Option<String>,
    #[serde(default)]
    pub merge_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecPolicy {
    pub a11y_hard: bool,
    pub motion_policy: String,
    pub contrast_preference: String,
    pub safe_mode: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecProvenance {
    pub created_by: String,
    pub created_at: String,
    pub source_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecLockState {
    pub locked_by: String,
    pub locked_at: String,
    pub rationale: String,
    pub structural_change: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_by: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecV1 {
    #[serde(default = "default_viewspec_schema_version")]
    pub schema_version: String,
    pub view_spec_id: String,
    pub scope: ViewSpecScope,
    pub intent: String,
    #[serde(default)]
    pub constraints: Vec<ConstraintRule>,
    pub layout_graph: LayoutGraph,
    #[serde(default)]
    pub style_tokens: BTreeMap<String, String>,
    #[serde(default)]
    pub component_refs: Vec<ComponentRef>,
    pub confidence: ViewSpecConfidence,
    #[serde(default)]
    pub lineage: ViewSpecLineage,
    pub policy: ViewSpecPolicy,
    pub provenance: ViewSpecProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lock: Option<ViewSpecLockState>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecIndexEntry {
    pub view_spec_id: String,
    pub scope_key: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecValidationIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecValidationResult {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<ViewSpecValidationIssue>,
    #[serde(default)]
    pub warnings: Vec<ViewSpecValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ViewSpecProposalStatus {
    Staged,
    UnderReview,
    Approved,
    Ratified,
    Rejected,
    Superseded,
    Merged,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecProposalReviewRecord {
    pub reviewed_by: String,
    pub reviewed_at: String,
    pub summary: String,
    #[serde(default)]
    pub checks: Vec<String>,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecProposalDecisionRecord {
    pub decided_by: String,
    pub decided_at: String,
    pub decision: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecProposalMergeRecord {
    pub merged_by: String,
    pub merged_at: String,
    pub merged_with_proposal_id: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecGovernanceRef {
    pub gate_level: String,
    pub gate_status: String,
    pub decision_gate_id: String,
    pub replay_contract_ref: String,
    pub source_of_truth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecScopeAdoptionRecord {
    pub scope_key: String,
    pub active_view_spec_id: String,
    pub adopted_from_proposal_id: String,
    pub adopted_at: String,
    pub adopted_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecProposalEnvelope {
    pub proposal_id: String,
    pub view_spec_id: String,
    #[serde(default)]
    pub scope_key: String,
    pub proposed_by: String,
    pub rationale: String,
    pub created_at: String,
    pub status: ViewSpecProposalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<ViewSpecProposalReviewRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<ViewSpecProposalDecisionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<ViewSpecProposalMergeRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_ref: Option<ViewSpecGovernanceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ViewSpecEventRecord {
    pub event_id: String,
    pub event_type: String,
    pub view_spec_id: String,
    pub scope_key: String,
    pub actor: String,
    pub timestamp: String,
    #[serde(default)]
    pub payload: Value,
}

fn default_viewspec_schema_version() -> String {
    VIEW_SPEC_SCHEMA_VERSION.to_string()
}

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn is_blank(value: &str) -> bool {
    value.trim().is_empty()
}

fn allowed_component_types() -> HashSet<&'static str> {
    [
        "Container",
        "Card",
        "Row",
        "Column",
        "Tabs",
        "Modal",
        "Divider",
        "TextField",
        "TextArea",
        "Select",
        "Checkbox",
        "Slider",
        "DateTimeInput",
        "MultipleChoice",
        "Text",
        "Heading",
        "Markdown",
        "CodeBlock",
        "DataTable",
        "StatusBadge",
        "Image",
        "Video",
        "AudioPlayer",
        "Button",
    ]
    .into_iter()
    .collect()
}

fn interactive_component_types() -> HashSet<&'static str> {
    [
        "TextField",
        "TextArea",
        "Select",
        "Checkbox",
        "Slider",
        "DateTimeInput",
        "MultipleChoice",
        "Button",
        "Tabs",
        "Modal",
    ]
    .into_iter()
    .collect()
}

fn valid_motion_policy(value: &str) -> bool {
    matches!(value, "system" | "reduced" | "full")
}

fn valid_contrast_preference(value: &str) -> bool {
    matches!(value, "system" | "more" | "less")
}

fn valid_source_mode(value: &str) -> bool {
    matches!(value, "human" | "agent" | "hybrid")
}

pub fn validate_viewspec(spec: &ViewSpecV1) -> ViewSpecValidationResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    if is_blank(&spec.view_spec_id) {
        errors.push(ViewSpecValidationIssue {
            code: "missing_field".to_string(),
            path: "viewSpecId".to_string(),
            message: "viewSpecId is required".to_string(),
        });
    }
    if is_blank(&spec.intent) {
        errors.push(ViewSpecValidationIssue {
            code: "missing_field".to_string(),
            path: "intent".to_string(),
            message: "intent is required".to_string(),
        });
    }

    if !valid_motion_policy(&spec.policy.motion_policy) {
        errors.push(ViewSpecValidationIssue {
            code: "invalid_policy".to_string(),
            path: "policy.motionPolicy".to_string(),
            message: "motionPolicy must be one of system|reduced|full".to_string(),
        });
    }
    if !valid_contrast_preference(&spec.policy.contrast_preference) {
        errors.push(ViewSpecValidationIssue {
            code: "invalid_policy".to_string(),
            path: "policy.contrastPreference".to_string(),
            message: "contrastPreference must be one of system|more|less".to_string(),
        });
    }
    if !valid_source_mode(&spec.provenance.source_mode) {
        errors.push(ViewSpecValidationIssue {
            code: "invalid_source_mode".to_string(),
            path: "provenance.sourceMode".to_string(),
            message: "sourceMode must be one of human|agent|hybrid".to_string(),
        });
    }

    if !(0.0..=1.0).contains(&spec.confidence.score) {
        errors.push(ViewSpecValidationIssue {
            code: "invalid_confidence".to_string(),
            path: "confidence.score".to_string(),
            message: "confidence score must be between 0.0 and 1.0".to_string(),
        });
    }

    let allowed = allowed_component_types();
    let interactive = interactive_component_types();
    for (idx, component) in spec.component_refs.iter().enumerate() {
        let comp_path = format!("componentRefs[{idx}]");
        if is_blank(&component.component_id) {
            errors.push(ViewSpecValidationIssue {
                code: "missing_field".to_string(),
                path: format!("{comp_path}.componentId"),
                message: "componentId is required".to_string(),
            });
        }
        if !allowed.contains(component.component_type.as_str()) {
            errors.push(ViewSpecValidationIssue {
                code: "non_catalog_component".to_string(),
                path: format!("{comp_path}.componentType"),
                message: format!(
                    "componentType '{}' is not in the A2UI v1 catalog",
                    component.component_type
                ),
            });
        }

        if spec.policy.a11y_hard && interactive.contains(component.component_type.as_str()) {
            let has_label = component
                .a11y
                .as_ref()
                .and_then(|a| a.label.as_deref())
                .map(|label| !is_blank(label))
                .unwrap_or(false);
            if !has_label {
                errors.push(ViewSpecValidationIssue {
                    code: "missing_a11y_label".to_string(),
                    path: format!("{comp_path}.a11y.label"),
                    message: "interactive components require a11y.label when policy.a11yHard=true"
                        .to_string(),
                });
            }
        }
    }

    if spec.layout_graph.nodes.is_empty() && !spec.component_refs.is_empty() {
        warnings.push(ViewSpecValidationIssue {
            code: "layout_graph_empty".to_string(),
            path: "layoutGraph.nodes".to_string(),
            message: "layoutGraph.nodes is empty; compiler will preserve componentRefs order"
                .to_string(),
        });
    }

    ViewSpecValidationResult {
        valid: errors.is_empty(),
        errors,
        warnings,
    }
}

pub fn compile_viewspec_to_render_surface(
    spec: &ViewSpecV1,
) -> Result<Value, ViewSpecValidationResult> {
    let validation = validate_viewspec(spec);
    if !validation.valid {
        return Err(validation);
    }

    let mut components = Vec::new();
    for component in &spec.component_refs {
        let mut node = json!({
            "id": component.component_id,
            "type": component.component_type,
            "props": component.props,
        });

        if !component.children.is_empty() {
            node["children"] = json!(component.children);
        }
        if let Some(a11y) = &component.a11y {
            node["a11y"] = json!(a11y);
        }

        components.push(node);
    }

    let token_version = spec
        .style_tokens
        .get("token_version")
        .cloned()
        .unwrap_or_else(|| "1.0.0".to_string());
    let theme_allowlist_id = spec
        .style_tokens
        .get("theme_allowlist_id")
        .cloned()
        .unwrap_or_else(|| "trusted-core".to_string());

    Ok(json!({
        "type": "RenderSurface",
        "surfaceId": format!("viewspec:{}", spec.view_spec_id),
        "title": spec.intent,
        "components": components,
        "meta": {
            "theme": spec.style_tokens.get("theme").cloned().unwrap_or_else(|| "cortex".to_string()),
            "context": spec.style_tokens.get("context").cloned().unwrap_or_else(|| "editor".to_string()),
            "density": spec.style_tokens.get("density").cloned().unwrap_or_else(|| "regular".to_string()),
            "tone": spec.style_tokens.get("tone").cloned().unwrap_or_else(|| "neutral".to_string()),
            "intent": spec.style_tokens.get("intent").cloned().unwrap_or_else(|| "primary".to_string()),
            "priority": spec.style_tokens.get("priority").cloned().unwrap_or_else(|| "p2".to_string()),
            "token_version": token_version,
            "motion_policy": spec.policy.motion_policy,
            "safe_mode": spec.policy.safe_mode,
            "theme_allowlist_id": theme_allowlist_id,
            "contrast_preference": spec.policy.contrast_preference,
            "view_spec_id": spec.view_spec_id,
            "source_mode": spec.provenance.source_mode,
        }
    }))
}

pub fn scope_key(scope: &ViewSpecScope) -> String {
    let mut parts = Vec::new();
    if let Some(space_id) = &scope.space_id {
        parts.push(format!("space-{}", sanitize_scope_token(space_id)));
    }
    if let Some(route_id) = &scope.route_id {
        parts.push(format!("route-{}", sanitize_scope_token(route_id)));
    }
    if let Some(role) = &scope.role {
        parts.push(format!("role-{}", sanitize_scope_token(role)));
    }

    if parts.is_empty() {
        "global".to_string()
    } else {
        parts.join("__")
    }
}

fn sanitize_scope_token(value: &str) -> String {
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

pub fn current_viewspec_key(scope: &ViewSpecScope, view_spec_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/current/{}/{}.json",
        scope_key(scope),
        sanitize_scope_token(view_spec_id)
    )
}

pub fn history_viewspec_key(scope: &ViewSpecScope, view_spec_id: &str, timestamp: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/history/{}/{}_{}.json",
        scope_key(scope),
        sanitize_scope_token(timestamp),
        sanitize_scope_token(view_spec_id)
    )
}

pub fn viewspec_events_key(date_yyyy_mm_dd: &str) -> String {
    format!("/cortex/ux/viewspecs/events/{date_yyyy_mm_dd}.jsonl")
}

pub fn proposal_store_key(scope: &ViewSpecScope, proposal_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/proposals/{}/{}.json",
        scope_key(scope),
        sanitize_scope_token(proposal_id)
    )
}

pub fn default_viewspec_policy() -> ViewSpecPolicy {
    ViewSpecPolicy {
        a11y_hard: true,
        motion_policy: "system".to_string(),
        contrast_preference: "system".to_string(),
        safe_mode: true,
    }
}

pub fn generate_candidate_viewspecs(
    scope: ViewSpecScope,
    intent: &str,
    constraints: &[ConstraintRule],
    count: usize,
    created_by: &str,
    source_mode: &str,
) -> Vec<ViewSpecV1> {
    let requested = count.clamp(1, 5);
    let mut candidates = Vec::new();

    for idx in 0..requested {
        let created_at = now_iso();
        let candidate_id = format!(
            "viewspec_candidate_{}_{}",
            Utc::now().timestamp_millis(),
            idx + 1
        );

        let mut style_tokens = BTreeMap::new();
        style_tokens.insert("theme".to_string(), "cortex".to_string());
        style_tokens.insert("context".to_string(), "editor".to_string());
        style_tokens.insert("density".to_string(), "regular".to_string());
        style_tokens.insert("priority".to_string(), "p2".to_string());
        style_tokens.insert("token_version".to_string(), "1.0.0".to_string());
        style_tokens.insert("theme_allowlist_id".to_string(), "trusted-core".to_string());

        let component_refs = default_component_refs_for_intent(intent, idx);
        let layout_graph = layout_from_components(&component_refs);

        candidates.push(ViewSpecV1 {
            schema_version: VIEW_SPEC_SCHEMA_VERSION.to_string(),
            view_spec_id: candidate_id,
            scope: scope.clone(),
            intent: intent.to_string(),
            constraints: constraints.to_vec(),
            layout_graph,
            style_tokens,
            component_refs,
            confidence: ViewSpecConfidence {
                score: 0.52,
                rationale:
                    "Initial candidate from deterministic scaffold generation; requires human lock."
                        .to_string(),
            },
            lineage: ViewSpecLineage::default(),
            policy: default_viewspec_policy(),
            provenance: ViewSpecProvenance {
                created_by: created_by.to_string(),
                created_at,
                source_mode: source_mode.to_string(),
            },
            lock: None,
        });
    }

    candidates
}

fn default_component_refs_for_intent(intent: &str, variant_offset: usize) -> Vec<ComponentRef> {
    let lower = intent.to_ascii_lowercase();

    let heading = ComponentRef {
        component_id: "view_header".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([("text".to_string(), Value::String(intent.to_string()))]),
        a11y: Some(ViewSpecA11y {
            label: Some("View heading".to_string()),
            ..ViewSpecA11y::default()
        }),
        children: Vec::new(),
    };

    let primary = if lower.contains("timeline") || lower.contains("history") {
        ComponentRef {
            component_id: "timeline_table".to_string(),
            component_type: "DataTable".to_string(),
            props: BTreeMap::new(),
            a11y: Some(ViewSpecA11y {
                label: Some("Timeline table".to_string()),
                ..ViewSpecA11y::default()
            }),
            children: Vec::new(),
        }
    } else if lower.contains("write") || lower.contains("edit") || lower.contains("compose") {
        ComponentRef {
            component_id: "editor_markdown".to_string(),
            component_type: "TextArea".to_string(),
            props: BTreeMap::from([("label".to_string(), Value::String("Content".to_string()))]),
            a11y: Some(ViewSpecA11y {
                label: Some("Content editor".to_string()),
                required: Some(true),
                ..ViewSpecA11y::default()
            }),
            children: Vec::new(),
        }
    } else {
        ComponentRef {
            component_id: "overview_markdown".to_string(),
            component_type: "Markdown".to_string(),
            props: BTreeMap::from([(
                "content".to_string(),
                Value::String("Candidate view scaffold".to_string()),
            )]),
            a11y: Some(ViewSpecA11y {
                label: Some("Overview content".to_string()),
                ..ViewSpecA11y::default()
            }),
            children: Vec::new(),
        }
    };

    let action = ComponentRef {
        component_id: format!("action_apply_{}", variant_offset + 1),
        component_type: "Button".to_string(),
        props: BTreeMap::from([
            ("label".to_string(), Value::String("Apply".to_string())),
            (
                "variant".to_string(),
                Value::String(if variant_offset % 2 == 0 {
                    "primary".to_string()
                } else {
                    "secondary".to_string()
                }),
            ),
        ]),
        a11y: Some(ViewSpecA11y {
            label: Some("Apply view".to_string()),
            required: Some(false),
            ..ViewSpecA11y::default()
        }),
        children: Vec::new(),
    };

    vec![heading, primary, action]
}

fn layout_from_components(components: &[ComponentRef]) -> LayoutGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();

    for (idx, component) in components.iter().enumerate() {
        nodes.push(LayoutNode {
            node_id: format!("node_{}", idx + 1),
            role: if idx == 0 {
                "header".to_string()
            } else if idx + 1 == components.len() {
                "action".to_string()
            } else {
                "content".to_string()
            },
            component_ref_id: component.component_id.clone(),
        });

        if idx > 0 {
            edges.push(LayoutEdge {
                from: format!("node_{}", idx),
                to: format!("node_{}", idx + 1),
                relation: "flows_to".to_string(),
            });
        }
    }

    LayoutGraph { nodes, edges }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_viewspec() -> ViewSpecV1 {
        let candidates = generate_candidate_viewspecs(
            ViewSpecScope {
                space_id: Some("space_alpha".to_string()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            },
            "Show project progress clearly",
            &[],
            1,
            "tester",
            "human",
        );
        candidates[0].clone()
    }

    #[test]
    fn reject_missing_a11y_label_when_a11y_hard() {
        let mut spec = base_viewspec();
        spec.component_refs.push(ComponentRef {
            component_id: "x".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::new(),
            a11y: None,
            children: Vec::new(),
        });
        let result = validate_viewspec(&spec);
        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.code == "missing_a11y_label")
        );
    }

    #[test]
    fn reject_non_catalog_component_type() {
        let mut spec = base_viewspec();
        spec.component_refs[0].component_type = "NonStandardWidget".to_string();
        let result = validate_viewspec(&spec);
        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.code == "non_catalog_component")
        );
    }

    #[test]
    fn reject_invalid_policy_values() {
        let mut spec = base_viewspec();
        spec.policy.motion_policy = "fast".to_string();
        spec.policy.contrast_preference = "high".to_string();
        let result = validate_viewspec(&spec);
        assert!(!result.valid);
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.path == "policy.motionPolicy")
        );
        assert!(
            result
                .errors
                .iter()
                .any(|err| err.path == "policy.contrastPreference")
        );
    }

    #[test]
    fn compile_is_deterministic_for_identical_input() {
        let spec = base_viewspec();
        let a = compile_viewspec_to_render_surface(&spec).expect("compile A should pass");
        let b = compile_viewspec_to_render_surface(&spec).expect("compile B should pass");
        assert_eq!(a, b);
    }

    #[test]
    fn scope_key_sanitizes_non_path_chars() {
        let key = scope_key(&ViewSpecScope {
            space_id: Some("space/alpha".to_string()),
            route_id: Some("/studio".to_string()),
            role: Some("ops team".to_string()),
        });
        assert_eq!(key, "space-space_alpha__route-_studio__role-ops_team");
    }
}
