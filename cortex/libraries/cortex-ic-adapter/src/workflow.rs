use crate::ic::resolve_canister_id;
use async_trait::async_trait;
use candid::{CandidType, Decode, Encode, Principal};
use cortex_runtime::RuntimeError;
use cortex_runtime::ports::{
    AttributionDomain, AttributionMode, ConsensusMode, ContributionAttributionBinding,
    DecisionClass, DecisionLineage, EpistemicAssessment, ExecutionProfile, ExecutionTopology,
    FileMetadata, GateOutcome, ReplayContract, TrustBoundary, WorkflowAdapter,
};
use cortex_runtime::workflow::adapter::WorkflowExecutionAdapter;
use cortex_domain::workflow::{
    WorkflowCheckpointResultV1, WorkflowDefinitionV1, WorkflowExecutionBindingV1,
    WorkflowExecutionPlanV1, WorkflowInstanceV1, WorkflowSignalV1, WorkflowSnapshotV1,
};
use ic_agent::identity::AnonymousIdentity;
use ic_agent::Agent;
use serde::Serialize;

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
enum ExecutionTopologyCandid {
    LocalOnly,
    Networked,
    Hybrid,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
enum ConsensusModeCandid {
    NoneLocal,
    ReplicatedConsensus,
    DelegatedConsensus,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
enum TrustBoundaryCandid {
    AttributedDefault,
    MixedAttribution,
    PrivacyPreferred,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq)]
struct ExecutionProfileCandid {
    execution_topology: ExecutionTopologyCandid,
    consensus_mode: ConsensusModeCandid,
    trust_boundary: TrustBoundaryCandid,
    updated_by: Principal,
    updated_at: u64,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
enum AttributionModeCandid {
    #[serde(rename = "attributed")]
    Attributed,
    #[serde(rename = "pseudonymous")]
    Pseudonymous,
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "delayed")]
    Delayed,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq)]
struct AttributionDomainCandid {
    id: String,
    mode: AttributionModeCandid,
    reattachment_policy: String,
    governance_visibility: String,
    auditability_level: String,
    weight_policy_ref: Option<String>,
    updated_by: Principal,
    updated_at: u64,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq)]
