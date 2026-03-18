use crate::RuntimeError;
use crate::gateway::local::{
    LocalGatewayMutationRecord, LocalGatewayMutationSubmit, LocalGatewayProbe,
    LocalGatewayQueueAction,
};
use crate::gateway::types::{
    GatewayIdempotencySemantics, GatewayRequestEnvelope, GatewayResponseEnvelope,
    GatewayRouteMetadata, GatewayTransactionBoundary,
};
use async_trait::async_trait;
pub use cortex_domain::agent::provider::ModelProviderPort;
use cortex_domain::events::ProjectedEvent;
use cortex_domain::streaming::types::{
    ArtifactRealtimeConnectAck, ArtifactRealtimeDisconnectAck, ArtifactRealtimeEnvelope,
    ArtifactRealtimePollResult,
};
use cortex_domain::theme::ThemePolicyPreferences;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn put(&self, key: &str, value: &Value) -> Result<(), RuntimeError>;
    async fn get(&self, key: &str) -> Result<Option<Value>, RuntimeError>;
}

#[async_trait]
pub trait NetworkAdapter: Send + Sync {
    async fn post_json(
        &self,
        endpoint: &str,
        idempotency_key: &str,
        body: &Value,
    ) -> Result<(), RuntimeError>;
}

pub trait TimeProvider: Send + Sync {
    fn now_unix_secs(&self) -> u64;
    fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError>;
}

#[async_trait]
pub trait GatewayHostAdapter: Send + Sync {
    fn route_exists(&self, method: &str, path: &str) -> bool;

    fn resolve_route(
        &self,
        method: &str,
        path: &str,
    ) -> Result<Option<GatewayRouteMetadata>, RuntimeError> {
        if !self.route_exists(method, path) {
            return Ok(None);
        }
        Ok(Some(GatewayRouteMetadata {
            path_template: path.to_string(),
            path_params: std::collections::BTreeMap::new(),
            idempotency_semantics: self.idempotency_semantics(method, path),
            transaction_boundary: self.transaction_boundary(method, path),
            expected_event_emissions: self.expected_event_emissions(method, path),
        }))
    }

    fn idempotency_semantics(&self, method: &str, path: &str) -> GatewayIdempotencySemantics;

    fn transaction_boundary(&self, method: &str, path: &str) -> GatewayTransactionBoundary;

    fn expected_event_emissions(&self, method: &str, path: &str) -> Vec<String>;

    async fn dispatch(
        &self,
        request: &GatewayRequestEnvelope,
    ) -> Result<GatewayResponseEnvelope, RuntimeError>;
}

#[async_trait]
pub trait UxContractStoreAdapter: Send + Sync {
    async fn read_text(&self, key: &str) -> Result<Option<String>, RuntimeError>;
    async fn write_text(
        &self,
        key: &str,
        content: &str,
        mime_type: &str,
    ) -> Result<(), RuntimeError>;
    async fn append_line(&self, key: &str, line: &str, mime_type: &str)
    -> Result<(), RuntimeError>;
    async fn list_prefix(&self, prefix: &str) -> Result<Vec<String>, RuntimeError>;
}

pub trait ThemePolicyAdapter: Send + Sync {
    fn current(&self) -> Result<ThemePolicyPreferences, RuntimeError>;
    fn persist(
        &self,
        requested: ThemePolicyPreferences,
    ) -> Result<ThemePolicyPreferences, RuntimeError>;
}

pub trait LocalGatewayOrchestrationAdapter: Send + Sync {
    fn queue_snapshot(&self) -> Result<Vec<LocalGatewayMutationRecord>, RuntimeError>;
    fn export_queue_json(&self) -> Result<String, RuntimeError>;
    fn apply_queue_action(
        &self,
        mutation_id: &str,
        action: LocalGatewayQueueAction,
    ) -> Result<(), RuntimeError>;
    fn probe(&self) -> Result<LocalGatewayProbe, RuntimeError>;
    fn set_online(&self, status: bool) -> Result<(), RuntimeError>;
    fn is_online(&self) -> Result<bool, RuntimeError>;
    fn submit_mutation(&self, mutation: LocalGatewayMutationSubmit)
    -> Result<String, RuntimeError>;
}

#[async_trait]
pub trait AgentProcessAdapter: Send + Sync {
    async fn spawn_supervised(&self, program: &str, args: &[String]) -> Result<(), RuntimeError>;
    async fn probe_canister_status(&self, canister: &str) -> Result<bool, RuntimeError>;
    async fn emit_log(&self, line: &str) -> Result<(), RuntimeError>;
}

