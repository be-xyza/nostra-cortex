use candid::{CandidType, Deserialize, Principal};
use ic_cdk::management_canister::raw_rand;
use ic_cdk_timers::TimerId;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{DefaultMemoryImpl, StableBTreeMap, Storable};
use nostra_workflow_core::{
    Action, Engine, Step, Transition, WorkflowDefinition, WorkflowId, WorkflowInstance,
    WorkflowStatus,
};
use serde::{Serialize, de::DeserializeOwned};
use std::borrow::Cow;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::time::Duration;

mod monitor;

use monitor::SystemStatus;

pub mod a2ui_types;
pub use a2ui_types::*;

pub mod a2ui_adapter;
pub mod flow_graph;
pub mod vfs;
use flow_graph::{FlowGraph, FlowLayout, FlowLayoutInput};

use sha2::{Digest, Sha256};

type Memory = VirtualMemory<DefaultMemoryImpl>;

const WORKFLOW_MAGIC: &[u8; 4] = b"NWF1";
const WORKFLOW_DEF_MAGIC: &[u8; 4] = b"NWD1";
const EPI_ASSESS_MAGIC: &[u8; 4] = b"NEA1";
const EPI_POLICY_MAGIC: &[u8; 4] = b"NEP1";
const EPI_AUTH_MAGIC: &[u8; 4] = b"NEQ1";
const EPI_OVERRIDE_MAGIC: &[u8; 4] = b"NEO1";
const EXEC_PROFILE_MAGIC: &[u8; 4] = b"NXP1";
const ATTR_DOMAIN_MAGIC: &[u8; 4] = b"NAD1";
const ATTR_BINDING_MAGIC: &[u8; 4] = b"NAB1";
const REPLAY_CONTRACT_MAGIC: &[u8; 4] = b"NRC1";
const CHAT_THREAD_MAGIC: &[u8; 4] = b"NCT1";
const CHAT_THREAD_MAX_TURNS: usize = 8;

fn decode_storable_payload<T: DeserializeOwned>(
    bytes: &[u8],
    magic: &[u8],
    label: &str,
    fallback: impl FnOnce() -> T,
) -> T {
    if let Some(payload) = bytes.strip_prefix(magic) {
        if let Ok(decoded) = postcard::from_bytes::<T>(payload) {
            return decoded;
        }
        ic_cdk::println!(
            "WorkflowEngine: failed to decode magic-prefixed {label} payload; attempting fallback paths"
        );
    } else if let Ok(decoded) = postcard::from_bytes::<T>(bytes) {
        ic_cdk::println!("WorkflowEngine: legacy {label} payload decoded without magic prefix");
        return decoded;
    } else {
        ic_cdk::println!(
            "WorkflowEngine: legacy {label} payload not recognized; applying recovery fallback"
        );
    }
    fallback()
}

fn decode_storable_key(bytes: &[u8], label: &str, fallback: &str) -> String {
    match String::from_utf8(bytes.to_vec()) {
        Ok(value) => value,
        Err(err) => {
            ic_cdk::println!(
                "WorkflowEngine: failed to decode {label} key as UTF-8; using fallback ({err})"
            );
            fallback.to_string()
        }
    }
}

fn fallback_workflow_definition() -> WorkflowDefinition {
    WorkflowDefinition {
        id: "legacy-recovered-definition".to_string(),
        steps: HashMap::new(),
        start_step_id: "start".to_string(),
    }
}

fn fallback_workflow_instance() -> WorkflowInstance {
    WorkflowInstance {
        id: "legacy-recovered-workflow".to_string(),
        definition: fallback_workflow_definition(),
        current_step_id: None,
        status: WorkflowStatus::Failed("legacy workflow decode failed".to_string()),
        context: nostra_workflow_core::Context::new(),
    }
}

fn fallback_epistemic_assessment() -> EpistemicAssessment {
    EpistemicAssessment {
        assessment_id: "legacy-recovered-assessment".to_string(),
        workflow_id: "workflow:legacy".to_string(),
        mutation_id: "mutation:legacy".to_string(),
        decision_class: DecisionClass::Standard,
        confidence_score: 0.0,
        source_reliability: 0.0,
        robustness_score: 0.0,
        voi_score: 0.0,
        regret_risk: 1.0,
        assumption_count: 0,
        evidence_count: 0,
        alternative_count: 0,
        gate_outcome: GateOutcome::RequireReview,
        reasons: vec!["legacy epistemic assessment decode failed".to_string()],
        created_at: 0,
    }
}

fn fallback_epistemic_policy() -> EpistemicPolicy {
    default_epistemic_policy()
}

fn fallback_epistemic_policy_authority() -> EpistemicPolicyAuthority {
    EpistemicPolicyAuthority::LocalAdmin
}

fn fallback_epistemic_override_ack() -> EpistemicOverrideAck {
    EpistemicOverrideAck {
        assessment_id: "legacy-recovered-assessment".to_string(),
        mutation_id: "mutation:legacy".to_string(),
        workflow_id: "workflow:legacy".to_string(),
        justification: "legacy override decode failed".to_string(),
        approved_by: Principal::anonymous(),
        approved_at: 0,
    }
}

fn fallback_execution_profile() -> ExecutionProfile {
    ExecutionProfile {
        execution_topology: ExecutionTopology::LocalOnly,
        consensus_mode: ConsensusMode::NoneLocal,
        trust_boundary: TrustBoundary::AttributedDefault,
        updated_by: Principal::anonymous(),
        updated_at: 0,
    }
}

fn fallback_attribution_domain() -> AttributionDomain {
    AttributionDomain {
        id: "legacy-default".to_string(),
        mode: AttributionMode::Attributed,
        reattachment_policy: "manual".to_string(),
        governance_visibility: "full".to_string(),
        auditability_level: "standard".to_string(),
        weight_policy_ref: None,
        updated_by: Principal::anonymous(),
        updated_at: 0,
    }
}

fn fallback_contribution_binding() -> ContributionAttributionBinding {
    ContributionAttributionBinding {
        contribution_id: "legacy".to_string(),
        space_id: "legacy".to_string(),
        domain_id: "legacy-default".to_string(),
        bound_by: Principal::anonymous(),
        bound_at: 0,
    }
}

fn fallback_replay_contract() -> ReplayContract {
    ReplayContract {
        mutation_id: "legacy".to_string(),
        workflow_id: "workflow:legacy".to_string(),
        action_target: "action_target:legacy".to_string(),
        adapter_set_ref: "nostra://workflow_engine/legacy_recovery".to_string(),
        execution_profile_ref: "system_execution_profile:unknown".to_string(),
        attribution_domain_ref: "system_attribution_domain:unknown".to_string(),
        deterministic_input_hash: "legacy".to_string(),
        lineage_id: None,
        policy_ref: None,
        policy_snapshot_ref: None,
        evidence_refs: Vec::new(),
        decision_digest: None,
        captured_at: 0,
    }
}

fn fallback_chat_thread_state() -> ChatThreadState {
    ChatThreadState {
        thread_id: "legacy-thread".to_string(),
        turns: Vec::new(),
        updated_at: 0,
    }
}

#[derive(Deserialize)]
struct StorableWorkflow(WorkflowInstance);