struct ContributionAttributionBindingCandid {
    contribution_id: String,
    space_id: String,
    domain_id: String,
    bound_by: Principal,
    bound_at: u64,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct ReplayContractCandid {
    mutation_id: String,
    workflow_id: String,
    action_target: String,
    adapter_set_ref: String,
    execution_profile_ref: String,
    attribution_domain_ref: String,
    deterministic_input_hash: String,
    lineage_id: Option<String>,
    policy_ref: Option<String>,
    policy_snapshot_ref: Option<String>,
    evidence_refs: Vec<String>,
    decision_digest: Option<String>,
    captured_at: u64,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct DecisionLineageCandid {
    mutation_id: String,
    workflow_id: String,
    lineage_id: String,
    action_target: String,
    decision_digest: String,
    policy_ref: Option<String>,
    policy_snapshot_ref: Option<String>,
    evidence_refs: Vec<String>,
    captured_at: u64,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct FileMetadataCandid {
    mime_type: String,
    size: u64,
    last_modified: u64,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
enum DecisionClassCandid {
    #[serde(rename = "governance")]
    Governance,
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "high_impact")]
    HighImpact,
    #[serde(rename = "standard")]
    Standard,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
enum GateOutcomeCandid {
    #[serde(rename = "pass")]
    Pass,
    #[serde(rename = "warn")]
    Warn,
    #[serde(rename = "require_review")]
    RequireReview,
    #[serde(rename = "require_simulation")]
    RequireSimulation,
    #[serde(rename = "block")]
    Block,
}

#[derive(CandidType, candid::Deserialize, Serialize, Clone, Debug, PartialEq)]
struct EpistemicAssessmentCandid {
    assessment_id: String,
    workflow_id: String,
    mutation_id: String,
    decision_class: DecisionClassCandid,
    confidence_score: f64,
    source_reliability: f64,
    robustness_score: f64,
    voi_score: f64,
    regret_risk: f64,
    assumption_count: u32,
    evidence_count: u32,
    alternative_count: u32,
    gate_outcome: GateOutcomeCandid,
    reasons: Vec<String>,
    created_at: u64,
}

#[derive(CandidType, candid::Deserialize, Clone, Debug, PartialEq, Eq)]
struct TextOperationResult {
    ok: Option<String>,
    err: Option<String>,
}

#[derive(Clone, Debug)]
pub struct WorkflowCanisterClient {
    host: String,
    canister_id: Principal,
}

impl WorkflowCanisterClient {
    pub async fn from_env() -> Result<Self, RuntimeError> {
        let host = std::env::var("NOSTRA_IC_HOST")
            .or_else(|_| std::env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());
        let canister_id_text =
            resolve_canister_id("CANISTER_ID_WORKFLOW_ENGINE", "workflow_engine")
                .await
                .map_err(RuntimeError::Network)?;
        let canister_id = Principal::from_text(canister_id_text).map_err(|err| {
            RuntimeError::Network(format!("invalid workflow_engine principal: {err}"))
        })?;
        Ok(Self { host, canister_id })
    }

    async fn agent(&self) -> Result<Agent, RuntimeError> {
        let agent = Agent::builder()
            .with_url(self.host.clone())
            .with_identity(AnonymousIdentity)
            .build()
            .map_err(|err| RuntimeError::Network(format!("failed to build ic-agent: {err}")))?;

        if self.host.contains("127.0.0.1") || self.host.contains("localhost") {
            agent
                .fetch_root_key()
                .await
                .map_err(|err| RuntimeError::Network(format!("failed to fetch root key: {err}")))?;
        }
        Ok(agent)
    }

    async fn query_text_result<T>(&self, method: &str, arg: Vec<u8>) -> Result<T, RuntimeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let agent = self.agent().await?;
        let bytes = agent
            .query(&self.canister_id, method)
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let result = Decode!(&bytes, TextOperationResult)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        match (result.ok, result.err) {
            (Some(raw), None) => serde_json::from_str(&raw)
                .map_err(|err| RuntimeError::Serialization(format!("json decode failed: {err}"))),
            (None, Some(err)) => Err(RuntimeError::Domain(err)),
            _ => Err(RuntimeError::Serialization(format!(
                "{method} returned an invalid canister result envelope"
            ))),
        }
    }

    async fn update_text_result<T>(&self, method: &str, arg: Vec<u8>) -> Result<T, RuntimeError>
    where
        T: serde::de::DeserializeOwned,
    {
        let agent = self.agent().await?;
        let bytes = agent
            .update(&self.canister_id, method)
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|err| RuntimeError::Network(format!("update failed: {err}")))?;
        let result = Decode!(&bytes, TextOperationResult)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        match (result.ok, result.err) {
            (Some(raw), None) => serde_json::from_str(&raw)
                .map_err(|err| RuntimeError::Serialization(format!("json decode failed: {err}"))),
            (None, Some(err)) => Err(RuntimeError::Domain(err)),
            _ => Err(RuntimeError::Serialization(format!(
                "{method} returned an invalid canister result envelope"
            ))),
        }
    }

    pub async fn compile_workflow_execution_plan(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError> {
        let definition_json = serde_json::to_string(definition)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let binding_json = serde_json::to_string(binding)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let arg = Encode!(&definition_json, &binding_json)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.query_text_result("compile_workflow_v1", arg).await
    }

    pub async fn start_workflow_instance(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError> {
        let definition_json = serde_json::to_string(definition)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let binding_json = serde_json::to_string(binding)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let arg = Encode!(&definition_json, &binding_json)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.update_text_result("start_workflow_v1", arg).await
    }

    pub async fn signal_workflow_instance(
        &self,
        instance_id: &str,
        signal: &WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        let signal_json = serde_json::to_string(signal)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let arg = Encode!(&instance_id.to_string(), &signal_json)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.update_text_result("signal_workflow_v1", arg).await
    }

    pub async fn snapshot_workflow_instance(
        &self,
        instance_id: &str,
    ) -> Result<WorkflowSnapshotV1, RuntimeError> {
        let arg = Encode!(&instance_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.query_text_result("snapshot_workflow_v1", arg).await
    }

    pub async fn cancel_workflow_instance(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        let arg = Encode!(&instance_id.to_string(), &reason.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        self.update_text_result("cancel_workflow_v1", arg).await
    }
}

#[async_trait]
impl WorkflowAdapter for WorkflowCanisterClient {
    async fn get_space_execution_profile(
        &self,
        space_id: &str,
    ) -> Result<Option<ExecutionProfile>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&space_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_space_execution_profile")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Option<ExecutionProfileCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.map(map_execution_profile))
    }

    async fn get_attribution_domains(
        &self,
        space_id: &str,
    ) -> Result<Vec<AttributionDomain>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&space_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_attribution_domains")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Vec<AttributionDomainCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.into_iter().map(map_attribution_domain).collect())
    }

    async fn get_replay_contract(
        &self,
        mutation_id: &str,
    ) -> Result<Option<ReplayContract>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&mutation_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_replay_contract")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Option<ReplayContractCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.map(map_replay_contract))
    }

    async fn get_epistemic_assessment_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<EpistemicAssessment>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&mutation_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_epistemic_assessment_by_mutation")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Option<EpistemicAssessmentCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.map(map_epistemic_assessment))
    }

    async fn get_contribution_attribution_binding(
        &self,
        contribution_id: &str,
    ) -> Result<Option<ContributionAttributionBinding>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&contribution_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_contribution_attribution_binding")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Option<ContributionAttributionBindingCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.map(map_contribution_binding))
    }

