use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;
use std::str::FromStr;

pub const TEMPORAL_WORKFLOW_SIGNAL_HUMAN_APPROVAL: &str = "human_approval";
pub const TEMPORAL_WORKFLOW_QUERY_RUN_SNAPSHOT: &str = "run_snapshot";
pub const AGENT_EXECUTION_EVENT_TYPE: &str = "AgentExecutionLifecycle";

/// Canonical action target contract aligned to shared/specs.md.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionTarget {
    pub protocol: String,
    pub address: String,
    pub method: String,
    #[serde(default)]
    pub payload: Vec<u8>,
}

/// Tool-safe intent emitted by the agent before host-level translation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "intent", rename_all = "snake_case")]
pub enum AgentIntent {
    CreateContextNode { node_id: String, content: String },
    ProposeSchemaMutation { schema_json: String },
    ExecuteSimulation { scenario_id: String },
    ApplyActionTarget { action_target: ActionTarget },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentRunStatus {
    Queued,
    Simulating,
    WaitingApproval,
    Applying,
    Completed,
    Rejected,
    Failed,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[serde(rename_all = "lowercase")]
pub enum AuthorityLevel {
    L0,
    L1,
    L2,
    L3,
    L4,
}

impl AuthorityLevel {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::L0 => "l0",
            Self::L1 => "l1",
            Self::L2 => "l2",
            Self::L3 => "l3",
            Self::L4 => "l4",
        }
    }

    pub const fn is_v1_supported(self) -> bool {
        matches!(self, Self::L0 | Self::L1 | Self::L2)
    }
}

impl Default for AuthorityLevel {
    fn default() -> Self {
        Self::L1
    }
}

