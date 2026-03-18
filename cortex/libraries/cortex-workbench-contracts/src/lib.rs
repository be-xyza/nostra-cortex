use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubApprovalEnvelope {
    pub approved_by: String,
    pub rationale: String,
    pub approved_at: String,
    pub decision_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubPipelineRunRequest {
    pub mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scenario_template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub publish_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub to_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub approval: Option<DpubApprovalEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubPipelineQueryRequest {
    pub kind: String,
    pub id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubPhaseResult {
    pub phase: String,
    pub status: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubPipelineRunReport {
    pub run_id: String,
    pub mode: String,
    pub status: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub finished_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_root_hash_before: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_root_hash_after: Option<String>,
    #[serde(default)]
    pub phase_results: Vec<DpubPhaseResult>,
    #[serde(default)]
    pub artifacts: Value,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubSimulationArtifact {
    pub file_name: String,
    pub bytes: u64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub modified_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubEditionEntry {
    pub version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_root_hash: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubRunHistoryItem {
    pub run_id: String,
    pub mode: String,
    pub actor_role: String,
    pub status: String,
    pub started_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graph_root_hash_after: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubLensCategorySummary {
    pub id: String,
    pub label: String,
    #[serde(default)]
    pub total: usize,
    #[serde(default)]
    pub active: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubLensCountSummary {
    pub id: String,
    pub category: String,
    pub label: String,
    #[serde(default)]
    pub count: usize,
    #[serde(default)]
    pub default_on: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubLensSummaryResponse {
    #[serde(default)]
    pub graph_root_hash: String,
    #[serde(default)]
    pub categories: Vec<DpubLensCategorySummary>,
    #[serde(default)]
    pub lenses: Vec<DpubLensCountSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubEditionTrendPoint {
    pub version: String,
    #[serde(default)]
    pub risk_score: usize,
    #[serde(default)]
    pub critical: usize,
    #[serde(default)]
    pub violation: usize,
    #[serde(default)]
    pub warning: usize,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recommended_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubEditionTrendResponse {
    pub goal: String,
    #[serde(default)]
    pub points: Vec<DpubEditionTrendPoint>,
    #[serde(default)]
    pub recommendation_changes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubLensEvaluateRequest {
    #[serde(default)]
    pub active_lenses: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubLensOverlayResponse {
    #[serde(default)]
    pub graph_root_hash: String,
    #[serde(default)]
    pub lens_state: Value,
    #[serde(default)]
    pub node_flags: Value,
    #[serde(default)]
    pub edge_flags: Value,
    #[serde(default)]
    pub counts: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubSystemReadyResponse {
    #[serde(default)]
    pub ready: bool,
    #[serde(default)]
    pub dfx_port_healthy: bool,
    #[serde(default)]
    pub gateway_port: u16,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct DpubSystemBuildResponse {
    #[serde(default)]
    pub build_id: String,
    #[serde(default)]
    pub build_time_utc: String,
    #[serde(default)]
    pub gateway_dispatch_mode: String,
    #[serde(default)]
    pub gateway_port: u16,
    #[serde(default)]
    pub workspace_root: String,
}