    async fn list_space_replay_contracts(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<ReplayContract>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&space_id.to_string(), &limit)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "list_space_replay_contracts")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Vec<ReplayContractCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.into_iter().map(map_replay_contract).collect())
    }

    async fn get_decision_lineage_by_mutation(
        &self,
        mutation_id: &str,
    ) -> Result<Option<DecisionLineage>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&mutation_id.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "get_decision_lineage_by_mutation")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Option<DecisionLineageCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.map(map_decision_lineage))
    }

    async fn list_space_decision_lineage(
        &self,
        space_id: &str,
        limit: u32,
    ) -> Result<Vec<DecisionLineage>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&space_id.to_string(), &limit)
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "list_space_decision_lineage")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Vec<DecisionLineageCandid>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded.into_iter().map(map_decision_lineage).collect())
    }

    async fn write_file(
        &self,
        path: &str,
        content: Vec<u8>,
        mime_type: &str,
    ) -> Result<(), RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&path.to_string(), &content, &mime_type.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .update(&self.canister_id, "write_file")
            .with_arg(arg)
            .call_and_wait()
            .await
            .map_err(|err| RuntimeError::Network(format!("update failed: {err}")))?;
        Decode!(&bytes, Result<(), String>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?
            .map_err(RuntimeError::Network)
    }

    async fn read_file(&self, path: &str) -> Result<Vec<u8>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&path.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "read_file")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        Decode!(&bytes, Result<Vec<u8>, String>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?
            .map_err(RuntimeError::Network)
    }

    async fn list_files(&self, prefix: &str) -> Result<Vec<(String, FileMetadata)>, RuntimeError> {
        let agent = self.agent().await?;
        let arg = Encode!(&prefix.to_string())
            .map_err(|err| RuntimeError::Serialization(err.to_string()))?;
        let bytes = agent
            .query(&self.canister_id, "list_files")
            .with_arg(arg)
            .call()
            .await
            .map_err(|err| RuntimeError::Network(format!("query failed: {err}")))?;
        let decoded = Decode!(&bytes, Vec<(String, FileMetadataCandid)>)
            .map_err(|err| RuntimeError::Serialization(format!("decode failed: {err}")))?;
        Ok(decoded
            .into_iter()
            .map(|(path, metadata)| (path, map_file_metadata(metadata)))
            .collect())
    }
}