impl FromStr for AuthorityLevel {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value.trim().to_ascii_lowercase().as_str() {
            "l0" => Ok(Self::L0),
            "l1" => Ok(Self::L1),
            "l2" => Ok(Self::L2),
            "l3" => Ok(Self::L3),
            "l4" => Ok(Self::L4),
            other => Err(format!("unsupported authority level '{other}'")),
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentExecutionPhase {
    Queued,
    Simulation,
    Evaluation,
    WaitingApproval,
    Applying,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentBenchmarkRecord {
    pub pass_rate: f64,
    pub latency_ms: u64,
    pub total_tokens: u64,
    #[serde(default)]
    pub assertions_passed: usize,
    #[serde(default)]
    pub assertions_total: usize,
    #[serde(default)]
    pub assertion_details: Vec<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct BenchmarkProjectionV1 {
    pub grade: String,
    pub latency_ms: u64,
    pub token_cost: f64,
    pub summary: String,
    pub assertions_passed: usize,
    pub assertions_total: usize,
}

impl AgentBenchmarkRecord {
    pub fn to_projection_v1(&self) -> BenchmarkProjectionV1 {
        let grade = if self.assertions_total > 0 {
            if self.assertions_passed >= self.assertions_total {
                "PASS"
            } else {
                "FAIL"
            }
        } else if self.pass_rate >= 0.95 {
            "PASS"
        } else if self.pass_rate >= 0.75 {
            "WARN"
        } else {
            "FAIL"
        };

        let summary = if self.assertions_total > 0 {
            format!(
                "{} assertions passed, {:.0}% pass rate across {} tokens.",
                self.assertions_passed,
                self.pass_rate * 100.0,
                self.total_tokens
            )
        } else {
            format!(
                "{:.0}% pass rate, {} ms latency, {} tokens.",
                self.pass_rate * 100.0,
                self.latency_ms,
                self.total_tokens
            )
        };

        BenchmarkProjectionV1 {
            grade: grade.to_string(),
            latency_ms: self.latency_ms,
            token_cost: self.total_tokens as f64,
            summary,
            assertions_passed: self.assertions_passed,
            assertions_total: self.assertions_total,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessCandidateMode {
    RecommendationOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessRunStatus {
    Recorded,
    Evaluated,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HarnessEvaluationVerdict {
    Winner,
    RunnerUp,
    Hold,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessSearchSpaceV1 {
    pub supported_knobs: Vec<String>,
    pub prompt_variant_search_enabled: bool,
    pub recommendation_only: bool,
}

impl HarnessSearchSpaceV1 {
    pub fn phase6_safe() -> Self {
        Self {
            supported_knobs: vec![
                "heap_context_packaging".to_string(),
                "provider_profile".to_string(),
                "tool_loop_policy".to_string(),
                "environment_bootstrap".to_string(),
            ],
            prompt_variant_search_enabled: false,
            recommendation_only: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessCandidateV1 {
    pub schema_version: String,
    pub candidate_id: String,
    pub created_at: String,
    #[serde(default)]
    pub parent_candidate_id: Option<String>,
    pub mode: HarnessCandidateMode,
    pub search_space: HarnessSearchSpaceV1,
    #[serde(default)]
    pub changed_knobs: BTreeMap<String, Value>,
    #[serde(default)]
    pub provenance_refs: Vec<String>,
    #[serde(default)]
    pub workflow_snapshot_ref: Option<String>,
    #[serde(default)]
    pub heap_artifact_refs: Vec<String>,
    #[serde(default)]
    pub replay_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentBootstrapV1 {
    pub schema_version: String,
    pub bootstrap_id: String,
    pub captured_at: String,
    pub cwd: String,
    #[serde(default)]
    pub workspace_root: Option<String>,
    #[serde(default)]
    pub provider_profile: Option<String>,
    #[serde(default)]
    pub toolchain: Vec<String>,
    #[serde(default)]
    pub path_hints: Vec<String>,
    #[serde(default)]
    pub constraints: Vec<String>,
    pub summary: String,
    pub prompt_override_verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessRunV1 {
    pub schema_version: String,
    pub run_id: String,
    pub candidate_id: String,
    pub status: HarnessRunStatus,
    pub started_at: String,
    #[serde(default)]
    pub finished_at: Option<String>,
    pub execution_ref: String,
    #[serde(default)]
    pub workflow_snapshot_ref: Option<String>,
    #[serde(default)]
    pub heap_artifact_refs: Vec<String>,
    #[serde(default)]
    pub replay_refs: Vec<String>,
    #[serde(default)]
    pub benchmark: Option<AgentBenchmarkRecord>,
    #[serde(default)]
    pub benchmark_projection: Option<BenchmarkProjectionV1>,
    #[serde(default)]
    pub notes: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HarnessEvaluationV1 {
    pub schema_version: String,
    pub evaluation_id: String,
    pub candidate_id: String,
    pub held_out_pack_id: String,
    pub verdict: HarnessEvaluationVerdict,
    pub rank: u32,
    pub total_score: f64,
    pub latency_ms: u64,
    pub token_cost: f64,
    pub summary: String,
    pub benchmark_projection: BenchmarkProjectionV1,
    #[serde(default)]
    pub execution_refs: Vec<String>,
    #[serde(default)]
    pub evaluator_ref: Option<String>,
    pub recommendation_only: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentExecutionRecord {
    pub schema_version: String,
    pub execution_id: String,
    pub attempt_id: String,
    pub agent_id: String,
    pub workflow_id: String,
    pub phase: AgentExecutionPhase,
    pub status: String,
    pub authority_scope: AuthorityLevel,
    pub input_snapshot_hash: String,
    pub output_snapshot_hash: String,
    pub timestamp: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub model_fingerprint: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub prompt_template_artifact_id: Option<String>,
    #[serde(default)]
    pub prompt_template_revision_id: Option<String>,
    #[serde(default)]
    pub prompt_execution_artifact_id: Option<String>,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub child_run_ids: Vec<String>,
    #[serde(default)]
    pub provider_trace_summary: Option<Value>,
    #[serde(default)]
    pub tool_state_hash: Option<String>,
    #[serde(default)]
    pub confidence: Option<f64>,
    #[serde(default)]
    pub promotion_level: Option<String>,
    #[serde(default)]
    pub started_at: Option<String>,
    #[serde(default)]
    pub ended_at: Option<String>,
    #[serde(default)]
    pub replay_contract_ref: Option<String>,
    #[serde(default)]
    pub lineage_id: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub benchmark: Option<AgentBenchmarkRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentRunEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub run_id: String,
    pub space_id: String,
    pub timestamp: String,
    #[serde(default)]
    pub sequence: u64,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AuthorityExecutionOutcome {
    pub accepted: bool,
    pub action_target: ActionTarget,
    pub applied_at: String,
    #[serde(default)]
    pub host_receipt: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalRunBinding {
    pub workflow_id: String,
    #[serde(default)]
    pub temporal_run_id: Option<String>,
    #[serde(default)]
    pub task_queue: Option<String>,
    #[serde(default)]
    pub namespace: Option<String>,
    #[serde(default)]
    pub projection_mode: Option<String>,
    #[serde(default)]
    pub status: Option<String>,
    #[serde(default)]
    pub last_projected_sequence: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalBridgeStartCommand {
    pub run_id: String,
    pub workflow_id: String,
    pub space_id: String,
    pub contribution_id: String,
    pub approval_timeout_seconds: u64,
    pub task_queue: String,
    pub namespace: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalBridgeSignalCommand {
    pub run_id: String,
    pub decision: String,
    #[serde(default)]
    pub rationale: Option<String>,
    pub actor: String,
    #[serde(default)]
    pub decision_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TemporalBridgeRunSnapshot {
    pub schema_version: String,
    pub run_id: String,
    pub workflow_id: String,
    pub space_id: String,
    pub contribution_id: String,
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub sequence: u64,
    #[serde(default)]
    pub events: Vec<AgentRunEvent>,
    #[serde(default)]
    pub simulation: Option<Value>,
    #[serde(default)]
    pub surface_update: Option<Value>,
    #[serde(default)]
    pub authority_outcome: Option<AuthorityExecutionOutcome>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub prompt_template_artifact_id: Option<String>,
    #[serde(default)]
    pub prompt_template_revision_id: Option<String>,
    #[serde(default)]
    pub prompt_execution_artifact_id: Option<String>,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub child_run_ids: Vec<String>,
    #[serde(default)]
    pub provider_trace_summary: Option<Value>,
    #[serde(default)]
    pub provider_trace: Option<Value>,
    pub approval_timeout_seconds: u64,
    pub terminal: bool,
    #[serde(default)]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShadowDivergenceSeverity {
    Critical,
    Warning,
    Info,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShadowDivergenceRecord {
    pub severity: ShadowDivergenceSeverity,
    pub code: String,
    pub message: String,
    #[serde(default)]
    pub expected: Option<Value>,
    #[serde(default)]
    pub actual: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ShadowComparisonSummary {
    pub compared_at: String,
    pub status: String,
    pub critical_count: usize,
    pub warning_count: usize,
    pub info_count: usize,
    #[serde(default)]
    pub divergences: Vec<ShadowDivergenceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentRun {
    pub run_id: String,
    pub workflow_id: String,
    pub space_id: String,
    pub contribution_id: String,
    #[serde(default)]
    pub agent_id: Option<String>,
    pub status: AgentRunStatus,
    pub started_at: String,
    pub updated_at: String,
    #[serde(default)]
    pub stream_channel: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub prompt_template_artifact_id: Option<String>,
    #[serde(default)]
    pub prompt_template_revision_id: Option<String>,
    #[serde(default)]
    pub prompt_execution_artifact_id: Option<String>,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub child_run_ids: Vec<String>,
    #[serde(default)]
    pub provider_trace_summary: Option<Value>,
    #[serde(default)]
    pub simulation: Option<Value>,
    #[serde(default)]
    pub surface_update: Option<Value>,
    #[serde(default)]
    pub authority_outcome: Option<AuthorityExecutionOutcome>,
    #[serde(default)]
    pub authority_level: Option<AuthorityLevel>,
    #[serde(default)]
    pub execution_id: Option<String>,
    #[serde(default)]
    pub attempt_id: Option<String>,
    #[serde(default)]
    pub temporal_binding: Option<TemporalRunBinding>,
    #[serde(default)]
    pub shadow_summary: Option<ShadowComparisonSummary>,
    #[serde(default)]
    pub approval_timeout_seconds: Option<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn authority_level_parsing_accepts_supported_values() {
        assert_eq!("l0".parse::<AuthorityLevel>().unwrap(), AuthorityLevel::L0);
        assert_eq!("L1".parse::<AuthorityLevel>().unwrap(), AuthorityLevel::L1);
        assert_eq!("l2".parse::<AuthorityLevel>().unwrap(), AuthorityLevel::L2);
        assert_eq!("l3".parse::<AuthorityLevel>().unwrap(), AuthorityLevel::L3);
        assert_eq!("l4".parse::<AuthorityLevel>().unwrap(), AuthorityLevel::L4);
        assert!("x1".parse::<AuthorityLevel>().is_err());
    }

    #[test]
    fn execution_record_supports_optional_extension_fields() {
        let record = AgentExecutionRecord {
            schema_version: "1.0.0".to_string(),
            execution_id: "exec-1".to_string(),
            attempt_id: "attempt-1".to_string(),
            agent_id: "agent-default".to_string(),
            workflow_id: "wf-1".to_string(),
            phase: AgentExecutionPhase::Queued,
            status: "queued".to_string(),
            authority_scope: AuthorityLevel::L1,
            input_snapshot_hash: "in".to_string(),
            output_snapshot_hash: "out".to_string(),
            timestamp: "2026-02-24T00:00:00Z".to_string(),
            space_id: None,
            model_fingerprint: None,
            provider: None,
            model: None,
            auth_mode: None,
            response_id: None,
            prompt_template_artifact_id: None,
            prompt_template_revision_id: None,
            prompt_execution_artifact_id: None,
            parent_run_id: None,
            child_run_ids: Vec::new(),
            provider_trace_summary: None,
            tool_state_hash: None,
            confidence: None,
            promotion_level: None,
            started_at: None,
            ended_at: None,
            replay_contract_ref: None,
            lineage_id: None,
            evidence_refs: Vec::new(),
            benchmark: None,
        };
        let encoded = serde_json::to_value(&record).unwrap();
        assert_eq!(encoded["schemaVersion"], "1.0.0");
        assert_eq!(encoded["authorityScope"], "l1");
        assert!(encoded.get("modelFingerprint").is_some());
    }

    #[test]
    fn benchmark_record_parsing_accepts_valid_json() {
        let json = r#"{
            "passRate": 0.95,
            "latencyMs": 1200,
            "totalTokens": 2500,
            "assertionsPassed": 19,
            "assertionsTotal": 20,
            "assertionDetails": [{"id": 1, "passed": true}]
        }"#;
        let record: AgentBenchmarkRecord = serde_json::from_str(json).unwrap();
        assert_eq!(record.pass_rate, 0.95);
        assert_eq!(record.latency_ms, 1200);
        assert_eq!(record.total_tokens, 2500);
        assert_eq!(record.assertions_passed, 19);
        assert_eq!(record.assertions_total, 20);
        assert_eq!(record.assertion_details.len(), 1);
    }

    #[test]
    fn benchmark_projection_derives_grade_and_summary() {
        let record = AgentBenchmarkRecord {
            pass_rate: 0.8,
            latency_ms: 900,
            total_tokens: 2400,
            assertions_passed: 3,
            assertions_total: 4,
            assertion_details: Vec::new(),
        };

        let projection = record.to_projection_v1();
        assert_eq!(projection.grade, "FAIL");
        assert_eq!(projection.latency_ms, 900);
        assert_eq!(projection.token_cost, 2400.0);
        assert_eq!(projection.assertions_passed, 3);
        assert_eq!(projection.assertions_total, 4);
        assert!(projection.summary.contains("3 assertions passed"));
    }

    #[test]
    fn phase6_safe_search_space_disables_prompt_variants() {
        let search_space = HarnessSearchSpaceV1::phase6_safe();
        assert_eq!(
            search_space.supported_knobs,
            vec![
                "heap_context_packaging",
                "provider_profile",
                "tool_loop_policy",
                "environment_bootstrap"
            ]
        );
        assert!(!search_space.prompt_variant_search_enabled);
        assert!(search_space.recommendation_only);
    }
}