impl Storable for StorableWorkflow {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(WORKFLOW_MAGIC.len() + payload.len());
        bytes.extend_from_slice(WORKFLOW_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableWorkflow(decode_storable_payload(
            bytes,
            WORKFLOW_MAGIC,
            "workflow instance",
            fallback_workflow_instance,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableWorkflowDefinition(WorkflowDefinition);

impl Storable for StorableWorkflowDefinition {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(WORKFLOW_DEF_MAGIC.len() + payload.len());
        bytes.extend_from_slice(WORKFLOW_DEF_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableWorkflowDefinition(decode_storable_payload(
            bytes,
            WORKFLOW_DEF_MAGIC,
            "workflow definition",
            fallback_workflow_definition,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableWorkflowId(String);

impl Storable for StorableWorkflowId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableWorkflowId(decode_storable_key(
            bytes.as_ref(),
            "workflow_id",
            "legacy-workflow-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 64,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableWorkflowDefinitionId(String);

impl Storable for StorableWorkflowDefinitionId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableWorkflowDefinitionId(decode_storable_key(
            bytes.as_ref(),
            "workflow_definition_id",
            "legacy-workflow-definition-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableMutationId(String);

impl Storable for StorableMutationId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableMutationId(decode_storable_key(
            bytes.as_ref(),
            "mutation_id",
            "legacy-mutation-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableAssessmentId(String);

impl Storable for StorableAssessmentId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableAssessmentId(decode_storable_key(
            bytes.as_ref(),
            "assessment_id",
            "legacy-assessment-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableSpaceId(String);

impl Storable for StorableSpaceId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableSpaceId(decode_storable_key(
            bytes.as_ref(),
            "space_id",
            "legacy-space-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableCompositeId(String);

impl Storable for StorableCompositeId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableCompositeId(decode_storable_key(
            bytes.as_ref(),
            "composite_id",
            "legacy-composite-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 256,
            is_fixed_size: false,
        };
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableContributionId(String);

impl Storable for StorableContributionId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableContributionId(decode_storable_key(
            bytes.as_ref(),
            "contribution_id",
            "legacy-contribution-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 256,
            is_fixed_size: false,
        };
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum EpistemicMode {
    #[serde(rename = "observe")]
    Observe,
    #[serde(rename = "soft_gate")]
    SoftGate,
    #[serde(rename = "hard_gate")]
    HardGate,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum DecisionClass {
    #[serde(rename = "governance")]
    Governance,
    #[serde(rename = "merge")]
    Merge,
    #[serde(rename = "high_impact")]
    HighImpact,
    #[serde(rename = "standard")]
    Standard,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum GateOutcome {
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
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

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EpistemicPolicy {
    pub mode: EpistemicMode,
    pub min_evidence: u32,
    pub min_alternatives: u32,
    pub min_robustness: f64,
    pub max_confidence_drift: f64,
    pub max_fork_pressure: f64,
    pub max_correction_density: f64,
    pub simulation_ttl_days: u32,
    pub enforced_decision_classes: Vec<DecisionClass>,
    pub block_on_soft: bool,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum EpistemicPolicyAuthority {
    LocalAdmin,
    GovernanceCanister(Principal),
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EpistemicOverrideAck {
    pub assessment_id: String,
    pub mutation_id: String,
    pub workflow_id: String,
    pub justification: String,
    pub approved_by: Principal,
    pub approved_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ExecutionTopology {
    LocalOnly,
    Networked,
    Hybrid,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum ConsensusMode {
    NoneLocal,
    ReplicatedConsensus,
    DelegatedConsensus,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum TrustBoundary {
    AttributedDefault,
    MixedAttribution,
    PrivacyPreferred,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ExecutionProfile {
    pub execution_topology: ExecutionTopology,
    pub consensus_mode: ConsensusMode,
    pub trust_boundary: TrustBoundary,
    pub updated_by: Principal,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, PartialEq, Eq)]
pub enum AttributionMode {
    #[serde(rename = "attributed")]
    Attributed,
    #[serde(rename = "pseudonymous")]
    Pseudonymous,
    #[serde(rename = "anonymous")]
    Anonymous,
    #[serde(rename = "delayed")]
    Delayed,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct AttributionDomain {
    pub id: String,
    pub mode: AttributionMode,
    pub reattachment_policy: String,
    pub governance_visibility: String,
    pub auditability_level: String,
    pub weight_policy_ref: Option<String>,
    pub updated_by: Principal,
    pub updated_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ContributionAttributionBinding {
    pub contribution_id: String,
    pub space_id: String,
    pub domain_id: String,
    pub bound_by: Principal,
    pub bound_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ReplayContract {
    pub mutation_id: String,
    pub workflow_id: String,
    pub action_target: String,
    pub adapter_set_ref: String,
    pub execution_profile_ref: String,
    pub attribution_domain_ref: String,
    pub deterministic_input_hash: String,
    pub lineage_id: Option<String>,
    pub policy_ref: Option<String>,
    pub policy_snapshot_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub decision_digest: Option<String>,
    pub captured_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct DecisionLineage {
    pub mutation_id: String,
    pub workflow_id: String,
    pub lineage_id: String,
    pub action_target: String,
    pub decision_digest: String,
    pub policy_ref: Option<String>,
    pub policy_snapshot_ref: Option<String>,
    pub evidence_refs: Vec<String>,
    pub captured_at: u64,
}

#[derive(Deserialize)]
struct StorableEpistemicAssessment(EpistemicAssessment);

impl Storable for StorableEpistemicAssessment {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(EPI_ASSESS_MAGIC.len() + payload.len());
        bytes.extend_from_slice(EPI_ASSESS_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableEpistemicAssessment(decode_storable_payload(
            bytes,
            EPI_ASSESS_MAGIC,
            "epistemic assessment",
            fallback_epistemic_assessment,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableEpistemicPolicy(EpistemicPolicy);

impl Storable for StorableEpistemicPolicy {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(EPI_POLICY_MAGIC.len() + payload.len());
        bytes.extend_from_slice(EPI_POLICY_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableEpistemicPolicy(decode_storable_payload(
            bytes,
            EPI_POLICY_MAGIC,
            "epistemic policy",
            fallback_epistemic_policy,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableEpistemicPolicyAuthority(EpistemicPolicyAuthority);

impl Storable for StorableEpistemicPolicyAuthority {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(EPI_AUTH_MAGIC.len() + payload.len());
        bytes.extend_from_slice(EPI_AUTH_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableEpistemicPolicyAuthority(decode_storable_payload(
            bytes,
            EPI_AUTH_MAGIC,
            "epistemic policy authority",
            fallback_epistemic_policy_authority,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableEpistemicOverrideAck(EpistemicOverrideAck);

impl Storable for StorableEpistemicOverrideAck {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(EPI_OVERRIDE_MAGIC.len() + payload.len());
        bytes.extend_from_slice(EPI_OVERRIDE_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableEpistemicOverrideAck(decode_storable_payload(
            bytes,
            EPI_OVERRIDE_MAGIC,
            "epistemic override",
            fallback_epistemic_override_ack,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableExecutionProfile(ExecutionProfile);

impl Storable for StorableExecutionProfile {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(EXEC_PROFILE_MAGIC.len() + payload.len());
        bytes.extend_from_slice(EXEC_PROFILE_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableExecutionProfile(decode_storable_payload(
            bytes,
            EXEC_PROFILE_MAGIC,
            "execution profile",
            fallback_execution_profile,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableAttributionDomain(AttributionDomain);

impl Storable for StorableAttributionDomain {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(ATTR_DOMAIN_MAGIC.len() + payload.len());
        bytes.extend_from_slice(ATTR_DOMAIN_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableAttributionDomain(decode_storable_payload(
            bytes,
            ATTR_DOMAIN_MAGIC,
            "attribution domain",
            fallback_attribution_domain,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableContributionAttributionBinding(ContributionAttributionBinding);

impl Storable for StorableContributionAttributionBinding {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(ATTR_BINDING_MAGIC.len() + payload.len());
        bytes.extend_from_slice(ATTR_BINDING_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableContributionAttributionBinding(decode_storable_payload(
            bytes,
            ATTR_BINDING_MAGIC,
            "attribution binding",
            fallback_contribution_binding,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(Deserialize)]
struct StorableReplayContract(ReplayContract);

impl Storable for StorableReplayContract {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(REPLAY_CONTRACT_MAGIC.len() + payload.len());
        bytes.extend_from_slice(REPLAY_CONTRACT_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableReplayContract(decode_storable_payload(
            bytes,
            REPLAY_CONTRACT_MAGIC,
            "replay contract",
            fallback_replay_contract,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatMessageEnvelope {
    pub text: String,
    #[serde(default)]
    pub thread_id: Option<String>,
    #[serde(default)]
    pub conversation_id: Option<String>,
    #[serde(default)]
    pub context_refs: Vec<String>,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatTurn {
    pub role: String,
    pub content: String,
    pub created_at: u64,
}

#[derive(CandidType, Deserialize, Serialize, Clone, Debug, Default, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ChatThreadState {
    pub thread_id: String,
    pub turns: Vec<ChatTurn>,
    pub updated_at: u64,
}

#[derive(Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct StorableChatThreadId(String);

impl Storable for StorableChatThreadId {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        Cow::Borrowed(self.0.as_bytes())
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        StorableChatThreadId(decode_storable_key(
            bytes.as_ref(),
            "chat_thread_id",
            "legacy-thread-id",
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Bounded {
            max_size: 128,
            is_fixed_size: false,
        };
}

#[derive(Deserialize)]
struct StorableChatThreadState(ChatThreadState);

impl Storable for StorableChatThreadState {
    fn to_bytes(&self) -> Cow<'_, [u8]> {
        let payload = postcard::to_stdvec(&self.0).unwrap();
        let mut bytes = Vec::with_capacity(CHAT_THREAD_MAGIC.len() + payload.len());
        bytes.extend_from_slice(CHAT_THREAD_MAGIC);
        bytes.extend_from_slice(&payload);
        Cow::Owned(bytes)
    }

    fn from_bytes(bytes: Cow<[u8]>) -> Self {
        let bytes = bytes.as_ref();
        StorableChatThreadState(decode_storable_payload(
            bytes,
            CHAT_THREAD_MAGIC,
            "chat thread state",
            fallback_chat_thread_state,
        ))
    }

    fn into_bytes(self) -> Vec<u8> {
        self.to_bytes().into_owned()
    }

    const BOUND: ic_stable_structures::storable::Bound =
        ic_stable_structures::storable::Bound::Unbounded;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static WORKFLOWS: RefCell<StableBTreeMap<StorableWorkflowId, StorableWorkflow, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0)))
        )
    );

    static WORKFLOW_DEFINITIONS: RefCell<StableBTreeMap<StorableWorkflowDefinitionId, StorableWorkflowDefinition, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
        )
    );

    static OFFLINE_CONFLICT_INDEX: RefCell<StableBTreeMap<StorableMutationId, StorableWorkflowId, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
        )
    );

    static EPISTEMIC_ASSESSMENTS: RefCell<StableBTreeMap<StorableAssessmentId, StorableEpistemicAssessment, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
        )
    );

    static EPISTEMIC_ASSESSMENT_BY_MUTATION: RefCell<StableBTreeMap<StorableMutationId, StorableAssessmentId, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(5)))
        )
    );

    static EPISTEMIC_POLICY_STORE: RefCell<StableBTreeMap<StorableWorkflowDefinitionId, StorableEpistemicPolicy, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(6)))
        )
    );

    static EPISTEMIC_POLICY_AUTHORITY_STORE: RefCell<StableBTreeMap<StorableWorkflowDefinitionId, StorableEpistemicPolicyAuthority, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(8)))
        )
    );

    static EPISTEMIC_OVERRIDE_STORE: RefCell<StableBTreeMap<StorableAssessmentId, StorableEpistemicOverrideAck, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(7)))
        )
    );

    static SPACE_EXECUTION_PROFILES: RefCell<StableBTreeMap<StorableSpaceId, StorableExecutionProfile, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(9)))
        )
    );

    static ATTRIBUTION_DOMAINS: RefCell<StableBTreeMap<StorableCompositeId, StorableAttributionDomain, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(10)))
        )
    );

    static CONTRIBUTION_ATTRIBUTION_BINDINGS: RefCell<StableBTreeMap<StorableContributionId, StorableContributionAttributionBinding, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(11)))
        )
    );

    static REPLAY_CONTRACTS: RefCell<StableBTreeMap<StorableMutationId, StorableReplayContract, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(12)))
        )
    );

    static CHAT_THREADS: RefCell<StableBTreeMap<StorableChatThreadId, StorableChatThreadState, Memory>> = RefCell::new(
        StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(13)))
        )
    );

    static EPISTEMIC_POLICY_ADMIN: RefCell<Option<Principal>> = RefCell::new(None);

    static TIMER_ID: RefCell<Option<TimerId>> = RefCell::new(None);
}

// --------------------------------------------------------------------------------
// Public API

// --------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictMutation {
    #[serde(default)]
    pub id: String,
    #[serde(default)]
    pub idempotency_key: String,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub kip_command: String,
    #[serde(default)]
    pub timestamp: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictEvent {
    #[serde(default)]
    pub kind: String,
    #[serde(default)]
    pub error: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
    #[serde(default)]
    pub mutation: OfflineConflictMutation,
}

#[derive(Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictDecision {
    #[serde(default)]
    pub mutation_id: String,
    #[serde(default)]
    pub decision: String,
    #[serde(default)]
    pub source: Option<String>,
    #[serde(default)]
    pub workflow_id: Option<String>,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug, Default)]
pub struct OfflineConflictSummary {
    pub mutation_id: String,
    pub workflow_id: String,
    pub kind: String,
    pub error: String,
    pub source: Option<String>,
    pub status: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubFile {
    #[serde(rename = "@context", default)]
    pub context: serde_json::Value,
    pub meta: DPubMeta,
    #[serde(default)]
    pub manifest: Option<DPubManifest>,
    pub content: Vec<DPubChapter>,
    #[serde(default)]
    pub editions: Vec<EditionSummary>,
    #[serde(default)]
    pub latest_edition: Option<String>,
    #[serde(default)]
    pub hypothesis: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubMeta {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub phase: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    pub provenance: DPubProvenance,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubProvenance {
    pub author_did: String,
    pub space_did: String,
    pub created_at: String,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubManifest {
    pub chapters: Vec<ManifestNode>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct ManifestNode {
    pub id: String,
    pub title_cache: String,
    #[serde(default)]
    pub children: Vec<ManifestNode>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DPubChapter {
    pub id: String,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(rename = "type")]
    pub content_type: String,
    pub blocks: Vec<Block>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum Block {
    #[serde(rename = "Block::Heading")]
    Heading { level: u8, content: String },
    #[serde(rename = "Block::Paragraph")]
    Paragraph { content: ContentValue },
    #[serde(rename = "Block::Reference")]
    Reference {
        ref_id: String,
        display_text: String,
    },
    #[serde(rename = "Block::VersionedReference")]
    VersionedReference {
        urn: String,
        display_text: String,
        #[serde(default)]
        version: Option<serde_json::Value>,
    },
    #[serde(untagged)]
    LegacyHtml { content: String },
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(untagged)]
pub enum ContentValue {
    String(String),
    Rich(Vec<RichTextSpan>),
}

#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
pub enum RichTextSpan {
    Text { value: String },
    Bold { value: String },
    Italic { value: String },
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EditionSummary {
    pub edition_id: String,
    pub version: String,
    pub published_at: String,
    pub content_root: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EditionManifest {
    pub edition_id: String,
    pub dpub_id: String,
    pub version: String,
    #[serde(default)]
    pub name: Option<String>,
    pub content_root: String,
    pub chapters: Vec<ChapterManifest>,
    pub published_at: String,
    pub publisher: String,
    #[serde(default)]
    pub previous_edition: Option<String>,
    pub metadata: EditionMetadata,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct EditionMetadata {
    pub license: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ChapterManifest {
    pub index: u32,
    pub contribution_ref: ContributionVersionRef,
    pub content_hash: String,
    pub title: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct ContributionVersionRef {
    pub contribution_id: String,
    pub version_hash: String,
}

#[derive(candid::CandidType, Deserialize, Serialize, Clone, Debug)]
pub struct PublishContext {
    pub commit_hash: String,
    #[serde(default)]
    pub source_ref: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct SnapshotManifestFile {
    path: String,
    sha256: String,
    size_bytes: u64,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct SnapshotManifestEntrypoints {
    dpub_path: String,
    edition_manifest_path: String,
    snapshot_path: String,
    snapshot_manifest_path: String,
    #[serde(default)]
    feed_path: Option<String>,
}

#[derive(Deserialize, Serialize, Clone, Debug)]
struct SnapshotManifest {
    bundle_id: String,
    commit_hash: String,
    generated_at: String,
    bundle_version: String,
    files: Vec<SnapshotManifestFile>,
    entrypoints: SnapshotManifestEntrypoints,
}

fn sha256_hex(bytes: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    hex::encode(hasher.finalize())
}

fn merkle_root_hex(leaf_hashes_hex: &[String]) -> String {
    if leaf_hashes_hex.is_empty() {
        return sha256_hex(b"");
    }
    if leaf_hashes_hex.len() == 1 {
        return leaf_hashes_hex[0].clone();
    }

    let mut level: Vec<String> = leaf_hashes_hex.to_vec();
    while level.len() > 1 {
        let mut next = Vec::with_capacity((level.len() + 1) / 2);
        for chunk in level.chunks(2) {
            let combined = if chunk.len() == 2 {
                format!("{}{}", chunk[0], chunk[1])
            } else {
                chunk[0].clone()
            };
            next.push(sha256_hex(combined.as_bytes()));
        }
        level = next;
    }
    level[0].clone()
}

fn clamp01(value: f64) -> f64 {
    value.clamp(0.0, 1.0)
}

fn default_epistemic_policy() -> EpistemicPolicy {
    EpistemicPolicy {
        mode: EpistemicMode::Observe,
        min_evidence: 2,
        min_alternatives: 2,
        min_robustness: 0.70,
        max_confidence_drift: 0.25,
        max_fork_pressure: 0.60,
        max_correction_density: 0.50,
        simulation_ttl_days: 30,
        enforced_decision_classes: vec![DecisionClass::Governance, DecisionClass::Merge],
        block_on_soft: false,
    }
}

fn default_epistemic_policy_authority() -> EpistemicPolicyAuthority {
    EpistemicPolicyAuthority::LocalAdmin
}

fn policy_store_key() -> StorableWorkflowDefinitionId {
    StorableWorkflowDefinitionId("default_epistemic_policy".to_string())
}

fn policy_authority_store_key() -> StorableWorkflowDefinitionId {
    StorableWorkflowDefinitionId("default_epistemic_policy_authority".to_string())
}

fn decision_class_label(class: &DecisionClass) -> &'static str {
    match class {
        DecisionClass::Governance => "governance",
        DecisionClass::Merge => "merge",
        DecisionClass::HighImpact => "high_impact",
        DecisionClass::Standard => "standard",
    }
}

fn parse_decision_class(value: Option<String>) -> DecisionClass {
    match value
        .unwrap_or_else(|| "standard".to_string())
        .trim()
        .to_lowercase()
        .as_str()
    {
        "governance" => DecisionClass::Governance,
        "merge" => DecisionClass::Merge,
        "high_impact" | "high-impact" => DecisionClass::HighImpact,
        _ => DecisionClass::Standard,
    }
}

fn is_enforced_class(policy: &EpistemicPolicy, class: &DecisionClass) -> bool {
    policy.enforced_decision_classes.iter().any(|c| c == class)
}

fn parse_ctx_f64(ctx: &nostra_workflow_core::Context, key: &str, default: f64) -> f64 {
    ctx.get(key)
        .and_then(|v| v.parse::<f64>().ok())
        .unwrap_or(default)
}

fn parse_ctx_u32(ctx: &nostra_workflow_core::Context, key: &str, default: u32) -> u32 {
    ctx.get(key)
        .and_then(|v| v.parse::<u32>().ok())
        .unwrap_or(default)
}

fn normalize_required_id(value: &str, field: &str) -> Result<String, String> {
    let normalized = value.trim();
    if normalized.is_empty() {
        return Err(format!("{field} is required"));
    }
    Ok(normalized.to_string())
}

fn attribution_domain_store_key(space_id: &str, domain_id: &str) -> StorableCompositeId {
    StorableCompositeId(format!("{space_id}::{domain_id}"))
}

fn attribution_domain_prefix(space_id: &str) -> String {
    format!("{space_id}::")
}

fn execution_profile_surface_ref(space_id: &str) -> String {
    format!("system_execution_profile:{space_id}")
}

fn attribution_domain_surface_ref(space_id: &str, domain_id: &str) -> String {
    format!("system_attribution_domain:{space_id}:{domain_id}")
}

fn replay_input_hash(payload: &serde_json::Value) -> String {
    match serde_json::to_vec(payload) {
        Ok(bytes) => sha256_hex(&bytes),
        Err(_) => sha256_hex(b"{}"),
    }
}

fn replay_decision_digest(
    mutation_id: &str,
    policy_ref: Option<&str>,
    policy_snapshot_ref: Option<&str>,
    evidence_refs: &[String],
) -> String {
    let canonical = serde_json::json!({
        "mutation_id": mutation_id,
        "policy_ref": policy_ref.unwrap_or("none"),
        "policy_snapshot_ref": policy_snapshot_ref.unwrap_or("none"),
        "evidence_refs": evidence_refs
    });
    replay_input_hash(&canonical)
}

fn policy_snapshot_ref(policy: &EpistemicPolicy) -> String {
    format!(
        "epistemic_policy:{:?}:e{}:a{}:r{:.3}",
        policy.mode, policy.min_evidence, policy.min_alternatives, policy.min_robustness
    )
}

fn replay_contract_matches_space(contract: &ReplayContract, space_id: &str) -> bool {
    let profile_ref = execution_profile_surface_ref(space_id);
    let domain_prefix = format!("system_attribution_domain:{space_id}:");
    contract.execution_profile_ref == profile_ref
        || contract.attribution_domain_ref.starts_with(&domain_prefix)
        || contract.workflow_id.contains(space_id)
}

fn decision_lineage_from_replay(contract: &ReplayContract) -> Option<DecisionLineage> {
    let lineage_id = contract.lineage_id.clone()?;
    let digest = contract.decision_digest.clone().unwrap_or_else(|| {
        replay_decision_digest(
            &contract.mutation_id,
            contract.policy_ref.as_deref(),
            contract.policy_snapshot_ref.as_deref(),
            &contract.evidence_refs,
        )
    });
    Some(DecisionLineage {
        mutation_id: contract.mutation_id.clone(),
        workflow_id: contract.workflow_id.clone(),
        lineage_id,
        action_target: contract.action_target.clone(),
        decision_digest: digest,
        policy_ref: contract.policy_ref.clone(),
        policy_snapshot_ref: contract.policy_snapshot_ref.clone(),
        evidence_refs: contract.evidence_refs.clone(),
        captured_at: contract.captured_at,
    })
}

fn lookup_execution_profile_ref(space_id: Option<&str>) -> String {
    let Some(space_id) = space_id.filter(|id| !id.trim().is_empty()) else {
        return "system_execution_profile:unknown".to_string();
    };
    let exists = SPACE_EXECUTION_PROFILES.with(|store| {
        store
            .borrow()
            .contains_key(&StorableSpaceId(space_id.to_string()))
    });
    if exists {
        execution_profile_surface_ref(space_id)
    } else {
        "system_execution_profile:unknown".to_string()
    }
}

fn lookup_binding_domain_ref(space_id: Option<&str>, mutation_id: &str) -> String {
    if let Some(space_id) = space_id {
        if let Some(domain_id) = CONTRIBUTION_ATTRIBUTION_BINDINGS.with(|bindings| {
            bindings
                .borrow()
                .get(&StorableContributionId(mutation_id.to_string()))
                .map(|binding| binding.0.domain_id)
        }) {
            return attribution_domain_surface_ref(space_id, &domain_id);
        }
    }
    "system_attribution_domain:unknown".to_string()
}

fn store_replay_contract(contract: ReplayContract) {
    REPLAY_CONTRACTS.with(|store| {
        store.borrow_mut().insert(
            StorableMutationId(contract.mutation_id.clone()),
            StorableReplayContract(contract),
        );
    });
}

fn contains_any_token(haystack: &str, terms: &[&str]) -> bool {
    terms.iter().any(|term| haystack.contains(term))
}

fn validate_override_quality(justification: &str) -> Result<(), String> {
    let normalized = justification.trim().to_ascii_lowercase();
    if normalized.is_empty() {
        return Err("override acknowledgement has missing required fields".to_string());
    }
    if normalized.split_whitespace().count() < 6 {
        return Err("override justification must include at least 6 words".to_string());
    }
    if !contains_any_token(&normalized, &["risk", "impact", "blast radius", "safety"]) {
        return Err("override justification must include risk/impact context".to_string());
    }
    if !contains_any_token(&normalized, &["rollback", "revert", "fallback", "backout"]) {
        return Err("override justification must include rollback/backout path".to_string());
    }
    if !contains_any_token(
        &normalized,
        &["evidence", "test", "log", "metric", "runbook"],
    ) {
        return Err(
            "override justification must include supporting evidence reference".to_string(),
        );
    }
    Ok(())
}

fn load_epistemic_policy() -> EpistemicPolicy {
    EPISTEMIC_POLICY_STORE.with(|store| {
        store
            .borrow()
            .get(&policy_store_key())
            .map(|s| s.0.clone())
            .unwrap_or_else(default_epistemic_policy)
    })
}

fn save_epistemic_policy(policy: EpistemicPolicy) {
    EPISTEMIC_POLICY_STORE.with(|store| {
        store
            .borrow_mut()
            .insert(policy_store_key(), StorableEpistemicPolicy(policy));
    });
}

fn load_epistemic_policy_authority() -> EpistemicPolicyAuthority {
    EPISTEMIC_POLICY_AUTHORITY_STORE.with(|store| {
        store
            .borrow()
            .get(&policy_authority_store_key())
            .map(|entry| entry.0.clone())
            .unwrap_or_else(default_epistemic_policy_authority)
    })
}

fn save_epistemic_policy_authority(authority: EpistemicPolicyAuthority) {
    EPISTEMIC_POLICY_AUTHORITY_STORE.with(|store| {
        store.borrow_mut().insert(
            policy_authority_store_key(),
            StorableEpistemicPolicyAuthority(authority),
        );
    });
}

fn resolve_policy_admin(caller: Principal) -> Result<(), String> {
    EPISTEMIC_POLICY_ADMIN.with(|admin| {
        let mut admin = admin.borrow_mut();
        match *admin {
            Some(existing) if existing != caller => {
                Err("Only policy admin may update epistemic policy".to_string())
            }
            Some(_) => Ok(()),
            None => {
                *admin = Some(caller);
                Ok(())
            }
        }
    })
}

fn ensure_controller(caller: Principal) -> Result<(), String> {
    if ic_cdk::api::is_controller(&caller) {
        Ok(())
    } else {
        Err("Only canister controllers may update epistemic policy authority".to_string())
    }
}

fn authorize_policy_update(caller: Principal) -> Result<(), String> {
    match load_epistemic_policy_authority() {
        EpistemicPolicyAuthority::LocalAdmin => resolve_policy_admin(caller),
        EpistemicPolicyAuthority::GovernanceCanister(governance_id) => {
            if caller == governance_id {
                Ok(())
            } else {
                Err(format!(
                    "Only governance canister {} may update epistemic policy",
                    governance_id
                ))
            }
        }
    }
}

#[derive(Clone, Debug)]
struct EpistemicSignalSnapshot {
    confidence_score: f64,
    source_reliability: f64,
    assumption_count: u32,
    evidence_count: u32,
    alternative_count: u32,
    confidence_drift: f64,
    fork_pressure: f64,
    correction_density: f64,
}

#[derive(Clone, Debug)]
struct EpistemicGateComputation {
    robustness_score: f64,
    voi_score: f64,
    regret_risk: f64,
    gate_outcome: GateOutcome,
    reasons: Vec<String>,
}

fn build_assessment_id(workflow_id: &str, mutation_id: &str) -> String {
    let material = format!("{}::{}", workflow_id, mutation_id);
    format!("epi_{}", sha256_hex(material.as_bytes()))
}

fn compute_epistemic_gate(
    policy: &EpistemicPolicy,
    decision_class: &DecisionClass,
    signals: &EpistemicSignalSnapshot,
) -> EpistemicGateComputation {
    let evidence_factor =
        clamp01(signals.evidence_count as f64 / policy.min_evidence.max(1) as f64);
    let alternatives_factor =
        clamp01(signals.alternative_count as f64 / policy.min_alternatives.max(1) as f64);
    let decision_impact = match decision_class {
        DecisionClass::Governance | DecisionClass::Merge => 1.0,
        DecisionClass::HighImpact => 0.8,
        DecisionClass::Standard => 0.4,
    };

    let robustness_score = clamp01(
        (0.45 * signals.confidence_score)
            + (0.25 * signals.source_reliability)
            + (0.15 * evidence_factor)
            + (0.15 * alternatives_factor)
            - (0.20 * signals.confidence_drift)
            - (0.15 * signals.fork_pressure)
            - (0.10 * signals.correction_density),
    );
    let voi_score = clamp01(((1.0 - robustness_score) * 0.70) + (decision_impact * 0.30));
    let regret_risk = clamp01(
        (signals.confidence_drift * 0.40)
            + (signals.fork_pressure * 0.25)
            + (signals.correction_density * 0.20)
            + ((1.0 - robustness_score) * 0.25),
    );

    let mut reasons = Vec::<String>::new();
    let mut gate_outcome = GateOutcome::Pass;

    if signals.evidence_count < policy.min_evidence {
        reasons.push(format!(
            "evidence_count {} below minimum {}",
            signals.evidence_count, policy.min_evidence
        ));
        gate_outcome = GateOutcome::RequireReview;
    }

    if signals.alternative_count < policy.min_alternatives
        && matches!(
            decision_class,
            DecisionClass::Governance | DecisionClass::Merge
        )
    {
        reasons.push(format!(
            "alternative_count {} below minimum {} for {} decisions",
            signals.alternative_count,
            policy.min_alternatives,
            decision_class_label(decision_class)
        ));
        gate_outcome = GateOutcome::RequireReview;
    }

    if robustness_score < policy.min_robustness && voi_score >= 0.35 {
        reasons.push(format!(
            "robustness {:.3} below threshold {:.3} with high VoI {:.3}",
            robustness_score, policy.min_robustness, voi_score
        ));
        gate_outcome = GateOutcome::RequireSimulation;
    }

    if signals.confidence_drift > policy.max_confidence_drift
        || signals.fork_pressure > policy.max_fork_pressure
        || signals.correction_density > policy.max_correction_density
    {
        reasons.push(format!(
            "alignment pressure drift={:.3} fork={:.3} correction={:.3}",
            signals.confidence_drift, signals.fork_pressure, signals.correction_density
        ));
        if gate_outcome == GateOutcome::Pass {
            gate_outcome = GateOutcome::Warn;
        }
    }

    if is_enforced_class(policy, decision_class) && robustness_score < 0.55 {
        gate_outcome = GateOutcome::Block;
        reasons.push("hard_gate robustness floor (0.55) not met".to_string());
    }

    match policy.mode {
        EpistemicMode::Observe => {
            if gate_outcome == GateOutcome::Block {
                gate_outcome = GateOutcome::Warn;
                reasons.push("observe mode downgrades block to warn".to_string());
            }
        }
        EpistemicMode::SoftGate => {
            if gate_outcome == GateOutcome::Block && !policy.block_on_soft {
                gate_outcome = GateOutcome::RequireReview;
                reasons.push("soft_gate mode downgrades block to require_review".to_string());
            }
        }
        EpistemicMode::HardGate => {
            if !is_enforced_class(policy, decision_class) && gate_outcome == GateOutcome::Block {
                gate_outcome = GateOutcome::RequireReview;
                reasons.push("decision class not in hard-gate scope".to_string());
            }
        }
    }

    EpistemicGateComputation {
        robustness_score,
        voi_score,
        regret_risk,
        gate_outcome,
        reasons,
    }
}

fn evaluate_epistemic_gate_internal(
    workflow_id: String,
    mutation_id: String,
    decision_class_raw: Option<String>,
) -> Result<EpistemicAssessment, String> {
    if workflow_id.trim().is_empty() {
        return Err("workflow_id is required".to_string());
    }
    if mutation_id.trim().is_empty() {
        return Err("mutation_id is required".to_string());
    }

    let decision_class = parse_decision_class(decision_class_raw);
    let policy = load_epistemic_policy();

    let workflow = WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(workflow_id.clone()))
            .map(|w| w.0)
    });

    let mut workflow_space_id: Option<String> = None;
    let mut workflow_domain_id: Option<String> = None;
    let mut action_target = format!("workflow:{workflow_id}");
    let mut adapter_set_ref = "nostra://workflow_engine/evaluate_epistemic_gate".to_string();
    let signals = if let Some(wf) = workflow {
        let ctx = wf.context;
        workflow_space_id = ctx.get("space_id").cloned();
        workflow_domain_id = ctx.get("attribution_domain_id").cloned();
        if let Some(target) = ctx.get("action_target") {
            action_target = target.clone();
        }
        if let Some(adapter_ref) = ctx.get("adapter_set_ref") {
            adapter_set_ref = adapter_ref.clone();
        }
        EpistemicSignalSnapshot {
            confidence_score: clamp01(parse_ctx_f64(&ctx, "confidence_score", 0.60)),
            source_reliability: clamp01(parse_ctx_f64(&ctx, "source_reliability", 0.70)),
            assumption_count: parse_ctx_u32(&ctx, "assumption_count", 1),
            evidence_count: parse_ctx_u32(&ctx, "evidence_count", 1),
            alternative_count: parse_ctx_u32(&ctx, "alternative_count", 1),
            confidence_drift: clamp01(parse_ctx_f64(&ctx, "confidence_drift", 0.0)),
            fork_pressure: clamp01(parse_ctx_f64(&ctx, "fork_pressure", 0.0)),
            correction_density: clamp01(parse_ctx_f64(&ctx, "correction_density", 0.0)),
        }
    } else {
        EpistemicSignalSnapshot {
            confidence_score: 0.60,
            source_reliability: 0.70,
            assumption_count: 1,
            evidence_count: 1,
            alternative_count: 1,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        }
    };
    let computation = compute_epistemic_gate(&policy, &decision_class, &signals);

    let created_at = ic_cdk::api::time() / 1_000_000_000;
    let assessment_id = build_assessment_id(&workflow_id, &mutation_id);
    let assessment = EpistemicAssessment {
        assessment_id: assessment_id.clone(),
        workflow_id,
        mutation_id: mutation_id.clone(),
        decision_class,
        confidence_score: signals.confidence_score,
        source_reliability: signals.source_reliability,
        robustness_score: computation.robustness_score,
        voi_score: computation.voi_score,
        regret_risk: computation.regret_risk,
        assumption_count: signals.assumption_count,
        evidence_count: signals.evidence_count,
        alternative_count: signals.alternative_count,
        gate_outcome: computation.gate_outcome,
        reasons: computation.reasons,
        created_at,
    };

    EPISTEMIC_ASSESSMENTS.with(|store| {
        store.borrow_mut().insert(
            StorableAssessmentId(assessment_id.clone()),
            StorableEpistemicAssessment(assessment.clone()),
        );
    });
    EPISTEMIC_ASSESSMENT_BY_MUTATION.with(|store| {
        store.borrow_mut().insert(
            StorableMutationId(mutation_id),
            StorableAssessmentId(assessment_id),
        );
    });

    let space_id = workflow_space_id
        .as_deref()
        .filter(|value| !value.trim().is_empty());
    let execution_profile_ref = lookup_execution_profile_ref(space_id);
    let attribution_domain_ref = if let (Some(space), Some(domain)) = (space_id, workflow_domain_id)
    {
        attribution_domain_surface_ref(space, &domain)
    } else {
        lookup_binding_domain_ref(space_id, &assessment.mutation_id)
    };
    let deterministic_input_hash = replay_input_hash(&serde_json::json!({
        "workflow_id": assessment.workflow_id.clone(),
        "mutation_id": assessment.mutation_id.clone(),
        "decision_class": decision_class_label(&assessment.decision_class),
        "signals": {
            "confidence_score": assessment.confidence_score,
            "source_reliability": assessment.source_reliability,
            "assumption_count": assessment.assumption_count,
            "evidence_count": assessment.evidence_count,
            "alternative_count": assessment.alternative_count,
            "robustness_score": assessment.robustness_score,
            "voi_score": assessment.voi_score,
            "regret_risk": assessment.regret_risk
        },
        "policy": {
            "mode": format!("{:?}", policy.mode),
            "min_evidence": policy.min_evidence,
            "min_alternatives": policy.min_alternatives,
            "min_robustness": policy.min_robustness,
            "max_confidence_drift": policy.max_confidence_drift,
            "max_fork_pressure": policy.max_fork_pressure,
            "max_correction_density": policy.max_correction_density,
            "simulation_ttl_days": policy.simulation_ttl_days,
            "block_on_soft": policy.block_on_soft
        },
        "execution_profile_ref": execution_profile_ref,
        "attribution_domain_ref": attribution_domain_ref
    }));
    let policy_ref = Some(policy_snapshot_ref(&policy));
    let evidence_refs = assessment
        .reasons
        .iter()
        .enumerate()
        .map(|(idx, reason)| format!("reason:{}:{}", idx + 1, reason))
        .collect::<Vec<_>>();
    let decision_digest = Some(replay_decision_digest(
        &assessment.mutation_id,
        policy_ref.as_deref(),
        policy_ref.as_deref(),
        &evidence_refs,
    ));
    store_replay_contract(ReplayContract {
        mutation_id: assessment.mutation_id.clone(),
        workflow_id: assessment.workflow_id.clone(),
        action_target,
        adapter_set_ref,
        execution_profile_ref,
        attribution_domain_ref,
        deterministic_input_hash,
        lineage_id: Some(format!("lineage:epistemic:{}", assessment.assessment_id)),
        policy_ref: policy_ref.clone(),
        policy_snapshot_ref: policy_ref,
        evidence_refs,
        decision_digest,
        captured_at: created_at,
    });

    Ok(assessment)
}

fn ordered_chapters_from_manifest(dpub: &DPubFile) -> Vec<DPubChapter> {
    let mut ordered: Vec<DPubChapter> = Vec::new();
    let mut seen: HashSet<String> = HashSet::new();

    if let Some(manifest) = dpub.manifest.as_ref() {
        for node in manifest.chapters.iter() {
            if let Some(ch) = dpub.content.iter().find(|c| c.id == node.id) {
                ordered.push(ch.clone());
                seen.insert(ch.id.clone());
            }
        }
    }

    for ch in dpub.content.iter() {
        if !seen.contains(&ch.id) {
            ordered.push(ch.clone());
        }
    }

    ordered
}

fn workflow_definition_from_template(template: &str) -> Option<WorkflowDefinition> {
    match template {
        "offline_conflict" => Some(build_offline_conflict_definition()),
        "approval" => Some(build_simple_template(
            "approval",
            "Approval Request",
            "Review and approve the request.",
        )),
        "governance" => Some(build_simple_template(
            "governance",
            "Governance Review",
            "Review and decide on the governance proposal.",
        )),
        _ => None,
    }
}

fn build_simple_template(id: &str, step_title: &str, step_description: &str) -> WorkflowDefinition {
    let mut steps = HashMap::new();

    let start = Step::new("start", step_title)
        .with_action(Action::UserTask {
            description: step_description.to_string(),
            candidate_roles: vec![],
            candidate_users: vec![],
            a2ui_schema: None,
        })
        .with_transition(Transition::to("done"));

    let done = Step::new("done", "Completed").with_action(Action::None);

    steps.insert("start".to_string(), start);
    steps.insert("done".to_string(), done);

    WorkflowDefinition {
        id: id.to_string(),
        steps,
        start_step_id: "start".to_string(),
    }
}

fn build_offline_conflict_definition() -> WorkflowDefinition {
    let mut steps = HashMap::new();

    let review = Step::new("review", "Resolve Offline Conflict")
        .with_action(Action::UserTask {
            description: "Resolve offline replay conflict.".to_string(),
            candidate_roles: vec![],
            candidate_users: vec![],
            a2ui_schema: Some(offline_conflict_a2ui_schema()),
        })
        .with_transition(Transition::to("resolved"));

    let resolved = Step::new("resolved", "Conflict resolved").with_action(Action::None);

    steps.insert("review".to_string(), review);
    steps.insert("resolved".to_string(), resolved);

    WorkflowDefinition {
        id: "offline_conflict".to_string(),
        steps,
        start_step_id: "review".to_string(),
    }
}

fn offline_conflict_a2ui_schema() -> String {
    let mut components = Vec::new();
    let mut card_props = HashMap::new();
    card_props.insert(
        "title".to_string(),
        serde_json::Value::String("Offline Conflict".to_string()),
    );
    card_props.insert(
        "description".to_string(),
        serde_json::Value::String(
            "Resolve the offline replay conflict (retry / fork / discard).".to_string(),
        ),
    );

    components.push(Component {
        id: "root".to_string(),
        component_type: ComponentType::Card,
        props: card_props,
        a11y: None,
        children: vec![
            "summary".to_string(),
            "error".to_string(),
            "command".to_string(),
            "actions".to_string(),
        ],
        data_bind: None,
    });

    components.push(Component {
        id: "summary".to_string(),
        component_type: ComponentType::Text,
        props: HashMap::from([(
            "text".to_string(),
            serde_json::Value::String("Conflict details will appear here.".to_string()),
        )]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "error".to_string(),
        component_type: ComponentType::Text,
        props: HashMap::from([
            (
                "text".to_string(),
                serde_json::Value::String("Error details will appear here.".to_string()),
            ),
            (
                "tone".to_string(),
                serde_json::Value::String("error".to_string()),
            ),
        ]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "command".to_string(),
        component_type: ComponentType::CodeBlock,
        props: HashMap::from([(
            "code".to_string(),
            serde_json::Value::String("KIP command preview".to_string()),
        )]),
        a11y: None,
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "actions".to_string(),
        component_type: ComponentType::Row,
        props: HashMap::new(),
        a11y: None,
        children: vec![
            "conflict_retry".to_string(),
            "conflict_fork".to_string(),
            "conflict_discard".to_string(),
        ],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_retry".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Retry".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_retry:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Retry")),
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_fork".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Fork".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_fork:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Fork")),
        children: vec![],
        data_bind: None,
    });

    components.push(Component {
        id: "conflict_discard".to_string(),
        component_type: ComponentType::Button,
        props: HashMap::from([
            (
                "label".to_string(),
                serde_json::Value::String("Discard".to_string()),
            ),
            (
                "action".to_string(),
                serde_json::Value::String("conflict_discard:{mutation_id}".to_string()),
            ),
        ]),
        a11y: Some(A11yProperties::with_label("Discard")),
        children: vec![],
        data_bind: None,
    });

    serde_json::to_string(&components).unwrap_or_else(|_| "[]".to_string())
}

async fn create_workflow_instance(definition: WorkflowDefinition) -> String {
    let rand_bytes = raw_rand().await.expect("Failed to generate randomness");
    let id = hex::encode(rand_bytes);

    let instance = WorkflowInstance::new(id.clone(), definition.clone());

    WORKFLOWS.with(|p| {
        p.borrow_mut()
            .insert(StorableWorkflowId(id.clone()), StorableWorkflow(instance));
    });

    WORKFLOW_DEFINITIONS.with(|p| {
        p.borrow_mut().insert(
            StorableWorkflowDefinitionId(definition.id.clone()),
            StorableWorkflowDefinition(definition),
        );
    });

    ic_cdk_timers::set_timer(Duration::from_secs(0), || ic_cdk::futures::spawn(tick()));

    id
}

#[ic_cdk::update]
async fn start_workflow(definition_json: String) -> String {
    let definition = match serde_json::from_str::<WorkflowDefinition>(&definition_json) {
        Ok(def) => def,
        Err(_) => match workflow_definition_from_template(definition_json.trim()) {
            Some(def) => def,
            None => {
                return format!(
                    "error: unknown workflow template or invalid definition ({})",
                    definition_json
                );
            }
        },
    };

    create_workflow_instance(definition).await
}

#[ic_cdk::query]
fn get_workflow(id: WorkflowId) -> Option<String> {
    WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(id))
            .map(|w| serde_json::to_string(&w.0).unwrap())
    })
}

// --------------------------------------------------------------------------------
// Flow Graph API (MVP)
// --------------------------------------------------------------------------------

#[ic_cdk::query]
fn get_flow_graph(workflow_id: String, version: Option<String>) -> Result<FlowGraph, String> {
    if workflow_id.trim().is_empty() {
        return Err("workflow_id is required".to_string());
    }
    let definition = WORKFLOW_DEFINITIONS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowDefinitionId(workflow_id.clone()))
            .map(|w| w.0.clone())
    });

    let definition = definition.ok_or_else(|| "workflow definition not found".to_string())?;
    Ok(flow_graph::derive_graph(&definition, version))
}

#[ic_cdk::query]
fn get_flow_layout(
    workflow_id: String,
    graph_version: Option<String>,
) -> Result<FlowLayout, String> {
    flow_graph::get_flow_layout(workflow_id, graph_version)
}

#[ic_cdk::update]
fn set_flow_layout(input: FlowLayoutInput) -> Result<FlowLayout, String> {
    flow_graph::set_flow_layout(input)
}

// --------------------------------------------------------------------------------
// VFS API (013)
// --------------------------------------------------------------------------------

#[ic_cdk::update]
fn write_file(path: String, content: Vec<u8>, mime_type: String) -> Result<(), String> {
    vfs::vfs_write(path, content, mime_type)
}

#[ic_cdk::query]
fn read_file(path: String) -> Result<Vec<u8>, String> {
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_files(prefix: String) -> Vec<(String, vfs::FileMetadata)> {
    vfs::vfs_list(prefix)
}

// --------------------------------------------------------------------------------
// dPub V1 API (Edition publication + feed)
// --------------------------------------------------------------------------------

fn normalize_publish_context(mut publish_context: PublishContext) -> PublishContext {
    if publish_context.commit_hash.trim().is_empty() {
        publish_context.commit_hash = "unknown-local".to_string();
    } else {
        publish_context.commit_hash = publish_context.commit_hash.trim().to_string();
    }
    publish_context.source_ref = publish_context
        .source_ref
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty());
    publish_context
}

async fn publish_dpub_edition_internal(
    dpub_path: String,
    edition_version: String,
    edition_name: Option<String>,
    override_token: Option<String>,
    publish_context: PublishContext,
) -> Result<EditionManifest, String> {
    let publish_context = normalize_publish_context(publish_context);
    let dpub_bytes = vfs::vfs_read(dpub_path.clone())?;
    let mut dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;

    let license = dpub.meta.license.clone().unwrap_or_default();
    if license.trim().is_empty() {
        return Err("Missing license".to_string());
    }
    if license.to_lowercase().contains("arranged")
        && override_token
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
    {
        return Err("License requires an explicit override token (Arranged)".to_string());
    }

    let published_at = ic_cdk::api::time().to_string();
    let edition_id = hex::encode(raw_rand().await.map_err(|e| format!("{:?}", e))?);
    let publisher = ic_cdk::api::msg_caller().to_text();
    let ordered_content = ordered_chapters_from_manifest(&dpub);

    let mut chapter_manifests: Vec<ChapterManifest> = Vec::new();
    let mut leaf_hashes: Vec<String> = Vec::new();
    for (i, ch) in ordered_content.iter().enumerate() {
        let bytes = serde_json::to_vec(&ch.blocks).map_err(|e| e.to_string())?;
        let content_hash = sha256_hex(&bytes);
        leaf_hashes.push(content_hash.clone());
        chapter_manifests.push(ChapterManifest {
            index: i as u32,
            contribution_ref: ContributionVersionRef {
                contribution_id: ch.id.clone(),
                version_hash: content_hash.clone(),
            },
            content_hash,
            title: ch.title.clone().unwrap_or_else(|| ch.id.clone()),
        });
    }

    let content_root = merkle_root_hex(&leaf_hashes);

    let manifest = EditionManifest {
        edition_id: edition_id.clone(),
        dpub_id: dpub.meta.id.clone(),
        version: edition_version.clone(),
        name: edition_name,
        content_root: content_root.clone(),
        chapters: chapter_manifests,
        published_at: published_at.clone(),
        publisher,
        previous_edition: dpub.latest_edition.clone(),
        metadata: EditionMetadata {
            license: license.clone(),
        },
    };

    // Snapshot is the full dPub JSON at time of publication (immutable edition view).
    let mut snapshot = dpub.clone();
    snapshot.content = ordered_content;
    snapshot.meta.version = Some(edition_version.clone());
    snapshot.meta.phase = Some("Archival".to_string());

    let base_dir = dpub_path
        .rsplit_once('/')
        .map(|(dir, _)| dir.to_string())
        .unwrap_or_else(|| "lib/dpubs".to_string());
    let edition_dir = format!("{}/editions/{}", base_dir, edition_version);
    let manifest_path = format!("{}/edition_manifest.json", edition_dir);
    let snapshot_path = format!("{}/snapshot.json", edition_dir);
    let snapshot_manifest_path = format!("{}/snapshot_manifest.json", edition_dir);

    let manifest_bytes = serde_json::to_vec(&manifest).map_err(|e| e.to_string())?;
    vfs::vfs_write(
        manifest_path.clone(),
        manifest_bytes.clone(),
        "application/json".to_string(),
    )?;

    let snapshot_bytes = serde_json::to_vec(&snapshot).map_err(|e| e.to_string())?;
    vfs::vfs_write(
        snapshot_path.clone(),
        snapshot_bytes.clone(),
        "application/json".to_string(),
    )?;

    // Update dPub index (latest edition + summary list)
    dpub.meta.version = Some(edition_version.clone());
    dpub.meta.phase = Some("Archival".to_string());
    dpub.latest_edition = Some(edition_version.clone());
    dpub.editions.push(EditionSummary {
        edition_id: edition_id.clone(),
        version: edition_version.clone(),
        published_at: published_at.clone(),
        content_root: content_root.clone(),
    });
    let dpub_updated_bytes = serde_json::to_vec(&dpub).map_err(|e| e.to_string())?;
    vfs::vfs_write(
        dpub_path.clone(),
        dpub_updated_bytes.clone(),
        "application/json".to_string(),
    )?;

    // Native feed (V1 canonical JSON feed).
    let feed_path = format!("{}/feed.json", base_dir);
    let feed = serde_json::json!({
        "type": "dpub.feed.v1",
        "dpub_id": dpub.meta.id,
        "latest_edition": dpub.latest_edition,
        "editions": dpub.editions,
    });
    let feed_bytes = serde_json::to_vec(&feed).map_err(|e| e.to_string())?;
    let wrote_feed = vfs::vfs_write(
        feed_path.clone(),
        feed_bytes.clone(),
        "application/json".to_string(),
    )
    .is_ok();

    let mut snapshot_files = vec![
        SnapshotManifestFile {
            path: manifest_path.clone(),
            sha256: sha256_hex(&manifest_bytes),
            size_bytes: manifest_bytes.len() as u64,
        },
        SnapshotManifestFile {
            path: snapshot_path.clone(),
            sha256: sha256_hex(&snapshot_bytes),
            size_bytes: snapshot_bytes.len() as u64,
        },
        SnapshotManifestFile {
            path: dpub_path.clone(),
            sha256: sha256_hex(&dpub_updated_bytes),
            size_bytes: dpub_updated_bytes.len() as u64,
        },
    ];
    if wrote_feed {
        snapshot_files.push(SnapshotManifestFile {
            path: feed_path.clone(),
            sha256: sha256_hex(&feed_bytes),
            size_bytes: feed_bytes.len() as u64,
        });
    }
    snapshot_files.sort_by(|a, b| a.path.cmp(&b.path));

    let snapshot_manifest = SnapshotManifest {
        bundle_id: format!("{}@{}", manifest.dpub_id, manifest.version),
        commit_hash: publish_context.commit_hash.clone(),
        generated_at: published_at.clone(),
        bundle_version: "1.0.0".to_string(),
        files: snapshot_files,
        entrypoints: SnapshotManifestEntrypoints {
            dpub_path: dpub_path.clone(),
            edition_manifest_path: manifest_path.clone(),
            snapshot_path: snapshot_path.clone(),
            snapshot_manifest_path: snapshot_manifest_path.clone(),
            feed_path: if wrote_feed {
                Some(feed_path.clone())
            } else {
                None
            },
        },
    };
    let snapshot_manifest_bytes =
        serde_json::to_vec(&snapshot_manifest).map_err(|e| e.to_string())?;
    vfs::vfs_write(
        snapshot_manifest_path.clone(),
        snapshot_manifest_bytes,
        "application/json".to_string(),
    )?;

    // Chronicle append (simple JSONL stored via VFS)
    let chronicle_path = "/lib/chronicle/edition_published.jsonl".to_string();
    let mut existing = String::new();
    if let Ok(bytes) = vfs::vfs_read(chronicle_path.clone()) {
        existing = String::from_utf8_lossy(&bytes).to_string();
    }
    let event = serde_json::json!({
        "type": "edition.published",
        "dpub_id": manifest.dpub_id,
        "edition_id": edition_id,
        "version": edition_version,
        "content_root": content_root,
        "published_at": published_at,
    });
    existing.push_str(&serde_json::to_string(&event).unwrap_or_else(|_| "{}".to_string()));
    existing.push('\n');
    let _ = vfs::vfs_write(
        chronicle_path,
        existing.into_bytes(),
        "application/jsonl".to_string(),
    );

    // Audit trace (glass box)
    let trace = serde_json::json!({
        "type": "audit_trace.v1",
        "action": "publish_edition",
        "dpub_id": dpub.meta.id,
        "edition_id": manifest.edition_id,
        "timestamp": ic_cdk::api::time().to_string(),
        "inputs": {
            "dpub_path": dpub_path.clone(),
            "edition_version": manifest.version,
            "override_token": override_token,
            "chapter_count": dpub.content.len(),
            "commit_hash": publish_context.commit_hash,
            "source_ref": publish_context.source_ref,
        },
        "outputs": {
            "content_root": manifest.content_root,
            "manifest_path": manifest_path,
            "snapshot_path": snapshot_path,
            "snapshot_manifest_path": snapshot_manifest_path,
        }
    });
    let trace_path = format!("/lib/audit_traces/publish_edition_{}.json", edition_id);
    let _ = vfs::vfs_write(
        trace_path,
        serde_json::to_vec(&trace).unwrap_or_default(),
        "application/json".to_string(),
    );

    Ok(manifest)
}

// Deprecated for new clients; retained for backward compatibility.
#[ic_cdk::update]
async fn publish_dpub_edition(
    dpub_path: String,
    edition_version: String,
    edition_name: Option<String>,
    override_token: Option<String>,
) -> Result<EditionManifest, String> {
    publish_dpub_edition_internal(
        dpub_path,
        edition_version,
        edition_name,
        override_token,
        PublishContext {
            commit_hash: "unknown-local".to_string(),
            source_ref: None,
        },
    )
    .await
}

#[ic_cdk::update]
async fn publish_dpub_edition_v2(
    dpub_path: String,
    edition_version: String,
    edition_name: Option<String>,
    override_token: Option<String>,
    publish_context: PublishContext,
) -> Result<EditionManifest, String> {
    publish_dpub_edition_internal(
        dpub_path,
        edition_version,
        edition_name,
        override_token,
        publish_context,
    )
    .await
}

fn is_valid_dpub_dir(base_dir: &str) -> bool {
    let trimmed = base_dir.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    let normalized = trimmed.trim_start_matches('/');
    normalized.starts_with("lib/dpubs/")
}

fn treaty_required(
    viewer_space: Option<&str>,
    dpub_space: &str,
    treaty_token: Option<&str>,
) -> bool {
    let viewer = viewer_space.map(|v| v.trim()).filter(|v| !v.is_empty());
    let Some(viewer) = viewer else {
        return false;
    };
    if viewer == dpub_space {
        return false;
    }
    treaty_token.unwrap_or_default().trim().is_empty()
}

fn build_dpub_feed(
    base_dir: &str,
    mut editions: Vec<serde_json::Value>,
    limit: usize,
) -> serde_json::Value {
    editions.reverse();
    editions.truncate(limit);
    serde_json::json!({
        "type": "dpub.feed.v1",
        "dpub_dir": base_dir,
        "items": editions,
    })
}

fn is_valid_vfs_path(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    if trimmed.contains("..") {
        return false;
    }
    let normalized = trimmed.trim_start_matches('/');
    normalized.starts_with("lib/")
}

fn dpub_base_dir_from_path(path: &str) -> Option<String> {
    let trimmed = path.trim().trim_start_matches('/');
    let mut parts = trimmed.split('/');
    if parts.next()? != "lib" {
        return None;
    }
    if parts.next()? != "dpubs" {
        return None;
    }
    let slug = parts.next()?;
    if slug.trim().is_empty() {
        return None;
    }
    Some(format!("lib/dpubs/{}", slug))
}

fn enforce_non_dpub_guard(
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    let viewer = viewer_space_did
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Viewer space required for guarded access".to_string())?;
    let token = treaty_token
        .map(|t| t.trim())
        .filter(|t| !t.is_empty())
        .ok_or_else(|| "Treaty token required for guarded access".to_string())?;
    let _ = (viewer, token);
    Ok(())
}

fn enforce_dpub_treaty(
    base_dir: &str,
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    if !is_valid_dpub_dir(base_dir) {
        return Err("Invalid dPub dir".to_string());
    }
    let viewer = viewer_space_did
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .ok_or_else(|| "Viewer space required for guarded access".to_string())?;
    let meta_path = format!("{}/dpub.json", base_dir);
    let dpub_bytes = vfs::vfs_read(meta_path)
        .map_err(|_| "Missing dPub metadata for treaty enforcement".to_string())?;
    let dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;
    let dpub_space = dpub.meta.provenance.space_did;
    if treaty_required(Some(viewer), &dpub_space, treaty_token) {
        return Err("Treaty required for cross-space access".to_string());
    }
    Ok(())
}

fn enforce_vfs_guarded(
    path_or_prefix: &str,
    viewer_space_did: Option<&str>,
    treaty_token: Option<&str>,
) -> Result<(), String> {
    if !is_valid_vfs_path(path_or_prefix) {
        return Err("Invalid VFS path".to_string());
    }
    if let Some(base_dir) = dpub_base_dir_from_path(path_or_prefix) {
        return enforce_dpub_treaty(&base_dir, viewer_space_did, treaty_token);
    }
    enforce_non_dpub_guard(viewer_space_did, treaty_token)
}

#[ic_cdk::query]
fn get_dpub_feed(
    dpub_dir: String,
    limit: u32,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<String, String> {
    // Minimal feed: list edition manifests under dpub_dir/editions/*
    let base_dir = dpub_dir.trim_end_matches('/').to_string();
    if !is_valid_dpub_dir(&base_dir) {
        return Err("Invalid dPub dir".to_string());
    }

    if let Some(viewer) = viewer_space_did
        .as_deref()
        .map(|v| v.trim())
        .filter(|v| !v.is_empty())
    {
        let meta_path = format!("{}/dpub.json", base_dir);
        let dpub_bytes = vfs::vfs_read(meta_path)
            .map_err(|_| "Missing dPub metadata for treaty enforcement".to_string())?;
        let dpub: DPubFile = serde_json::from_slice(&dpub_bytes).map_err(|e| e.to_string())?;
        let dpub_space = dpub.meta.provenance.space_did;
        if treaty_required(Some(viewer), &dpub_space, treaty_token.as_deref()) {
            return Err("Treaty required for cross-space feed access".to_string());
        }
    }

    let prefix = format!("{}/editions/", base_dir);
    let mut entries = vfs::vfs_list(prefix.clone());
    entries.sort_by(|a, b| a.0.cmp(&b.0));

    let mut editions: Vec<serde_json::Value> = Vec::new();
    for (path, _) in entries {
        if !path.ends_with("edition_manifest.json") {
            continue;
        }
        if let Ok(bytes) = vfs::vfs_read(path) {
            if let Ok(v) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                editions.push(v);
            }
        }
    }

    let feed = build_dpub_feed(&base_dir, editions, limit as usize);
    serde_json::to_string(&feed).map_err(|e| e.to_string())
}

#[ic_cdk::query]
fn read_dpub_file_guarded(
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    let base_dir = dpub_base_dir_from_path(&path).ok_or_else(|| "Invalid dPub path".to_string())?;
    enforce_dpub_treaty(
        &base_dir,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_dpub_files_guarded(
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, vfs::FileMetadata)>, String> {
    let base_dir =
        dpub_base_dir_from_path(&prefix).ok_or_else(|| "Invalid dPub prefix".to_string())?;
    enforce_dpub_treaty(
        &base_dir,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    Ok(vfs::vfs_list(prefix))
}

#[ic_cdk::query]
fn read_vfs_guarded(
    path: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<u8>, String> {
    enforce_vfs_guarded(&path, viewer_space_did.as_deref(), treaty_token.as_deref())?;
    vfs::vfs_read(path)
}

#[ic_cdk::query]
fn list_vfs_guarded(
    prefix: String,
    viewer_space_did: Option<String>,
    treaty_token: Option<String>,
) -> Result<Vec<(String, vfs::FileMetadata)>, String> {
    enforce_vfs_guarded(
        &prefix,
        viewer_space_did.as_deref(),
        treaty_token.as_deref(),
    )?;
    Ok(vfs::vfs_list(prefix))
}

#[ic_cdk::update]
async fn tick() {
    let keys: Vec<StorableWorkflowId> = WORKFLOWS.with(|p| {
        // Fix: Use generic iteration or handle tuple explicitly
        p.borrow().iter().map(|x| x.key().clone()).collect()
    });

    for key in keys {
        let mut workflow = WORKFLOWS.with(|p| p.borrow().get(&key).unwrap().0);

        Engine::step(&mut workflow);

        monitor::record_workflow_status_change(&workflow.status);

        if matches!(workflow.status, WorkflowStatus::Completed) {
            trigger_proposals(&workflow);
        }

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(key, StorableWorkflow(workflow));
        });
    }
}

// --------------------------------------------------------------------------------
// Governance Adapter Hooks (Nostra Commons Bridge)
// --------------------------------------------------------------------------------

fn trigger_proposals(workflow: &WorkflowInstance) {
    ic_cdk::println!(
        "WorkflowEngine: Evaluating Governance rules for completed workflow {}",
        workflow.id
    );
    // Future: Call to Nostra Commons SIQS rule check
    // let result = call_nostra_commons_evaluate(&workflow);
    // if result.requires_ratification {
    //      create_governance_proposal(&workflow);
    // }
}

#[derive(Clone, Debug, candid::CandidType, serde::Deserialize)]
pub struct EvaluatedSimulationReport {
    pub workflow_id: String,
    pub confidence_score: f64,
}

#[ic_cdk::update]
pub fn evaluate_simulation(report: EvaluatedSimulationReport) -> Result<(), String> {
    let workflow_id = StorableWorkflowId(report.workflow_id.clone());
    let mut workflow = WORKFLOWS
        .with(|p| p.borrow().get(&workflow_id))
        .ok_or_else(|| format!("Workflow not found: {}", report.workflow_id))?
        .0;

    if report.confidence_score < 0.8 {
        ic_cdk::println!(
            "Workflow {} missed confidence threshold ({}). Forking to Governance Node.",
            report.workflow_id,
            report.confidence_score
        );
        trigger_proposals(&workflow);
        workflow.status = WorkflowStatus::Paused; // Branch execution gracefully
    } else {
        ic_cdk::println!(
            "Workflow {} passed confidence threshold ({}). Continuing execution.",
            report.workflow_id,
            report.confidence_score
        );
    }

    WORKFLOWS.with(|p| {
        p.borrow_mut()
            .insert(workflow_id, StorableWorkflow(workflow));
    });

    Ok(())
}

// --------------------------------------------------------------------------------
// System Hooks
// --------------------------------------------------------------------------------

#[ic_cdk::init]
fn init() {
    let timer_id = ic_cdk_timers::set_timer_interval(Duration::from_secs(1), || {
        ic_cdk::futures::spawn(tick());
    });

    TIMER_ID.with(|t| *t.borrow_mut() = Some(timer_id));
}

fn index_conflict(mutation_id: &str, workflow_id: &str) {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow_mut().insert(
            StorableMutationId(mutation_id.to_string()),
            StorableWorkflowId(workflow_id.to_string()),
        );
    });
}

fn remove_conflict_index(mutation_id: &str) {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow_mut()
            .remove(&StorableMutationId(mutation_id.to_string()));
    });
}

fn lookup_conflict_workflow(mutation_id: &str) -> Option<String> {
    OFFLINE_CONFLICT_INDEX.with(|p| {
        p.borrow()
            .get(&StorableMutationId(mutation_id.to_string()))
            .map(|id| id.0.clone())
    })
}

fn attach_conflict_to_workflow(workflow_id: &str, event: &OfflineConflictEvent) {
    let workflow = WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(workflow_id.to_string()))
            .map(|w| w.0)
    });

    if let Some(mut workflow) = workflow {
        workflow
            .context
            .set("mutation_id", event.mutation.id.clone());
        workflow.context.set("conflict_kind", event.kind.clone());
        workflow.context.set("conflict_error", event.error.clone());
        workflow
            .context
            .set("source", event.source.clone().unwrap_or_default());
        workflow.context.log(format!(
            "Offline conflict received: {} ({})",
            event.mutation.id, event.kind
        ));

        Engine::step(&mut workflow);

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(
                StorableWorkflowId(workflow_id.to_string()),
                StorableWorkflow(workflow),
            );
        });
    }
}

fn workflow_status_label(status: &WorkflowStatus) -> String {
    match status {
        WorkflowStatus::Running => "Running".to_string(),
        WorkflowStatus::Paused => "Paused".to_string(),
        WorkflowStatus::Completed => "Completed".to_string(),
        WorkflowStatus::Failed(msg) => format!("Failed: {}", msg),
    }
}

fn parse_offline_conflict_value(value: &serde_json::Value) -> Option<OfflineConflictEvent> {
    if let Some(payload) = value.get("payload") {
        let payload_value = if let Some(s) = payload.as_str() {
            serde_json::from_str::<serde_json::Value>(s).ok()?
        } else {
            payload.clone()
        };
        let mut event = parse_offline_conflict_value(&payload_value)?;
        if event.workflow_id.is_none() {
            if let Some(wid) = value.get("workflow_id").and_then(|v| v.as_str()) {
                event.workflow_id = Some(wid.to_string());
            }
        }
        return Some(event);
    }

    let mut event: OfflineConflictEvent = serde_json::from_value(value.clone()).ok()?;
    if event.workflow_id.is_none() {
        if let Some(wid) = value.get("workflow_id").and_then(|v| v.as_str()) {
            event.workflow_id = Some(wid.to_string());
        }
    }
    if event.kind.is_empty() {
        event.kind = "Conflict".to_string();
    }
    if event.error.is_empty() {
        event.error = "Unknown conflict".to_string();
    }
    if event.mutation.id.is_empty() {
        return None;
    }
    Some(event)
}

fn render_offline_conflict_surface(event: &OfflineConflictEvent, workflow_id: &str) -> A2UIMessage {
    let mut props = HashMap::new();
    props.insert(
        "title".to_string(),
        serde_json::Value::String(format!("Offline {} ({})", event.kind, workflow_id)),
    );
    props.insert(
        "context".to_string(),
        serde_json::Value::String("inbox".to_string()),
    );
    props.insert(
        "tone".to_string(),
        serde_json::Value::String("critical".to_string()),
    );
    props.insert(
        "priority".to_string(),
        serde_json::Value::String("p0".to_string()),
    );
    props.insert(
        "workflow_id".to_string(),
        serde_json::Value::String(workflow_id.to_string()),
    );
    props.insert(
        "source".to_string(),
        serde_json::Value::String(
            event
                .source
                .clone()
                .unwrap_or_else(|| "workflow-engine".to_string()),
        ),
    );
    props.insert(
        "mutation_id".to_string(),
        serde_json::Value::String(event.mutation.id.clone()),
    );

    let cmd = if event.mutation.kip_command.len() > 240 {
        format!("{}...", &event.mutation.kip_command[..240])
    } else {
        event.mutation.kip_command.clone()
    };

    let retry_id = format!("conflict_retry:{}", event.mutation.id);
    let fork_id = format!("conflict_fork:{}", event.mutation.id);
    let discard_id = format!("conflict_discard:{}", event.mutation.id);

    let components = vec![
        Component {
            id: "root".to_string(),
            component_type: ComponentType::Card,
            props,
            a11y: None,
            children: vec![
                "summary".to_string(),
                "error".to_string(),
                "command".to_string(),
                "actions".to_string(),
            ],
            data_bind: None,
        },
        Component {
            id: "summary".to_string(),
            component_type: ComponentType::Text,
            props: HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String(format!(
                        "Mutation ID: {}\nAttempts: {}",
                        event.mutation.id, event.mutation.attempts
                    )),
                ),
                (
                    "tone".to_string(),
                    serde_json::Value::String("muted".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "error".to_string(),
            component_type: ComponentType::Text,
            props: HashMap::from([
                (
                    "text".to_string(),
                    serde_json::Value::String(format!("Error: {}", event.error)),
                ),
                (
                    "tone".to_string(),
                    serde_json::Value::String("error".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "command".to_string(),
            component_type: ComponentType::CodeBlock,
            props: HashMap::from([("code".to_string(), serde_json::Value::String(cmd))]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "actions".to_string(),
            component_type: ComponentType::Row,
            props: HashMap::new(),
            a11y: None,
            children: vec![retry_id.clone(), fork_id.clone(), discard_id.clone()],
            data_bind: None,
        },
        Component {
            id: retry_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Retry".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_retry:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Retry")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: fork_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Fork".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_fork:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Fork")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: discard_id,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Discard".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!("conflict_discard:{}", event.mutation.id)),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Discard")),
            children: vec![],
            data_bind: None,
        },
    ];

    A2UIMessage::RenderSurface {
        surface_id: format!("offline_conflict_{}", workflow_id),
        title: "Offline Conflict".to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some("critical".to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some("p0".to_string()),
            intent: Some("primary".to_string()),
            severity: Some("critical".to_string()),
            workflow_id: Some(workflow_id.to_string()),
            mutation_id: Some(event.mutation.id.clone()),
            space_id: event.mutation.space_id.clone(),
            execution_profile_ref: None,
            attribution_domain_ref: None,
            gate_level: Some("release_blocker".to_string()),
            gate_status: Some("conflict".to_string()),
            decision_gate_id: Some(format!("conflict_gate:{}", event.mutation.id)),
            replay_contract_ref: Some(format!("system_replay_contract:{}", event.mutation.id)),
            action_target_ref: Some(format!("mutation:{}", event.mutation.id)),
            actor_ref: None,
            policy_ref: None,
            lineage_id: None,
            source_of_truth: Some("canister".to_string()),
            source: event.source.clone().or(Some("workflow-engine".to_string())),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

fn render_decision_ack(decision: &OfflineConflictDecision, workflow_id: &str) -> A2UIMessage {
    let mut props = HashMap::new();
    props.insert(
        "title".to_string(),
        serde_json::Value::String("Conflict Decision Recorded".to_string()),
    );
    props.insert(
        "description".to_string(),
        serde_json::Value::String(format!(
            "Workflow {} recorded decision '{}' for mutation {}.",
            workflow_id, decision.decision, decision.mutation_id
        )),
    );
    props.insert(
        "workflow_id".to_string(),
        serde_json::Value::String(workflow_id.to_string()),
    );
    props.insert(
        "mutation_id".to_string(),
        serde_json::Value::String(decision.mutation_id.clone()),
    );
    props.insert(
        "decision".to_string(),
        serde_json::Value::String(decision.decision.clone()),
    );
    props.insert(
        "source".to_string(),
        serde_json::Value::String(
            decision
                .source
                .clone()
                .unwrap_or_else(|| "workflow-engine".to_string()),
        ),
    );

    let components = vec![Component {
        id: "root".to_string(),
        component_type: ComponentType::Card,
        props,
        a11y: None,
        children: vec![],
        data_bind: None,
    }];

    A2UIMessage::RenderSurface {
        surface_id: format!("offline_conflict_decision_{}", workflow_id),
        title: "Decision Recorded".to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some("info".to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some("p1".to_string()),
            intent: Some("secondary".to_string()),
            severity: Some("info".to_string()),
            workflow_id: Some(workflow_id.to_string()),
            mutation_id: Some(decision.mutation_id.clone()),
            space_id: None,
            execution_profile_ref: None,
            attribution_domain_ref: None,
            gate_level: Some("informational".to_string()),
            gate_status: Some("acknowledged".to_string()),
            decision_gate_id: Some(format!("conflict_decision:{}", decision.mutation_id)),
            replay_contract_ref: Some(format!("system_replay_contract:{}", decision.mutation_id)),
            action_target_ref: Some(format!("mutation:{}", decision.mutation_id)),
            actor_ref: None,
            policy_ref: None,
            lineage_id: None,
            source_of_truth: Some("canister".to_string()),
            source: decision
                .source
                .clone()
                .or(Some("workflow-engine".to_string())),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

async fn handle_offline_conflict(value: serde_json::Value) -> String {
    let mut event = match parse_offline_conflict_value(&value) {
        Some(event) => event,
        None => {
            let response = A2UIMessage::Error {
                message: "Invalid offline_conflict payload".to_string(),
            };
            return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
        }
    };

    if event.workflow_id.is_none() {
        event.workflow_id =
            Some(create_workflow_instance(build_offline_conflict_definition()).await);
    }

    let workflow_id = event.workflow_id.clone().unwrap_or_default();
    index_conflict(&event.mutation.id, &workflow_id);
    attach_conflict_to_workflow(&workflow_id, &event);

    let response = render_offline_conflict_surface(&event, &workflow_id);
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

async fn handle_offline_conflict_decision(value: serde_json::Value) -> String {
    let decision: OfflineConflictDecision = serde_json::from_value(value).unwrap_or_default();
    let workflow_id = decision
        .workflow_id
        .clone()
        .or_else(|| lookup_conflict_workflow(&decision.mutation_id));

    let Some(workflow_id) = workflow_id else {
        let response = A2UIMessage::Error {
            message: "Unknown workflow for decision".to_string(),
        };
        return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    };

    let workflow = WORKFLOWS.with(|p| {
        p.borrow()
            .get(&StorableWorkflowId(workflow_id.clone()))
            .map(|w| w.0)
    });

    if let Some(mut workflow) = workflow {
        let mut payload = HashMap::new();
        if !decision.decision.is_empty() {
            payload.insert("decision".to_string(), decision.decision.clone());
        }
        if !decision.mutation_id.is_empty() {
            payload.insert("mutation_id".to_string(), decision.mutation_id.clone());
        }
        if let Some(source) = decision.source.clone() {
            payload.insert("source".to_string(), source);
        }

        Engine::complete_user_task(&mut workflow, Some(payload));

        WORKFLOWS.with(|p| {
            p.borrow_mut().insert(
                StorableWorkflowId(workflow_id.clone()),
                StorableWorkflow(workflow),
            );
        });

        remove_conflict_index(&decision.mutation_id);

        let response = render_decision_ack(&decision, &workflow_id);
        return serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string());
    }

    let response = A2UIMessage::Error {
        message: "Workflow instance not found".to_string(),
    };
    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

#[ic_cdk::query]
fn list_offline_conflicts() -> Vec<OfflineConflictSummary> {
    OFFLINE_CONFLICT_INDEX.with(|index| {
        index
            .borrow()
            .iter()
            .map(|entry| {
                let mutation_id = entry.key().0.clone();
                let workflow_id = entry.value().0.clone();

                let workflow = WORKFLOWS.with(|p| {
                    p.borrow()
                        .get(&StorableWorkflowId(workflow_id.clone()))
                        .map(|w| w.0)
                });

                if let Some(wf) = workflow {
                    let ctx = &wf.context;
                    let mutation_id = ctx.get("mutation_id").cloned().unwrap_or(mutation_id);
                    let kind = ctx
                        .get("conflict_kind")
                        .cloned()
                        .unwrap_or_else(|| "Conflict".to_string());
                    let error = ctx
                        .get("conflict_error")
                        .cloned()
                        .unwrap_or_else(|| "Unknown".to_string());
                    let source = ctx.get("source").cloned();
                    let status = workflow_status_label(&wf.status);

                    OfflineConflictSummary {
                        mutation_id,
                        workflow_id,
                        kind,
                        error,
                        source,
                        status,
                    }
                } else {
                    OfflineConflictSummary {
                        mutation_id,
                        workflow_id,
                        kind: "Conflict".to_string(),
                        error: "Unknown".to_string(),
                        source: None,
                        status: "Missing".to_string(),
                    }
                }
            })
            .collect()
    })
}

fn render_blackwell_surface(
    assessment: &EpistemicAssessment,
    policy: &EpistemicPolicy,
) -> A2UIMessage {
    let (surface_prefix, tone, priority, severity, title) = match assessment.gate_outcome {
        GateOutcome::Pass => (
            "blackwell_info",
            "info",
            "p2",
            "info",
            "Blackwell Assessment: Pass",
        ),
        GateOutcome::Warn => (
            "blackwell_warning",
            "warning",
            "p1",
            "warning",
            "Blackwell Warning",
        ),
        GateOutcome::RequireReview => (
            "blackwell_gate",
            "critical",
            "p0",
            "critical",
            "Blackwell Gate: Review Required",
        ),
        GateOutcome::RequireSimulation => (
            "blackwell_gate",
            "critical",
            "p0",
            "critical",
            "Blackwell Gate: Simulation Required",
        ),
        GateOutcome::Block => (
            "blackwell_gate",
            "critical",
            "p0",
            "critical",
            "Blackwell Gate: Blocked",
        ),
    };

    let mut props = HashMap::new();
    props.insert(
        "title".to_string(),
        serde_json::Value::String(title.to_string()),
    );
    props.insert(
        "summary".to_string(),
        serde_json::Value::String(format!(
            "Class={} Robustness={:.3} VoI={:.3} Regret={:.3}",
            decision_class_label(&assessment.decision_class),
            assessment.robustness_score,
            assessment.voi_score,
            assessment.regret_risk
        )),
    );
    props.insert(
        "mode".to_string(),
        serde_json::Value::String(format!("{:?}", policy.mode)),
    );
    props.insert(
        "assessment_id".to_string(),
        serde_json::Value::String(assessment.assessment_id.clone()),
    );
    props.insert(
        "workflow_id".to_string(),
        serde_json::Value::String(assessment.workflow_id.clone()),
    );
    props.insert(
        "mutation_id".to_string(),
        serde_json::Value::String(assessment.mutation_id.clone()),
    );

    let reasons = if assessment.reasons.is_empty() {
        "No explicit concerns".to_string()
    } else {
        assessment.reasons.join("\n")
    };

    let add_evidence = format!("blackwell_add_evidence:{}", assessment.mutation_id);
    let run_simulation = format!("blackwell_run_simulation:{}", assessment.mutation_id);
    let request_review = format!("blackwell_request_review:{}", assessment.mutation_id);
    let override_action = format!("blackwell_override:{}", assessment.mutation_id);

    let components = vec![
        Component {
            id: "root".to_string(),
            component_type: ComponentType::Card,
            props,
            a11y: None,
            children: vec!["reasons".to_string(), "actions".to_string()],
            data_bind: None,
        },
        Component {
            id: "reasons".to_string(),
            component_type: ComponentType::Text,
            props: HashMap::from([(
                "text".to_string(),
                serde_json::Value::String(format!("Reasons:\n{}", reasons)),
            )]),
            a11y: None,
            children: vec![],
            data_bind: None,
        },
        Component {
            id: "actions".to_string(),
            component_type: ComponentType::Row,
            props: HashMap::new(),
            a11y: None,
            children: vec![
                add_evidence.clone(),
                run_simulation.clone(),
                request_review.clone(),
                override_action.clone(),
            ],
            data_bind: None,
        },
        Component {
            id: add_evidence,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Add Evidence".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!(
                        "blackwell_add_evidence:{}",
                        assessment.mutation_id
                    )),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Add Evidence")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: run_simulation,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Run Simulation".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!(
                        "blackwell_run_simulation:{}",
                        assessment.mutation_id
                    )),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Run Simulation")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: request_review,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Request Review".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!(
                        "blackwell_request_review:{}",
                        assessment.mutation_id
                    )),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Request Review")),
            children: vec![],
            data_bind: None,
        },
        Component {
            id: override_action,
            component_type: ComponentType::Button,
            props: HashMap::from([
                (
                    "label".to_string(),
                    serde_json::Value::String("Override".to_string()),
                ),
                (
                    "action".to_string(),
                    serde_json::Value::String(format!(
                        "blackwell_override:{}",
                        assessment.mutation_id
                    )),
                ),
            ]),
            a11y: Some(A11yProperties::with_label("Override")),
            children: vec![],
            data_bind: None,
        },
    ];

    let replay_contract = REPLAY_CONTRACTS.with(|store| {
        store
            .borrow()
            .get(&StorableMutationId(assessment.mutation_id.clone()))
            .map(|entry| entry.0.clone())
    });
    let (execution_profile_ref, attribution_domain_ref, replay_contract_ref) = replay_contract
        .map(|contract| {
            (
                Some(contract.execution_profile_ref),
                Some(contract.attribution_domain_ref),
                Some(format!("system_replay_contract:{}", contract.mutation_id)),
            )
        })
        .unwrap_or((None, None, None));
    let gate_level = if matches!(
        assessment.gate_outcome,
        GateOutcome::Pass | GateOutcome::Warn
    ) {
        "informational"
    } else {
        "release_blocker"
    };
    let gate_status = match assessment.gate_outcome {
        GateOutcome::Pass => "pass",
        GateOutcome::Warn => "warn",
        GateOutcome::RequireReview => "require_review",
        GateOutcome::RequireSimulation => "require_simulation",
        GateOutcome::Block => "block",
    };

    A2UIMessage::RenderSurface {
        surface_id: format!("{}:{}", surface_prefix, assessment.mutation_id),
        title: title.to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some(tone.to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some(priority.to_string()),
            intent: Some("primary".to_string()),
            severity: Some(severity.to_string()),
            workflow_id: Some(assessment.workflow_id.clone()),
            mutation_id: Some(assessment.mutation_id.clone()),
            space_id: None,
            execution_profile_ref,
            attribution_domain_ref,
            gate_level: Some(gate_level.to_string()),
            gate_status: Some(gate_status.to_string()),
            decision_gate_id: Some(format!("blackwell_gate:{}", assessment.mutation_id)),
            replay_contract_ref,
            action_target_ref: Some(format!("mutation:{}", assessment.mutation_id)),
            actor_ref: None,
            policy_ref: None,
            lineage_id: None,
            source_of_truth: Some("canister".to_string()),
            source: Some("workflow-engine".to_string()),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

fn render_blackwell_override_ack(ack: &EpistemicOverrideAck) -> A2UIMessage {
    let components = vec![Component {
        id: "root".to_string(),
        component_type: ComponentType::Card,
        props: HashMap::from([
            (
                "title".to_string(),
                serde_json::Value::String("Blackwell Override Recorded".to_string()),
            ),
            (
                "description".to_string(),
                serde_json::Value::String(format!(
                    "Override for assessment {} recorded.",
                    ack.assessment_id
                )),
            ),
            (
                "assessment_id".to_string(),
                serde_json::Value::String(ack.assessment_id.clone()),
            ),
            (
                "workflow_id".to_string(),
                serde_json::Value::String(ack.workflow_id.clone()),
            ),
            (
                "mutation_id".to_string(),
                serde_json::Value::String(ack.mutation_id.clone()),
            ),
        ]),
        a11y: None,
        children: vec![],
        data_bind: None,
    }];

    A2UIMessage::RenderSurface {
        surface_id: format!("blackwell_override_ack:{}", ack.mutation_id),
        title: "Override Recorded".to_string(),
        root: None,
        components,
        meta: Some(A2UIMeta {
            theme: Some("cortex".to_string()),
            tone: Some("info".to_string()),
            context: Some("inbox".to_string()),
            density: Some("compact".to_string()),
            priority: Some("p1".to_string()),
            intent: Some("secondary".to_string()),
            severity: Some("info".to_string()),
            workflow_id: Some(ack.workflow_id.clone()),
            mutation_id: Some(ack.mutation_id.clone()),
            space_id: None,
            execution_profile_ref: None,
            attribution_domain_ref: None,
            gate_level: Some("informational".to_string()),
            gate_status: Some("override_acknowledged".to_string()),
            decision_gate_id: Some(format!("blackwell_override_ack:{}", ack.mutation_id)),
            replay_contract_ref: Some(format!("system_replay_contract:{}", ack.mutation_id)),
            action_target_ref: Some(format!("mutation:{}", ack.mutation_id)),
            actor_ref: Some(ack.approved_by.to_text()),
            policy_ref: None,
            lineage_id: None,
            source_of_truth: Some("canister".to_string()),
            source: Some("workflow-engine".to_string()),
            timestamp: Some(ic_cdk::api::time() / 1_000_000_000),
        }),
    }
}

async fn handle_epistemic_gate_request(value: serde_json::Value) -> String {
    let workflow_id = value
        .get("workflow_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let mutation_id = value
        .get("mutation_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let decision_class = value
        .get("decision_class")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());

    match evaluate_epistemic_gate_internal(workflow_id, mutation_id, decision_class) {
        Ok(assessment) => {
            let policy = load_epistemic_policy();
            let response = render_blackwell_surface(&assessment, &policy);
            serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
        }
        Err(err) => serde_json::to_string(&A2UIMessage::Error { message: err })
            .unwrap_or_else(|_| "{}".to_string()),
    }
}

async fn handle_blackwell_override_request(value: serde_json::Value) -> String {
    let assessment_id = value
        .get("assessment_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let workflow_id = value
        .get("workflow_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let mutation_id = value
        .get("mutation_id")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();
    let justification = value
        .get("justification")
        .and_then(|v| v.as_str())
        .unwrap_or_default()
        .to_string();

    let ack = EpistemicOverrideAck {
        assessment_id,
        mutation_id,
        workflow_id,
        justification,
        approved_by: ic_cdk::api::msg_caller(),
        approved_at: ic_cdk::api::time() / 1_000_000_000,
    };

    match ack_epistemic_override(ack.clone()) {
        Ok(_) => serde_json::to_string(&render_blackwell_override_ack(&ack))
            .unwrap_or_else(|_| "{}".to_string()),
        Err(err) => serde_json::to_string(&A2UIMessage::Error { message: err })
            .unwrap_or_else(|_| "{}".to_string()),
    }
}

#[ic_cdk::update]
fn evaluate_epistemic_gate(
    workflow_id: String,
    mutation_id: String,
    decision_class: Option<String>,
) -> Result<EpistemicAssessment, String> {
    evaluate_epistemic_gate_internal(workflow_id, mutation_id, decision_class)
}

#[ic_cdk::query]
fn get_epistemic_assessment(assessment_id: String) -> Option<EpistemicAssessment> {
    EPISTEMIC_ASSESSMENTS.with(|store| {
        store
            .borrow()
            .get(&StorableAssessmentId(assessment_id))
            .map(|entry| entry.0.clone())
    })
}

#[ic_cdk::query]
fn list_epistemic_assessments(workflow_id: String, limit: u32) -> Vec<EpistemicAssessment> {
    let mut rows = EPISTEMIC_ASSESSMENTS.with(|store| {
        store
            .borrow()
            .iter()
            .map(|entry| entry.value().0.clone())
            .filter(|entry| workflow_id.trim().is_empty() || entry.workflow_id == workflow_id)
            .collect::<Vec<_>>()
    });

    rows.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    rows.truncate(limit as usize);
    rows
}

#[ic_cdk::query]
fn get_epistemic_assessment_by_mutation(mutation_id: String) -> Option<EpistemicAssessment> {
    let mutation_id = mutation_id.trim();
    if mutation_id.is_empty() {
        return None;
    }
    let assessment_id = EPISTEMIC_ASSESSMENT_BY_MUTATION.with(|store| {
        store
            .borrow()
            .get(&StorableMutationId(mutation_id.to_string()))
            .map(|entry| entry.0)
    })?;
    EPISTEMIC_ASSESSMENTS.with(|store| {
        store
            .borrow()
            .get(&StorableAssessmentId(assessment_id))
            .map(|entry| entry.0.clone())
    })
}

#[ic_cdk::query]
fn get_epistemic_policy() -> EpistemicPolicy {
    load_epistemic_policy()
}

#[ic_cdk::query]
fn get_epistemic_policy_authority() -> EpistemicPolicyAuthority {
    load_epistemic_policy_authority()
}

#[ic_cdk::update]
fn set_epistemic_policy(policy: EpistemicPolicy) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    authorize_policy_update(caller)?;

    if !(0.0..=1.0).contains(&policy.min_robustness)
        || !(0.0..=1.0).contains(&policy.max_confidence_drift)
        || !(0.0..=1.0).contains(&policy.max_fork_pressure)
        || !(0.0..=1.0).contains(&policy.max_correction_density)
    {
        return Err("policy threshold values must be within 0.0..=1.0".to_string());
    }

    save_epistemic_policy(policy);
    Ok(())
}

#[ic_cdk::update]
fn set_epistemic_policy_authority(authority: EpistemicPolicyAuthority) -> Result<(), String> {
    let caller = ic_cdk::api::msg_caller();
    ensure_controller(caller)?;
    save_epistemic_policy_authority(authority);
    Ok(())
}

#[ic_cdk::update]
fn ack_epistemic_override(ack: EpistemicOverrideAck) -> Result<(), String> {
    if ack.assessment_id.trim().is_empty()
        || ack.mutation_id.trim().is_empty()
        || ack.workflow_id.trim().is_empty()
    {
        return Err("override acknowledgement has missing required fields".to_string());
    }
    validate_override_quality(&ack.justification)?;

    let caller = ic_cdk::api::msg_caller();
    if ack.approved_by != caller {
        return Err("approved_by must match msg_caller".to_string());
    }

    let exists = EPISTEMIC_ASSESSMENTS.with(|store| {
        store
            .borrow()
            .contains_key(&StorableAssessmentId(ack.assessment_id.clone()))
    });
    if !exists {
        return Err("assessment not found".to_string());
    }

    EPISTEMIC_OVERRIDE_STORE.with(|store| {
        store.borrow_mut().insert(
            StorableAssessmentId(ack.assessment_id.clone()),
            StorableEpistemicOverrideAck(ack),
        );
    });
    Ok(())
}

#[ic_cdk::update]
fn set_space_execution_profile(space_id: String, profile: ExecutionProfile) -> Result<(), String> {
    let space_id = normalize_required_id(&space_id, "space_id")?;
    let mut profile = profile;
    profile.updated_by = ic_cdk::api::msg_caller();
    profile.updated_at = ic_cdk::api::time() / 1_000_000_000;
    SPACE_EXECUTION_PROFILES.with(|store| {
        store
            .borrow_mut()
            .insert(StorableSpaceId(space_id), StorableExecutionProfile(profile));
    });
    Ok(())
}

#[ic_cdk::query]
fn get_space_execution_profile(space_id: String) -> Option<ExecutionProfile> {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return None;
    }
    SPACE_EXECUTION_PROFILES.with(|store| {
        store
            .borrow()
            .get(&StorableSpaceId(space_id.to_string()))
            .map(|entry| entry.0.clone())
    })
}

#[ic_cdk::update]
fn upsert_attribution_domain(space_id: String, domain: AttributionDomain) -> Result<(), String> {
    let space_id = normalize_required_id(&space_id, "space_id")?;
    let mut domain = domain;
    domain.id = normalize_required_id(&domain.id, "domain.id")?;
    domain.updated_by = ic_cdk::api::msg_caller();
    domain.updated_at = ic_cdk::api::time() / 1_000_000_000;
    let key = attribution_domain_store_key(&space_id, &domain.id);
    ATTRIBUTION_DOMAINS.with(|store| {
        store
            .borrow_mut()
            .insert(key, StorableAttributionDomain(domain));
    });
    Ok(())
}

#[ic_cdk::query]
fn get_attribution_domains(space_id: String) -> Vec<AttributionDomain> {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return Vec::new();
    }
    let prefix = attribution_domain_prefix(space_id);
    let mut domains = ATTRIBUTION_DOMAINS.with(|store| {
        store
            .borrow()
            .iter()
            .filter(|entry| entry.key().0.starts_with(&prefix))
            .map(|entry| entry.value().0.clone())
            .collect::<Vec<_>>()
    });
    domains.sort_by(|a, b| a.id.cmp(&b.id));
    domains
}

#[ic_cdk::update]
fn bind_contribution_attribution_domain(
    contribution_id: String,
    space_id: String,
    domain_id: String,
) -> Result<(), String> {
    let contribution_id = normalize_required_id(&contribution_id, "contribution_id")?;
    let space_id = normalize_required_id(&space_id, "space_id")?;
    let domain_id = normalize_required_id(&domain_id, "domain_id")?;
    let domain_key = attribution_domain_store_key(&space_id, &domain_id);

    let domain_exists = ATTRIBUTION_DOMAINS.with(|store| store.borrow().contains_key(&domain_key));
    if !domain_exists {
        return Err(format!(
            "attribution domain {} not found in space {}",
            domain_id, space_id
        ));
    }

    let now = ic_cdk::api::time() / 1_000_000_000;
    let caller = ic_cdk::api::msg_caller();
    let binding = ContributionAttributionBinding {
        contribution_id: contribution_id.clone(),
        space_id: space_id.clone(),
        domain_id: domain_id.clone(),
        bound_by: caller,
        bound_at: now,
    };
    CONTRIBUTION_ATTRIBUTION_BINDINGS.with(|store| {
        store.borrow_mut().insert(
            StorableContributionId(contribution_id.clone()),
            StorableContributionAttributionBinding(binding),
        );
    });

    let execution_profile_ref = lookup_execution_profile_ref(Some(&space_id));
    let attribution_domain_ref = attribution_domain_surface_ref(&space_id, &domain_id);
    let deterministic_input_hash = replay_input_hash(&serde_json::json!({
        "contribution_id": contribution_id.clone(),
        "space_id": space_id.clone(),
        "domain_id": domain_id.clone(),
        "bound_by": caller.to_text(),
        "bound_at": now
    }));
    let evidence_refs = vec![
        format!("space:{}", space_id),
        format!("domain:{}", domain_id),
        format!("bound_by:{}", caller.to_text()),
    ];
    let decision_digest = Some(replay_decision_digest(
        &contribution_id,
        None,
        None,
        &evidence_refs,
    ));
    store_replay_contract(ReplayContract {
        mutation_id: contribution_id.clone(),
        workflow_id: format!("binding:{}", space_id),
        action_target: format!("contribution:{}", contribution_id),
        adapter_set_ref: "nostra://workflow_engine/bind_contribution_attribution_domain"
            .to_string(),
        execution_profile_ref,
        attribution_domain_ref,
        deterministic_input_hash,
        lineage_id: Some(format!("lineage:binding:{}:{}", space_id, contribution_id)),
        policy_ref: None,
        policy_snapshot_ref: None,
        evidence_refs,
        decision_digest,
        captured_at: now,
    });

    Ok(())
}

#[ic_cdk::query]
fn get_contribution_attribution_binding(
    contribution_id: String,
) -> Option<ContributionAttributionBinding> {
    let contribution_id = contribution_id.trim();
    if contribution_id.is_empty() {
        return None;
    }
    CONTRIBUTION_ATTRIBUTION_BINDINGS.with(|store| {
        store
            .borrow()
            .get(&StorableContributionId(contribution_id.to_string()))
            .map(|entry| entry.0.clone())
    })
}

#[ic_cdk::query]
fn get_replay_contract(mutation_id: String) -> Option<ReplayContract> {
    let mutation_id = mutation_id.trim();
    if mutation_id.is_empty() {
        return None;
    }
    REPLAY_CONTRACTS.with(|store| {
        store
            .borrow()
            .get(&StorableMutationId(mutation_id.to_string()))
            .map(|entry| entry.0.clone())
    })
}

#[ic_cdk::query]
fn list_space_replay_contracts(space_id: String, limit: u32) -> Vec<ReplayContract> {
    let normalized = space_id.trim();
    if normalized.is_empty() {
        return Vec::new();
    }
    let mut rows = REPLAY_CONTRACTS.with(|store| {
        store
            .borrow()
            .iter()
            .map(|entry| entry.value().0.clone())
            .filter(|entry| replay_contract_matches_space(entry, normalized))
            .collect::<Vec<_>>()
    });
    rows.sort_by(|a, b| b.captured_at.cmp(&a.captured_at));
    rows.truncate(limit as usize);
    rows
}

#[ic_cdk::query]
fn get_decision_lineage_by_mutation(mutation_id: String) -> Option<DecisionLineage> {
    get_replay_contract(mutation_id).and_then(|contract| decision_lineage_from_replay(&contract))
}

#[ic_cdk::query]
fn list_space_decision_lineage(space_id: String, limit: u32) -> Vec<DecisionLineage> {
    list_space_replay_contracts(space_id, limit.saturating_mul(2).max(limit))
        .into_iter()
        .filter_map(|contract| decision_lineage_from_replay(&contract))
        .take(limit as usize)
        .collect()
}

fn normalize_chat_thread_id(
    thread_id: Option<String>,
    conversation_id: Option<String>,
) -> Option<String> {
    thread_id
        .or(conversation_id)
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn chat_thread_key(thread_id: &str) -> StorableChatThreadId {
    StorableChatThreadId(thread_id.to_string())
}

fn load_chat_thread_state(thread_id: &str) -> ChatThreadState {
    CHAT_THREADS.with(|threads| {
        threads
            .borrow()
            .get(&chat_thread_key(thread_id))
            .map(|state| state.0)
            .unwrap_or_else(|| ChatThreadState {
                thread_id: thread_id.to_string(),
                turns: Vec::new(),
                updated_at: 0,
            })
    })
}

fn persist_chat_thread_state(state: ChatThreadState) {
    let key = chat_thread_key(state.thread_id.as_str());
    CHAT_THREADS.with(|threads| {
        threads.borrow_mut().insert(key, StorableChatThreadState(state));
    });
}

fn append_chat_turn(state: &mut ChatThreadState, role: &str, content: String) {
    state.turns.push(ChatTurn {
        role: role.to_string(),
        content,
        created_at: current_chat_timestamp(),
    });
    if state.turns.len() > CHAT_THREAD_MAX_TURNS {
        let excess = state.turns.len() - CHAT_THREAD_MAX_TURNS;
        state.turns.drain(0..excess);
    }
    state.updated_at = current_chat_timestamp();
}

fn render_chat_turn_summary(turns: &[ChatTurn]) -> String {
    let mut lines = Vec::new();
    for turn in turns.iter().rev().take(4).collect::<Vec<_>>().into_iter().rev() {
        lines.push(format!("{}: {}", turn.role, turn.content));
    }
    lines.join("\n")
}

fn current_chat_timestamp() -> u64 {
    #[cfg(all(not(test), target_arch = "wasm32"))]
    {
        ic_cdk::api::time() / 1_000_000_000
    }

    #[cfg(any(test, not(target_arch = "wasm32")))]
    {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_secs())
            .unwrap_or(0)
    }
}

fn handle_chat_message(value: serde_json::Value) -> String {
    let Ok(envelope) = serde_json::from_value::<ChatMessageEnvelope>(value) else {
        return "invalid chat message envelope".to_string();
    };
    let Some(thread_id) =
        normalize_chat_thread_id(envelope.thread_id.clone(), envelope.conversation_id.clone())
    else {
        return "chat message requires a thread_id or conversation_id".to_string();
    };
    if envelope.text.trim().is_empty() {
        return "chat message text is required".to_string();
    }

    let mut state = load_chat_thread_state(thread_id.as_str());
    if state.thread_id.is_empty() {
        state.thread_id = thread_id.clone();
    }
    append_chat_turn(&mut state, "user", envelope.text.trim().to_string());

    let prior_turn_count = state.turns.len().saturating_sub(1);
    let recent_context = render_chat_turn_summary(&state.turns);
    let mut response_sections = vec![
        format!(
            "Thread {thread_id} resumed with {prior_turn_count} prior turn{}.",
            if prior_turn_count == 1 { "" } else { "s" }
        ),
        format!("Latest request: {}", envelope.text.trim()),
    ];

    if !envelope.context_refs.is_empty() {
        response_sections.push(format!(
            "Context refs: {}",
            envelope.context_refs.join(", ")
        ));
    }

    if !recent_context.is_empty() {
        response_sections.push(format!("Recent context:\n{recent_context}"));
    }

    let response = response_sections.join("\n");
    append_chat_turn(&mut state, "assistant", response.clone());
    persist_chat_thread_state(state);
    response
}

#[ic_cdk::update]
async fn process_message(message: String) -> String {
    if let Ok(value) = serde_json::from_str::<serde_json::Value>(&message) {
        if let Some(msg_type) = value.get("type").and_then(|v| v.as_str()) {
            match msg_type {
                "offline_conflict" => return handle_offline_conflict(value).await,
                "offline_conflict_decision" => {
                    return handle_offline_conflict_decision(value).await;
                }
                "epistemic_gate_request" => return handle_epistemic_gate_request(value).await,
                "blackwell_override_ack" => return handle_blackwell_override_request(value).await,
                "chat_message" => return handle_chat_message(value),
                _ => {}
            }
        }

        if value.get("text").is_some()
            && (value.get("threadId").is_some() || value.get("conversationId").is_some())
        {
            return handle_chat_message(value);
        }
    }

    let mut props = HashMap::new();
    props.insert(
        "content".to_string(),
        serde_json::Value::String(format!("ECHO: {}", message)),
    );

    let components = vec![Component {
        id: "response_card".to_string(),
        component_type: ComponentType::Card,
        props,
        a11y: None,
        children: vec![],
        data_bind: None,
    }];

    let response = A2UIMessage::RenderSurface {
        surface_id: "chat_stream".to_string(),
        title: "Console Response".to_string(),
        root: None,
        components,
        meta: None,
    };

    serde_json::to_string(&response).unwrap_or_else(|_| "{}".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicU64, Ordering};

    static CHAT_TEST_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_thread_id() -> String {
        format!(
            "thread-test-{}",
            CHAT_TEST_COUNTER.fetch_add(1, Ordering::Relaxed)
        )
    }

    fn high_confidence_signals() -> EpistemicSignalSnapshot {
        EpistemicSignalSnapshot {
            confidence_score: 0.90,
            source_reliability: 0.90,
            assumption_count: 1,
            evidence_count: 2,
            alternative_count: 2,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        }
    }

    #[test]
    fn test_is_valid_dpub_dir() {
        assert!(is_valid_dpub_dir("lib/dpubs/example"));
        assert!(is_valid_dpub_dir("/lib/dpubs/example"));
        assert!(!is_valid_dpub_dir("lib/books/example"));
        assert!(!is_valid_dpub_dir("../lib/dpubs/example"));
        assert!(!is_valid_dpub_dir(""));
    }

    #[test]
    fn test_treaty_required() {
        assert!(!treaty_required(None, "space:a", None));
        assert!(!treaty_required(Some("space:a"), "space:a", None));
        assert!(treaty_required(Some("space:b"), "space:a", None));
        assert!(!treaty_required(Some("space:b"), "space:a", Some("token")));
        assert!(!treaty_required(Some(""), "space:a", None));
    }

    #[test]
    fn test_build_dpub_feed_orders_and_limits() {
        let editions = vec![json!({"version": "1.0.0"}), json!({"version": "1.0.1"})];
        let feed = build_dpub_feed("lib/dpubs/x", editions, 1);
        let items = feed.get("items").and_then(|v| v.as_array()).unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(
            items[0].get("version").and_then(|v| v.as_str()),
            Some("1.0.1")
        );
    }

    #[test]
    fn test_dpub_base_dir_from_path() {
        assert_eq!(
            dpub_base_dir_from_path("lib/dpubs/my-dpub/editions/1.0.0"),
            Some("lib/dpubs/my-dpub".to_string())
        );
        assert_eq!(
            dpub_base_dir_from_path("/lib/dpubs/my-dpub/dpub.json"),
            Some("lib/dpubs/my-dpub".to_string())
        );
        assert_eq!(dpub_base_dir_from_path("lib/books/x"), None);
        assert_eq!(dpub_base_dir_from_path("lib/dpubs/"), None);
    }

    #[test]
    fn test_is_valid_vfs_path() {
        assert!(is_valid_vfs_path("lib/dpubs/x"));
        assert!(is_valid_vfs_path("/lib/chronicle/events.jsonl"));
        assert!(!is_valid_vfs_path("../lib/dpubs/x"));
        assert!(!is_valid_vfs_path(""));
    }

    #[test]
    fn test_enforce_non_dpub_guard_requires_token() {
        assert!(enforce_non_dpub_guard(Some("space:a"), None).is_err());
        assert!(enforce_non_dpub_guard(Some("space:a"), Some(" ")).is_err());
        assert!(enforce_non_dpub_guard(None, Some("token")).is_err());
        assert!(enforce_non_dpub_guard(Some("space:a"), Some("token")).is_ok());
    }

    #[test]
    fn test_gate_evidence_threshold_boundary() {
        let policy = default_epistemic_policy();
        let pass = compute_epistemic_gate(
            &policy,
            &DecisionClass::Governance,
            &high_confidence_signals(),
        );
        assert_eq!(pass.gate_outcome, GateOutcome::Pass);

        let mut below = high_confidence_signals();
        below.evidence_count = policy.min_evidence.saturating_sub(1);
        let review = compute_epistemic_gate(&policy, &DecisionClass::Governance, &below);
        assert_eq!(review.gate_outcome, GateOutcome::RequireReview);
    }

    #[test]
    fn test_gate_alternatives_apply_to_governance_and_merge_only() {
        let policy = default_epistemic_policy();
        let mut signals = high_confidence_signals();
        signals.alternative_count = 0;

        let governance = compute_epistemic_gate(&policy, &DecisionClass::Governance, &signals);
        assert_eq!(governance.gate_outcome, GateOutcome::RequireReview);

        let standard = compute_epistemic_gate(&policy, &DecisionClass::Standard, &signals);
        assert_eq!(standard.gate_outcome, GateOutcome::Pass);
    }

    #[test]
    fn test_gate_low_robustness_with_high_voi_requires_simulation() {
        let policy = default_epistemic_policy();
        let signals = EpistemicSignalSnapshot {
            confidence_score: 0.40,
            source_reliability: 0.40,
            assumption_count: 2,
            evidence_count: 2,
            alternative_count: 2,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        };
        let result = compute_epistemic_gate(&policy, &DecisionClass::Governance, &signals);
        assert_eq!(result.gate_outcome, GateOutcome::RequireSimulation);
    }

    #[test]
    fn test_gate_pressure_signals_produce_warn() {
        let policy = default_epistemic_policy();
        let mut signals = high_confidence_signals();
        signals.confidence_drift = policy.max_confidence_drift + 0.01;
        let result = compute_epistemic_gate(&policy, &DecisionClass::Standard, &signals);
        assert_eq!(result.gate_outcome, GateOutcome::Warn);
    }

    #[test]
    fn test_observe_mode_never_blocks() {
        let mut policy = default_epistemic_policy();
        policy.mode = EpistemicMode::Observe;
        let signals = EpistemicSignalSnapshot {
            confidence_score: 0.05,
            source_reliability: 0.05,
            assumption_count: 1,
            evidence_count: 2,
            alternative_count: 2,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        };
        let result = compute_epistemic_gate(&policy, &DecisionClass::Governance, &signals);
        assert_ne!(result.gate_outcome, GateOutcome::Block);
    }

    #[test]
    fn test_soft_gate_downgrades_block_when_disabled() {
        let mut policy = default_epistemic_policy();
        policy.mode = EpistemicMode::SoftGate;
        policy.block_on_soft = false;
        let signals = EpistemicSignalSnapshot {
            confidence_score: 0.05,
            source_reliability: 0.05,
            assumption_count: 1,
            evidence_count: 2,
            alternative_count: 2,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        };
        let result = compute_epistemic_gate(&policy, &DecisionClass::Governance, &signals);
        assert_eq!(result.gate_outcome, GateOutcome::RequireReview);
    }

    #[test]
    fn test_hard_gate_blocks_only_enforced_classes() {
        let mut policy = default_epistemic_policy();
        policy.mode = EpistemicMode::HardGate;

        let signals = EpistemicSignalSnapshot {
            confidence_score: 0.05,
            source_reliability: 0.05,
            assumption_count: 1,
            evidence_count: 2,
            alternative_count: 2,
            confidence_drift: 0.0,
            fork_pressure: 0.0,
            correction_density: 0.0,
        };

        let governance = compute_epistemic_gate(&policy, &DecisionClass::Governance, &signals);
        assert_eq!(governance.gate_outcome, GateOutcome::Block);

        let standard = compute_epistemic_gate(&policy, &DecisionClass::Standard, &signals);
        assert_ne!(standard.gate_outcome, GateOutcome::Block);
    }

    #[test]
    fn chat_message_envelope_resumes_and_persists_thread_history() {
        let thread_id = unique_thread_id();

        let first = handle_chat_message(json!({
            "type": "chat_message",
            "threadId": thread_id,
            "text": "Summarize the current status.",
            "contextRefs": ["artifact-1"]
        }));

        assert!(first.contains("Thread thread-test-"));
        assert!(first.contains("resumed with 0 prior turn"));
        assert!(first.contains("Context refs: artifact-1"));

        let second = handle_chat_message(json!({
            "type": "chat_message",
            "threadId": thread_id,
            "text": "Continue the same thread."
        }));

        assert!(second.contains("resumed with 2 prior turns"));
        assert!(second.contains("Latest request: Continue the same thread."));

        let state = load_chat_thread_state(thread_id.as_str());
        assert_eq!(state.thread_id, thread_id);
        assert_eq!(state.turns.len(), 4);
        assert_eq!(state.turns[0].role, "user");
        assert_eq!(state.turns[1].role, "assistant");
        assert_eq!(state.turns[2].role, "user");
        assert_eq!(state.turns[3].role, "assistant");
    }
}

// Custom export for JSON interface
ic_cdk::export_candid!();