fn map_execution_profile(source: ExecutionProfileCandid) -> ExecutionProfile {
    ExecutionProfile {
        execution_topology: match source.execution_topology {
            ExecutionTopologyCandid::LocalOnly => ExecutionTopology::LocalOnly,
            ExecutionTopologyCandid::Networked => ExecutionTopology::Networked,
            ExecutionTopologyCandid::Hybrid => ExecutionTopology::Hybrid,
        },
        consensus_mode: match source.consensus_mode {
            ConsensusModeCandid::NoneLocal => ConsensusMode::NoneLocal,
            ConsensusModeCandid::ReplicatedConsensus => ConsensusMode::ReplicatedConsensus,
            ConsensusModeCandid::DelegatedConsensus => ConsensusMode::DelegatedConsensus,
        },
        trust_boundary: match source.trust_boundary {
            TrustBoundaryCandid::AttributedDefault => TrustBoundary::AttributedDefault,
            TrustBoundaryCandid::MixedAttribution => TrustBoundary::MixedAttribution,
            TrustBoundaryCandid::PrivacyPreferred => TrustBoundary::PrivacyPreferred,
        },
        updated_by: source.updated_by.to_text(),
        updated_at: source.updated_at,
    }
}

fn map_attribution_domain(source: AttributionDomainCandid) -> AttributionDomain {
    AttributionDomain {
        id: source.id,
        mode: match source.mode {
            AttributionModeCandid::Attributed => AttributionMode::Attributed,
            AttributionModeCandid::Pseudonymous => AttributionMode::Pseudonymous,
            AttributionModeCandid::Anonymous => AttributionMode::Anonymous,
            AttributionModeCandid::Delayed => AttributionMode::Delayed,
        },
        reattachment_policy: source.reattachment_policy,
        governance_visibility: source.governance_visibility,
        auditability_level: source.auditability_level,
        weight_policy_ref: source.weight_policy_ref,
        updated_by: source.updated_by.to_text(),
        updated_at: source.updated_at,
    }
}

fn map_contribution_binding(
    source: ContributionAttributionBindingCandid,
) -> ContributionAttributionBinding {
    ContributionAttributionBinding {
        contribution_id: source.contribution_id,
        space_id: source.space_id,
        domain_id: source.domain_id,
        bound_by: source.bound_by.to_text(),
        bound_at: source.bound_at,
    }
}

fn map_replay_contract(source: ReplayContractCandid) -> ReplayContract {
    ReplayContract {
        mutation_id: source.mutation_id,
        workflow_id: source.workflow_id,
        action_target: source.action_target,
        adapter_set_ref: source.adapter_set_ref,
        execution_profile_ref: source.execution_profile_ref,
        attribution_domain_ref: source.attribution_domain_ref,
        deterministic_input_hash: source.deterministic_input_hash,
        lineage_id: source.lineage_id,
        policy_ref: source.policy_ref,
        policy_snapshot_ref: source.policy_snapshot_ref,
        evidence_refs: source.evidence_refs,
        decision_digest: source.decision_digest,
        captured_at: source.captured_at,
    }
}

fn map_decision_lineage(source: DecisionLineageCandid) -> DecisionLineage {
    DecisionLineage {
        mutation_id: source.mutation_id,
        workflow_id: source.workflow_id,
        lineage_id: source.lineage_id,
        action_target: source.action_target,
        decision_digest: source.decision_digest,
        policy_ref: source.policy_ref,
        policy_snapshot_ref: source.policy_snapshot_ref,
        evidence_refs: source.evidence_refs,
        captured_at: source.captured_at,
    }
}

fn map_file_metadata(source: FileMetadataCandid) -> FileMetadata {
    FileMetadata {
        mime_type: source.mime_type,
        size: source.size,
        last_modified: source.last_modified,
    }
}

