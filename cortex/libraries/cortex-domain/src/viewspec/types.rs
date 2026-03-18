use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const VIEW_SPEC_SCHEMA_VERSION: &str = "1.0.0";
pub const VIEWSPEC_INDEX_KEY: &str = "/cortex/ux/viewspecs/current/index.json";

pub fn default_viewspec_schema_version() -> String {
    VIEW_SPEC_SCHEMA_VERSION.to_string()
}

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