pub trait LogAdapter: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn append_projected_event(&self, event: &ProjectedEvent) -> Result<(), RuntimeError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceScopeRequest {
    pub space_id: String,
    pub action_target: String,
    pub domain_mode: String,
    pub gate_level: String,
    #[serde(default)]
    pub actor_principal: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct GovernanceScopeEvaluation {
    pub allowed: bool,
    pub reason: String,
    pub effective_weight: f64,
    pub requires_review: bool,
    pub gate_decision: String,
    pub required_actions: Vec<String>,
    #[serde(default)]
    pub policy_ref: Option<String>,
    pub policy_version: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ActorRoleBinding {
    pub space_id: String,
    pub principal: String,
    pub role: String,
    #[serde(default)]
    pub source_ref: Option<String>,
    pub updated_at: u64,
}

#[async_trait]
pub trait GovernanceAdapter: Send + Sync {
    async fn evaluate_action_scope(
        &self,
        request: GovernanceScopeRequest,
    ) -> Result<GovernanceScopeEvaluation, RuntimeError>;

    async fn get_actor_role_binding(
        &self,
        space_id: &str,
        principal: &str,
    ) -> Result<Option<ActorRoleBinding>, RuntimeError>;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionTopology {
    LocalOnly,
    Networked,
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ConsensusMode {
    NoneLocal,
    ReplicatedConsensus,
    DelegatedConsensus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum TrustBoundary {
    AttributedDefault,
    MixedAttribution,
    PrivacyPreferred,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ExecutionProfile {
    pub execution_topology: ExecutionTopology,
    pub consensus_mode: ConsensusMode,
    pub trust_boundary: TrustBoundary,
    pub updated_by: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttributionMode {
    Attributed,
    Pseudonymous,
    Anonymous,
    Delayed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AttributionDomain {
    pub id: String,
    pub mode: AttributionMode,
    pub reattachment_policy: String,
    pub governance_visibility: String,
    pub auditability_level: String,
    #[serde(default)]
    pub weight_policy_ref: Option<String>,
    pub updated_by: String,
    pub updated_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ContributionAttributionBinding {
    pub contribution_id: String,
    pub space_id: String,
    pub domain_id: String,
    pub bound_by: String,
    pub bound_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ReplayContract {
    pub mutation_id: String,
    pub workflow_id: String,
    pub action_target: String,
    pub adapter_set_ref: String,
    pub execution_profile_ref: String,
    pub attribution_domain_ref: String,
    pub deterministic_input_hash: String,
    #[serde(default)]
    pub lineage_id: Option<String>,
    #[serde(default)]
    pub policy_ref: Option<String>,
    #[serde(default)]
    pub policy_snapshot_ref: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    #[serde(default)]
    pub decision_digest: Option<String>,
    pub captured_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DecisionLineage {
    pub mutation_id: String,
    pub workflow_id: String,
    pub lineage_id: String,
    pub action_target: String,
    pub decision_digest: String,
    #[serde(default)]
    pub policy_ref: Option<String>,
    #[serde(default)]
    pub policy_snapshot_ref: Option<String>,
    #[serde(default)]
    pub evidence_refs: Vec<String>,
    pub captured_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct FileMetadata {
    pub mime_type: String,
    pub size: u64,
    pub last_modified: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecisionClass {
    Governance,
    Merge,
    HighImpact,
    Standard,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GateOutcome {
    Pass,
    Warn,
    RequireReview,
    RequireSimulation,
    Block,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EpistemicAssessment {
    pub assessment_id: String,
    pub workflow_id: String,
    pub mutation_id: String,
    pub decision_class: DecisionClass,
    pub confidence_score: f64,
    pub source_reliability: f64,
    pub robustness_score: f64,
    pub voi_score: f64,
    pub regret_risk: f64,
    pub assumption_count: u32,
    pub evidence_count: u32,
    pub alternative_count: u32,
    pub gate_outcome: GateOutcome,
    pub reasons: Vec<String>,
    pub created_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvaluationLoopRequest {
    pub run_id: String,
    pub workflow_id: String,
    pub space_id: String,
    pub authority_level: String,
    #[serde(default)]
    pub simulation: Option<Value>,
    #[serde(default)]
    pub action_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AgentEvaluationLoopResult {
    pub gate_outcome: GateOutcome,
    pub allowed: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default)]
    pub confidence_score: Option<f64>,
    #[serde(default)]
    pub source_reliability: Option<f64>,
    #[serde(default)]
    pub policy_ref: Option<String>,
}

#[async_trait]
pub trait AgentEvaluationLoopAdapter: Send + Sync {
    async fn evaluate(
        &self,
        request: AgentEvaluationLoopRequest,
    ) -> Result<AgentEvaluationLoopResult, RuntimeError>;
}

#[async_trait]
pub trait WorkflowAdapter: Send + Sync {
    async fn get_space_execution_profile(
        &self,
        space_id: &str,
    ) -> Result<Option<ExecutionProfile>, RuntimeError>;

    async fn get_attribution_domains(
        &self,
        space_id: &str,
    ) -> Result<Vec<AttributionDomain>, RuntimeError>;

    async fn get_replay_contract(
        &self,
        mutation_id: &str,
    ) -> Result<Option<ReplayContract>, RuntimeError>;

    async fn get_epistemic_assessment_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<EpistemicAssessment>, RuntimeError>;

    async fn get_contribution_attribution_binding(
        &self,
        contribution_id: &str,
    ) -> Result<Option<ContributionAttributionBinding>, RuntimeError>;

    async fn list_space_replay_contracts(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<ReplayContract>, RuntimeError>;

    async fn get_decision_lineage_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<DecisionLineage>, RuntimeError>;

    async fn list_space_decision_lineage(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<DecisionLineage>, RuntimeError>;

    async fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
        mime_type: &str,
    ) -> Result<(), RuntimeError>;

    async fn read_file(&self, path: &str) -> Result<Vec<u8>, RuntimeError>;

    async fn list_files(&self, prefix: &str) -> Result<Vec<(String, FileMetadata)>, RuntimeError>;
}

#[async_trait]
pub trait StreamingTransportAdapter: Send + Sync {
    async fn connect(
        &self,
        actor_id: &str,
        artifact_id: &str,
        client_nonce: u64,
    ) -> Result<ArtifactRealtimeConnectAck, RuntimeError>;

    async fn disconnect(
        &self,
        actor_id: &str,
        artifact_id: &str,
        client_nonce: Option<u64>,
    ) -> Result<ArtifactRealtimeDisconnectAck, RuntimeError>;

    async fn publish(
        &self,
        envelope: &ArtifactRealtimeEnvelope,
        client_nonce: u64,
        timestamp_ms: u64,
    ) -> Result<(), RuntimeError>;

    async fn poll(&self, nonce: u64) -> Result<ArtifactRealtimePollResult, RuntimeError>;
}