fn map_epistemic_assessment(source: EpistemicAssessmentCandid) -> EpistemicAssessment {
    EpistemicAssessment {
        assessment_id: source.assessment_id,
        workflow_id: source.workflow_id,
        mutation_id: source.mutation_id,
        decision_class: match source.decision_class {
            DecisionClassCandid::Governance => DecisionClass::Governance,
            DecisionClassCandid::Merge => DecisionClass::Merge,
            DecisionClassCandid::HighImpact => DecisionClass::HighImpact,
            DecisionClassCandid::Standard => DecisionClass::Standard,
        },
        confidence_score: source.confidence_score,
        source_reliability: source.source_reliability,
        robustness_score: source.robustness_score,
        voi_score: source.voi_score,
        regret_risk: source.regret_risk,
        assumption_count: source.assumption_count,
        evidence_count: source.evidence_count,
        alternative_count: source.alternative_count,
        gate_outcome: match source.gate_outcome {
            GateOutcomeCandid::Pass => GateOutcome::Pass,
            GateOutcomeCandid::Warn => GateOutcome::Warn,
            GateOutcomeCandid::RequireReview => GateOutcome::RequireReview,
            GateOutcomeCandid::RequireSimulation => GateOutcome::RequireSimulation,
            GateOutcomeCandid::Block => GateOutcome::Block,
        },
        reasons: source.reasons,
        created_at: source.created_at,
    }
}

#[async_trait]
pub trait WorkflowExecutionCanister: Send + Sync {
    async fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError>;

