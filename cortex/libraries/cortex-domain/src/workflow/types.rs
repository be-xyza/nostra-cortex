use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

pub const WORKFLOW_SCHEMA_VERSION: &str = "1.0.0";
pub const WORKFLOW_CONTEXT_SECTIONS: &[&str] = &[
    "inputs",
    "artifacts",
    "memory_refs",
    "evaluation",
    "control",
    "visibility",
];
pub const WORKFLOW_INDEX_KEY: &str = "/cortex/workflows/definitions/current/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub role: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowMotifKind {
    Sequential,
    ParallelCompare,
    RepairLoop,
    FanOutJoin,
    HumanGate,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowGenerationMode {
    #[default]
    DeterministicScaffold,
    MotifHybrid,
}

impl WorkflowGenerationMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::DeterministicScaffold => "deterministic_scaffold",
            Self::MotifHybrid => "motif_hybrid",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowConstraintRule {
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
pub struct WorkflowConfidence {
    pub score: f32,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLineage {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_workflow_draft_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_definition_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fork_reason: Option<String>,
    #[serde(default)]
    pub merge_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProvenance {
    pub created_by: String,
    pub created_at: String,
    pub source_mode: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGovernanceRef {
    pub gate_level: String,
    pub gate_status: String,
    pub decision_gate_id: String,
    pub replay_contract_ref: String,
    pub source_of_truth: String,
    pub lineage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
    pub definition_digest: String,
    pub binding_digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct ContextEnvelopeV1 {
    #[serde(default)]
    pub inputs: BTreeMap<String, Value>,
    #[serde(default)]
    pub artifacts: Vec<String>,
    #[serde(default)]
    pub memory_refs: Vec<String>,
    #[serde(default)]
    pub evaluation: BTreeMap<String, Value>,
    #[serde(default)]
    pub control: BTreeMap<String, Value>,
    #[serde(default)]
    pub visibility: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContextContractV1 {
    #[serde(default = "default_context_sections")]
    pub allowed_sections: Vec<String>,
}

impl Default for ContextContractV1 {
    fn default() -> Self {
        Self {
            allowed_sections: default_context_sections(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowNodeKind {
    CapabilityCall,
    HumanCheckpoint,
    EvaluationGate,
    Parallel,
    Switch,
    Loop,
    SubflowRef,
    Terminal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCheckpointPolicyV1 {
    pub resume_allowed: bool,
    pub cancel_allowed: bool,
    #[serde(default)]
    pub pause_allowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowLoopPolicyV1 {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_iterations: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub termination_expression: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowNodeV1 {
    pub node_id: String,
    pub label: String,
    pub kind: WorkflowNodeKind,
    #[serde(default)]
    pub reads: Vec<String>,
    #[serde(default)]
    pub writes: Vec<String>,
    #[serde(default)]
    pub evidence_outputs: Vec<String>,
    #[serde(default)]
    pub authority_requirements: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_policy: Option<WorkflowCheckpointPolicyV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub loop_policy: Option<WorkflowLoopPolicyV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subflow_ref: Option<String>,
    #[serde(default)]
    pub config: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowEdgeV1 {
    pub edge_id: String,
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGraphV1 {
    #[serde(default)]
    pub nodes: Vec<WorkflowNodeV1>,
    #[serde(default)]
    pub edges: Vec<WorkflowEdgeV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDraftPolicyV1 {
    pub recommendation_only: bool,
    pub require_review: bool,
    pub allow_shadow_execution: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowIntentV1 {
    #[serde(default = "default_workflow_schema_version")]
    pub schema_version: String,
    pub workflow_intent_id: String,
    pub scope: WorkflowScope,
    pub intent: String,
    pub motif_kind: WorkflowMotifKind,
    #[serde(default)]
    pub constraints: Vec<WorkflowConstraintRule>,
    pub authority_ceiling: String,
    pub provenance: WorkflowProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDraftV1 {
    #[serde(default = "default_workflow_schema_version")]
    pub schema_version: String,
    pub workflow_draft_id: String,
    pub scope: WorkflowScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_ref: Option<String>,
    pub intent: String,
    pub motif_kind: WorkflowMotifKind,
    #[serde(default)]
    pub constraints: Vec<WorkflowConstraintRule>,
    pub graph: WorkflowGraphV1,
    #[serde(default)]
    pub context_contract: ContextContractV1,
    pub confidence: WorkflowConfidence,
    #[serde(default)]
    pub lineage: WorkflowLineage,
    pub policy: WorkflowDraftPolicyV1,
    pub provenance: WorkflowProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowDefinitionV1 {
    #[serde(default = "default_workflow_schema_version")]
    pub schema_version: String,
    pub definition_id: String,
    pub scope: WorkflowScope,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent_ref: Option<String>,
    pub intent: String,
    pub motif_kind: WorkflowMotifKind,
    #[serde(default)]
    pub constraints: Vec<WorkflowConstraintRule>,
    pub graph: WorkflowGraphV1,
    #[serde(default)]
    pub context_contract: ContextContractV1,
    pub confidence: WorkflowConfidence,
    #[serde(default)]
    pub lineage: WorkflowLineage,
    pub policy: WorkflowDraftPolicyV1,
    pub provenance: WorkflowProvenance,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_ref: Option<WorkflowGovernanceRef>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowValidationIssue {
    pub code: String,
    pub path: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowValidationResult {
    pub valid: bool,
    #[serde(default)]
    pub errors: Vec<WorkflowValidationIssue>,
    #[serde(default)]
    pub warnings: Vec<WorkflowValidationIssue>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCompileResult {
    pub valid: bool,
    pub normalized_graph: Value,
    pub serverless_workflow_projection: Value,
    pub a2ui_surface_projection: Value,
    pub flow_graph_projection: Value,
    #[serde(default)]
    pub warnings: Vec<WorkflowValidationIssue>,
    pub digest: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowGenerationTrace {
    pub strategy: String,
    #[serde(default)]
    pub seed_refs: Vec<String>,
    #[serde(default)]
    pub policy_flags: BTreeMap<String, bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCandidateEnvelope {
    pub candidate_id: String,
    pub workflow_draft: WorkflowDraftV1,
    pub validation: WorkflowValidationResult,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compile_result: Option<WorkflowCompileResult>,
    pub generation_trace: WorkflowGenerationTrace,
    pub input_hash: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCandidateSet {
    pub candidate_set_id: String,
    pub scope_key: String,
    pub intent: String,
    pub motif_kind: WorkflowMotifKind,
    #[serde(default)]
    pub constraints: Vec<WorkflowConstraintRule>,
    pub mode: WorkflowGenerationMode,
    pub created_by: String,
    pub created_at: String,
    #[serde(default)]
    pub candidates: Vec<WorkflowCandidateEnvelope>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCandidateSetIndexEntry {
    pub candidate_set_id: String,
    pub scope_key: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowProposalStatus {
    Staged,
    UnderReview,
    Approved,
    Ratified,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProposalReviewRecord {
    pub reviewed_by: String,
    pub reviewed_at: String,
    pub summary: String,
    #[serde(default)]
    pub checks: Vec<String>,
    pub approved: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProposalDecisionRecord {
    pub decided_by: String,
    pub decided_at: String,
    pub decision: String,
    pub rationale: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowProposalEnvelope {
    pub proposal_id: String,
    pub workflow_draft_id: String,
    pub definition_id: String,
    pub scope_key: String,
    pub proposed_by: String,
    pub rationale: String,
    pub created_at: String,
    pub status: WorkflowProposalStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review: Option<WorkflowProposalReviewRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub decision: Option<WorkflowProposalDecisionRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_ref: Option<WorkflowGovernanceRef>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowScopeAdoptionRecord {
    pub scope_key: String,
    pub active_definition_id: String,
    pub adopted_from_proposal_id: String,
    pub adopted_at: String,
    pub adopted_by: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowExecutionAdapterKind {
    LocalDurableWorkerV1,
    WorkflowEngineCanisterV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowExecutionProfileKind {
    Sync,
    Async,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionBindingV1 {
    #[serde(default = "default_workflow_schema_version")]
    pub schema_version: String,
    pub binding_id: String,
    pub definition_id: String,
    pub adapter: WorkflowExecutionAdapterKind,
    pub execution_profile: WorkflowExecutionProfileKind,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_policy: Option<WorkflowCheckpointPolicyV1>,
    #[serde(default)]
    pub runtime_limits: BTreeMap<String, Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance_ref: Option<WorkflowGovernanceRef>,
    pub provenance: WorkflowProvenance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowExecutionPlanV1 {
    pub plan_id: String,
    pub definition_id: String,
    pub binding_id: String,
    pub adapter: WorkflowExecutionAdapterKind,
    pub projection: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowInstanceStatus {
    Queued,
    Running,
    WaitingCheckpoint,
    Paused,
    Completed,
    Cancelled,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowInstanceV1 {
    #[serde(default = "default_workflow_schema_version")]
    pub schema_version: String,
    pub instance_id: String,
    pub definition_id: String,
    pub binding_id: String,
    pub status: WorkflowInstanceStatus,
    pub scope: WorkflowScope,
    pub created_at: String,
    pub updated_at: String,
    pub definition_digest: String,
    pub binding_digest: String,
    pub source_of_truth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub replay_contract_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSignalV1 {
    pub signal_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowTraceEventV1 {
    pub event_id: String,
    pub instance_id: String,
    pub event_type: String,
    pub sequence: u64,
    pub timestamp: String,
    #[serde(default)]
    pub payload: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowCheckpointStatus {
    Pending,
    Resolved,
    Cancelled,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCheckpointV1 {
    pub checkpoint_id: String,
    pub instance_id: String,
    pub node_id: String,
    pub kind: WorkflowNodeKind,
    pub status: WorkflowCheckpointStatus,
    pub created_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub surface_ref: Option<String>,
    pub policy: WorkflowCheckpointPolicyV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowCheckpointResultV1 {
    pub instance_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<String>,
    pub status: WorkflowCheckpointStatus,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowOutcomeStatus {
    Completed,
    Failed,
    Cancelled,
    Paused,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowOutcomeV1 {
    pub outcome_id: String,
    pub instance_id: String,
    pub status: WorkflowOutcomeStatus,
    pub completed_at: String,
    pub summary: String,
    #[serde(default)]
    pub contribution_refs: Vec<String>,
    #[serde(default)]
    pub global_event_refs: Vec<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSnapshotV1 {
    pub instance: WorkflowInstanceV1,
    #[serde(default)]
    pub trace: Vec<WorkflowTraceEventV1>,
    #[serde(default)]
    pub checkpoints: Vec<WorkflowCheckpointV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub outcome: Option<WorkflowOutcomeV1>,
}

pub fn default_workflow_schema_version() -> String {
    WORKFLOW_SCHEMA_VERSION.to_string()
}

pub fn default_context_sections() -> Vec<String> {
    WORKFLOW_CONTEXT_SECTIONS
        .iter()
        .map(|section| (*section).to_string())
        .collect()
}
