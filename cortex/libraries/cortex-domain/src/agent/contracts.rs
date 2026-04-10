use serde::{Deserialize, Serialize};
use serde_json::Value;
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
        assert!(encoded.get("provider").is_some());
        assert!(encoded.get("promptTemplateArtifactId").is_some());
        assert!(encoded.get("childRunIds").is_some());
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
}