    async fn start(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError>;

    async fn signal(
        &self,
        instance_id: &str,
        signal: &WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError>;

    async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError>;

    async fn cancel(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError>;
}

#[async_trait]
impl WorkflowExecutionCanister for WorkflowCanisterClient {
    async fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError> {
        self.compile_workflow_execution_plan(definition, binding).await
    }

    async fn start(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError> {
        self.start_workflow_instance(definition, binding).await
    }

    async fn signal(
        &self,
        instance_id: &str,
        signal: &WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.signal_workflow_instance(instance_id, signal).await
    }

    async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError> {
        self.snapshot_workflow_instance(instance_id).await
    }

    async fn cancel(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.cancel_workflow_instance(instance_id, reason).await
    }
}

#[derive(Clone, Debug)]
pub struct WorkflowEngineCanisterExecutionAdapter<C = WorkflowCanisterClient> {
    client: C,
}

impl WorkflowEngineCanisterExecutionAdapter<WorkflowCanisterClient> {
    pub async fn from_env() -> Result<Self, RuntimeError> {
        Ok(Self {
            client: WorkflowCanisterClient::from_env().await?,
        })
    }
}

impl<C> WorkflowEngineCanisterExecutionAdapter<C> {
    pub fn new(client: C) -> Self {
        Self { client }
    }
}

#[async_trait]
impl<C> WorkflowExecutionAdapter for WorkflowEngineCanisterExecutionAdapter<C>
where
    C: WorkflowExecutionCanister + Send + Sync,
{
    async fn compile(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowExecutionPlanV1, RuntimeError> {
        validate_supported_definition(definition)?;
        self.client.compile(definition, binding).await
    }

    async fn start(
        &self,
        definition: &WorkflowDefinitionV1,
        binding: &WorkflowExecutionBindingV1,
    ) -> Result<WorkflowInstanceV1, RuntimeError> {
        self.client.start(definition, binding).await
    }

    async fn signal(
        &self,
        instance_id: &str,
        signal: WorkflowSignalV1,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.client.signal(instance_id, &signal).await
    }

    async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError> {
        self.client.snapshot(instance_id).await
    }

    async fn cancel(
        &self,
        instance_id: &str,
        reason: &str,
    ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
        self.client.cancel(instance_id, reason).await
    }
}

fn validate_supported_definition(definition: &WorkflowDefinitionV1) -> Result<(), RuntimeError> {
    let unsupported = definition
        .graph
        .nodes
        .iter()
        .filter_map(|node| {
            let code = match node.kind {
                cortex_domain::workflow::WorkflowNodeKind::EvaluationGate => Some("evaluation_gate"),
                cortex_domain::workflow::WorkflowNodeKind::Parallel => Some("parallel"),
                cortex_domain::workflow::WorkflowNodeKind::Switch => Some("switch"),
                cortex_domain::workflow::WorkflowNodeKind::SubflowRef => Some("subflow_ref"),
                _ => None,
            }?;
            Some(format!(
                "WF_CANISTER_UNSUPPORTED_NODE_KIND:{}:{}",
                node.node_id, code
            ))
        })
        .collect::<Vec<_>>();

    if unsupported.is_empty() {
        Ok(())
    } else {
        Err(RuntimeError::Domain(format!(
            "workflow_engine_canister_v1 cannot compile unsupported node kinds: {}",
            unsupported.join(", ")
        )))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use cortex_domain::workflow::{
        ContextContractV1, WorkflowCheckpointPolicyV1, WorkflowCheckpointStatus,
        WorkflowConfidence, WorkflowDraftPolicyV1, WorkflowExecutionAdapterKind,
        WorkflowExecutionProfileKind, WorkflowGraphV1, WorkflowInstanceStatus,
        WorkflowMotifKind, WorkflowNodeKind, WorkflowNodeV1, WorkflowProvenance, WorkflowScope,
    };
    use serde_json::json;

    #[derive(Clone, Default)]
    struct FakeExecutionCanister;

    #[async_trait]
    impl WorkflowExecutionCanister for FakeExecutionCanister {
        async fn compile(
            &self,
            definition: &WorkflowDefinitionV1,
            binding: &WorkflowExecutionBindingV1,
        ) -> Result<WorkflowExecutionPlanV1, RuntimeError> {
            Ok(WorkflowExecutionPlanV1 {
                plan_id: "plan-1".to_string(),
                definition_id: definition.definition_id.clone(),
                binding_id: binding.binding_id.clone(),
                adapter: binding.adapter.clone(),
                projection: json!({ "runtime": "workflow_engine_canister_v1" }),
            })
        }

        async fn start(
            &self,
            definition: &WorkflowDefinitionV1,
            binding: &WorkflowExecutionBindingV1,
        ) -> Result<WorkflowInstanceV1, RuntimeError> {
            Ok(WorkflowInstanceV1 {
                schema_version: "1.0.0".to_string(),
                instance_id: "instance-1".to_string(),
                definition_id: definition.definition_id.clone(),
                binding_id: binding.binding_id.clone(),
                status: WorkflowInstanceStatus::WaitingCheckpoint,
                scope: definition.scope.clone(),
                created_at: "1".to_string(),
                updated_at: "1".to_string(),
                definition_digest: "sha256:def".to_string(),
                binding_digest: "sha256:binding".to_string(),
                source_of_truth: "workflow_engine_canister_v1".to_string(),
                replay_contract_ref: None,
                lineage_id: None,
                degraded_reason: None,
            })
        }

        async fn signal(
            &self,
            instance_id: &str,
            _signal: &WorkflowSignalV1,
        ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
            Ok(WorkflowCheckpointResultV1 {
                instance_id: instance_id.to_string(),
                checkpoint_id: Some("checkpoint-1".to_string()),
                status: WorkflowCheckpointStatus::Resolved,
                updated_at: "2".to_string(),
            })
        }

        async fn snapshot(&self, instance_id: &str) -> Result<WorkflowSnapshotV1, RuntimeError> {
            Ok(WorkflowSnapshotV1 {
                instance: WorkflowInstanceV1 {
                    schema_version: "1.0.0".to_string(),
                    instance_id: instance_id.to_string(),
                    definition_id: "definition-1".to_string(),
                    binding_id: "binding-1".to_string(),
                    status: WorkflowInstanceStatus::Completed,
                    scope: WorkflowScope {
                        space_id: Some("space-alpha".to_string()),
                        route_id: None,
                        role: None,
                    },
                    created_at: "1".to_string(),
                    updated_at: "2".to_string(),
                    definition_digest: "sha256:def".to_string(),
                    binding_digest: "sha256:binding".to_string(),
                    source_of_truth: "workflow_engine_canister_v1".to_string(),
                    replay_contract_ref: None,
                    lineage_id: None,
                    degraded_reason: None,
                },
                trace: Vec::new(),
                checkpoints: Vec::new(),
                outcome: None,
            })
        }

        async fn cancel(
            &self,
            instance_id: &str,
            _reason: &str,
        ) -> Result<WorkflowCheckpointResultV1, RuntimeError> {
            Ok(WorkflowCheckpointResultV1 {
                instance_id: instance_id.to_string(),
                checkpoint_id: Some("checkpoint-1".to_string()),
                status: WorkflowCheckpointStatus::Cancelled,
                updated_at: "3".to_string(),
            })
        }
    }

    fn sample_definition() -> WorkflowDefinitionV1 {
        WorkflowDefinitionV1 {
            schema_version: "1.0.0".to_string(),
            definition_id: "definition-1".to_string(),
            scope: WorkflowScope {
                space_id: Some("space-alpha".to_string()),
                route_id: None,
                role: None,
            },
            intent_ref: None,
            intent: "test".to_string(),
            motif_kind: WorkflowMotifKind::Sequential,
            constraints: Vec::new(),
            graph: WorkflowGraphV1 {
                nodes: vec![WorkflowNodeV1 {
                    node_id: "node-1".to_string(),
                    label: "Checkpoint".to_string(),
                    kind: WorkflowNodeKind::HumanCheckpoint,
                    reads: Vec::new(),
                    writes: Vec::new(),
                    evidence_outputs: Vec::new(),
                    authority_requirements: Vec::new(),
                    checkpoint_policy: Some(WorkflowCheckpointPolicyV1 {
                        resume_allowed: true,
                        cancel_allowed: true,
                        pause_allowed: true,
                        timeout_seconds: Some(120),
                    }),
                    loop_policy: None,
                    subflow_ref: None,
                    config: json!({}),
                }],
                edges: Vec::new(),
            },
            context_contract: ContextContractV1::default(),
            confidence: WorkflowConfidence {
                score: 0.9,
                rationale: "test".to_string(),
            },
            lineage: Default::default(),
            policy: WorkflowDraftPolicyV1 {
                recommendation_only: false,
                require_review: true,
                allow_shadow_execution: false,
            },
            provenance: WorkflowProvenance {
                created_by: "tester".to_string(),
                created_at: "0".to_string(),
                source_mode: "test".to_string(),
            },
            governance_ref: None,
            digest: Some("sha256:def".to_string()),
        }
    }

    fn sample_binding() -> WorkflowExecutionBindingV1 {
        WorkflowExecutionBindingV1 {
            schema_version: "1.0.0".to_string(),
            binding_id: "binding-1".to_string(),
            definition_id: "definition-1".to_string(),
            adapter: WorkflowExecutionAdapterKind::WorkflowEngineCanisterV1,
            execution_profile: WorkflowExecutionProfileKind::Async,
            checkpoint_policy: None,
            runtime_limits: Default::default(),
            governance_ref: None,
            provenance: WorkflowProvenance {
                created_by: "tester".to_string(),
                created_at: "0".to_string(),
                source_mode: "test".to_string(),
            },
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn execution_adapter_round_trips_through_canister_client_trait() {
        let adapter = WorkflowEngineCanisterExecutionAdapter::new(FakeExecutionCanister);
        let definition = sample_definition();
        let binding = sample_binding();

        let plan = adapter.compile(&definition, &binding).await.expect("compile");
        assert_eq!(plan.projection["runtime"], "workflow_engine_canister_v1");

        let instance = adapter.start(&definition, &binding).await.expect("start");
        assert_eq!(instance.source_of_truth, "workflow_engine_canister_v1");

        let signal = adapter
            .signal(
                instance.instance_id.as_str(),
                WorkflowSignalV1 {
                    signal_type: "approve".to_string(),
                    checkpoint_id: Some("checkpoint-1".to_string()),
                    payload: json!({ "decision": "approve" }),
                },
            )
            .await
            .expect("signal");
        assert_eq!(signal.status, WorkflowCheckpointStatus::Resolved);

        let snapshot = adapter
            .snapshot(instance.instance_id.as_str())
            .await
            .expect("snapshot");
        assert_eq!(snapshot.instance.status, WorkflowInstanceStatus::Completed);

        let cancelled = adapter
            .cancel(instance.instance_id.as_str(), "because")
            .await
            .expect("cancel");
        assert_eq!(cancelled.status, WorkflowCheckpointStatus::Cancelled);
    }
}
