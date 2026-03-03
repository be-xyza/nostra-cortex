use crate::gateway::state::{AgentApprovalSignal, GatewayState};
use crate::services::acp_adapter::{
    AcpAdapter, AcpPolicyConfig, AcpPolicyError, FsReadTextFileRequest, FsWriteTextFileRequest,
    TerminalCreateRequest,
};
use crate::services::acp_metrics::get_acp_metrics_snapshot;
use crate::services::acp_protocol::{JsonRpcRequest, handle_rpc_request, is_acp_pilot_enabled};
use crate::services::agent_evaluation_loop::evaluate_agent_gate;
use crate::services::agent_execution_events::{emit_agent_execution_record, hash_json_value};
use crate::services::artifact_collab_crdt::{
    ArtifactCollabCheckpoint, ArtifactCrdtConflict, ArtifactCrdtState, ArtifactCrdtUpdateEnvelope,
    apply_update as apply_crdt_update, build_replace_markdown_update,
    init_state as init_crdt_state, materialize_markdown as materialize_crdt_markdown,
    state_hash as crdt_state_hash,
};
use crate::services::brand_policy::{BrandPolicyBundle, BrandPolicyRegistryService};
use crate::services::cortex_ux::{
    ArtifactAuditEvent, ShellLayoutSpec, UX_STATUS_APPROVED, UX_STATUS_BLOCKED_MISSING_BASELINE,
    UX_STATUS_CANDIDATE, UX_STATUS_NEW, UX_STATUS_OVERDUE_REMEASUREMENT, UX_STATUS_REJECTED,
    UX_STATUS_REMEASURED, UX_STATUS_SHIPPED, UxFeedbackEvent, UxFeedbackQueueItem,
    UxLayoutEvaluationRequest, UxPromotionApproval, UxPromotionDecision, UxPromotionRejection,
    ViewCapabilityManifest, ViewCapabilityMatrixRow, default_artifact_capability_manifest,
    default_persisted_shell_contract, evaluate_cuqs, has_route_access,
    load_persisted_shell_contract, resolve_capability_matrix, resolve_pattern_contracts,
    resolve_shell_layout_spec, resolve_view_capability_manifests, role_rank,
    save_persisted_shell_contract, valid_feedback_status,
};
use crate::services::cortex_ux_store::{
    CortexReplayResult, CortexSyncStatus, cortex_ux_store_manager, is_cortex_ux_local_path,
    to_cortex_vfs_key,
};
use crate::services::file_system_service::FileSystemService;
use crate::services::governance_client::{ActionScopeEvaluation, GovernanceClient};
use crate::services::heap_mapper::{
    EmitHeapBlockRequest, HeapBlockProjection, canonicalize_emit_heap_block,
    map_emit_heap_block_to_agui_mutations, parse_emit_heap_block,
    parse_iso_timestamp as parse_heap_iso_timestamp, project_heap_block, validate_emit_heap_block,
};
use crate::services::siq_types::{
    SiqCoverage, SiqDependencyClosure, SiqGateSummary, SiqGraphProjection, SiqHealth,
    SiqRunArtifact,
};
use crate::services::streaming_transport::{
    ArtifactRealtimeAckCursor, ArtifactRealtimeBacklogItem, ArtifactRealtimeEnvelope,
    ArtifactRealtimeIntegrityReport, ArtifactRealtimePollResult, ArtifactRealtimeResyncResult,
    ArtifactRealtimeTransportStatus, CortexRealtimeSloBreachEvent, CortexRealtimeSloStatus,
    streaming_transport_manager,
};
use crate::services::terminal_service::{
    AcpTerminalOutputRequest, AcpTerminalWaitRequest, TerminalService,
};
use crate::services::theme_policy::{
    ThemePolicyPreferences, current_theme_policy, persist_theme_policy,
};
use crate::services::viewspec::{
    ConstraintRule, VIEWSPEC_INDEX_KEY, ViewSpecConfidence, ViewSpecEventRecord,
    ViewSpecGovernanceRef, ViewSpecIndexEntry, ViewSpecLockState, ViewSpecProposalDecisionRecord,
    ViewSpecProposalEnvelope, ViewSpecProposalMergeRecord, ViewSpecProposalReviewRecord,
    ViewSpecProposalStatus, ViewSpecScope, ViewSpecScopeAdoptionRecord, ViewSpecV1,
    ViewSpecValidationResult, compile_viewspec_to_render_surface, current_viewspec_key,
    history_viewspec_key, now_iso as viewspec_now_iso, proposal_store_key, scope_key,
    validate_viewspec, viewspec_events_key,
};
use crate::services::viewspec_learning::{
    LearningPolicyV1, LearningReplayResult, SpaceLearningProfileV1,
    VIEWSPEC_LEARNING_SIGNAL_INDEX_KEY, ViewSpecLearningSignal, extract_space_id_from_payload,
    is_supported_event_type, learning_profile_key, learning_replay_key, learning_signals_key,
    normalize_event_type, recompute_viewspec_confidence, replay_space_learning_profile,
    reset_space_learning_profile, sanitize_token as sanitize_learning_token,
    validate_learning_signal,
};
use crate::services::viewspec_synthesis::{
    VIEWSPEC_CANDIDATE_SET_INDEX_KEY, ViewSpecCandidateEnvelope, ViewSpecCandidateSet,
    ViewSpecCandidateSetIndexEntry, ViewSpecGenerationMode, blocked_count, candidate_set_store_key,
    compute_candidate_input_hash, generate_candidate_set,
};
use crate::services::workflow_engine_client::{
    AttributionDomain, EpistemicAssessment, ExecutionProfile, ReplayContract, WorkflowEngineClient,
};
use axum::{
    Router,
    body::{Body, to_bytes},
    extract::{
        Path, Query, Request, State,
        ws::{Message, WebSocket, WebSocketUpgrade},
    },
    http::{HeaderMap, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Json, Response},
    routing::{get, post, put},
};
use candid::Principal;
use chrono::{DateTime, Datelike, Timelike, Utc};
use cortex_domain::agent::contracts::{
    ActionTarget, AgentExecutionPhase, AgentExecutionRecord, AgentIntent, AgentRun, AgentRunEvent,
    AgentRunStatus, AuthorityExecutionOutcome, AuthorityLevel, ShadowComparisonSummary,
    ShadowDivergenceRecord, ShadowDivergenceSeverity, TEMPORAL_WORKFLOW_QUERY_RUN_SNAPSHOT,
    TEMPORAL_WORKFLOW_SIGNAL_HUMAN_APPROVAL, TemporalBridgeRunSnapshot,
    TemporalBridgeSignalCommand, TemporalBridgeStartCommand, TemporalRunBinding,
};
use cortex_domain::brand::policy::{
    BrandPolicyDocument, TemporalWindow, normalize_brand_policy_document,
};
use cortex_domain::capabilities::navigation_graph::{
    CapabilityEdge as DomainCapabilityEdge, CapabilityId as DomainCapabilityId,
    CapabilityNode as DomainCapabilityNode, EdgeRelationship as DomainEdgeRelationship,
    IntentType as DomainIntentType, OperationalFrequency, PlatformCapabilityCatalog,
    SpaceCapabilityGraph, SpaceCapabilityNodeOverride, SurfacingHeuristic,
};
use cortex_domain::graph::{EdgeKind as DomainEdgeKind, Graph as DomainGraph, Node as DomainNode};
use cortex_domain::integrity::{
    Constraint as DomainConstraint, Direction as DomainDirection,
    EdgeSelector as DomainEdgeSelector, IntegrityPredicate as DomainIntegrityPredicate,
    IntegrityRule as DomainIntegrityRule, IntegrityScope as DomainIntegrityScope,
    NodeSelector as DomainNodeSelector, Severity as DomainSeverity,
};
use cortex_domain::simulation::scenario::{
    ScenarioConstraints as DomainScenarioConstraints,
    ScenarioDefinition as DomainScenarioDefinition, ScenarioMetadata as DomainScenarioMetadata,
    ScenarioRound as DomainScenarioRound, ScenarioRoundAction as DomainScenarioRoundAction,
};
use cortex_domain::simulation::session::{
    SimulationAction as DomainSimulationAction, run_deterministic_session as run_domain_session,
};
use cortex_domain::ux::{CompilationContext, compile_navigation_plan};
use futures_util::{sink::SinkExt, stream::StreamExt};
use nostra_extraction::contribution_graph::{
    DoctorReport, EditionDiffReport, ContributionGraphV1, PathAssessmentBundleV1, assess_path,
    diff_editions, doctor, ingest_and_write, publish_edition, query_graph, simulate,
    validate_research_portfolio,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fs;
use std::fs::OpenOptions;
use std::future::Future;
use std::io::Write;
use std::net::SocketAddr;
use std::path::{Path as FsPath, PathBuf};
use std::process::Command;
use std::str::FromStr;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, Mutex};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tower::util::ServiceExt;

#[cfg(feature = "temporal-sdk-native")]
use squads_temporal_client::{
    ClientOptionsBuilder as TemporalClientOptionsBuilder, WorkflowClientTrait,
    WorkflowOptions as TemporalWorkflowOptions,
};
#[cfg(feature = "temporal-sdk-native")]
use squads_temporal_sdk_core_protos::{
    coresdk::{AsJsonPayloadExt, FromJsonPayloadExt, IntoPayloadsExt},
    temporal::api::{
        common::v1::Payload, enums::v1::WorkflowIdConflictPolicy, query::v1::WorkflowQuery,
    },
};

pub struct GatewayService;
pub(crate) const LEGACY_BYPASS_HEADER: &str = "x-cortex-runtime-legacy-bypass";

#[derive(Default, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RuntimeDispatchTelemetryRoute {
    request_count: u64,
    replay_hit_count: u64,
    latency_ms_samples: Vec<u64>,
    status_class_counts: BTreeMap<String, u64>,
    error_class_counts: BTreeMap<String, u64>,
    transaction_boundary_counts: BTreeMap<String, u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct RuntimeDispatchTelemetryState {
    schema_version: String,
    updated_at: String,
    total_requests: u64,
    total_replay_hits: u64,
    routes: BTreeMap<String, RuntimeDispatchTelemetryRoute>,
}

impl Default for RuntimeDispatchTelemetryState {
    fn default() -> Self {
        Self {
            schema_version: "runtime-dispatch-telemetry/v1".to_string(),
            updated_at: now_iso(),
            total_requests: 0,
            total_replay_hits: 0,
            routes: BTreeMap::new(),
        }
    }
}

#[derive(Serialize)]
struct SystemStatus {
    dfx_running: bool,
    version: String,
    replica_port: u16,
}

#[derive(Serialize)]
struct SystemReady {
    ready: bool,
    gateway_port: u16,
    dfx_port_healthy: bool,
    notes: Vec<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SystemBuild {
    build_id: String,
    build_time_utc: String,
    gateway_dispatch_mode: String,
    gateway_port: u16,
    workspace_root: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SystemBrandPolicyResponse {
    policy: BrandPolicyDocument,
    policy_version: u64,
    policy_digest: String,
    active_temporal_state: String,
    server_time_utc: String,
    source_of_truth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    degraded_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_normalization: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct BrandPolicyCacheRecord {
    policy: BrandPolicyDocument,
    policy_version: u64,
    policy_digest: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct LocalGatewayQueueMutationRecord {
    mutation_id: String,
    idempotency_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    space_id: Option<String>,
    kip_command: String,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    timestamp_iso: Option<String>,
    attempts: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_attempt_at: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    last_attempt_at_iso: Option<String>,
    conflict_state: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct LocalGatewayQueueSnapshot {
    queue_size: usize,
    conflict_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    oldest_timestamp_iso: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    newest_timestamp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    newest_timestamp_iso: Option<String>,
    items: Vec<LocalGatewayQueueMutationRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct LocalGatewayQueueActionResponse {
    accepted: bool,
    mutation_id: String,
    action: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexCapabilityMatrixResponse {
    schema_version: String,
    generated_at: String,
    layout_spec: ShellLayoutSpec,
    view_capabilities: Vec<ViewCapabilityManifest>,
    patterns: Vec<crate::services::cortex_ux::PatternContract>,
    matrix: Vec<ViewCapabilityMatrixRow>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityGraphResponse {
    schema_version: String,
    generated_at: String,
    source_of_truth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    layout_hints: Option<SystemCapabilityGraphLayoutHints>,
    #[serde(skip_serializing_if = "Option::is_none")]
    legend: Option<SystemCapabilityGraphLegend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    capabilities_version: Option<String>,
    nodes: Vec<SystemCapabilityGraphNode>,
    edges: Vec<SystemCapabilityGraphEdge>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityGraphLayoutHints {
    engine: String,
    seed: String,
    cluster_by: String,
    groups: Vec<SystemCapabilityGraphLayoutGroup>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityGraphLayoutGroup {
    key: String,
    label: String,
    order: usize,
    color: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityGraphLegend {
    intent_type_colors: BTreeMap<String, String>,
    relationship_styles: BTreeMap<String, String>,
    lock_semantics: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityGraphNode {
    id: String,
    title: String,
    description: String,
    intent_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    promotion_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    invariant_violations: Option<Vec<serde_json::Value>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    cluster_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    domain: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    locked_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    visibility_state: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    health: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    inspector: Option<SystemCapabilityNodeInspector>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct SystemCapabilityNodeInspector {
    #[serde(skip_serializing_if = "Option::is_none")]
    route_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pattern_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_role: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    required_role_rank: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    operator_critical: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    promotion_status: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct SystemCapabilityGraphEdge {
    from: String,
    to: String,
    relationship: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    relationship_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    confidence: Option<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    rationale: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    directionality: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SpaceCapabilityGraphUpsertResponse {
    accepted: bool,
    space_id: String,
    capability_graph_hash: String,
    capability_graph_version: String,
    stored_at: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct SpaceNavigationPlanQuery {
    actor_role: Option<String>,
    intent: Option<String>,
    density: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexLayoutEvaluationResponse {
    evaluation: crate::services::cortex_ux::UxCandidateEvaluation,
    #[serde(skip_serializing_if = "Option::is_none")]
    promotion_decision: Option<UxPromotionDecision>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackAck {
    accepted: bool,
    event_id: String,
    stored_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackQuery {
    status: Option<String>,
    route_id: Option<String>,
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackQueueResponse {
    schema_version: String,
    generated_at: String,
    items: Vec<UxFeedbackQueueItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackTriageRequest {
    queue_id: String,
    status: String,
    #[serde(default)]
    priority: Option<String>,
    #[serde(default)]
    assigned_to: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    baseline_metric_date: Option<String>,
    #[serde(default)]
    post_release_metric_date: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackTriageResponse {
    accepted: bool,
    item: UxFeedbackQueueItem,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct CortexPromotionHistoryQuery {
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexPromotionHistoryResponse {
    schema_version: String,
    generated_at: String,
    decisions: Vec<UxPromotionDecision>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct CortexCloseoutTasksQuery {
    contribution_id: Option<String>,
    as_of: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CortexCloseoutTaskRecord {
    task_id: String,
    title: String,
    owner: String,
    status: String,
    due_at_utc: String,
    kind: String,
    #[serde(default)]
    acceptance: Vec<String>,
    #[serde(default)]
    evidence_paths: Vec<String>,
    #[serde(default)]
    validation_commands: Vec<String>,
    #[serde(default)]
    depends_on: Vec<String>,
    last_updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CortexCloseoutTaskLedger {
    schema_version: String,
    contribution_id: String,
    generated_at: String,
    tasks: Vec<CortexCloseoutTaskRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CortexCloseoutTaskView {
    #[serde(flatten)]
    task: CortexCloseoutTaskRecord,
    overdue: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
struct CortexCloseoutTaskSummary {
    total: usize,
    overdue: usize,
    complete: usize,
    completion_ratio: f64,
    status_counts: BTreeMap<String, usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct CortexCloseoutTasksResponse {
    schema_version: String,
    generated_at: String,
    as_of: String,
    contribution_id: String,
    source_path: String,
    summary: CortexCloseoutTaskSummary,
    tasks: Vec<CortexCloseoutTaskView>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCandidateRequest {
    intent: String,
    #[serde(default)]
    scope: Option<ViewSpecScope>,
    #[serde(default)]
    generation_mode: Option<String>,
    #[serde(default)]
    candidate_set_id: Option<String>,
    #[serde(default)]
    actor_id: Option<String>,
    #[serde(default)]
    actor_role: Option<String>,
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    constraints: Vec<ConstraintRule>,
    #[serde(default)]
    count: Option<usize>,
    #[serde(default)]
    created_by: Option<String>,
    #[serde(default)]
    source_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCandidatesResponse {
    schema_version: String,
    generated_at: String,
    candidate_set_id: String,
    candidates: Vec<ViewSpecCandidateEnvelope>,
    blocked_count: u32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCandidateSetResponse {
    schema_version: String,
    generated_at: String,
    candidate_set: ViewSpecCandidateSet,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCandidateStageRequest {
    candidate_id: String,
    staged_by: String,
    rationale: String,
    expected_input_hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCandidateStageResponse {
    accepted: bool,
    view_spec_id: String,
    scope_key: String,
    stored_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecValidateRequest {
    view_spec: ViewSpecV1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecValidationResponse {
    schema_version: String,
    generated_at: String,
    validation: ViewSpecValidationResult,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCompileRequest {
    view_spec: ViewSpecV1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecCompileResponse {
    schema_version: String,
    generated_at: String,
    validation: ViewSpecValidationResult,
    compiled_surface: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLockRequest {
    view_spec_id: String,
    #[serde(default)]
    scope: Option<ViewSpecScope>,
    locked_by: String,
    rationale: String,
    #[serde(default)]
    structural_change: Option<bool>,
    #[serde(default)]
    approved_by: Option<String>,
    #[serde(default)]
    approved_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecForkRequest {
    #[serde(default)]
    target_scope: Option<ViewSpecScope>,
    fork_reason: String,
    forked_by: String,
    #[serde(default)]
    new_view_spec_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposeRequest {
    proposed_by: String,
    rationale: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecAckResponse {
    accepted: bool,
    view_spec_id: String,
    scope_key: String,
    stored_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalResponse {
    accepted: bool,
    proposal: ViewSpecProposalEnvelope,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLookupQuery {
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    route_id: Option<String>,
    #[serde(default)]
    role: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningSignalRequest {
    #[serde(default)]
    signal_id: Option<String>,
    event_type: String,
    view_spec_id: String,
    #[serde(default)]
    space_id: Option<String>,
    actor: String,
    #[serde(default)]
    timestamp: Option<String>,
    #[serde(default)]
    payload: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningSignalResponse {
    accepted: bool,
    signal: ViewSpecLearningSignal,
    stored_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningProfileResponse {
    schema_version: String,
    generated_at: String,
    profile: SpaceLearningProfileV1,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningRecomputeRequest {
    actor: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningRecomputeResponse {
    accepted: bool,
    profile: SpaceLearningProfileV1,
    replay: LearningReplayResult,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningResetRequest {
    actor: String,
    #[serde(default)]
    reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecConfidenceRecomputeRequest {
    #[serde(default)]
    scope: Option<ViewSpecScope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecConfidenceRecomputeResponse {
    view_spec_id: String,
    space_id: String,
    confidence: ViewSpecConfidence,
    profile_version: u64,
    signal_count: u64,
    policy: LearningPolicyV1,
    persisted: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecLearningSignalIndexEntry {
    date: String,
    key: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentEventRequest {
    run_id: String,
    space_id: String,
    mode: String,
    surface_variant: String,
    event_type: String,
    timestamp: String,
    #[serde(default)]
    payload: Value,
    #[serde(default)]
    build_id: Option<String>,
    host: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentEventRecord {
    event_id: String,
    run_id: String,
    space_id: String,
    mode: String,
    surface_variant: String,
    event_type: String,
    timestamp: String,
    payload: Value,
    #[serde(default)]
    build_id: Option<String>,
    host: String,
    stored_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentEventResponse {
    accepted: bool,
    stored_key: String,
    event_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentMetrics {
    #[serde(default)]
    time_to_first_interaction_ms: Option<u64>,
    #[serde(default)]
    task_completion_ms: Option<u64>,
    #[serde(default)]
    approval_decision_count: u64,
    #[serde(default)]
    spatial_interaction_count: u64,
    #[serde(default)]
    adapter_fallback_rate: f64,
    #[serde(default)]
    error_event_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentComplexityDelta {
    #[serde(default)]
    bundle_delta_kb: Option<f64>,
    #[serde(default)]
    runtime_overhead_ms: Option<f64>,
    #[serde(default)]
    adapter_fallback_rate: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct SpatialExperimentRunSummary {
    schema_version: String,
    generated_at: String,
    run_id: String,
    space_id: String,
    mode: String,
    surface_variant: String,
    host: String,
    #[serde(default)]
    build_id: Option<String>,
    metrics: SpatialExperimentMetrics,
    improvement_score: f64,
    recommendation: String,
    complexity_delta: SpatialExperimentComplexityDelta,
    #[serde(default)]
    verdict_rationale: Option<String>,
    event_count: u64,
    event_key: String,
}

const VIEWSPEC_PROPOSAL_INDEX_KEY: &str = "/cortex/ux/viewspecs/proposals/index.json";
const VIEWSPEC_ACTIVE_SCOPE_INDEX_KEY: &str = "/cortex/ux/viewspecs/active/index.json";
const VIEWSPEC_REPLAY_INDEX_KEY: &str = "/cortex/ux/viewspecs/replay/index.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalIndexEntry {
    proposal_id: String,
    scope_key: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecReplayIndexEntry {
    proposal_id: String,
    run_id: String,
    key: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalListQuery {
    #[serde(default)]
    scope_key: Option<String>,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalListResponse {
    schema_version: String,
    generated_at: String,
    proposals: Vec<ViewSpecProposalEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalReviewRequest {
    reviewed_by: String,
    summary: String,
    #[serde(default)]
    checks: Vec<String>,
    approved: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalDecisionRequest {
    decided_by: String,
    rationale: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalMergeRequest {
    merged_by: String,
    merged_with_proposal_id: String,
    rationale: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecProposalActionResponse {
    accepted: bool,
    proposal: ViewSpecProposalEnvelope,
    gate_level: String,
    gate_status: String,
    decision_gate_id: String,
    replay_contract_ref: String,
    source_of_truth: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    degraded_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ViewSpecActiveQuery {
    #[serde(default)]
    scope_key: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecActiveResponse {
    schema_version: String,
    generated_at: String,
    active: Vec<ViewSpecScopeAdoptionRecord>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecReplayArtifact {
    schema_version: String,
    run_id: String,
    proposal_id: String,
    scope_key: String,
    generated_at: String,
    proposal: ViewSpecProposalEnvelope,
    #[serde(default)]
    lineage: Value,
    #[serde(default)]
    gate_metadata: Value,
    signal_count: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecDigestArtifact {
    schema_version: String,
    proposal_id: String,
    digest: String,
    generated_at: String,
    scope_key: String,
    status: String,
    #[serde(default)]
    payload: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecReplayResponse {
    schema_version: String,
    generated_at: String,
    replay: ViewSpecReplayArtifact,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ViewSpecDigestResponse {
    schema_version: String,
    generated_at: String,
    digest: ViewSpecDigestArtifact,
}

#[derive(Debug, Clone, PartialEq)]
struct ViewSpecGovernanceDecisionGate {
    gate_level: String,
    gate_status: String,
    decision_gate_id: String,
    replay_contract_ref: String,
    source_of_truth: String,
    degraded_reason: Option<String>,
    actor_principal: String,
    actor_role: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactDocumentV2 {
    artifact_id: String,
    title: String,
    markdown_source: String,
    rich_content: ArtifactRichContentProjection,
    content_hash: String,
    status: String,
    created_at: String,
    updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    published_at: Option<String>,
    head_revision_id: String,
    version: u64,
    route_id: String,
    owner_role: String,
    source_of_truth: String,
    fallback_active: bool,
    /// Optional: A2UI Heap Block metadata (Initiative 124 Phase 3)
    #[serde(skip_serializing_if = "Option::is_none", default)]
    agui_initial_ui_json: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    agui_tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    agui_mentions: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_workspace_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_block_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_emitted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_file_keys: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_mirror_mentions_to_relations: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_relation_map_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    heap_files_key_format: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapProjectionRecord {
    projection: HeapBlockProjection,
    surface_json: Value,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pinned_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    deleted_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct EmitHeapBlockResponse {
    schema_version: String,
    accepted: bool,
    artifact_id: String,
    block_id: String,
    op_id: String,
    idempotent: bool,
    #[serde(default)]
    warnings: Vec<String>,
    projection: HeapBlockProjection,
    source_of_truth: String,
    fallback_active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct HeapBlocksQuery {
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    tag: Option<String>,
    #[serde(default)]
    mention: Option<String>,
    #[serde(default)]
    page_link: Option<String>,
    #[serde(default)]
    attribute: Option<String>,
    #[serde(default)]
    block_type: Option<String>,
    #[serde(default)]
    has_files: Option<bool>,
    #[serde(default)]
    from_ts: Option<String>,
    #[serde(default)]
    changed_since: Option<String>,
    #[serde(default)]
    to_ts: Option<String>,
    #[serde(default)]
    limit: Option<usize>,
    #[serde(default)]
    cursor: Option<String>,
    #[serde(default)]
    include_deleted: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapBlockListItem {
    projection: HeapBlockProjection,
    surface_json: Value,
    #[serde(default)]
    warnings: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pinned_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    deleted_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapBlocksResponse {
    schema_version: String,
    generated_at: String,
    count: usize,
    has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
    items: Vec<HeapBlockListItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapDeletedListItem {
    artifact_id: String,
    deleted_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapChangedBlocksResponse {
    schema_version: String,
    generated_at: String,
    count: usize,
    has_more: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    next_cursor: Option<String>,
    changed: Vec<HeapBlockListItem>,
    deleted: Vec<HeapDeletedListItem>,
}

#[derive(Default)]
struct HeapGatewayUsageMetrics {
    blocks_changed_since_alias_hits: AtomicU64,
    blocks_page_link_filter_hits: AtomicU64,
    changed_blocks_endpoint_hits: AtomicU64,
    changed_blocks_changed_since_alias_hits: AtomicU64,
    changed_blocks_page_link_filter_hits: AtomicU64,
}

static HEAP_GATEWAY_USAGE_METRICS: LazyLock<HeapGatewayUsageMetrics> =
    LazyLock::new(HeapGatewayUsageMetrics::default);

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct HeapGatewayUsageSnapshot {
    blocks_changed_since_alias_hits: u64,
    blocks_page_link_filter_hits: u64,
    changed_blocks_endpoint_hits: u64,
    changed_blocks_changed_since_alias_hits: u64,
    changed_blocks_page_link_filter_hits: u64,
}

fn record_heap_blocks_changed_since_alias_usage() {
    HEAP_GATEWAY_USAGE_METRICS
        .blocks_changed_since_alias_hits
        .fetch_add(1, Ordering::Relaxed);
}

fn record_heap_blocks_page_link_filter_usage() {
    HEAP_GATEWAY_USAGE_METRICS
        .blocks_page_link_filter_hits
        .fetch_add(1, Ordering::Relaxed);
}

fn record_heap_changed_blocks_endpoint_usage() {
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_endpoint_hits
        .fetch_add(1, Ordering::Relaxed);
}

fn record_heap_changed_blocks_changed_since_alias_usage() {
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_changed_since_alias_hits
        .fetch_add(1, Ordering::Relaxed);
}

fn record_heap_changed_blocks_page_link_filter_usage() {
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_page_link_filter_hits
        .fetch_add(1, Ordering::Relaxed);
}

#[cfg(test)]
fn heap_gateway_usage_snapshot() -> HeapGatewayUsageSnapshot {
    HeapGatewayUsageSnapshot {
        blocks_changed_since_alias_hits: HEAP_GATEWAY_USAGE_METRICS
            .blocks_changed_since_alias_hits
            .load(Ordering::Relaxed),
        blocks_page_link_filter_hits: HEAP_GATEWAY_USAGE_METRICS
            .blocks_page_link_filter_hits
            .load(Ordering::Relaxed),
        changed_blocks_endpoint_hits: HEAP_GATEWAY_USAGE_METRICS
            .changed_blocks_endpoint_hits
            .load(Ordering::Relaxed),
        changed_blocks_changed_since_alias_hits: HEAP_GATEWAY_USAGE_METRICS
            .changed_blocks_changed_since_alias_hits
            .load(Ordering::Relaxed),
        changed_blocks_page_link_filter_hits: HEAP_GATEWAY_USAGE_METRICS
            .changed_blocks_page_link_filter_hits
            .load(Ordering::Relaxed),
    }
}

#[cfg(test)]
fn reset_heap_gateway_usage_metrics() {
    HEAP_GATEWAY_USAGE_METRICS
        .blocks_changed_since_alias_hits
        .store(0, Ordering::Relaxed);
    HEAP_GATEWAY_USAGE_METRICS
        .blocks_page_link_filter_hits
        .store(0, Ordering::Relaxed);
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_endpoint_hits
        .store(0, Ordering::Relaxed);
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_changed_since_alias_hits
        .store(0, Ordering::Relaxed);
    HEAP_GATEWAY_USAGE_METRICS
        .changed_blocks_page_link_filter_hits
        .store(0, Ordering::Relaxed);
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapBlockActionResponse {
    accepted: bool,
    artifact_id: String,
    action: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct HeapEmitRejectionEvent {
    timestamp: String,
    actor_id: String,
    actor_role: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    request_id: Option<String>,
    code: String,
    message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCreateRequest {
    #[serde(default)]
    artifact_id: Option<String>,
    title: String,
    #[serde(default)]
    content: Option<String>,
    #[serde(default)]
    markdown_source: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactSaveRequest {
    lease_id: String,
    expected_revision_id: String,
    markdown_source: String,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    notes: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactCheckoutRequest {
    #[serde(default)]
    lease_ttl_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactLeaseRenewRequest {
    lease_id: String,
    #[serde(default)]
    lease_ttl_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactLeaseReleaseRequest {
    lease_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactPublishRequest {
    #[serde(default)]
    lease_id: Option<String>,
    #[serde(default)]
    expected_revision_id: Option<String>,
    #[serde(default)]
    notes: Option<String>,
    #[serde(default)]
    governance: Option<ArtifactGovernanceEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactRichContentProjection {
    hash: String,
    block_count: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactRevision {
    artifact_id: String,
    revision_id: String,
    revision_number: u64,
    markdown_source: String,
    content_hash: String,
    created_at: String,
    created_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    parent_revision_id: Option<String>,
    published: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactLease {
    artifact_id: String,
    lease_id: String,
    holder_id: String,
    holder_role: String,
    acquired_at: String,
    renewed_at: String,
    expires_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactRevisionListResponse {
    artifact_id: String,
    head_revision_id: String,
    revisions: Vec<ArtifactRevision>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabSessionOpenRequest {
    #[serde(default)]
    lease_ttl_secs: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabSessionCloseRequest {
    session_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabOpRequest {
    session_id: String,
    expected_head_revision_id: String,
    #[serde(default)]
    proposed_base_revision_id: Option<String>,
    op_type: String,
    payload_markdown: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabSession {
    artifact_id: String,
    session_id: String,
    lease_id: String,
    holder_id: String,
    holder_role: String,
    base_revision_id: String,
    opened_at: String,
    expires_at: String,
    #[serde(default)]
    last_sequence: u64,
    active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabOp {
    artifact_id: String,
    session_id: String,
    op_id: String,
    sequence: u64,
    op_type: String,
    actor_id: String,
    proposed_base_revision_id: String,
    expected_head_revision_id: String,
    applied_head_revision_id: String,
    created_at: String,
    merge_status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactMergeResult {
    artifact_id: String,
    session_id: String,
    merge_status: String,
    head_revision_id: String,
    merged_markdown: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    conflict_summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabCursor {
    line: u64,
    column: u64,
    #[serde(default)]
    selection_start: Option<u64>,
    #[serde(default)]
    selection_end: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabPresence {
    artifact_id: String,
    session_id: String,
    actor_id: String,
    actor_role: String,
    last_seen_at: String,
    expires_at: String,
    #[serde(default)]
    cursor: Option<ArtifactCollabCursor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabBatchOperation {
    op_id: String,
    lamport: u64,
    markdown_source: String,
    #[serde(default)]
    stream_channel: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabOpBatchRequest {
    session_id: String,
    batch_sequence: u64,
    #[serde(default)]
    expected_head_revision_id: Option<String>,
    operations: Vec<ArtifactCollabBatchOperation>,
    #[serde(default)]
    cursor: Option<ArtifactCollabCursor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabOpsQuery {
    #[serde(default)]
    since_sequence: Option<u64>,
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCrdtStateResponse {
    schema_version: String,
    artifact_id: String,
    head_revision_id: String,
    materialized_markdown: String,
    op_count: u64,
    source_of_truth: String,
    fallback_active: bool,
    sessions: Vec<ArtifactCollabSession>,
    presence: Vec<ArtifactCollabPresence>,
    conflicts: Vec<ArtifactCrdtConflict>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabCheckpointRequest {
    #[serde(default)]
    max_ops_after_compaction: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactCollabForceResolveRequest {
    session_id: String,
    markdown_source: String,
    approved_by: String,
    rationale: String,
    approved_at: String,
    #[serde(default)]
    governance: Option<ArtifactGovernanceEnvelope>,
    #[serde(default)]
    cursor: Option<ArtifactCollabCursor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtifactPrivilegeDecisionProof {
    decision_id: String,
    signature: String,
    signer: String,
    #[serde(default)]
    algorithm: Option<String>,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtifactGovernanceEnvelope {
    approved_by: String,
    rationale: String,
    approved_at: String,
    actor_id: String,
    decision_proof: ArtifactPrivilegeDecisionProof,
    #[serde(default)]
    nonce: Option<String>,
    #[serde(default)]
    expires_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactRealtimeConnectRequest {
    #[serde(default)]
    actor_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactRealtimeDisconnectRequest {
    #[serde(default)]
    actor_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactRealtimeBacklogQuery {
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct ArtifactRealtimeAckResetRequest {
    #[serde(default)]
    governance: Option<ArtifactGovernanceEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
struct ArtifactRealtimeSubscribe {
    action: String,
    #[serde(default)]
    actor_id: Option<String>,
    #[serde(default)]
    artifact_id: Option<String>,
    #[serde(default)]
    nonce: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexSourceState {
    schema_version: String,
    generated_at: String,
    source_of_truth: String,
    fallback_active: bool,
    primary_available: bool,
    local_mirror_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexDriftReport {
    schema_version: String,
    generated_at: String,
    source_of_truth: String,
    drift_detected: bool,
    route_diff: Vec<String>,
    capability_diff: Vec<String>,
    pattern_diff: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct UxLifecycleTransitionEvent {
    event_id: String,
    queue_id: String,
    route_id: String,
    view_id: String,
    from_status: String,
    to_status: String,
    actor_id: String,
    transitioned_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct UxRemeasurementRecord {
    queue_id: String,
    route_id: String,
    view_id: String,
    candidate_id: String,
    baseline_metric_date: String,
    post_release_metric_date: String,
    remeasured_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexPromoteCandidateRequest {
    queue_id: String,
    candidate_id: String,
    route_id: String,
    view_capability_id: String,
    approved_by: String,
    rationale: String,
    approved_at: String,
    baseline_metric_date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexMarkShippedRequest {
    queue_id: String,
    candidate_id: String,
    route_id: String,
    view_capability_id: String,
    shipped_at: String,
    post_release_metric_date: String,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexMarkRemeasuredRequest {
    queue_id: String,
    candidate_id: String,
    route_id: String,
    view_capability_id: String,
    remeasured_at: String,
    post_release_metric_date: String,
    #[serde(default)]
    summary: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackOverdueQuery {
    #[serde(default)]
    days: Option<i64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct CortexFeedbackOverdueResponse {
    schema_version: String,
    generated_at: String,
    threshold_days: i64,
    items: Vec<UxFeedbackQueueItem>,
}

#[derive(Serialize)]
struct CanisterInfo {
    name: String,
    id: String,
    status: String,
}

#[derive(Deserialize)]
struct WorkflowReadRequest {
    path: String,
}

#[derive(Deserialize)]
struct WorkflowSaveRequest {
    path: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkflowCatalogEntry {
    name: String,
    path: String,
    source: String,
    status: String,
    description: Option<String>,
    launch_template: Option<String>,
    read_only: bool,
    automation: Option<WorkflowAutomationDescriptor>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkflowAutomationDescriptor {
    automation_key: String,
    enabled: bool,
    paused: bool,
    interval_secs: u64,
    active_workflow_id: Option<String>,
    last_workflow_id: Option<String>,
    last_run_at: Option<String>,
    last_status: Option<String>,
    can_run_now: bool,
    can_pause: bool,
    can_resume: bool,
    pause_reason: Option<String>,
}

#[derive(Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct WorkerAcpAutomationStatus {
    #[serde(default)]
    automation_key: Option<String>,
    enabled: bool,
    paused: bool,
    #[serde(default)]
    interval_secs: Option<u64>,
    active_workflow_id: Option<String>,
    #[serde(default)]
    last_workflow_id: Option<String>,
    #[serde(default)]
    last_run_at: Option<String>,
    #[serde(default)]
    last_status: Option<String>,
}

const TESTING_SCHEMA_VERSION: &str = "1.0.0";
const TESTING_STALE_AFTER_SECS: u64 = 24 * 60 * 60;
const SIQ_SCHEMA_VERSION: &str = "1.0.0";
const SIQ_STALE_AFTER_SECS: u64 = 24 * 60 * 60;
const MOTOKO_GRAPH_SCHEMA_VERSION: &str = "1.0.0";
const MOTOKO_GRAPH_STALE_AFTER_SECS: u64 = 24 * 60 * 60;
const CORTEX_CLOSEOUT_TRACKING_SCHEMA_VERSION: &str = "1.0.0";
const CORTEX_CLOSEOUT_DEFAULT_INITIATIVE: &str = "116-cortex-realtime-ga-trust-hardening";
const EMBEDDED_BRAND_POLICY_JSON: &str =
    include_str!("../../../../../shared/standards/branding/brand_policy_v1.json");
const DEFAULT_BRAND_POLICY_CACHE_FILE: &str = "brand_policy_registry_v1.json";

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestCatalogEntry {
    test_id: String,
    name: String,
    layer: String,
    stack: String,
    owner: String,
    path: String,
    command: String,
    artifacts: Vec<String>,
    gate_level: String,
    destructive: bool,
    tags: Vec<String>,
    last_seen_commit: String,
    updated_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestCatalogArtifact {
    schema_version: String,
    generated_at: String,
    tests: Vec<TestCatalogEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestRunResult {
    test_id: String,
    status: String,
    duration_ms: u64,
    error_summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestRunArtifact {
    schema_version: String,
    run_id: String,
    started_at: String,
    finished_at: String,
    agent_id: String,
    environment: String,
    git_commit: String,
    results: Vec<TestRunResult>,
    artifacts: Vec<String>,
    warnings: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestGateFailure {
    code: String,
    message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestGateCounts {
    pass: u64,
    fail: u64,
    warn: u64,
    pending: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct TestGateSummaryArtifact {
    schema_version: String,
    generated_at: String,
    mode: String,
    catalog_valid: bool,
    run_artifacts_valid: bool,
    required_blockers_pass: bool,
    overall_verdict: String,
    latest_run_id: Option<String>,
    failures: Vec<TestGateFailure>,
    counts: TestGateCounts,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TestGateLatestResponse {
    summary: TestGateSummaryArtifact,
    surface: Option<Value>,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
struct TestingRunsQuery {
    limit: Option<usize>,
    status: Option<String>,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct TestingHealthResponse {
    status: String,
    testing_log_dir: String,
    schema_version: String,
    catalog_exists: bool,
    gate_exists: bool,
    runs_count: usize,
    catalog_last_modified: Option<u64>,
    gate_last_modified: Option<u64>,
    latest_run_last_modified: Option<u64>,
    catalog_fresh: bool,
    gate_fresh: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
struct SiqRunsQuery {
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphStatus {
    gate_result: String,
    posture: String,
    authority_mode: String,
    runtime_dependency_promotion: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphWorkload {
    path: String,
    workload: i64,
    edge_workload: i64,
    seconds_per_edge: f64,
    cycles_per_edge: f64,
    memory_per_edge_bytes: f64,
    walk_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphStability {
    path: String,
    steps_total: i64,
    steps_pass: i64,
    steps_blocked: i64,
    first_attempt_pass: i64,
    retries_consumed: i64,
    port_conflicts: i64,
    reliability_percent: f64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphWorkflowStage {
    stage: String,
    status: String,
    evidence: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphEvidence {
    gate_file: String,
    m4_metrics_file: String,
    m8_metrics_file: String,
    stability_file: String,
    analysis_file: String,
    m8_pass_count: i64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphSnapshot {
    schema_version: String,
    generated_at: String,
    contribution_id: String,
    status: MotokoGraphStatus,
    workloads: Vec<MotokoGraphWorkload>,
    stability: Vec<MotokoGraphStability>,
    workflow_stages: Vec<MotokoGraphWorkflowStage>,
    evidence: MotokoGraphEvidence,
    history_event_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphDecisionEvent {
    schema_version: String,
    decision_event_id: String,
    captured_at: String,
    contribution: String,
    decision_date: String,
    selected_option: String,
    rationale: String,
    posture_before: String,
    posture_after: String,
    authority_mode: String,
    evidence_refs: Vec<String>,
    steward: String,
    owner: String,
    follow_up_actions: Vec<String>,
    source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    applied_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringCheck {
    name: String,
    required: bool,
    status: String,
    details: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringRun {
    schema_version: String,
    run_id: String,
    started_at: String,
    finished_at: String,
    gateway_base: String,
    overall_status: String,
    required_failures: u64,
    warnings: u64,
    checks: Vec<MotokoGraphMonitoringCheck>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringWindowSummary {
    total_runs: u64,
    pass_runs: u64,
    warn_runs: u64,
    fail_runs: u64,
    reliability_percent: f64,
    warning_rate_percent: f64,
    required_failure_rate_percent: f64,
    gateway_warning_rate_percent: f64,
    mean_duration_seconds: f64,
    p95_duration_seconds: f64,
    last_success_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringWindows {
    #[serde(rename = "7d")]
    days_7: MotokoGraphMonitoringWindowSummary,
    #[serde(rename = "30d")]
    days_30: MotokoGraphMonitoringWindowSummary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringTrend {
    schema_version: String,
    generated_at: String,
    windows: MotokoGraphMonitoringWindows,
    latest: MotokoGraphMonitoringLatest,
    last_applied_decision_event_id: Option<String>,
    next_action: String,
    advisory_recommendation: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct MotokoGraphMonitoringLatest {
    run_id: String,
    overall_status: String,
    required_failures: u64,
    warnings: u64,
    duration_seconds: f64,
    started_at: String,
    finished_at: String,
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
struct DecisionCaptureRequest {
    schema_version: String,
    contribution: String,
    decision_date: String,
    selected_option: String,
    rationale: String,
    posture_before: String,
    posture_after: String,
    authority_mode: String,
    evidence_refs: Vec<String>,
    steward: String,
    owner: String,
    follow_up_actions: Vec<String>,
    source: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionCaptureResponse {
    decision_event_id: String,
    status: String,
    path: String,
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct MotokoGraphHealthResponse {
    status: String,
    schema_version: String,
    kg_log_dir: String,
    snapshot_exists: bool,
    history_count: usize,
    pending_count: usize,
    snapshot_last_modified: Option<u64>,
    snapshot_fresh: bool,
}

#[derive(Deserialize, Debug, Clone, PartialEq, Default)]
struct MotokoGraphMonitoringRunsQuery {
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(rename_all = "camelCase")]
struct ErrorResponse {
    error: String,
    error_code: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubApprovalEnvelope {
    approved_by: String,
    rationale: String,
    approved_at: String,
    decision_ref: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubPipelineRunRequest {
    mode: String,
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    scenario_template_id: Option<String>,
    #[serde(default)]
    publish_version: Option<String>,
    #[serde(default)]
    from_version: Option<String>,
    #[serde(default)]
    to_version: Option<String>,
    #[serde(default)]
    approval: Option<DpubApprovalEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubPipelineQueryRequest {
    kind: String,
    id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubRunHistoryQuery {
    #[serde(default)]
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubEditionDiffQuery {
    from: String,
    to: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubBlastRadiusQuery {
    #[serde(default)]
    contribution_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubStewardPacketExportRequest {
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    from_version: Option<String>,
    #[serde(default)]
    to_version: Option<String>,
    #[serde(default)]
    approval: Option<DpubApprovalEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubPhaseResult {
    phase: String,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    message: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubPipelineRunReport {
    run_id: String,
    mode: String,
    status: String,
    started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    finished_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash_after: Option<String>,
    #[serde(default)]
    phase_results: Vec<DpubPhaseResult>,
    #[serde(default)]
    artifacts: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubRunRecord {
    schema_version: String,
    run_id: String,
    mode: String,
    actor_role: String,
    actor_id: String,
    started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    finished_at: Option<String>,
    status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash_before: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash_after: Option<String>,
    #[serde(default)]
    phase_results: Vec<DpubPhaseResult>,
    #[serde(default)]
    artifacts: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    approval: Option<DpubApprovalEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubRunHistoryItem {
    run_id: String,
    mode: String,
    actor_role: String,
    status: String,
    started_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    duration_ms: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash_after: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubSimulationArtifact {
    file_name: String,
    bytes: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    modified_at: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubEditionEntry {
    version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    graph_root_hash: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubWorkbenchOverview {
    health: Value,
    latest_graph_metrics: Value,
    latest_path_summary: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    latest_run_summary: Option<DpubRunHistoryItem>,
    #[serde(skip_serializing_if = "Option::is_none")]
    siq_run_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    siq_graph_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    siq_overall_verdict: Option<String>,
    artifact_paths: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubBlastRadiusResponse {
    contribution_id: String,
    depends_on: Vec<String>,
    depended_by: Vec<String>,
    invalidates: Vec<String>,
    invalidated_by: Vec<String>,
    supersedes: Vec<String>,
    superseded_by: Vec<String>,
    references: Vec<String>,
    referenced_by: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubStewardPacketExportResponse {
    packet_path: String,
    goal: String,
    from_version: String,
    to_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubLensSummaryCategory {
    id: String,
    label: String,
    total: usize,
    active: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubLensSummaryLens {
    id: String,
    category: String,
    label: String,
    count: usize,
    default_on: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubLensSummaryResponse {
    graph_root_hash: String,
    categories: Vec<DpubLensSummaryCategory>,
    lenses: Vec<DpubLensSummaryLens>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubEditionTrendQuery {
    #[serde(default)]
    goal: Option<String>,
    #[serde(default)]
    window: Option<usize>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubEditionTrendPoint {
    version: String,
    risk_score: usize,
    critical: usize,
    violation: usize,
    warning: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    recommended_path: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubEditionTrendResponse {
    goal: String,
    points: Vec<DpubEditionTrendPoint>,
    recommendation_changes: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubLensEvaluateRequest {
    #[serde(default)]
    active_lenses: Vec<String>,
    #[serde(default)]
    goal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
#[serde(rename_all = "camelCase")]
struct DpubLensOverlayResponse {
    graph_root_hash: String,
    lens_state: Value,
    node_flags: Value,
    edge_flags: Value,
    counts: Value,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionSurfaceEnvelope {
    surface_id: String,
    workflow_id: String,
    mutation_id: String,
    status: String,
    required_actions: Vec<String>,
    evidence_refs: Vec<String>,
    last_updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    source_of_truth: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    lineage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    auth_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    payload: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionActionRequest {
    #[serde(default)]
    space_id: Option<String>,
    #[serde(default)]
    decision_gate_id: Option<String>,
    #[serde(default)]
    workflow_id: Option<String>,
    #[serde(default)]
    mutation_id: Option<String>,
    #[serde(default)]
    action_target: Option<String>,
    #[serde(default)]
    domain_mode: Option<String>,
    #[serde(default)]
    gate_level: Option<String>,
    #[serde(default)]
    actor_ref: Option<String>,
    #[serde(default)]
    risk_statement: Option<String>,
    #[serde(default)]
    rollback_path: Option<String>,
    #[serde(default)]
    evidence_refs: Vec<String>,
    #[serde(default)]
    note: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionActionRecord {
    schema_version: String,
    action_id: String,
    action: String,
    decision_gate_id: String,
    workflow_id: String,
    mutation_id: String,
    action_target: String,
    risk_statement: String,
    rollback_path: String,
    evidence_refs: Vec<String>,
    lineage_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    policy_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    actor_ref: Option<String>,
    note: Option<String>,
    created_at: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionPlaneResponse {
    space_id: String,
    surfaces: Vec<DecisionSurfaceEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    digest: Option<DecisionSurfaceEnvelope>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DecisionTelemetrySnapshot {
    schema_version: String,
    updated_at: String,
    decision_gate_samples: u64,
    latency_ms_p95: Option<u64>,
    gate_status_counts: BTreeMap<String, u64>,
    source_of_truth_counts: BTreeMap<String, u64>,
    degraded_reason_counts: BTreeMap<String, u64>,
    fallback_usage_total: u64,
    cache_usage_total: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    scope_space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_decision_gate_samples: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_fallback_usage_total: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    global_cache_usage_total: Option<u64>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DecisionTelemetryScopeState {
    decision_gate_samples: u64,
    #[serde(default)]
    latency_ms_samples: Vec<u64>,
    #[serde(default)]
    gate_status_counts: BTreeMap<String, u64>,
    #[serde(default)]
    source_of_truth_counts: BTreeMap<String, u64>,
    #[serde(default)]
    degraded_reason_counts: BTreeMap<String, u64>,
    fallback_usage_total: u64,
    cache_usage_total: u64,
}

impl Default for DecisionTelemetryScopeState {
    fn default() -> Self {
        Self {
            decision_gate_samples: 0,
            latency_ms_samples: Vec::new(),
            gate_status_counts: BTreeMap::new(),
            source_of_truth_counts: BTreeMap::new(),
            degraded_reason_counts: BTreeMap::new(),
            fallback_usage_total: 0,
            cache_usage_total: 0,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
struct DecisionTelemetryState {
    schema_version: String,
    updated_at: String,
    #[serde(flatten)]
    global: DecisionTelemetryScopeState,
    #[serde(default)]
    by_space: BTreeMap<String, DecisionTelemetryScopeState>,
}

impl Default for DecisionTelemetryState {
    fn default() -> Self {
        Self {
            schema_version: "1.0.0".to_string(),
            updated_at: now_iso(),
            global: DecisionTelemetryScopeState::default(),
            by_space: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone)]
struct VerifiedDecisionActor {
    principal: String,
    role: String,
    signed: bool,
    auth_status: String,
    auth_reason: Option<String>,
}

impl GatewayService {
    pub async fn start(port: u16) {
        let state = GatewayState::new();
        let app = Router::new()
            .route("/ws", get(ws_handler))
            .route("/ws/cortex/collab", get(ws_collab_handler))
            .route("/api/system/status", get(get_system_status))
            .route("/api/system/ready", get(get_system_ready))
            .route("/api/system/build", get(get_system_build))
            .route("/api/system/brand-policy", get(get_system_brand_policy))
            .route(
                "/api/system/capability-graph",
                get(get_system_capability_graph),
            )
            .route(
                "/api/system/capability-catalog",
                get(get_system_capability_catalog),
            )
            .route(
                "/api/system/ux/workbench",
                get(crate::services::workbench_ux::get_workbench_ux_viewspec),
            )
            .route("/api/spaces/create", post(post_create_space))
            .route(
                "/api/spaces/:space_id/capability-graph",
                get(get_space_capability_graph).put(put_space_capability_graph),
            )
            .route(
                "/api/spaces/:space_id/navigation-plan",
                get(get_space_navigation_plan),
            )
            .route(
                "/api/system/local-gateway/queue",
                get(get_local_gateway_queue),
            )
            .route(
                "/api/system/local-gateway/queue/export",
                get(get_local_gateway_queue_export),
            )
            .route(
                "/api/system/local-gateway/queue/:mutation_id/retry",
                post(post_local_gateway_queue_retry),
            )
            .route(
                "/api/system/local-gateway/queue/:mutation_id/discard",
                post(post_local_gateway_queue_discard),
            )
            .route(
                "/api/system/local-gateway/queue/:mutation_id/fork",
                post(post_local_gateway_queue_fork),
            )
            .route(
                "/api/system/execution-profile/:space_id",
                get(get_system_execution_profile),
            )
            .route(
                "/api/system/attribution-domains/:space_id",
                get(get_system_attribution_domains),
            )
            .route(
                "/api/system/governance-scope/:space_id",
                get(get_system_governance_scope),
            )
            .route(
                "/api/system/replay-contract/:mutation_id",
                get(get_system_replay_contract),
            )
            .route(
                "/api/system/decision-gates/latest",
                get(get_system_decision_gates_latest),
            )
            .route(
                "/api/system/decision-plane/:space_id",
                get(get_system_decision_plane),
            )
            .route(
                "/api/system/mutation-gates/:space_id/:mutation_id",
                get(get_system_mutation_gates),
            )
            .route("/api/system/decision/ack", post(post_system_decision_ack))
            .route(
                "/api/system/decision/escalate",
                post(post_system_decision_escalate),
            )
            .route(
                "/api/system/decision/telemetry",
                get(get_system_decision_telemetry),
            )
            .route(
                "/api/system/decision-telemetry/:space_id",
                get(get_system_decision_telemetry_by_space),
            )
            .route("/api/cortex/layout/spec", get(get_cortex_layout_spec))
            .route(
                "/api/cortex/preferences/theme-policy",
                get(get_cortex_theme_policy),
            )
            .route(
                "/api/cortex/preferences/theme-policy",
                put(put_cortex_theme_policy),
            )
            .route(
                "/api/cortex/layout/source-state",
                get(get_cortex_layout_source_state),
            )
            .route(
                "/api/cortex/layout/drift-report",
                get(get_cortex_layout_drift_report),
            )
            .route(
                "/api/cortex/runtime/sync-status",
                get(get_cortex_runtime_sync_status),
            )
            .route(
                "/api/cortex/runtime/sync/replay",
                post(post_cortex_runtime_sync_replay),
            )
            .route(
                "/api/cortex/runtime/slo/status",
                get(get_cortex_runtime_slo_status),
            )
            .route(
                "/api/cortex/runtime/slo/breaches",
                get(get_cortex_runtime_slo_breaches),
            )
            .route(
                "/api/cortex/runtime/closeout/tasks",
                get(get_cortex_runtime_closeout_tasks),
            )
            .route(
                "/api/cortex/layout/evaluate",
                post(post_cortex_layout_evaluate),
            )
            .route("/api/cortex/layout/spec", post(post_cortex_layout_spec))
            .route(
                "/api/cortex/viewspecs/candidates",
                post(post_cortex_viewspec_candidates),
            )
            .route(
                "/api/cortex/viewspecs/candidates/:candidate_set_id",
                get(get_cortex_viewspec_candidate_set),
            )
            .route(
                "/api/cortex/viewspecs/candidates/:candidate_set_id/stage",
                post(post_cortex_viewspec_candidate_stage),
            )
            .route(
                "/api/cortex/viewspecs/learning/signals",
                post(post_cortex_viewspec_learning_signals),
            )
            .route(
                "/api/cortex/viewspecs/learning/profiles/:space_id",
                get(get_cortex_viewspec_learning_profile),
            )
            .route(
                "/api/cortex/viewspecs/learning/profiles/:space_id/recompute",
                post(post_cortex_viewspec_learning_profile_recompute),
            )
            .route(
                "/api/cortex/viewspecs/learning/profiles/:space_id/reset",
                post(post_cortex_viewspec_learning_profile_reset),
            )
            .route(
                "/api/cortex/viewspecs/experiments/spatial/events",
                post(post_cortex_viewspec_spatial_experiment_event),
            )
            .route(
                "/api/cortex/viewspecs/experiments/spatial/runs/:run_id",
                get(get_cortex_viewspec_spatial_experiment_run),
            )
            .route(
                "/api/cortex/viewspecs/validate",
                post(post_cortex_viewspec_validate),
            )
            .route(
                "/api/cortex/viewspecs/compile",
                post(post_cortex_viewspec_compile),
            )
            .route(
                "/api/cortex/viewspecs/lock",
                post(post_cortex_viewspec_lock),
            )
            .route(
                "/api/cortex/viewspecs/proposals",
                get(get_cortex_viewspec_proposals),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id",
                get(get_cortex_viewspec_proposal),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/review",
                post(post_cortex_viewspec_proposal_review),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/ratify",
                post(post_cortex_viewspec_proposal_ratify),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/reject",
                post(post_cortex_viewspec_proposal_reject),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/merge",
                post(post_cortex_viewspec_proposal_merge),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/replay",
                get(get_cortex_viewspec_proposal_replay),
            )
            .route(
                "/api/cortex/viewspecs/proposals/:proposal_id/digest",
                get(get_cortex_viewspec_proposal_digest),
            )
            .route(
                "/api/cortex/viewspecs/active",
                get(get_cortex_viewspec_active),
            )
            .route(
                "/api/cortex/viewspecs/:view_spec_id",
                get(get_cortex_viewspec),
            )
            .route(
                "/api/cortex/viewspecs/:view_spec_id/fork",
                post(post_cortex_viewspec_fork),
            )
            .route(
                "/api/cortex/viewspecs/:view_spec_id/propose",
                post(post_cortex_viewspec_propose),
            )
            .route(
                "/api/cortex/viewspecs/:view_spec_id/confidence/recompute",
                post(post_cortex_viewspec_confidence_recompute),
            )
            .route("/api/cortex/feedback/ux", post(post_cortex_feedback_ux))
            .route("/api/cortex/feedback/ux", get(get_cortex_feedback_ux))
            .route(
                "/api/cortex/feedback/triage",
                post(post_cortex_feedback_triage),
            )
            .route(
                "/api/cortex/feedback/ux/promote-candidate",
                post(post_cortex_feedback_promote_candidate),
            )
            .route(
                "/api/cortex/feedback/ux/mark-shipped",
                post(post_cortex_feedback_mark_shipped),
            )
            .route(
                "/api/cortex/feedback/ux/mark-remeasured",
                post(post_cortex_feedback_mark_remeasured),
            )
            .route(
                "/api/cortex/feedback/ux/overdue",
                get(get_cortex_feedback_overdue),
            )
            .route(
                "/api/cortex/feedback/promotions/approve",
                post(post_cortex_promotion_approve),
            )
            .route(
                "/api/cortex/feedback/promotions/reject",
                post(post_cortex_promotion_reject),
            )
            .route(
                "/api/cortex/feedback/promotions/history",
                get(get_cortex_promotion_history),
            )
            .route(
                "/api/cortex/views/capability-matrix",
                get(get_cortex_capability_matrix),
            )
            .route("/api/cortex/studio/heap/emit", post(post_cortex_heap_emit))
            .route(
                "/api/cortex/studio/heap/blocks",
                get(get_cortex_heap_blocks),
            )
            .route(
                "/api/cortex/studio/heap/changed_blocks",
                get(get_cortex_heap_changed_blocks),
            )
            .route(
                "/api/cortex/studio/heap/blocks/:artifact_id/pin",
                post(post_cortex_heap_block_pin),
            )
            .route(
                "/api/cortex/studio/heap/blocks/:artifact_id/delete",
                post(post_cortex_heap_block_delete),
            )
            .route(
                "/api/cortex/studio/heap/blocks/context",
                post(post_cortex_heap_blocks_context),
            )
            .route(
                "/api/cortex/studio/heap/blocks/:artifact_id/export",
                get(get_cortex_heap_block_export),
            )
            .route(
                "/api/cortex/studio/heap/blocks/:artifact_id/history",
                get(get_cortex_heap_block_history),
            )
            .route(
                "/api/cortex/studio/artifacts",
                post(post_cortex_artifact_create),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id",
                get(get_cortex_artifact),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/publish",
                post(post_cortex_artifact_publish),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/checkout",
                post(post_cortex_artifact_checkout),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/lease/renew",
                post(post_cortex_artifact_lease_renew),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/lease/release",
                post(post_cortex_artifact_lease_release),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/save",
                post(post_cortex_artifact_save),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/revisions",
                get(get_cortex_artifact_revisions),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/revisions/:revision_id",
                get(get_cortex_artifact_revision),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/session/open",
                post(post_cortex_artifact_collab_session_open),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/op",
                post(post_cortex_artifact_collab_op),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/session/close",
                post(post_cortex_artifact_collab_session_close),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/session",
                get(get_cortex_artifact_collab_session),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/state",
                get(get_cortex_artifact_collab_state),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/op/batch",
                post(post_cortex_artifact_collab_op_batch),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/ops",
                get(get_cortex_artifact_collab_ops),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/presence",
                get(get_cortex_artifact_collab_presence),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/checkpoint",
                post(post_cortex_artifact_collab_checkpoint),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/force-resolve",
                post(post_cortex_artifact_collab_force_resolve),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/status",
                get(get_cortex_artifact_collab_realtime_status),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/connect",
                post(post_cortex_artifact_collab_realtime_connect),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/disconnect",
                post(post_cortex_artifact_collab_realtime_disconnect),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/backlog",
                get(get_cortex_artifact_collab_realtime_backlog),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/integrity",
                get(get_cortex_artifact_collab_realtime_integrity),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/resync",
                post(post_cortex_artifact_collab_realtime_resync),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/ack",
                get(get_cortex_artifact_collab_realtime_ack),
            )
            .route(
                "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/ack/reset",
                post(post_cortex_artifact_collab_realtime_ack_reset),
            )
            .route("/api/canisters", get(list_canisters))
            .route("/api/ingest", post(ingest_document))
            .route("/api/workflows", get(list_workflows))
            .route("/api/workflows/catalog", get(list_workflow_catalog))
            .route("/api/workflow/read", post(read_workflow))
            .route("/api/workflow/save", post(save_workflow))
            .route("/api/workflow/run", post(run_workflow))
            .route("/api/acp/rpc", post(acp_rpc))
            .route("/api/acp/fs/read_text_file", post(acp_read_text_file))
            .route("/api/acp/fs/write_text_file", post(acp_write_text_file))
            .route("/api/acp/terminal/create", post(acp_terminal_create))
            .route("/api/acp/terminal/output", post(acp_terminal_output))
            .route(
                "/api/acp/terminal/wait_for_exit",
                post(acp_terminal_wait_for_exit),
            )
            .route("/api/acp/terminal/kill", post(acp_terminal_kill))
            .route("/api/acp/terminal/release", post(acp_terminal_release))
            .route("/api/search", post(search_vector))
            .route("/api/health", get(health_check))
            .route("/api/metrics/acp", get(get_acp_metrics))
            .route("/api/metrics/resilience", get(get_resilience_metrics))
            .route("/api/testing/catalog", get(get_testing_catalog))
            .route("/api/testing/runs", get(get_testing_runs))
            .route("/api/testing/runs/:run_id", get(get_testing_run))
            .route("/api/testing/gates/latest", get(get_testing_gates_latest))
            .route("/api/testing/health", get(get_testing_health))
            .route("/api/system/siq/coverage", get(get_siq_coverage))
            .route(
                "/api/system/siq/dependency-closure",
                get(get_siq_dependency_closure),
            )
            .route("/api/system/siq/gates/latest", get(get_siq_gates_latest))
            .route(
                "/api/system/siq/graph-projection",
                get(get_siq_graph_projection),
            )
            .route("/api/system/siq/runs", get(get_siq_runs))
            .route("/api/system/siq/runs/:run_id", get(get_siq_run))
            .route("/api/system/siq/health", get(get_siq_health))
            .route(
                "/api/kg/motoko-graph/snapshot",
                get(get_motoko_graph_snapshot),
            )
            .route(
                "/api/kg/motoko-graph/decision-history",
                get(get_motoko_graph_decision_history),
            )
            .route("/api/kg/motoko-graph/health", get(get_motoko_graph_health))
            .route(
                "/api/kg/motoko-graph/monitoring-trends",
                get(get_motoko_graph_monitoring_trends),
            )
            .route(
                "/api/kg/motoko-graph/monitoring-runs",
                get(get_motoko_graph_monitoring_runs),
            )
            .route(
                "/api/kg/motoko-graph/decision-capture",
                post(capture_motoko_graph_decision),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/overview",
                get(get_contribution_graph_overview),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/graph",
                get(get_contribution_graph_graph),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/path-assessment",
                get(get_contribution_graph_path_assessment),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/lens-summary",
                get(get_contribution_graph_lens_summary),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/edition-trends",
                get(get_contribution_graph_edition_trends),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/doctor",
                get(get_contribution_graph_doctor),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/simulations",
                get(get_contribution_graph_simulations),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/editions",
                get(get_contribution_graph_editions),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/edition-diff",
                get(get_contribution_graph_edition_diff),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/runs",
                get(get_contribution_graph_runs),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/runs/:run_id",
                get(get_contribution_graph_run),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/pipeline/run",
                post(post_contribution_graph_pipeline_run),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/pipeline/query",
                post(post_contribution_graph_pipeline_query),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/lens/evaluate",
                post(post_contribution_graph_lens_evaluate),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/violations/by-node",
                get(get_contribution_graph_violations_by_node),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/blast-radius",
                get(get_contribution_graph_blast_radius),
            )
            .route(
                "/api/kg/spaces/:space_id/contribution-graph/steward-packet/export",
                post(post_contribution_graph_steward_packet_export),
            )
            .route(
                "/api/kg/spaces/:space_id/agents/contributions",
                post(post_agent_contribution),
            )
            .route(
                "/api/kg/spaces/:space_id/agents/contributions/:run_id",
                get(get_agent_contribution_run),
            )
            .route(
                "/api/kg/spaces/:space_id/agents/contributions/:run_id/approval",
                post(post_agent_contribution_approval),
            )
            .with_state(state)
            .layer(middleware::from_fn(runtime_gateway_dispatch_middleware))
            .layer(tower_http::cors::CorsLayer::permissive());
        register_runtime_dispatch_router(app.clone());

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = match tokio::net::TcpListener::bind(addr).await {
            Ok(listener) => listener,
            Err(err) => {
                if err.kind() == std::io::ErrorKind::AddrInUse {
                    tracing::warn!(
                        "Gateway port {} already in use; skipping gateway start",
                        port
                    );
                } else {
                    tracing::error!("Failed to bind gateway on {}: {}", addr, err);
                }
                return;
            }
        };

        tracing::info!("Gateway listening on {}", addr);
        if let Err(err) = axum::serve(listener, app).await {
            tracing::error!("Gateway server stopped with error: {}", err);
        }
    }
}

fn runtime_dispatch_router_cell() -> &'static Mutex<Option<Router>> {
    static ROUTER: LazyLock<Mutex<Option<Router>>> = LazyLock::new(|| Mutex::new(None));
    &ROUTER
}

fn register_runtime_dispatch_router(router: Router) {
    if let Ok(mut guard) = runtime_dispatch_router_cell().lock() {
        *guard = Some(router);
    }
}

fn runtime_dispatch_router() -> Option<Router> {
    runtime_dispatch_router_cell()
        .lock()
        .ok()
        .and_then(|guard| guard.clone())
}

pub async fn dispatch_legacy_api_request_in_process(
    request: &cortex_runtime::gateway::types::GatewayRequestEnvelope,
) -> Result<cortex_runtime::gateway::types::GatewayResponseEnvelope, cortex_runtime::RuntimeError> {
    let Some(router) = runtime_dispatch_router() else {
        return Err(cortex_runtime::RuntimeError::Domain(
            "legacy in-process router unavailable".to_string(),
        ));
    };

    let mut uri = request.path.clone();
    if let Some(query) = request.query.as_ref() {
        if !query.trim().is_empty() {
            uri.push('?');
            uri.push_str(query);
        }
    }

    let mut builder = axum::http::Request::builder()
        .method(request.method.as_str())
        .uri(uri);
    for (key, value) in &request.headers {
        if key.eq_ignore_ascii_case(LEGACY_BYPASS_HEADER) {
            continue;
        }
        builder = builder.header(key.as_str(), value.as_str());
    }
    builder = builder.header(LEGACY_BYPASS_HEADER, "1");

    let body = match request.body.as_ref() {
        Some(body) => serde_json::to_vec(body)
            .map_err(|err| cortex_runtime::RuntimeError::Serialization(err.to_string()))?,
        None => Vec::new(),
    };

    let inner_request = builder
        .body(Body::from(body))
        .map_err(|err| cortex_runtime::RuntimeError::Domain(err.to_string()))?;

    let response = router
        .oneshot(inner_request)
        .await
        .map_err(|err| cortex_runtime::RuntimeError::Domain(err.to_string()))?;

    gateway_http_response_to_envelope(response).await
}

async fn gateway_http_response_to_envelope(
    response: Response,
) -> Result<cortex_runtime::gateway::types::GatewayResponseEnvelope, cortex_runtime::RuntimeError> {
    let status = response.status().as_u16();
    let mut headers = BTreeMap::new();
    for (key, value) in response.headers() {
        if key.as_str().eq_ignore_ascii_case("transfer-encoding")
            || key.as_str().eq_ignore_ascii_case("content-length")
            || key.as_str().eq_ignore_ascii_case("connection")
        {
            continue;
        }
        if let Ok(value) = value.to_str() {
            headers.insert(key.as_str().to_string(), value.to_string());
        }
    }

    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .map_err(|err| cortex_runtime::RuntimeError::Domain(err.to_string()))?;
    let body = serde_json::from_slice::<Value>(&bytes).unwrap_or(Value::Null);

    Ok(cortex_runtime::gateway::types::GatewayResponseEnvelope {
        status,
        headers,
        body,
        route_template: None,
        error: None,
        dispatch_error: None,
        transaction_boundary:
            cortex_runtime::gateway::types::GatewayTransactionBoundary::HostManaged,
        event_emissions: Vec::new(),
        idempotency: Default::default(),
    })
}

async fn runtime_gateway_dispatch_middleware(request: Request, next: Next) -> Response {
    if !is_api_request(request.uri().path()) || has_legacy_bypass_header(&request) {
        return next.run(request).await;
    }

    let started_at = Instant::now();
    let envelope = match request_to_gateway_envelope(request).await {
        Ok(envelope) => envelope,
        Err(response) => return response,
    };
    let request_method = envelope.method.clone();
    let request_path = envelope.path.clone();

    match crate::gateway::runtime_host::dispatch_request(envelope).await {
        Ok(response) => {
            record_runtime_dispatch_telemetry_success(
                &request_method,
                &request_path,
                &response,
                started_at.elapsed().as_millis() as u64,
            );
            gateway_response_to_http_response(response)
        }
        Err(err) => {
            let error_class = classify_runtime_dispatch_error(&err.to_string());
            record_runtime_dispatch_telemetry_failure(
                &request_method,
                &request_path,
                error_class,
                started_at.elapsed().as_millis() as u64,
            );
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "runtime_gateway_dispatch_failed",
                    "errorClass": error_class,
                    "message": err.to_string(),
                })),
            )
                .into_response()
        }
    }
}

fn is_api_request(path: &str) -> bool {
    if !(path == "/api" || path.starts_with("/api/")) {
        return false;
    }
    !is_local_legacy_bypass_api_route(path)
}

fn is_local_legacy_bypass_api_route(path: &str) -> bool {
    path == "/api/system"
        || path.starts_with("/api/system/")
        || path == "/api/spaces/create"
        || path.starts_with("/api/spaces/")
}

fn has_legacy_bypass_header(request: &Request) -> bool {
    request
        .headers()
        .get(LEGACY_BYPASS_HEADER)
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim() == "1")
        .unwrap_or(false)
}

#[cfg(test)]
mod runtime_dispatch_route_classification_tests {
    use super::{is_api_request, is_local_legacy_bypass_api_route};

    #[test]
    fn local_legacy_bypass_routes_are_not_runtime_dispatched() {
        assert!(is_local_legacy_bypass_api_route("/api/system"));
        assert!(is_local_legacy_bypass_api_route("/api/system/ux/workbench"));
        assert!(is_local_legacy_bypass_api_route("/api/spaces/create"));
        assert!(is_local_legacy_bypass_api_route(
            "/api/spaces/nostra-governance-v0/navigation-plan"
        ));
        assert!(!is_api_request("/api/system/ux/workbench"));
        assert!(!is_api_request("/api/spaces/create"));
        assert!(!is_api_request(
            "/api/spaces/nostra-governance-v0/navigation-plan"
        ));
    }

    #[test]
    fn runtime_routes_are_dispatched_for_api_non_bypass_paths() {
        assert!(is_api_request(
            "/api/kg/spaces/nostra-governance-v0/contribution-graph/overview"
        ));
        assert!(is_api_request("/api/cortex/views/capability-matrix"));
        assert!(!is_api_request("/ws"));
    }
}

async fn request_to_gateway_envelope(
    request: Request,
) -> Result<cortex_runtime::gateway::types::GatewayRequestEnvelope, Response> {
    let (parts, body) = request.into_parts();
    let body = to_bytes(body, 2 * 1024 * 1024).await.map_err(|err| {
        (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "request_body_read_failed",
                "message": err.to_string(),
            })),
        )
            .into_response()
    })?;

    let mut envelope = cortex_runtime::gateway::types::GatewayRequestEnvelope::new(
        parts.method.as_str(),
        parts.uri.path(),
    );
    envelope.query = parts.uri.query().map(str::to_string);
    for (key, value) in &parts.headers {
        if let Ok(value) = value.to_str() {
            envelope
                .headers
                .insert(key.as_str().to_string(), value.to_string());
        }
    }
    envelope.idempotency_key = parts
        .headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    envelope.actor_id = parts
        .headers
        .get("x-actor-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    envelope.request_id = parts
        .headers
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string);
    if !body.is_empty() {
        envelope.body = match serde_json::from_slice::<Value>(&body) {
            Ok(value) => Some(value),
            Err(_) => Some(Value::String(String::from_utf8_lossy(&body).to_string())),
        };
    }

    Ok(envelope)
}

fn gateway_response_to_http_response(
    response: cortex_runtime::gateway::types::GatewayResponseEnvelope,
) -> Response {
    let status = StatusCode::from_u16(response.status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR);
    let mut http_response = (status, Json(response.body)).into_response();
    for (key, value) in response.headers {
        if let Ok(name) = key.parse::<axum::http::header::HeaderName>() {
            if let Ok(value) = value.parse::<axum::http::header::HeaderValue>() {
                http_response.headers_mut().insert(name, value);
            }
        }
    }
    http_response
}

async fn list_workflows() -> Json<Vec<crate::services::file_system_service::WorkflowFile>> {
    let files = crate::services::file_system_service::FileSystemService::list_workflows();
    Json(files)
}

async fn list_workflow_catalog() -> Json<Vec<WorkflowCatalogEntry>> {
    let mut catalog = Vec::new();
    let files = crate::services::file_system_service::FileSystemService::list_workflows();
    for flow in files {
        catalog.push(WorkflowCatalogEntry {
            name: flow.name,
            path: flow.path,
            source: "filesystem".to_string(),
            status: "available".to_string(),
            description: Some("Local workflow file".to_string()),
            launch_template: None,
            read_only: false,
            automation: None,
        });
    }

    let worker_status = fetch_worker_acp_status().await;
    catalog.push(build_acp_native_entry(worker_status));

    Json(catalog)
}

async fn fetch_worker_acp_status() -> Option<WorkerAcpAutomationStatus> {
    let response = reqwest::Client::new()
        .get("http://127.0.0.1:3003/automations/acp/status")
        .send()
        .await
        .ok()?;

    if !response.status().is_success() {
        return None;
    }

    response.json::<WorkerAcpAutomationStatus>().await.ok()
}

fn build_acp_native_entry(status: Option<WorkerAcpAutomationStatus>) -> WorkflowCatalogEntry {
    let derived_status = match status.as_ref() {
        Some(s) if !s.enabled => "disabled".to_string(),
        Some(s) if s.paused => "paused".to_string(),
        Some(s) if s.active_workflow_id.is_some() => "running".to_string(),
        Some(_) => "ready".to_string(),
        None => "worker-unreachable".to_string(),
    };
    let automation = match status {
        Some(s) => {
            let can_run_now = s.enabled && !s.paused;
            let can_pause = s.enabled && !s.paused;
            let can_resume = s.enabled && s.paused;
            let pause_reason = if s.paused {
                Some("Paused by operator or policy.".to_string())
            } else {
                None
            };
            Some(WorkflowAutomationDescriptor {
                automation_key: s
                    .automation_key
                    .unwrap_or_else(|| "acp_pilot_ops".to_string()),
                enabled: s.enabled,
                paused: s.paused,
                interval_secs: s.interval_secs.unwrap_or(0),
                active_workflow_id: s.active_workflow_id,
                last_workflow_id: s.last_workflow_id,
                last_run_at: s.last_run_at,
                last_status: s.last_status,
                can_run_now,
                can_pause,
                can_resume,
                pause_reason,
            })
        }
        None => Some(WorkflowAutomationDescriptor {
            automation_key: "acp_pilot_ops".to_string(),
            enabled: false,
            paused: false,
            interval_secs: 0,
            active_workflow_id: None,
            last_workflow_id: None,
            last_run_at: None,
            last_status: Some("worker-unreachable".to_string()),
            can_run_now: false,
            can_pause: false,
            can_resume: false,
            pause_reason: Some("Worker API unreachable from Cortex Desktop gateway.".to_string()),
        }),
    };

    WorkflowCatalogEntry {
        name: "acp_pilot_ops".to_string(),
        path: "cortex://worker/workflow-template/acp_pilot_ops".to_string(),
        source: "cortex-worker".to_string(),
        status: derived_status,
        description: Some(
            "Native ACP pilot automation workflow (collect_metrics -> evaluate_slo -> publish_evidence -> steward_gate)"
                .to_string(),
        ),
        launch_template: Some("acp_pilot_ops".to_string()),
        read_only: true,
        automation,
    }
}

pub fn workspace_root() -> PathBuf {
    if let Ok(path) = std::env::var("NOSTRA_WORKSPACE_ROOT") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }

    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(FsPath::parent)
        .and_then(FsPath::parent)
        .map(FsPath::to_path_buf)
        .unwrap_or(manifest_dir)
}

fn workspace_logs_dir() -> PathBuf {
    workspace_root().join("logs")
}

fn workspace_research_dir() -> PathBuf {
    workspace_root().join("research")
}

fn cortex_ux_log_dir() -> PathBuf {
    std::env::var("NOSTRA_CORTEX_UX_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_logs_dir().join("cortex").join("ux"))
}

fn cortex_ux_feedback_log_path() -> PathBuf {
    cortex_ux_log_dir().join("feedback_events.jsonl")
}

fn cortex_ux_evaluation_log_path() -> PathBuf {
    cortex_ux_log_dir().join("candidate_evaluations.jsonl")
}

fn cortex_ux_promotion_log_path() -> PathBuf {
    cortex_ux_log_dir().join("promotion_decisions.jsonl")
}

fn cortex_ux_feedback_queue_path() -> PathBuf {
    cortex_ux_log_dir().join("feedback_queue.json")
}

fn cortex_ux_artifacts_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_store.json")
}

fn cortex_ux_heap_projection_store_path() -> PathBuf {
    cortex_ux_log_dir().join("heap_projection_store.json")
}

fn cortex_ux_heap_emit_rejections_log_path() -> PathBuf {
    cortex_ux_log_dir().join("heap_emit_rejections.jsonl")
}

fn cortex_ux_artifact_audit_log_path() -> PathBuf {
    cortex_ux_log_dir().join("artifact_audit_events.jsonl")
}

fn cortex_ux_artifact_revision_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_revisions.json")
}

fn cortex_ux_artifact_lease_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_leases.json")
}

fn cortex_ux_artifact_collab_sessions_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_collab_sessions.json")
}

fn cortex_ux_artifact_collab_ops_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_collab_ops.json")
}

fn cortex_ux_artifact_crdt_dir(artifact_id: &str) -> PathBuf {
    cortex_ux_log_dir()
        .join("artifacts")
        .join("crdt")
        .join(artifact_id)
}

fn cortex_ux_artifact_crdt_snapshot_path(artifact_id: &str) -> PathBuf {
    cortex_ux_artifact_crdt_dir(artifact_id).join("snapshot.bin")
}

fn cortex_ux_artifact_crdt_ops_path(artifact_id: &str) -> PathBuf {
    cortex_ux_artifact_crdt_dir(artifact_id).join("ops.jsonl")
}

fn cortex_ux_artifact_crdt_presence_path(artifact_id: &str) -> PathBuf {
    cortex_ux_artifact_crdt_dir(artifact_id).join("presence.json")
}

fn cortex_ux_lifecycle_event_log_path() -> PathBuf {
    cortex_ux_log_dir().join("feedback_lifecycle_events.jsonl")
}

fn cortex_ux_remeasurement_store_path() -> PathBuf {
    cortex_ux_log_dir().join("feedback_remeasurements.json")
}

fn cortex_ux_primary_backend() -> &'static str {
    if std::env::var("CANISTER_ID_WORKFLOW_ENGINE").is_ok() {
        "workflow_engine_vfs"
    } else {
        "local_json"
    }
}

fn cortex_ux_source_state() -> CortexSourceState {
    let primary_available = std::env::var("CANISTER_ID_WORKFLOW_ENGINE").is_ok();
    let source_of_truth = cortex_ux_primary_backend().to_string();
    CortexSourceState {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        source_of_truth,
        fallback_active: !primary_available,
        primary_available,
        local_mirror_path: cortex_ux_log_dir().display().to_string(),
    }
}

fn hash_markdown(markdown_source: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(markdown_source.as_bytes());
    hex::encode(hasher.finalize())
}

fn hash_json_hex<T: Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hex::encode(hasher.finalize())
}

fn estimate_markdown_blocks(markdown_source: &str) -> usize {
    markdown_source
        .lines()
        .filter(|line| !line.trim().is_empty())
        .count()
        .max(1)
}

fn read_artifact_revisions() -> Vec<ArtifactRevision> {
    read_json_file_vec(&cortex_ux_artifact_revision_store_path())
}

fn write_artifact_revisions(items: &[ArtifactRevision]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifact_revision_store_path(), items)
}

fn read_artifact_leases() -> Vec<ArtifactLease> {
    read_json_file_vec(&cortex_ux_artifact_lease_store_path())
}

fn write_artifact_leases(items: &[ArtifactLease]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifact_lease_store_path(), items)
}

fn read_remeasurements() -> Vec<UxRemeasurementRecord> {
    read_json_file_vec(&cortex_ux_remeasurement_store_path())
}

fn write_remeasurements(items: &[UxRemeasurementRecord]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_remeasurement_store_path(), items)
}

fn read_collab_sessions() -> Vec<ArtifactCollabSession> {
    read_json_file_vec(&cortex_ux_artifact_collab_sessions_store_path())
}

fn write_collab_sessions(items: &[ArtifactCollabSession]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifact_collab_sessions_store_path(), items)
}

fn read_collab_ops() -> Vec<ArtifactCollabOp> {
    read_json_file_vec(&cortex_ux_artifact_collab_ops_store_path())
}

fn write_collab_ops(items: &[ArtifactCollabOp]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifact_collab_ops_store_path(), items)
}

fn read_artifact_crdt_state(artifact_id: &str, seed_markdown: &str) -> ArtifactCrdtState {
    read_json_file::<ArtifactCrdtState>(&cortex_ux_artifact_crdt_snapshot_path(artifact_id))
        .unwrap_or_else(|| init_crdt_state(artifact_id, seed_markdown, now_iso()))
}

fn write_artifact_crdt_state(artifact_id: &str, state: &ArtifactCrdtState) -> Result<(), String> {
    write_json_file(&cortex_ux_artifact_crdt_snapshot_path(artifact_id), state)
}

fn read_artifact_crdt_ops(artifact_id: &str) -> Vec<ArtifactCrdtUpdateEnvelope> {
    read_jsonl_vec(&cortex_ux_artifact_crdt_ops_path(artifact_id))
}

fn write_artifact_crdt_ops(
    artifact_id: &str,
    ops: &[ArtifactCrdtUpdateEnvelope],
) -> Result<(), String> {
    write_jsonl_vec(&cortex_ux_artifact_crdt_ops_path(artifact_id), ops)
}

fn read_artifact_crdt_presence(artifact_id: &str) -> Vec<ArtifactCollabPresence> {
    read_json_file_vec(&cortex_ux_artifact_crdt_presence_path(artifact_id))
}

fn write_artifact_crdt_presence(
    artifact_id: &str,
    items: &[ArtifactCollabPresence],
) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifact_crdt_presence_path(artifact_id), items)
}

fn clean_expired_presence(items: Vec<ArtifactCollabPresence>) -> Vec<ArtifactCollabPresence> {
    items
        .into_iter()
        .filter(|item| !iso_timestamp_expired(&item.expires_at))
        .collect()
}

fn upsert_artifact_presence(
    artifact_id: &str,
    session_id: &str,
    actor_id: &str,
    actor_role: &str,
    cursor: Option<ArtifactCollabCursor>,
    ttl_secs: i64,
) -> Result<Vec<ArtifactCollabPresence>, String> {
    let mut presence = clean_expired_presence(read_artifact_crdt_presence(artifact_id));
    presence.retain(|entry| !(entry.session_id == session_id && entry.actor_id == actor_id));
    let now = Utc::now();
    presence.push(ArtifactCollabPresence {
        artifact_id: artifact_id.to_string(),
        session_id: session_id.to_string(),
        actor_id: actor_id.to_string(),
        actor_role: actor_role.to_string(),
        last_seen_at: now.to_rfc3339(),
        expires_at: (now + chrono::Duration::seconds(ttl_secs.max(10))).to_rfc3339(),
        cursor,
    });
    write_artifact_crdt_presence(artifact_id, &presence)?;
    Ok(presence)
}

fn revision_markdown(revisions: &[ArtifactRevision], revision_id: &str) -> Option<String> {
    revisions
        .iter()
        .find(|revision| revision.revision_id == revision_id)
        .map(|revision| revision.markdown_source.clone())
}

fn three_way_merge(base: &str, head: &str, proposed: &str) -> ArtifactMergeResult {
    if head == base {
        return ArtifactMergeResult {
            artifact_id: String::new(),
            session_id: String::new(),
            merge_status: "merged_fast_forward".to_string(),
            head_revision_id: String::new(),
            merged_markdown: proposed.to_string(),
            conflict_summary: None,
        };
    }
    if proposed == base || proposed == head {
        return ArtifactMergeResult {
            artifact_id: String::new(),
            session_id: String::new(),
            merge_status: "head_retained".to_string(),
            head_revision_id: String::new(),
            merged_markdown: head.to_string(),
            conflict_summary: None,
        };
    }

    ArtifactMergeResult {
        artifact_id: String::new(),
        session_id: String::new(),
        merge_status: "merge_required".to_string(),
        head_revision_id: String::new(),
        merged_markdown: format!(
            "<<<<<<< HEAD\n{}\n=======\n{}\n>>>>>>> PROPOSED\n",
            head, proposed
        ),
        conflict_summary: Some("Non-head proposal requires explicit merge resolution.".to_string()),
    }
}

fn transition_feedback_queue_item(
    queue_id: &str,
    to_status: &str,
    actor_id: &str,
    reason: Option<String>,
    baseline_metric_date: Option<&str>,
    post_release_metric_date: Option<&str>,
) -> Result<Option<UxFeedbackQueueItem>, String> {
    let mut items = read_feedback_queue_items();
    let mut updated = None;
    for item in &mut items {
        if item.queue_id != queue_id {
            continue;
        }
        let from_status = item.status.clone();
        item.status = to_status.to_string();
        if let Some(date) = baseline_metric_date {
            item.baseline_metric_date = Some(date.to_string());
        }
        if let Some(date) = post_release_metric_date {
            item.post_release_metric_date = Some(date.to_string());
        }
        item.updated_at = now_iso();
        let event = UxLifecycleTransitionEvent {
            event_id: format!("ux_transition_{}", Utc::now().timestamp_millis()),
            queue_id: item.queue_id.clone(),
            route_id: item.route_id.clone(),
            view_id: item.view_id.clone(),
            from_status,
            to_status: item.status.clone(),
            actor_id: actor_id.to_string(),
            transitioned_at: item.updated_at.clone(),
            reason: reason.clone(),
        };
        append_json_line(&cortex_ux_lifecycle_event_log_path(), &event)?;
        updated = Some(item.clone());
        break;
    }
    if updated.is_some() {
        write_feedback_queue_items(&items)?;
    }
    Ok(updated)
}

fn upsert_artifact_revision(
    artifact_id: &str,
    markdown_source: &str,
    created_by: &str,
    parent_revision_id: Option<String>,
    published: bool,
) -> ArtifactRevision {
    let existing = read_artifact_revisions();
    let next_num = existing
        .iter()
        .filter(|rev| rev.artifact_id == artifact_id)
        .map(|rev| rev.revision_number)
        .max()
        .unwrap_or(0)
        + 1;
    ArtifactRevision {
        artifact_id: artifact_id.to_string(),
        revision_id: format!("rev_{}_{}", artifact_id, Utc::now().timestamp_millis()),
        revision_number: next_num,
        markdown_source: markdown_source.to_string(),
        content_hash: hash_markdown(markdown_source),
        created_at: now_iso(),
        created_by: created_by.to_string(),
        parent_revision_id,
        published,
    }
}

fn require_active_lease(
    artifact_id: &str,
    lease_id: &str,
    actor_id: &str,
) -> Result<ArtifactLease, axum::response::Response> {
    let leases = read_artifact_leases();
    let Some(lease) = leases
        .into_iter()
        .find(|entry| entry.artifact_id == artifact_id && entry.lease_id == lease_id)
    else {
        return Err(cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_LEASE_REQUIRED",
            "Active artifact lease is required.",
            Some(json!({ "artifactId": artifact_id, "leaseId": lease_id })),
        ));
    };
    if lease.holder_id != actor_id {
        return Err(cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_LEASE_OWNERSHIP_MISMATCH",
            "Lease holder does not match actor.",
            Some(
                json!({ "artifactId": artifact_id, "holderId": lease.holder_id, "actorId": actor_id }),
            ),
        ));
    }
    if DateTime::parse_from_rfc3339(&lease.expires_at)
        .map(|ts| ts.with_timezone(&Utc) < Utc::now())
        .unwrap_or(true)
    {
        return Err(cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_LEASE_EXPIRED",
            "Artifact lease has expired.",
            Some(
                json!({ "artifactId": artifact_id, "leaseId": lease_id, "expiresAt": lease.expires_at }),
            ),
        ));
    }
    Ok(lease)
}

fn iso_timestamp_expired(timestamp: &str) -> bool {
    DateTime::parse_from_rfc3339(timestamp)
        .map(|ts| ts.with_timezone(&Utc) < Utc::now())
        .unwrap_or(true)
}

fn run_store_future<F, T>(future: F) -> Result<T, String>
where
    F: Future<Output = Result<T, String>> + Send,
    T: Send,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        if handle.runtime_flavor() == tokio::runtime::RuntimeFlavor::MultiThread {
            tokio::task::block_in_place(|| handle.block_on(future))
        } else {
            std::thread::scope(|scope| {
                let join = scope.spawn(move || {
                    let runtime = tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                        .map_err(|err| err.to_string())?;
                    runtime.block_on(future)
                });
                join.join()
                    .map_err(|_| "scoped store runtime panicked".to_string())?
            })
        }
    } else {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|err| err.to_string())?;
        runtime.block_on(future)
    }
}

fn cortex_store_key_for_path(path: &FsPath) -> Option<String> {
    if !is_cortex_ux_local_path(path) {
        return None;
    }
    to_cortex_vfs_key(path)
}

fn cortex_store_mime_for_path(path: &FsPath) -> &'static str {
    if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
        "application/x-ndjson"
    } else {
        "application/json"
    }
}

fn append_json_line<T: Serialize>(path: &FsPath, value: &T) -> Result<(), String> {
    if let Some(key) = cortex_store_key_for_path(path) {
        let line = serde_json::to_string(value).map_err(|err| err.to_string())?;
        return run_store_future(cortex_ux_store_manager().append_line(
            &key,
            &line,
            cortex_store_mime_for_path(path),
        ))
        .map(|_| ());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let line = serde_json::to_string(value).map_err(|err| err.to_string())?;
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|err| err.to_string())?;
    writeln!(file, "{}", line).map_err(|err| err.to_string())
}

fn read_json_file_vec<T: DeserializeOwned>(path: &FsPath) -> Vec<T> {
    if let Some(key) = cortex_store_key_for_path(path) {
        if let Ok(outcome) = run_store_future(cortex_ux_store_manager().read_text(&key)) {
            if let Some(raw) = outcome.text {
                return serde_json::from_str::<Vec<T>>(&raw).unwrap_or_default();
            }
            return Vec::new();
        }
        return Vec::new();
    }

    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return Vec::new(),
    };
    serde_json::from_str::<Vec<T>>(&raw).unwrap_or_default()
}

fn write_json_file_vec<T: Serialize>(path: &FsPath, items: &[T]) -> Result<(), String> {
    if let Some(key) = cortex_store_key_for_path(path) {
        let encoded = serde_json::to_string_pretty(items).map_err(|err| err.to_string())?;
        return run_store_future(cortex_ux_store_manager().write_text(
            &key,
            &encoded,
            cortex_store_mime_for_path(path),
        ))
        .map(|_| ());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let encoded = serde_json::to_string_pretty(items).map_err(|err| err.to_string())?;
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn read_jsonl_vec<T: DeserializeOwned>(path: &FsPath) -> Vec<T> {
    if let Some(key) = cortex_store_key_for_path(path) {
        if let Ok(outcome) = run_store_future(cortex_ux_store_manager().read_text(&key)) {
            if let Some(raw) = outcome.text {
                return raw
                    .lines()
                    .filter_map(|line| serde_json::from_str::<T>(line).ok())
                    .collect();
            }
            return Vec::new();
        }
        return Vec::new();
    }

    let raw = match fs::read_to_string(path) {
        Ok(raw) => raw,
        Err(_) => return Vec::new(),
    };
    raw.lines()
        .filter_map(|line| serde_json::from_str::<T>(line).ok())
        .collect()
}

fn write_jsonl_vec<T: Serialize>(path: &FsPath, items: &[T]) -> Result<(), String> {
    let encoded = items
        .iter()
        .map(|item| serde_json::to_string(item).map_err(|err| err.to_string()))
        .collect::<Result<Vec<_>, _>>()?
        .join("\n");

    if let Some(key) = cortex_store_key_for_path(path) {
        return run_store_future(cortex_ux_store_manager().write_text(
            &key,
            &encoded,
            cortex_store_mime_for_path(path),
        ))
        .map(|_| ());
    }

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn read_json_file<T: DeserializeOwned>(path: &FsPath) -> Option<T> {
    if let Some(key) = cortex_store_key_for_path(path) {
        if let Ok(outcome) = run_store_future(cortex_ux_store_manager().read_text(&key)) {
            return outcome
                .text
                .and_then(|raw| serde_json::from_str::<T>(&raw).ok());
        }
        return None;
    }
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<T>(&raw).ok())
}

fn write_json_file<T: Serialize>(path: &FsPath, value: &T) -> Result<(), String> {
    let encoded = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    if let Some(key) = cortex_store_key_for_path(path) {
        return run_store_future(cortex_ux_store_manager().write_text(
            &key,
            &encoded,
            cortex_store_mime_for_path(path),
        ))
        .map(|_| ());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, encoded).map_err(|err| err.to_string())
}

fn parse_metric_date(date: &str) -> bool {
    DateTime::parse_from_rfc3339(date).is_ok()
}

fn closeout_contribution_id(raw: Option<&str>) -> Result<String, String> {
    let value = raw.unwrap_or(CORTEX_CLOSEOUT_DEFAULT_INITIATIVE).trim();
    if value.is_empty() {
        return Ok(CORTEX_CLOSEOUT_DEFAULT_INITIATIVE.to_string());
    }
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
    {
        return Ok(value.to_string());
    }
    Err("contribution_id must only contain [A-Za-z0-9_-].".to_string())
}

fn closeout_tasks_path_for_contribution(contribution_id: &str) -> PathBuf {
    if let Ok(path) = std::env::var("CORTEX_CLOSEOUT_TASKS_PATH") {
        if !path.trim().is_empty() {
            return PathBuf::from(path);
        }
    }
    workspace_research_dir()
        .join(contribution_id)
        .join("TASKS.json")
}

fn closeout_task_is_overdue(task: &CortexCloseoutTaskRecord, as_of: DateTime<Utc>) -> bool {
    let status = task.status.to_ascii_lowercase();
    if status == "complete" || status == "waived" {
        return false;
    }
    DateTime::parse_from_rfc3339(&task.due_at_utc)
        .map(|ts| as_of > ts.with_timezone(&Utc))
        .unwrap_or(false)
}

fn cortex_ux_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn actor_role_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-cortex-role")
        .and_then(|value| value.to_str().ok())
        .map(str::to_ascii_lowercase)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "operator".to_string())
}

fn actor_id_from_headers(headers: &HeaderMap) -> String {
    headers
        .get("x-cortex-actor")
        .and_then(|value| value.to_str().ok())
        .map(str::to_string)
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "cortex-desktop".to_string())
}

fn idempotency_key_from_headers(headers: &HeaderMap) -> Option<String> {
    headers
        .get("x-idempotency-key")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn artifact_realtime_channel(artifact_id: &str) -> String {
    format!("cortex:artifact:{artifact_id}")
}

fn realtime_feature_enabled() -> bool {
    std::env::var("CORTEX_COLLAB_REALTIME")
        .ok()
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            if matches!(
                normalized.as_str(),
                "0" | "false" | "no" | "off" | "disabled"
            ) {
                return false;
            }
            matches!(normalized.as_str(), "1" | "true" | "yes" | "on" | "enabled")
        })
        .unwrap_or(true)
}

fn artifact_governance_signature_secret() -> Option<String> {
    std::env::var("NOSTRA_ARTIFACT_GOVERNANCE_SIGNING_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(decision_signature_secret)
}

fn artifact_governance_require_secret() -> bool {
    std::env::var("NOSTRA_ARTIFACT_GOVERNANCE_REQUIRE_SECRET")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

fn governance_nonce_registry() -> &'static Mutex<HashSet<String>> {
    static NONCES: LazyLock<Mutex<HashSet<String>>> = LazyLock::new(|| Mutex::new(HashSet::new()));
    &NONCES
}

fn require_governance_envelope(
    actor_id: &str,
    envelope: Option<&ArtifactGovernanceEnvelope>,
) -> Result<(), axum::response::Response> {
    let Some(envelope) = envelope else {
        return Err(cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "ARTIFACT_GOVERNANCE_ENVELOPE_REQUIRED",
            "Signed governance envelope is required for this action.",
            None,
        ));
    };
    if envelope.approved_by.trim().is_empty()
        || envelope.rationale.trim().is_empty()
        || envelope.approved_at.trim().is_empty()
        || envelope.actor_id.trim().is_empty()
        || envelope.decision_proof.decision_id.trim().is_empty()
        || envelope.decision_proof.signature.trim().is_empty()
        || envelope.decision_proof.signer.trim().is_empty()
    {
        return Err(cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_GOVERNANCE_ENVELOPE",
            "approvedBy, rationale, approvedAt, actorId, and decisionProof fields are required.",
            None,
        ));
    }
    if !parse_metric_date(&envelope.approved_at) {
        return Err(cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_GOVERNANCE_APPROVED_AT",
            "governance.approvedAt must be RFC3339.",
            None,
        ));
    }
    let nonce = envelope
        .nonce
        .as_deref()
        .or(envelope.decision_proof.nonce.as_deref())
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if nonce.is_empty() {
        return Err(cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "ARTIFACT_GOVERNANCE_NONCE_REQUIRED",
            "governance.nonce is required for privileged mutations.",
            None,
        ));
    }
    let expires_at = envelope
        .expires_at
        .as_deref()
        .or(envelope.decision_proof.expires_at.as_deref())
        .map(str::trim)
        .unwrap_or_default()
        .to_string();
    if expires_at.is_empty() || !parse_metric_date(&expires_at) {
        return Err(cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "ARTIFACT_GOVERNANCE_EXPIRES_AT_REQUIRED",
            "governance.expiresAt must be RFC3339.",
            None,
        ));
    }
    if iso_timestamp_expired(&expires_at) {
        return Err(cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_GOVERNANCE_EXPIRED",
            "governance.expiresAt is in the past.",
            Some(json!({ "expiresAt": expires_at })),
        ));
    }
    if envelope.actor_id != actor_id {
        return Err(cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_GOVERNANCE_ACTOR_MISMATCH",
            "governance.actorId must match x-cortex-actor header.",
            Some(json!({ "headerActorId": actor_id, "governanceActorId": envelope.actor_id })),
        ));
    }
    {
        let registry = governance_nonce_registry();
        let locked = registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        if locked.contains(&nonce) {
            return Err(cortex_ux_error(
                StatusCode::CONFLICT,
                "ARTIFACT_GOVERNANCE_NONCE_REPLAY",
                "governance nonce has already been used.",
                Some(json!({ "nonce": nonce })),
            ));
        }
    }

    let material = format!(
        "{}|{}|{}|{}|{}|{}",
        actor_id,
        envelope.approved_by.trim(),
        envelope.approved_at.trim(),
        envelope.decision_proof.decision_id.trim(),
        nonce,
        expires_at
    );
    if let Some(secret) = artifact_governance_signature_secret() {
        let expected = signature_hash(&secret, &material);
        if !constant_time_eq(
            &expected,
            &envelope
                .decision_proof
                .signature
                .trim()
                .to_ascii_lowercase(),
        ) {
            return Err(cortex_ux_error(
                StatusCode::FORBIDDEN,
                "ARTIFACT_GOVERNANCE_SIGNATURE_INVALID",
                "Governance signature verification failed.",
                Some(json!({
                    "decisionId": envelope.decision_proof.decision_id,
                    "signer": envelope.decision_proof.signer
                })),
            ));
        }
    } else if artifact_governance_require_secret() {
        return Err(cortex_ux_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "ARTIFACT_GOVERNANCE_SECRET_MISSING",
            "Artifact governance signature verification is required but no signing secret is configured.",
            None,
        ));
    }

    {
        let registry = governance_nonce_registry();
        let mut locked = registry
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        locked.insert(nonce);
    }
    Ok(())
}

fn feedback_dedupe_key(event: &UxFeedbackEvent) -> String {
    let action = event
        .action_id
        .clone()
        .unwrap_or_else(|| "none".to_string())
        .to_ascii_lowercase();
    format!(
        "{}|{}|{}|{}",
        event.route_id.to_ascii_lowercase(),
        event.view_id.to_ascii_lowercase(),
        event.friction_tag.to_ascii_lowercase(),
        action
    )
}

fn feedback_default_priority(severity: &str) -> String {
    match severity.to_ascii_lowercase().as_str() {
        "critical" | "error" => "high".to_string(),
        "warn" | "warning" => "medium".to_string(),
        _ => "low".to_string(),
    }
}

fn read_feedback_queue_items() -> Vec<UxFeedbackQueueItem> {
    read_json_file_vec(&cortex_ux_feedback_queue_path())
}

fn write_feedback_queue_items(items: &[UxFeedbackQueueItem]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_feedback_queue_path(), items)
}

fn upsert_feedback_queue_item(event: &UxFeedbackEvent) -> Result<UxFeedbackQueueItem, String> {
    let mut items = read_feedback_queue_items();
    let key = feedback_dedupe_key(event);
    let now = now_iso();
    if let Some(existing) = items.iter_mut().find(|item| item.dedupe_key == key) {
        existing.updated_at = now;
        existing.event_count = existing.event_count.saturating_add(1);
        if existing.status == UX_STATUS_REJECTED {
            existing.status = UX_STATUS_NEW.to_string();
        }
        let cloned = existing.clone();
        write_feedback_queue_items(&items)?;
        return Ok(cloned);
    }

    let item = UxFeedbackQueueItem {
        queue_id: format!("ux_queue_{}", Utc::now().timestamp_millis()),
        dedupe_key: key,
        route_id: event.route_id.clone(),
        view_id: event.view_id.clone(),
        friction_tag: event.friction_tag.clone(),
        severity: event.severity.clone(),
        status: UX_STATUS_NEW.to_string(),
        priority: feedback_default_priority(&event.severity),
        assigned_to: None,
        notes: None,
        baseline_metric_date: None,
        post_release_metric_date: None,
        first_seen_at: event.timestamp.clone(),
        updated_at: event.timestamp.clone(),
        event_count: 1,
    };
    items.push(item.clone());
    write_feedback_queue_items(&items)?;
    Ok(item)
}

fn update_feedback_queue_item(
    req: &CortexFeedbackTriageRequest,
) -> Result<Option<UxFeedbackQueueItem>, String> {
    let mut items = read_feedback_queue_items();
    let mut result = None;
    for item in &mut items {
        if item.queue_id != req.queue_id {
            continue;
        }
        item.status = req.status.clone();
        item.priority = req
            .priority
            .clone()
            .unwrap_or_else(|| item.priority.clone())
            .to_ascii_lowercase();
        item.assigned_to = req.assigned_to.clone().or_else(|| item.assigned_to.clone());
        item.notes = req.notes.clone().or_else(|| item.notes.clone());
        item.baseline_metric_date = req
            .baseline_metric_date
            .clone()
            .or_else(|| item.baseline_metric_date.clone());
        item.post_release_metric_date = req
            .post_release_metric_date
            .clone()
            .or_else(|| item.post_release_metric_date.clone());
        item.updated_at = now_iso();
        result = Some(item.clone());
        break;
    }
    if result.is_some() {
        write_feedback_queue_items(&items)?;
    }
    Ok(result)
}

fn update_feedback_status_for_candidate(
    route_id: &str,
    view_id: &str,
    status: &str,
    baseline_metric_date: Option<&str>,
    post_release_metric_date: Option<&str>,
) -> Result<(), String> {
    let mut items = read_feedback_queue_items();
    let now = now_iso();
    for item in &mut items {
        if item.route_id == route_id && item.view_id == view_id {
            item.status = status.to_string();
            item.updated_at = now.clone();
            if let Some(date) = baseline_metric_date {
                item.baseline_metric_date = Some(date.to_string());
            }
            if let Some(date) = post_release_metric_date {
                item.post_release_metric_date = Some(date.to_string());
            }
        }
    }
    write_feedback_queue_items(&items)
}

fn read_artifacts_store() -> Vec<ArtifactDocumentV2> {
    read_json_file_vec(&cortex_ux_artifacts_store_path())
}

fn write_artifacts_store(items: &[ArtifactDocumentV2]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_artifacts_store_path(), items)
}

fn read_heap_projection_store() -> Vec<HeapProjectionRecord> {
    read_json_file_vec(&cortex_ux_heap_projection_store_path())
}

fn write_heap_projection_store(items: &[HeapProjectionRecord]) -> Result<(), String> {
    write_json_file_vec(&cortex_ux_heap_projection_store_path(), items)
}

fn append_heap_emit_rejection(event: &HeapEmitRejectionEvent) -> Result<(), String> {
    append_json_line(&cortex_ux_heap_emit_rejections_log_path(), event)
}

fn append_artifact_audit(
    artifact_id: &str,
    action: &str,
    actor_role: &str,
    actor_id: &str,
    route_id: &str,
    notes: Option<String>,
) -> Result<(), String> {
    let event = ArtifactAuditEvent {
        audit_id: format!("artifact_audit_{}", Utc::now().timestamp_millis()),
        artifact_id: artifact_id.to_string(),
        action: action.to_string(),
        actor_role: actor_role.to_string(),
        actor_id: actor_id.to_string(),
        route_id: route_id.to_string(),
        timestamp: now_iso(),
        notes,
    };
    append_json_line(&cortex_ux_artifact_audit_log_path(), &event)
}

async fn store_read_json<T: DeserializeOwned>(key: &str) -> Result<Option<T>, String> {
    let outcome = cortex_ux_store_manager().read_text(key).await?;
    match outcome.text {
        Some(raw) => serde_json::from_str::<T>(&raw)
            .map(Some)
            .map_err(|err| err.to_string()),
        None => Ok(None),
    }
}

async fn store_write_json<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    let encoded = serde_json::to_string_pretty(value).map_err(|err| err.to_string())?;
    cortex_ux_store_manager()
        .write_text(key, &encoded, "application/json")
        .await
        .map(|_| ())
}

async fn store_append_jsonl<T: Serialize>(key: &str, value: &T) -> Result<(), String> {
    let line = serde_json::to_string(value).map_err(|err| err.to_string())?;
    cortex_ux_store_manager()
        .append_line(key, &line, "application/x-ndjson")
        .await
        .map(|_| ())
}

async fn store_read_jsonl<T: DeserializeOwned>(key: &str) -> Result<Vec<T>, String> {
    let outcome = cortex_ux_store_manager().read_text(key).await?;
    let Some(raw) = outcome.text else {
        return Ok(Vec::new());
    };

    let mut rows = Vec::new();
    for (line_idx, line) in raw.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let parsed = serde_json::from_str::<T>(trimmed).map_err(|err| {
            format!(
                "Failed to parse JSONL at {} line {}: {}",
                key,
                line_idx + 1,
                err
            )
        })?;
        rows.push(parsed);
    }
    Ok(rows)
}

fn viewspec_event_key_today() -> String {
    viewspec_events_key(Utc::now().format("%Y-%m-%d").to_string().as_str())
}

fn viewspec_scope_from_lookup(query: &ViewSpecLookupQuery) -> Option<ViewSpecScope> {
    if query.space_id.is_none() && query.route_id.is_none() && query.role.is_none() {
        return None;
    }
    Some(ViewSpecScope {
        space_id: query.space_id.clone(),
        route_id: query.route_id.clone(),
        role: query.role.clone(),
    })
}

async fn read_viewspec_index() -> Result<BTreeMap<String, ViewSpecIndexEntry>, String> {
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecIndexEntry>>(VIEWSPEC_INDEX_KEY)
            .await?
            .unwrap_or_default(),
    )
}

async fn write_viewspec_index(index: &BTreeMap<String, ViewSpecIndexEntry>) -> Result<(), String> {
    store_write_json(VIEWSPEC_INDEX_KEY, index).await
}

async fn read_viewspec_candidate_set_index()
-> Result<BTreeMap<String, ViewSpecCandidateSetIndexEntry>, String> {
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecCandidateSetIndexEntry>>(
            VIEWSPEC_CANDIDATE_SET_INDEX_KEY,
        )
        .await?
        .unwrap_or_default(),
    )
}

async fn write_viewspec_candidate_set_index(
    index: &BTreeMap<String, ViewSpecCandidateSetIndexEntry>,
) -> Result<(), String> {
    store_write_json(VIEWSPEC_CANDIDATE_SET_INDEX_KEY, index).await
}

async fn read_viewspec_learning_signal_index()
-> Result<BTreeMap<String, ViewSpecLearningSignalIndexEntry>, String> {
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecLearningSignalIndexEntry>>(
            VIEWSPEC_LEARNING_SIGNAL_INDEX_KEY,
        )
        .await?
        .unwrap_or_default(),
    )
}

async fn write_viewspec_learning_signal_index(
    index: &BTreeMap<String, ViewSpecLearningSignalIndexEntry>,
) -> Result<(), String> {
    store_write_json(VIEWSPEC_LEARNING_SIGNAL_INDEX_KEY, index).await
}

async fn read_viewspec_proposal_index()
-> Result<BTreeMap<String, ViewSpecProposalIndexEntry>, String> {
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecProposalIndexEntry>>(
            VIEWSPEC_PROPOSAL_INDEX_KEY,
        )
        .await?
        .unwrap_or_default(),
    )
}

async fn write_viewspec_proposal_index(
    index: &BTreeMap<String, ViewSpecProposalIndexEntry>,
) -> Result<(), String> {
    store_write_json(VIEWSPEC_PROPOSAL_INDEX_KEY, index).await
}

async fn read_viewspec_active_scope_index()
-> Result<BTreeMap<String, ViewSpecScopeAdoptionRecord>, String> {
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecScopeAdoptionRecord>>(
            VIEWSPEC_ACTIVE_SCOPE_INDEX_KEY,
        )
        .await?
        .unwrap_or_default(),
    )
}

async fn write_viewspec_active_scope_index(
    index: &BTreeMap<String, ViewSpecScopeAdoptionRecord>,
) -> Result<(), String> {
    store_write_json(VIEWSPEC_ACTIVE_SCOPE_INDEX_KEY, index).await
}

async fn read_viewspec_replay_index() -> Result<BTreeMap<String, ViewSpecReplayIndexEntry>, String>
{
    Ok(
        store_read_json::<BTreeMap<String, ViewSpecReplayIndexEntry>>(VIEWSPEC_REPLAY_INDEX_KEY)
            .await?
            .unwrap_or_default(),
    )
}

async fn write_viewspec_replay_index(
    index: &BTreeMap<String, ViewSpecReplayIndexEntry>,
) -> Result<(), String> {
    store_write_json(VIEWSPEC_REPLAY_INDEX_KEY, index).await
}

fn sanitize_viewspec_candidate_set_token(value: &str) -> String {
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

fn viewspec_proposal_events_key(date_yyyy_mm_dd: &str) -> String {
    format!("/cortex/ux/viewspecs/proposals/events/{date_yyyy_mm_dd}.jsonl")
}

fn viewspec_governance_events_key(date_yyyy_mm_dd: &str) -> String {
    format!("/cortex/ux/viewspecs/governance/events/{date_yyyy_mm_dd}.jsonl")
}

fn viewspec_proposal_history_key(scope_key: &str, proposal_id: &str, timestamp: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/proposals/history/{}/{}_{}.json",
        scope_key,
        sanitize_viewspec_candidate_set_token(timestamp),
        sanitize_viewspec_candidate_set_token(proposal_id),
    )
}

fn viewspec_active_scope_key(scope_key: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/active/{}.json",
        sanitize_viewspec_candidate_set_token(scope_key)
    )
}

fn viewspec_replay_key(proposal_id: &str, run_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/replay/{}/{}.json",
        sanitize_viewspec_candidate_set_token(proposal_id),
        sanitize_viewspec_candidate_set_token(run_id),
    )
}

fn viewspec_replay_digest_latest_key(proposal_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/replay/{}/digest_latest.json",
        sanitize_viewspec_candidate_set_token(proposal_id),
    )
}

fn spatial_experiment_events_key(date_yyyy_mm_dd: &str) -> String {
    format!("/cortex/ux/viewspecs/experiments/spatial/events/{date_yyyy_mm_dd}.jsonl")
}

fn spatial_experiment_run_summary_key(run_id: &str) -> String {
    format!(
        "/cortex/ux/viewspecs/experiments/spatial/runs/{}.json",
        sanitize_viewspec_candidate_set_token(run_id)
    )
}

fn spatial_experiment_event_date(timestamp: &str) -> String {
    timestamp
        .get(0..10)
        .map(str::to_string)
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
}

fn spatial_experiment_event_id(run_id: &str, event_type: &str) -> String {
    format!(
        "spatial_evt_{}_{}_{}",
        Utc::now().timestamp_millis(),
        sanitize_viewspec_candidate_set_token(run_id),
        sanitize_viewspec_candidate_set_token(event_type),
    )
}

fn spatial_experiment_event_supported(event_type: &str) -> bool {
    matches!(
        event_type,
        "run_start"
            | "mode_switch"
            | "button_click"
            | "approval"
            | "spatial_shape_click"
            | "spatial_adapter_loaded"
            | "spatial_adapter_fallback"
            | "spatial_adapter_replay"
            | "spatial_adapter_replay_failed"
            | "run_end"
    )
}

fn spatial_experiment_recommendation(
    improvement_score: f64,
    metrics: &SpatialExperimentMetrics,
) -> String {
    if metrics.error_event_count > 0 || metrics.adapter_fallback_rate > 0.2 {
        return "no_go".to_string();
    }
    if improvement_score >= 3.0 && metrics.approval_decision_count > 0 {
        return "go".to_string();
    }
    "hold".to_string()
}

fn normalize_spatial_recommendation(value: &str) -> String {
    match value.trim().to_ascii_lowercase().as_str() {
        "go" => "go".to_string(),
        "no_go" | "no-go" | "nogo" => "no_go".to_string(),
        "hold" => "hold".to_string(),
        _ => "hold".to_string(),
    }
}

async fn store_viewspec_candidate_set(
    scope: &ViewSpecScope,
    candidate_set: &ViewSpecCandidateSet,
) -> Result<(), String> {
    let key = candidate_set_store_key(scope, &candidate_set.candidate_set_id);
    store_write_json(key.as_str(), candidate_set).await?;

    let mut index = read_viewspec_candidate_set_index().await?;
    index.insert(
        candidate_set.candidate_set_id.clone(),
        ViewSpecCandidateSetIndexEntry {
            candidate_set_id: candidate_set.candidate_set_id.clone(),
            scope_key: candidate_set.scope_key.clone(),
            updated_at: viewspec_now_iso(),
        },
    );
    write_viewspec_candidate_set_index(&index).await
}

async fn load_viewspec_candidate_set(
    candidate_set_id: &str,
) -> Result<Option<ViewSpecCandidateSet>, String> {
    let index = read_viewspec_candidate_set_index().await?;
    let Some(entry) = index.get(candidate_set_id) else {
        return Ok(None);
    };
    let key = format!(
        "/cortex/ux/viewspecs/candidates/{}/{}.json",
        entry.scope_key,
        sanitize_viewspec_candidate_set_token(candidate_set_id)
    );
    store_read_json::<ViewSpecCandidateSet>(key.as_str()).await
}

fn viewspec_learning_signal_id(event_type: &str) -> String {
    format!(
        "viewspec_learning_{}_{}",
        Utc::now().timestamp_millis(),
        sanitize_learning_token(event_type)
    )
}

fn viewspec_learning_signal_date(timestamp: &str) -> String {
    timestamp
        .get(0..10)
        .map(str::to_string)
        .unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string())
}

async fn append_viewspec_learning_signal_record(
    signal: &ViewSpecLearningSignal,
) -> Result<(), String> {
    validate_learning_signal(signal)?;
    let date = viewspec_learning_signal_date(&signal.timestamp);
    let key = learning_signals_key(date.as_str());
    store_append_jsonl(key.as_str(), signal).await?;

    let mut index = read_viewspec_learning_signal_index().await?;
    index.insert(
        date.clone(),
        ViewSpecLearningSignalIndexEntry {
            date,
            key,
            updated_at: viewspec_now_iso(),
        },
    );
    write_viewspec_learning_signal_index(&index).await
}

async fn load_viewspec_learning_profile(
    space_id: &str,
) -> Result<Option<SpaceLearningProfileV1>, String> {
    store_read_json::<SpaceLearningProfileV1>(learning_profile_key(space_id).as_str()).await
}

async fn store_viewspec_learning_profile(profile: &SpaceLearningProfileV1) -> Result<(), String> {
    store_write_json(learning_profile_key(&profile.space_id).as_str(), profile).await
}

async fn store_viewspec_learning_replay(
    space_id: &str,
    replay: &LearningReplayResult,
) -> Result<String, String> {
    let key = learning_replay_key(space_id, &replay.run_id);
    store_write_json(key.as_str(), replay).await?;
    Ok(key)
}

async fn load_viewspec_learning_signals(
    space_id: &str,
) -> Result<Vec<ViewSpecLearningSignal>, String> {
    let index = read_viewspec_learning_signal_index().await?;
    let mut entries = index.values().cloned().collect::<Vec<_>>();
    entries.sort_by(|a, b| a.date.cmp(&b.date));

    let mut signals = Vec::new();
    for entry in entries {
        let records = store_read_jsonl::<ViewSpecLearningSignal>(entry.key.as_str()).await?;
        signals.extend(
            records
                .into_iter()
                .filter(|signal| signal.space_id.trim() == space_id),
        );
    }

    signals.sort_by(|a, b| {
        a.timestamp
            .cmp(&b.timestamp)
            .then_with(|| a.signal_id.cmp(&b.signal_id))
    });
    Ok(signals)
}

async fn emit_viewspec_learning_signal(
    event_type: &str,
    spec: &ViewSpecV1,
    actor: &str,
    payload: Value,
) -> Result<ViewSpecLearningSignal, String> {
    let Some(space_id) = spec
        .scope
        .space_id
        .as_ref()
        .map(|value| value.trim().to_string())
    else {
        return Err("space_id is required for learning signal emission".to_string());
    };
    if space_id.is_empty() {
        return Err("space_id is required for learning signal emission".to_string());
    }

    let signal = ViewSpecLearningSignal {
        signal_id: viewspec_learning_signal_id(event_type),
        event_type: normalize_event_type(event_type),
        view_spec_id: spec.view_spec_id.clone(),
        space_id: space_id.clone(),
        actor: actor.trim().to_string(),
        timestamp: viewspec_now_iso(),
        payload,
    };
    append_viewspec_learning_signal_record(&signal).await?;
    Ok(signal)
}

async fn append_viewspec_candidate_set_event(
    event_type: &str,
    candidate_set: &ViewSpecCandidateSet,
    actor: &str,
    payload: Value,
) -> Result<(), String> {
    let record = ViewSpecEventRecord {
        event_id: format!("viewspec_evt_{}", Utc::now().timestamp_millis()),
        event_type: event_type.to_string(),
        view_spec_id: candidate_set.candidate_set_id.clone(),
        scope_key: candidate_set.scope_key.clone(),
        actor: actor.to_string(),
        timestamp: viewspec_now_iso(),
        payload,
    };
    store_append_jsonl(viewspec_event_key_today().as_str(), &record).await
}

async fn append_viewspec_event(
    event_type: &str,
    spec: &ViewSpecV1,
    actor: &str,
    payload: Value,
) -> Result<(), String> {
    let record = ViewSpecEventRecord {
        event_id: format!("viewspec_evt_{}", Utc::now().timestamp_millis()),
        event_type: event_type.to_string(),
        view_spec_id: spec.view_spec_id.clone(),
        scope_key: scope_key(&spec.scope),
        actor: actor.to_string(),
        timestamp: viewspec_now_iso(),
        payload,
    };
    store_append_jsonl(viewspec_event_key_today().as_str(), &record).await
}

async fn append_viewspec_proposal_event(
    event_type: &str,
    proposal: &ViewSpecProposalEnvelope,
    actor: &str,
    payload: Value,
) -> Result<(), String> {
    let record = ViewSpecEventRecord {
        event_id: format!("viewspec_evt_{}", Utc::now().timestamp_millis()),
        event_type: event_type.to_string(),
        view_spec_id: proposal.view_spec_id.clone(),
        scope_key: proposal.scope_key.clone(),
        actor: actor.to_string(),
        timestamp: viewspec_now_iso(),
        payload,
    };
    let date = Utc::now().format("%Y-%m-%d").to_string();
    store_append_jsonl(viewspec_proposal_events_key(&date).as_str(), &record).await
}

async fn append_viewspec_governance_event(
    event_type: &str,
    proposal: &ViewSpecProposalEnvelope,
    actor: &str,
    payload: Value,
) -> Result<(), String> {
    let record = ViewSpecEventRecord {
        event_id: format!("viewspec_evt_{}", Utc::now().timestamp_millis()),
        event_type: event_type.to_string(),
        view_spec_id: proposal.view_spec_id.clone(),
        scope_key: proposal.scope_key.clone(),
        actor: actor.to_string(),
        timestamp: viewspec_now_iso(),
        payload,
    };
    let date = Utc::now().format("%Y-%m-%d").to_string();
    store_append_jsonl(viewspec_governance_events_key(&date).as_str(), &record).await
}

async fn store_viewspec_proposal(
    scope: &ViewSpecScope,
    proposal: &ViewSpecProposalEnvelope,
) -> Result<(), String> {
    let scope_value = scope_key(scope);
    let proposal_key = proposal_store_key(scope, &proposal.proposal_id);
    let history_key = viewspec_proposal_history_key(
        &scope_value,
        &proposal.proposal_id,
        &Utc::now().timestamp_millis().to_string(),
    );
    store_write_json(proposal_key.as_str(), proposal).await?;
    store_write_json(history_key.as_str(), proposal).await?;

    let mut index = read_viewspec_proposal_index().await?;
    index.insert(
        proposal.proposal_id.clone(),
        ViewSpecProposalIndexEntry {
            proposal_id: proposal.proposal_id.clone(),
            scope_key: scope_value,
            updated_at: viewspec_now_iso(),
        },
    );
    write_viewspec_proposal_index(&index).await
}

async fn load_viewspec_proposal(
    proposal_id: &str,
) -> Result<Option<ViewSpecProposalEnvelope>, String> {
    let index = read_viewspec_proposal_index().await?;
    let Some(entry) = index.get(proposal_id) else {
        return Ok(None);
    };
    let key = format!(
        "/cortex/ux/viewspecs/proposals/{}/{}.json",
        entry.scope_key,
        sanitize_viewspec_candidate_set_token(proposal_id)
    );
    match store_read_json::<ViewSpecProposalEnvelope>(key.as_str()).await? {
        Some(mut proposal) => {
            if proposal.scope_key.trim().is_empty() {
                proposal.scope_key = entry.scope_key.clone();
            }
            Ok(Some(proposal))
        }
        None => Ok(None),
    }
}

async fn list_viewspec_proposals() -> Result<Vec<ViewSpecProposalEnvelope>, String> {
    let index = read_viewspec_proposal_index().await?;
    let mut rows = Vec::new();
    for entry in index.values() {
        let key = format!(
            "/cortex/ux/viewspecs/proposals/{}/{}.json",
            entry.scope_key,
            sanitize_viewspec_candidate_set_token(entry.proposal_id.as_str())
        );
        if let Some(mut proposal) =
            store_read_json::<ViewSpecProposalEnvelope>(key.as_str()).await?
        {
            if proposal.scope_key.trim().is_empty() {
                proposal.scope_key = entry.scope_key.clone();
            }
            rows.push(proposal);
        }
    }
    rows.sort_by(|a, b| a.created_at.cmp(&b.created_at));
    Ok(rows)
}

async fn store_viewspec_scope_adoption(record: &ViewSpecScopeAdoptionRecord) -> Result<(), String> {
    let key = viewspec_active_scope_key(&record.scope_key);
    store_write_json(key.as_str(), record).await?;
    let mut index = read_viewspec_active_scope_index().await?;
    index.insert(record.scope_key.clone(), record.clone());
    write_viewspec_active_scope_index(&index).await
}

async fn load_viewspec_active_scope(
    scope_key_value: &str,
) -> Result<Option<ViewSpecScopeAdoptionRecord>, String> {
    let key = viewspec_active_scope_key(scope_key_value);
    store_read_json::<ViewSpecScopeAdoptionRecord>(key.as_str()).await
}

async fn list_viewspec_active_scopes() -> Result<Vec<ViewSpecScopeAdoptionRecord>, String> {
    let index = read_viewspec_active_scope_index().await?;
    let mut rows = index.values().cloned().collect::<Vec<_>>();
    rows.sort_by(|a, b| a.scope_key.cmp(&b.scope_key));
    Ok(rows)
}

async fn store_viewspec_replay_artifact(
    proposal_id: &str,
    replay: &ViewSpecReplayArtifact,
    digest: &ViewSpecDigestArtifact,
) -> Result<(), String> {
    let replay_key = viewspec_replay_key(proposal_id, &replay.run_id);
    store_write_json(replay_key.as_str(), replay).await?;
    store_write_json(
        viewspec_replay_digest_latest_key(proposal_id).as_str(),
        digest,
    )
    .await?;

    let mut index = read_viewspec_replay_index().await?;
    index.insert(
        proposal_id.to_string(),
        ViewSpecReplayIndexEntry {
            proposal_id: proposal_id.to_string(),
            run_id: replay.run_id.clone(),
            key: replay_key,
            updated_at: viewspec_now_iso(),
        },
    );
    write_viewspec_replay_index(&index).await
}

async fn load_viewspec_latest_replay_artifact(
    proposal_id: &str,
) -> Result<Option<ViewSpecReplayArtifact>, String> {
    let index = read_viewspec_replay_index().await?;
    let Some(entry) = index.get(proposal_id) else {
        return Ok(None);
    };
    store_read_json::<ViewSpecReplayArtifact>(entry.key.as_str()).await
}

async fn store_viewspec(
    spec: &ViewSpecV1,
    event_type: &str,
    actor: &str,
    payload: Value,
) -> Result<String, String> {
    let scope = scope_key(&spec.scope);
    let current_key = current_viewspec_key(&spec.scope, &spec.view_spec_id);
    let history_key = history_viewspec_key(
        &spec.scope,
        &spec.view_spec_id,
        &Utc::now().timestamp_millis().to_string(),
    );

    store_write_json(current_key.as_str(), spec).await?;
    store_write_json(history_key.as_str(), spec).await?;

    let mut index = read_viewspec_index().await?;
    index.insert(
        spec.view_spec_id.clone(),
        ViewSpecIndexEntry {
            view_spec_id: spec.view_spec_id.clone(),
            scope_key: scope.clone(),
            updated_at: viewspec_now_iso(),
        },
    );
    write_viewspec_index(&index).await?;
    append_viewspec_event(event_type, spec, actor, payload).await?;
    Ok(scope)
}

async fn load_viewspec(
    view_spec_id: &str,
    scope: Option<ViewSpecScope>,
) -> Result<Option<ViewSpecV1>, String> {
    if let Some(scope) = scope {
        let key = current_viewspec_key(&scope, view_spec_id);
        return store_read_json::<ViewSpecV1>(key.as_str()).await;
    }

    let index = read_viewspec_index().await?;
    let Some(entry) = index.get(view_spec_id) else {
        return Ok(None);
    };
    let key = format!(
        "/cortex/ux/viewspecs/current/{}/{}.json",
        entry.scope_key,
        sanitize_viewspec_id_token(view_spec_id)
    );
    store_read_json::<ViewSpecV1>(key.as_str()).await
}

fn sanitize_viewspec_id_token(value: &str) -> String {
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

async fn get_cortex_layout_spec() -> Json<ShellLayoutSpec> {
    Json(resolve_shell_layout_spec())
}

async fn get_cortex_layout_source_state() -> Json<CortexSourceState> {
    Json(cortex_ux_source_state())
}

async fn get_cortex_runtime_sync_status() -> Json<CortexSyncStatus> {
    Json(cortex_ux_store_manager().sync_status().await)
}

async fn post_cortex_runtime_sync_replay() -> axum::response::Response {
    match cortex_ux_store_manager().replay_pending().await {
        Ok(result) => Json::<CortexReplayResult>(result).into_response(),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CORTEX_RUNTIME_REPLAY_FAILED",
            "Failed to replay pending Cortex UX writes to workflow engine VFS.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_cortex_runtime_slo_status() -> Json<CortexRealtimeSloStatus> {
    Json(streaming_transport_manager().slo_status().await)
}

async fn get_cortex_runtime_slo_breaches() -> Json<Vec<CortexRealtimeSloBreachEvent>> {
    Json(streaming_transport_manager().slo_breaches().await)
}

async fn get_cortex_runtime_closeout_tasks(
    Query(query): Query<CortexCloseoutTasksQuery>,
) -> axum::response::Response {
    let contribution_id = match closeout_contribution_id(query.contribution_id.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "INVALID_CONTRIBUTION_ID",
                "Invalid closeout contribution identifier.",
                Some(json!({ "reason": err })),
            );
        }
    };
    let as_of = match query.as_of.as_deref() {
        Some(raw) => match DateTime::parse_from_rfc3339(raw) {
            Ok(ts) => ts.with_timezone(&Utc),
            Err(err) => {
                return cortex_ux_error(
                    StatusCode::BAD_REQUEST,
                    "INVALID_AS_OF",
                    "as_of must be RFC3339 UTC timestamp.",
                    Some(json!({ "asOf": raw, "reason": err.to_string() })),
                );
            }
        },
        None => Utc::now(),
    };

    let path = closeout_tasks_path_for_contribution(&contribution_id);
    let raw = match fs::read_to_string(&path) {
        Ok(data) => data,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::NOT_FOUND,
                "CLOSEOUT_TASKS_NOT_FOUND",
                "Closeout task ledger is not available for the requested contribution.",
                Some(
                    json!({ "contributionId": contribution_id, "path": path.display().to_string(), "reason": err.to_string() }),
                ),
            );
        }
    };

    let ledger = match serde_json::from_str::<CortexCloseoutTaskLedger>(&raw) {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "CLOSEOUT_TASKS_INVALID",
                "Closeout task ledger is malformed JSON.",
                Some(
                    json!({ "contributionId": contribution_id, "path": path.display().to_string(), "reason": err.to_string() }),
                ),
            );
        }
    };

    let mut tasks: Vec<CortexCloseoutTaskView> = ledger
        .tasks
        .into_iter()
        .map(|task| {
            let overdue = closeout_task_is_overdue(&task, as_of);
            CortexCloseoutTaskView { task, overdue }
        })
        .collect();
    tasks.sort_by(|left, right| {
        left.task
            .due_at_utc
            .cmp(&right.task.due_at_utc)
            .then(left.task.task_id.cmp(&right.task.task_id))
    });

    let mut status_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut complete = 0usize;
    let mut overdue = 0usize;
    for item in &tasks {
        let status = item.task.status.to_ascii_lowercase();
        *status_counts.entry(status.clone()).or_insert(0) += 1;
        if status == "complete" {
            complete += 1;
        }
        if item.overdue {
            overdue += 1;
        }
    }
    let total = tasks.len();
    let completion_ratio = if total == 0 {
        0.0
    } else {
        ((complete as f64 / total as f64) * 10000.0).round() / 10000.0
    };

    Json(CortexCloseoutTasksResponse {
        schema_version: CORTEX_CLOSEOUT_TRACKING_SCHEMA_VERSION.to_string(),
        generated_at: now_iso(),
        as_of: as_of.to_rfc3339(),
        contribution_id: contribution_id.clone(),
        source_path: path.display().to_string(),
        summary: CortexCloseoutTaskSummary {
            total,
            overdue,
            complete,
            completion_ratio,
            status_counts,
        },
        tasks,
    })
    .into_response()
}

async fn get_cortex_layout_drift_report() -> Json<CortexDriftReport> {
    let persisted =
        load_persisted_shell_contract().unwrap_or_else(|_| default_persisted_shell_contract());
    let defaults = default_persisted_shell_contract();

    let persisted_routes: HashSet<String> = persisted
        .layout_spec
        .navigation_graph
        .entries
        .iter()
        .map(|entry| entry.route_id.clone())
        .collect();
    let default_routes: HashSet<String> = defaults
        .layout_spec
        .navigation_graph
        .entries
        .iter()
        .map(|entry| entry.route_id.clone())
        .collect();

    let mut route_diff = Vec::new();
    for route in persisted_routes.difference(&default_routes) {
        route_diff.push(format!("persisted_only:{route}"));
    }
    for route in default_routes.difference(&persisted_routes) {
        route_diff.push(format!("default_only:{route}"));
    }
    route_diff.sort();

    let persisted_caps: HashSet<String> = persisted
        .view_capabilities
        .iter()
        .map(|cap| format!("{}|{}", cap.route_id, cap.pattern_id))
        .collect();
    let default_caps: HashSet<String> = defaults
        .view_capabilities
        .iter()
        .map(|cap| format!("{}|{}", cap.route_id, cap.pattern_id))
        .collect();
    let mut capability_diff = Vec::new();
    for item in persisted_caps.difference(&default_caps) {
        capability_diff.push(format!("persisted_only:{item}"));
    }
    for item in default_caps.difference(&persisted_caps) {
        capability_diff.push(format!("default_only:{item}"));
    }
    capability_diff.sort();

    let persisted_patterns: HashSet<String> = persisted
        .patterns
        .iter()
        .map(|p| p.pattern_id.clone())
        .collect();
    let default_patterns: HashSet<String> = defaults
        .patterns
        .iter()
        .map(|p| p.pattern_id.clone())
        .collect();
    let mut pattern_diff = Vec::new();
    for item in persisted_patterns.difference(&default_patterns) {
        pattern_diff.push(format!("persisted_only:{item}"));
    }
    for item in default_patterns.difference(&persisted_patterns) {
        pattern_diff.push(format!("default_only:{item}"));
    }
    pattern_diff.sort();

    Json(CortexDriftReport {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        source_of_truth: cortex_ux_source_state().source_of_truth,
        drift_detected: !route_diff.is_empty()
            || !capability_diff.is_empty()
            || !pattern_diff.is_empty(),
        route_diff,
        capability_diff,
        pattern_diff,
    })
}

async fn post_cortex_layout_spec(
    Json(contract): Json<crate::services::cortex_ux::PersistedShellLayoutSpec>,
) -> axum::response::Response {
    if let Err(err) = save_persisted_shell_contract(&contract) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_LAYOUT_SPEC",
            "Persisted layout contract failed validation or could not be saved.",
            Some(json!({ "reason": err })),
        );
    }
    Json(json!({
        "accepted": true,
        "storedAt": now_iso(),
        "layoutId": contract.layout_spec.layout_id,
    }))
    .into_response()
}

async fn post_cortex_viewspec_candidates(
    Json(request): Json<ViewSpecCandidateRequest>,
) -> axum::response::Response {
    if request.intent.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_CANDIDATE_REQUEST",
            "intent is required.",
            None,
        );
    }

    let mut scope = request.scope.unwrap_or_default();
    if scope.space_id.is_none() {
        scope.space_id = request.space_id.clone();
    }
    let mut source_mode = request
        .source_mode
        .unwrap_or_else(|| "hybrid".to_string())
        .to_ascii_lowercase();
    if !matches!(source_mode.as_str(), "human" | "agent" | "hybrid") {
        source_mode = "hybrid".to_string();
    }
    let created_by = request.created_by.unwrap_or_else(|| {
        request
            .actor_id
            .clone()
            .unwrap_or_else(|| "cortex-viewspec-agent".to_string())
    });
    let generation_mode = ViewSpecGenerationMode::parse(request.generation_mode.as_deref());
    let candidate_set = generate_candidate_set(
        scope.clone(),
        &request.intent,
        &request.constraints,
        request.count.unwrap_or(3),
        &created_by,
        &source_mode,
        generation_mode,
        request.candidate_set_id.clone(),
    );

    if let Err(err) = store_viewspec_candidate_set(&scope, &candidate_set).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_CANDIDATE_SET_STORE_FAILED",
            "Failed to persist ViewSpec candidate set.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = append_viewspec_candidate_set_event(
        "viewspec_candidates_generated",
        &candidate_set,
        &created_by,
        json!({
            "candidateSetId": candidate_set.candidate_set_id.clone(),
            "mode": candidate_set.mode.as_str(),
            "candidateCount": candidate_set.candidates.len(),
            "blockedCount": blocked_count(&candidate_set.candidates),
            "actorRole": request.actor_role,
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_EVENT_STORE_FAILED",
            "Failed to append ViewSpec candidate generation event.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecCandidatesResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        candidate_set_id: candidate_set.candidate_set_id,
        blocked_count: blocked_count(&candidate_set.candidates),
        candidates: candidate_set.candidates,
    })
    .into_response()
}

async fn get_cortex_viewspec_candidate_set(
    Path(candidate_set_id): Path<String>,
) -> axum::response::Response {
    if candidate_set_id.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_CANDIDATE_SET_ID",
            "candidate_set_id is required.",
            None,
        );
    }

    match load_viewspec_candidate_set(candidate_set_id.as_str()).await {
        Ok(Some(candidate_set)) => Json(ViewSpecCandidateSetResponse {
            schema_version: "1.0.0".to_string(),
            generated_at: viewspec_now_iso(),
            candidate_set,
        })
        .into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_CANDIDATE_SET_NOT_FOUND",
            "ViewSpec candidate set not found.",
            Some(json!({ "candidateSetId": candidate_set_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_CANDIDATE_SET_LOAD_FAILED",
            "Failed to load ViewSpec candidate set.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_viewspec_candidate_stage(
    Path(candidate_set_id): Path<String>,
    Json(request): Json<ViewSpecCandidateStageRequest>,
) -> axum::response::Response {
    if request.candidate_id.trim().is_empty()
        || request.staged_by.trim().is_empty()
        || request.rationale.trim().is_empty()
        || request.expected_input_hash.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_STAGE_REQUEST",
            "candidateId, stagedBy, rationale, and expectedInputHash are required.",
            None,
        );
    }

    let Some(candidate_set) = (match load_viewspec_candidate_set(candidate_set_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_CANDIDATE_SET_LOAD_FAILED",
                "Failed to load ViewSpec candidate set.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_CANDIDATE_SET_NOT_FOUND",
            "ViewSpec candidate set not found.",
            Some(json!({ "candidateSetId": candidate_set_id })),
        );
    };

    let Some(candidate) = candidate_set
        .candidates
        .iter()
        .find(|candidate| candidate.candidate_id == request.candidate_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_CANDIDATE_NOT_FOUND",
            "ViewSpec candidate not found in candidate set.",
            Some(json!({
                "candidateSetId": candidate_set_id,
                "candidateId": request.candidate_id
            })),
        );
    };

    let recomputed_hash = compute_candidate_input_hash(
        &candidate.view_spec,
        &candidate.generation_trace,
        &candidate_set.mode,
        &candidate_set.intent,
        &candidate_set.constraints,
        &candidate.view_spec.scope,
    );
    if recomputed_hash != candidate.input_hash {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_CANDIDATE_TAMPERED",
            "Candidate input hash does not match persisted candidate content.",
            Some(json!({
                "candidateSetId": candidate_set_id,
                "candidateId": request.candidate_id
            })),
        );
    }
    if request.expected_input_hash != candidate.input_hash {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_STAGE_HASH_MISMATCH",
            "expectedInputHash does not match candidate input hash.",
            Some(json!({
                "candidateSetId": candidate_set_id,
                "candidateId": request.candidate_id
            })),
        );
    }

    let validation = validate_viewspec(&candidate.view_spec);
    if !validation.valid {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_STAGE_BLOCKED",
            "Candidate ViewSpec failed validation and cannot be staged.",
            Some(json!({ "validation": validation })),
        );
    }

    let scope = match store_viewspec(
        &candidate.view_spec,
        "viewspec_candidate_staged",
        &request.staged_by,
        json!({
            "candidateSetId": candidate_set_id,
            "candidateId": request.candidate_id,
            "rationale": request.rationale,
            "expectedInputHash": request.expected_input_hash,
        }),
    )
    .await
    {
        Ok(scope) => scope,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_STORE_FAILED",
                "Failed to persist staged ViewSpec candidate.",
                Some(json!({ "reason": err })),
            );
        }
    };

    if let Err(err) = emit_viewspec_learning_signal(
        "candidate_staged",
        &candidate.view_spec,
        &request.staged_by,
        json!({
            "candidateSetId": candidate_set_id,
            "candidateId": request.candidate_id,
            "rationale": request.rationale,
            "expectedInputHash": request.expected_input_hash
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append candidate staging learning signal.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecCandidateStageResponse {
        accepted: true,
        view_spec_id: candidate.view_spec.view_spec_id.clone(),
        scope_key: scope,
        stored_at: viewspec_now_iso(),
    })
    .into_response()
}

async fn post_cortex_viewspec_learning_signals(
    Json(request): Json<ViewSpecLearningSignalRequest>,
) -> axum::response::Response {
    if request.event_type.trim().is_empty()
        || request.view_spec_id.trim().is_empty()
        || request.actor.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LEARNING_SIGNAL_REQUEST",
            "eventType, viewSpecId, and actor are required.",
            None,
        );
    }

    let event_type = normalize_event_type(&request.event_type);
    if !is_supported_event_type(&event_type) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_VIEWSPEC_LEARNING_EVENT",
            "eventType is not supported for Phase 3 learning.",
            Some(json!({
                "eventType": request.event_type,
            })),
        );
    }

    let requested_space = request
        .space_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string);
    let payload_space = extract_space_id_from_payload(&request.payload);
    let source_spec = match load_viewspec(request.view_spec_id.as_str(), None).await {
        Ok(spec) => spec,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to resolve ViewSpec while deriving learning signal scope.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let scope_space = source_spec
        .as_ref()
        .and_then(|spec| spec.scope.space_id.as_ref())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    if let (Some(a), Some(b)) = (requested_space.as_ref(), payload_space.as_ref()) {
        if a != b {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "VIEWSPEC_LEARNING_SPACE_MISMATCH",
                "Provided spaceId does not match payload space identity.",
                Some(json!({
                    "spaceId": a,
                    "payloadSpaceId": b,
                })),
            );
        }
    }

    if let (Some(a), Some(b)) = (requested_space.as_ref(), scope_space.as_ref()) {
        if a != b {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "VIEWSPEC_LEARNING_SPACE_MISMATCH",
                "Provided spaceId does not match ViewSpec scope.",
                Some(json!({
                    "spaceId": a,
                    "viewSpecScopeSpaceId": b,
                })),
            );
        }
    }

    if let (Some(a), Some(b)) = (payload_space.as_ref(), scope_space.as_ref()) {
        if a != b {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "VIEWSPEC_LEARNING_SPACE_MISMATCH",
                "Payload space identity does not match ViewSpec scope.",
                Some(json!({
                    "payloadSpaceId": a,
                    "viewSpecScopeSpaceId": b,
                })),
            );
        }
    }

    let resolved_space = requested_space.or(payload_space).or(scope_space);
    let Some(space_id) = resolved_space else {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_LEARNING_SPACE_REQUIRED",
            "spaceId is required (request, payload, or ViewSpec scope).",
            Some(json!({ "viewSpecId": request.view_spec_id })),
        );
    };

    let signal = ViewSpecLearningSignal {
        signal_id: request
            .signal_id
            .unwrap_or_else(|| viewspec_learning_signal_id(event_type.as_str())),
        event_type,
        view_spec_id: request.view_spec_id,
        space_id,
        actor: request.actor.trim().to_string(),
        timestamp: request.timestamp.unwrap_or_else(viewspec_now_iso),
        payload: request.payload,
    };

    if let Err(err) = validate_learning_signal(&signal) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LEARNING_SIGNAL",
            "Learning signal validation failed.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = append_viewspec_learning_signal_record(&signal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append ViewSpec learning signal.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecLearningSignalResponse {
        accepted: true,
        signal,
        stored_at: viewspec_now_iso(),
    })
    .into_response()
}

async fn post_cortex_viewspec_spatial_experiment_event(
    Json(request): Json<SpatialExperimentEventRequest>,
) -> axum::response::Response {
    if request.run_id.trim().is_empty()
        || request.space_id.trim().is_empty()
        || request.mode.trim().is_empty()
        || request.surface_variant.trim().is_empty()
        || request.event_type.trim().is_empty()
        || request.timestamp.trim().is_empty()
        || request.host.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPATIAL_EXPERIMENT_EVENT_REQUEST",
            "runId, spaceId, mode, surfaceVariant, eventType, timestamp, and host are required.",
            None,
        );
    }

    if request.run_id.contains('/')
        || request.run_id.contains('\\')
        || request.run_id.contains("..")
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPATIAL_EXPERIMENT_RUN_ID",
            "runId contains invalid path characters.",
            Some(json!({ "runId": request.run_id })),
        );
    }

    let event_type = request.event_type.trim().to_ascii_lowercase();
    if !spatial_experiment_event_supported(event_type.as_str()) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "UNSUPPORTED_SPATIAL_EXPERIMENT_EVENT",
            "eventType is not supported for spatial experiment ingestion.",
            Some(json!({
                "eventType": request.event_type,
            })),
        );
    }

    let date = spatial_experiment_event_date(request.timestamp.as_str());
    let stored_key = spatial_experiment_events_key(date.as_str());
    let event_id = spatial_experiment_event_id(&request.run_id, event_type.as_str());
    let record = SpatialExperimentEventRecord {
        event_id: event_id.clone(),
        run_id: request.run_id.trim().to_string(),
        space_id: request.space_id.trim().to_string(),
        mode: request.mode.trim().to_string(),
        surface_variant: request.surface_variant.trim().to_string(),
        event_type: event_type.clone(),
        timestamp: request.timestamp.trim().to_string(),
        payload: request.payload.clone(),
        build_id: request
            .build_id
            .as_ref()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
        host: request.host.trim().to_string(),
        stored_at: viewspec_now_iso(),
    };

    if let Err(err) = store_append_jsonl(stored_key.as_str(), &record).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SPATIAL_EXPERIMENT_EVENT_STORE_FAILED",
            "Failed to append spatial experiment event.",
            Some(json!({ "reason": err })),
        );
    }

    if event_type == "run_end" {
        let metrics = request
            .payload
            .get("metrics")
            .cloned()
            .and_then(|value| serde_json::from_value::<SpatialExperimentMetrics>(value).ok())
            .unwrap_or_default();

        let complexity_delta = request
            .payload
            .get("complexityDelta")
            .cloned()
            .and_then(|value| {
                serde_json::from_value::<SpatialExperimentComplexityDelta>(value).ok()
            })
            .unwrap_or_else(|| SpatialExperimentComplexityDelta {
                bundle_delta_kb: None,
                runtime_overhead_ms: None,
                adapter_fallback_rate: metrics.adapter_fallback_rate,
            });

        let derived_improvement = {
            let interaction_bonus = metrics.spatial_interaction_count as f64 * 1.5
                + metrics.approval_decision_count as f64 * 2.0;
            let speed_bonus = metrics
                .time_to_first_interaction_ms
                .map(|ms| ((5000_i64 - ms as i64).max(0) as f64) / 1000.0)
                .unwrap_or(0.0);
            let penalty =
                metrics.error_event_count as f64 * 3.0 + metrics.adapter_fallback_rate * 10.0;
            ((interaction_bonus + speed_bonus - penalty) * 100.0).round() / 100.0
        };

        let improvement_score = request
            .payload
            .get("improvementScore")
            .and_then(Value::as_f64)
            .unwrap_or(derived_improvement);

        let recommendation = request
            .payload
            .get("recommendation")
            .and_then(Value::as_str)
            .map(normalize_spatial_recommendation)
            .unwrap_or_else(|| spatial_experiment_recommendation(improvement_score, &metrics));

        let verdict_rationale = request
            .payload
            .get("verdictRationale")
            .and_then(Value::as_str)
            .map(str::to_string)
            .filter(|value| !value.trim().is_empty());

        let event_count =
            match store_read_jsonl::<SpatialExperimentEventRecord>(stored_key.as_str()).await {
                Ok(events) => events
                    .iter()
                    .filter(|item| item.run_id == request.run_id.trim())
                    .count() as u64,
                Err(_) => 0,
            };

        let summary = SpatialExperimentRunSummary {
            schema_version: "1.0.0".to_string(),
            generated_at: viewspec_now_iso(),
            run_id: request.run_id.trim().to_string(),
            space_id: request.space_id.trim().to_string(),
            mode: request.mode.trim().to_string(),
            surface_variant: request.surface_variant.trim().to_string(),
            host: request.host.trim().to_string(),
            build_id: request
                .build_id
                .as_ref()
                .map(|value| value.trim().to_string())
                .filter(|value| !value.is_empty()),
            metrics,
            improvement_score,
            recommendation,
            complexity_delta,
            verdict_rationale,
            event_count,
            event_key: stored_key.clone(),
        };

        if let Err(err) = store_write_json(
            spatial_experiment_run_summary_key(request.run_id.as_str()).as_str(),
            &summary,
        )
        .await
        {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SPATIAL_EXPERIMENT_SUMMARY_STORE_FAILED",
                "Failed to persist spatial experiment run summary.",
                Some(json!({ "reason": err })),
            );
        }
    }

    Json(SpatialExperimentEventResponse {
        accepted: true,
        stored_key,
        event_id,
    })
    .into_response()
}

async fn get_cortex_viewspec_spatial_experiment_run(
    Path(run_id): Path<String>,
) -> axum::response::Response {
    if run_id.trim().is_empty()
        || run_id.contains('/')
        || run_id.contains('\\')
        || run_id.contains("..")
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPATIAL_EXPERIMENT_RUN_ID",
            "runId contains invalid path characters.",
            Some(json!({ "runId": run_id })),
        );
    }

    let key = spatial_experiment_run_summary_key(run_id.as_str());
    match store_read_json::<SpatialExperimentRunSummary>(key.as_str()).await {
        Ok(Some(summary)) => Json(summary).into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "SPATIAL_EXPERIMENT_RUN_NOT_FOUND",
            "Spatial experiment run summary was not found.",
            Some(json!({ "runId": run_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SPATIAL_EXPERIMENT_RUN_LOAD_FAILED",
            "Failed to load spatial experiment run summary.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_cortex_viewspec_learning_profile(
    Path(space_id): Path<String>,
) -> axum::response::Response {
    let space_id = space_id.trim().to_string();
    if space_id.is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LEARNING_SPACE",
            "space_id is required.",
            None,
        );
    }

    match load_viewspec_learning_profile(space_id.as_str()).await {
        Ok(Some(profile)) => Json(ViewSpecLearningProfileResponse {
            schema_version: "1.0.0".to_string(),
            generated_at: viewspec_now_iso(),
            profile,
        })
        .into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_LEARNING_PROFILE_NOT_FOUND",
            "Learning profile not found for space.",
            Some(json!({ "spaceId": space_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_PROFILE_LOAD_FAILED",
            "Failed to load learning profile.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_viewspec_learning_profile_recompute(
    Path(space_id): Path<String>,
    Json(request): Json<ViewSpecLearningRecomputeRequest>,
) -> axum::response::Response {
    let space_id = space_id.trim().to_string();
    if space_id.is_empty() || request.actor.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LEARNING_RECOMPUTE_REQUEST",
            "space_id and actor are required.",
            None,
        );
    }

    let signals = match load_viewspec_learning_signals(space_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_SIGNAL_LOAD_FAILED",
                "Failed to load learning signals for recompute.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let base_profile_version = match load_viewspec_learning_profile(space_id.as_str()).await {
        Ok(Some(profile)) => profile.profile_version,
        Ok(None) => 0,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_PROFILE_LOAD_FAILED",
                "Failed to load prior learning profile.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let (mut profile, mut replay) =
        replay_space_learning_profile(space_id.as_str(), &signals, base_profile_version);
    profile.policy.auto_apply_enabled = false;
    profile.policy.global_merge_enabled = false;
    if profile.policy.apply_mode.trim().is_empty() {
        profile.policy.apply_mode = "advisory".to_string();
    }
    if let Some(reason) = request.reason.as_ref().map(|value| value.trim()) {
        if !reason.is_empty() {
            replay.warnings.push(format!("recompute_reason={reason}"));
        }
    }
    replay
        .warnings
        .push(format!("recompute_actor={}", request.actor.trim()));

    if let Err(err) = store_viewspec_learning_profile(&profile).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_PROFILE_STORE_FAILED",
            "Failed to persist learning profile.",
            Some(json!({ "reason": err })),
        );
    }
    let replay_key = match store_viewspec_learning_replay(space_id.as_str(), &replay).await {
        Ok(key) => key,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_REPLAY_STORE_FAILED",
                "Failed to persist learning replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };

    Json(ViewSpecLearningRecomputeResponse {
        accepted: true,
        profile,
        replay: LearningReplayResult {
            warnings: {
                let mut warnings = replay.warnings.clone();
                warnings.push(format!("replay_key={replay_key}"));
                warnings
            },
            ..replay
        },
    })
    .into_response()
}

async fn post_cortex_viewspec_learning_profile_reset(
    Path(space_id): Path<String>,
    Json(request): Json<ViewSpecLearningResetRequest>,
) -> axum::response::Response {
    let space_id = space_id.trim().to_string();
    if space_id.is_empty() || request.actor.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LEARNING_RESET_REQUEST",
            "space_id and actor are required.",
            None,
        );
    }

    let base_profile_version = match load_viewspec_learning_profile(space_id.as_str()).await {
        Ok(Some(profile)) => profile.profile_version,
        Ok(None) => 0,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_PROFILE_LOAD_FAILED",
                "Failed to load prior learning profile.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let (mut profile, replay) = reset_space_learning_profile(
        space_id.as_str(),
        base_profile_version,
        request.actor.trim(),
        request.reason.as_deref(),
    );
    profile.policy.auto_apply_enabled = false;
    profile.policy.global_merge_enabled = false;
    if profile.policy.apply_mode.trim().is_empty() {
        profile.policy.apply_mode = "advisory".to_string();
    }

    if let Err(err) = store_viewspec_learning_profile(&profile).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_PROFILE_STORE_FAILED",
            "Failed to persist reset learning profile.",
            Some(json!({ "reason": err })),
        );
    }
    let replay_key = match store_viewspec_learning_replay(space_id.as_str(), &replay).await {
        Ok(key) => key,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_REPLAY_STORE_FAILED",
                "Failed to persist reset replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };

    Json(ViewSpecLearningRecomputeResponse {
        accepted: true,
        profile,
        replay: LearningReplayResult {
            warnings: {
                let mut warnings = replay.warnings.clone();
                warnings.push(format!("replay_key={replay_key}"));
                warnings
            },
            ..replay
        },
    })
    .into_response()
}

async fn post_cortex_viewspec_confidence_recompute(
    Path(view_spec_id): Path<String>,
    Json(request): Json<ViewSpecConfidenceRecomputeRequest>,
) -> axum::response::Response {
    if view_spec_id.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_ID",
            "view_spec_id is required.",
            None,
        );
    }

    let Some(spec) = (match load_viewspec(view_spec_id.as_str(), request.scope).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for confidence recompute.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found.",
            Some(json!({ "viewSpecId": view_spec_id })),
        );
    };

    let Some(space_id) = spec
        .scope
        .space_id
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
    else {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_SPACE_REQUIRED",
            "ViewSpec confidence recompute requires a space-scoped ViewSpec.",
            Some(json!({ "viewSpecId": spec.view_spec_id })),
        );
    };

    let Some(profile) = (match load_viewspec_learning_profile(space_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LEARNING_PROFILE_LOAD_FAILED",
                "Failed to load learning profile.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_LEARNING_PROFILE_NOT_FOUND",
            "Learning profile not found for space.",
            Some(json!({ "spaceId": space_id })),
        );
    };

    let confidence = recompute_viewspec_confidence(&spec, &profile);
    Json(ViewSpecConfidenceRecomputeResponse {
        view_spec_id: spec.view_spec_id.clone(),
        space_id,
        confidence,
        profile_version: profile.profile_version,
        signal_count: profile.signal_count,
        policy: profile.policy.clone(),
        persisted: false,
    })
    .into_response()
}

async fn post_cortex_viewspec_validate(
    Json(request): Json<ViewSpecValidateRequest>,
) -> axum::response::Response {
    let validation = validate_viewspec(&request.view_spec);
    Json(ViewSpecValidationResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        validation,
    })
    .into_response()
}

async fn post_cortex_viewspec_compile(
    Json(request): Json<ViewSpecCompileRequest>,
) -> axum::response::Response {
    let validation = validate_viewspec(&request.view_spec);
    if !validation.valid {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_COMPILE_REQUEST",
            "ViewSpec failed validation and cannot be compiled to A2UI.",
            Some(json!({ "validation": validation })),
        );
    }
    let compiled = match compile_viewspec_to_render_surface(&request.view_spec) {
        Ok(value) => value,
        Err(validation) => {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "INVALID_VIEWSPEC_COMPILE_REQUEST",
                "ViewSpec failed validation and cannot be compiled to A2UI.",
                Some(json!({ "validation": validation })),
            );
        }
    };

    Json(ViewSpecCompileResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        validation,
        compiled_surface: compiled,
    })
    .into_response()
}

async fn post_cortex_viewspec_lock(
    Json(request): Json<ViewSpecLockRequest>,
) -> axum::response::Response {
    if request.view_spec_id.trim().is_empty()
        || request.locked_by.trim().is_empty()
        || request.rationale.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_LOCK_REQUEST",
            "viewSpecId, lockedBy, and rationale are required.",
            None,
        );
    }

    let Some(mut spec) = (match load_viewspec(request.view_spec_id.as_str(), request.scope).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec from storage.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found.",
            Some(json!({ "viewSpecId": request.view_spec_id })),
        );
    };

    let structural_change = request.structural_change.unwrap_or(true);
    if structural_change
        && (request
            .approved_by
            .as_deref()
            .unwrap_or_default()
            .trim()
            .is_empty()
            || request
                .approved_at
                .as_deref()
                .unwrap_or_default()
                .trim()
                .is_empty())
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_HITL_REQUIRED",
            "Structural changes require approvedBy and approvedAt metadata.",
            Some(json!({
                "viewSpecId": request.view_spec_id,
                "structuralChange": structural_change
            })),
        );
    }

    spec.lock = Some(ViewSpecLockState {
        locked_by: request.locked_by.clone(),
        locked_at: viewspec_now_iso(),
        rationale: request.rationale.clone(),
        structural_change,
        approved_by: request.approved_by.clone(),
        approved_at: request.approved_at.clone(),
    });

    let scope = match store_viewspec(
        &spec,
        "viewspec_locked",
        &request.locked_by,
        json!({
            "rationale": request.rationale,
            "structuralChange": structural_change,
            "approvedBy": request.approved_by,
            "approvedAt": request.approved_at
        }),
    )
    .await
    {
        Ok(scope) => scope,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_STORE_FAILED",
                "Failed to persist locked ViewSpec.",
                Some(json!({ "reason": err })),
            );
        }
    };

    if let Err(err) = emit_viewspec_learning_signal(
        "viewspec_locked",
        &spec,
        &request.locked_by,
        json!({
            "rationale": request.rationale,
            "structuralChange": structural_change,
            "approvedBy": request.approved_by,
            "approvedAt": request.approved_at
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append lock learning signal.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecAckResponse {
        accepted: true,
        view_spec_id: spec.view_spec_id,
        scope_key: scope,
        stored_at: viewspec_now_iso(),
    })
    .into_response()
}

async fn get_cortex_viewspec(
    Path(view_spec_id): Path<String>,
    Query(query): Query<ViewSpecLookupQuery>,
) -> axum::response::Response {
    let scope = viewspec_scope_from_lookup(&query);
    match load_viewspec(view_spec_id.as_str(), scope).await {
        Ok(Some(spec)) => Json(spec).into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found.",
            Some(json!({ "viewSpecId": view_spec_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LOAD_FAILED",
            "Failed to load ViewSpec from storage.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_viewspec_fork(
    Path(view_spec_id): Path<String>,
    Json(request): Json<ViewSpecForkRequest>,
) -> axum::response::Response {
    if request.fork_reason.trim().is_empty() || request.forked_by.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_FORK_REQUEST",
            "forkReason and forkedBy are required.",
            None,
        );
    }

    let Some(source_spec) = (match load_viewspec(view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load source ViewSpec.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "Source ViewSpec not found.",
            Some(json!({ "viewSpecId": view_spec_id })),
        );
    };

    let mut forked = source_spec.clone();
    forked.view_spec_id = request.new_view_spec_id.unwrap_or_else(|| {
        format!(
            "{}_fork_{}",
            source_spec.view_spec_id,
            Utc::now().timestamp_millis()
        )
    });
    forked.scope = request
        .target_scope
        .unwrap_or_else(|| source_spec.scope.clone());
    forked.lineage.parent_view_spec_id = Some(source_spec.view_spec_id.clone());
    forked.lineage.fork_reason = Some(request.fork_reason.clone());
    forked.provenance.created_by = request.forked_by.clone();
    forked.provenance.created_at = viewspec_now_iso();
    forked.provenance.source_mode = "human".to_string();
    forked.lock = None;
    forked.confidence.score = 0.45;
    forked.confidence.rationale =
        "Forked candidate starts with reset confidence until re-validated.".to_string();

    let validation = validate_viewspec(&forked);
    if !validation.valid {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_FORKED_VIEWSPEC",
            "Forked ViewSpec is invalid.",
            Some(json!({ "validation": validation })),
        );
    }

    let scope = match store_viewspec(
        &forked,
        "viewspec_forked",
        &request.forked_by,
        json!({
            "sourceViewSpecId": source_spec.view_spec_id,
            "forkReason": request.fork_reason
        }),
    )
    .await
    {
        Ok(scope) => scope,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_STORE_FAILED",
                "Failed to persist forked ViewSpec.",
                Some(json!({ "reason": err })),
            );
        }
    };

    if let Err(err) = emit_viewspec_learning_signal(
        "viewspec_forked",
        &forked,
        &request.forked_by,
        json!({
            "sourceViewSpecId": source_spec.view_spec_id,
            "forkReason": request.fork_reason
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append fork learning signal.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecAckResponse {
        accepted: true,
        view_spec_id: forked.view_spec_id,
        scope_key: scope,
        stored_at: viewspec_now_iso(),
    })
    .into_response()
}

async fn post_cortex_viewspec_propose(
    Path(view_spec_id): Path<String>,
    Json(request): Json<ViewSpecProposeRequest>,
) -> axum::response::Response {
    if request.proposed_by.trim().is_empty() || request.rationale.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_PROPOSAL_REQUEST",
            "proposedBy and rationale are required.",
            None,
        );
    }

    let Some(spec) = (match load_viewspec(view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for proposal.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found for proposal.",
            Some(json!({ "viewSpecId": view_spec_id })),
        );
    };

    let validation = validate_viewspec(&spec);
    if !validation.valid {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_PROPOSAL_BLOCKED",
            "Proposal blocked because ViewSpec did not pass validation.",
            Some(json!({ "validation": validation })),
        );
    }

    let proposal = ViewSpecProposalEnvelope {
        proposal_id: format!("viewspec_proposal_{}", Utc::now().timestamp_millis()),
        view_spec_id: spec.view_spec_id.clone(),
        scope_key: scope_key(&spec.scope),
        proposed_by: request.proposed_by.clone(),
        rationale: request.rationale.clone(),
        created_at: viewspec_now_iso(),
        status: ViewSpecProposalStatus::Staged,
        review: None,
        decision: None,
        merge: None,
        governance_ref: None,
    };
    if let Err(err) = store_viewspec_proposal(&spec.scope, &proposal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_STORE_FAILED",
            "Failed to persist ViewSpec proposal.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = append_viewspec_proposal_event(
        "viewspec_proposed",
        &proposal,
        &request.proposed_by,
        json!({
            "proposalId": proposal.proposal_id.clone(),
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_EVENT_STORE_FAILED",
            "Failed to append ViewSpec proposal event.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = append_viewspec_event(
        "viewspec_proposed",
        &spec,
        &request.proposed_by,
        json!({
            "proposalId": proposal.proposal_id.clone(),
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_EVENT_STORE_FAILED",
            "Failed to append ViewSpec proposal event.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = emit_viewspec_learning_signal(
        "viewspec_proposed",
        &spec,
        &request.proposed_by,
        json!({
            "proposalId": proposal.proposal_id.clone(),
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append proposal learning signal.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecProposalResponse {
        accepted: true,
        proposal,
    })
    .into_response()
}

async fn get_cortex_viewspec_proposals(
    Query(query): Query<ViewSpecProposalListQuery>,
) -> axum::response::Response {
    let mut proposals = match list_viewspec_proposals().await {
        Ok(rows) => rows,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load ViewSpec proposals.",
                Some(json!({ "reason": err })),
            );
        }
    };

    if let Some(scope_key_value) = query
        .scope_key
        .as_ref()
        .map(|value| value.trim().to_string())
    {
        if !scope_key_value.is_empty() {
            proposals.retain(|proposal| proposal.scope_key == scope_key_value);
        }
    }
    if let Some(status) = query.status.as_ref().map(|value| value.trim().to_string()) {
        if !status.is_empty() {
            proposals.retain(|proposal| {
                viewspec_proposal_status_matches(status.as_str(), &proposal.status)
            });
        }
    }
    proposals.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    if let Some(limit) = query.limit {
        proposals.truncate(limit.min(proposals.len()));
    }

    Json(ViewSpecProposalListResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        proposals,
    })
    .into_response()
}

async fn get_cortex_viewspec_proposal(Path(proposal_id): Path<String>) -> axum::response::Response {
    match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(Some(proposal)) => Json(proposal).into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_LOAD_FAILED",
            "Failed to load ViewSpec proposal.",
            Some(json!({ "reason": err })),
        ),
    }
}

fn governance_ref_from_gate(gate: &ViewSpecGovernanceDecisionGate) -> ViewSpecGovernanceRef {
    ViewSpecGovernanceRef {
        gate_level: gate.gate_level.clone(),
        gate_status: gate.gate_status.clone(),
        decision_gate_id: gate.decision_gate_id.clone(),
        replay_contract_ref: gate.replay_contract_ref.clone(),
        source_of_truth: gate.source_of_truth.clone(),
        degraded_reason: gate.degraded_reason.clone(),
    }
}

async fn post_cortex_viewspec_proposal_review(
    headers: HeaderMap,
    Path(proposal_id): Path<String>,
    Json(request): Json<ViewSpecProposalReviewRequest>,
) -> axum::response::Response {
    if request.reviewed_by.trim().is_empty() || request.summary.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_PROPOSAL_REVIEW_REQUEST",
            "reviewedBy and summary are required.",
            None,
        );
    }

    let Some(mut proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load ViewSpec proposal for review.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };

    if !matches!(
        proposal.status,
        ViewSpecProposalStatus::Staged | ViewSpecProposalStatus::UnderReview
    ) {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_PROPOSAL_REVIEW_INVALID_STATE",
            "Proposal review is only allowed from staged or under_review states.",
            Some(json!({
                "proposalId": proposal_id,
                "status": viewspec_proposal_status_name(&proposal.status)
            })),
        );
    }

    let Some(spec) = (match load_viewspec(proposal.view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for proposal review.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found for proposal review.",
            Some(json!({ "viewSpecId": proposal.view_spec_id })),
        );
    };
    let space_id = spec
        .scope
        .space_id
        .clone()
        .unwrap_or_else(|| "space-default".to_string());
    let gate = match resolve_viewspec_governance_gate(
        &headers,
        proposal.proposal_id.as_str(),
        &space_id,
        "governance:viewspec:review",
        "informational",
        Some(request.reviewed_by.as_str()),
        false,
    )
    .await
    {
        Ok(value) => value,
        Err(response) => return response,
    };

    proposal.review = Some(ViewSpecProposalReviewRecord {
        reviewed_by: request.reviewed_by.clone(),
        reviewed_at: viewspec_now_iso(),
        summary: request.summary.clone(),
        checks: request.checks.clone(),
        approved: request.approved,
    });
    proposal.status = if request.approved {
        ViewSpecProposalStatus::Approved
    } else {
        ViewSpecProposalStatus::UnderReview
    };
    proposal.governance_ref = Some(governance_ref_from_gate(&gate));

    if let Err(err) = store_viewspec_proposal(&spec.scope, &proposal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_STORE_FAILED",
            "Failed to persist proposal review.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_proposal_event(
        "viewspec_proposal_reviewed",
        &proposal,
        &request.reviewed_by,
        json!({
            "approved": request.approved,
            "summary": request.summary,
            "checks": request.checks
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_EVENT_STORE_FAILED",
            "Failed to append proposal review event.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_governance_event(
        "governance:viewspec:review",
        &proposal,
        &request.reviewed_by,
        json!({
            "gateLevel": gate.gate_level,
            "gateStatus": gate.gate_status,
            "decisionGateId": gate.decision_gate_id,
            "sourceOfTruth": gate.source_of_truth,
            "degradedReason": gate.degraded_reason
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_GOVERNANCE_EVENT_STORE_FAILED",
            "Failed to append governance review evidence.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecProposalActionResponse {
        accepted: true,
        proposal,
        gate_level: gate.gate_level,
        gate_status: gate.gate_status,
        decision_gate_id: gate.decision_gate_id,
        replay_contract_ref: gate.replay_contract_ref,
        source_of_truth: gate.source_of_truth,
        degraded_reason: gate.degraded_reason,
    })
    .into_response()
}

async fn post_cortex_viewspec_proposal_ratify(
    headers: HeaderMap,
    Path(proposal_id): Path<String>,
    Json(request): Json<ViewSpecProposalDecisionRequest>,
) -> axum::response::Response {
    if request.decided_by.trim().is_empty() || request.rationale.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_PROPOSAL_RATIFY_REQUEST",
            "decidedBy and rationale are required.",
            None,
        );
    }
    let Some(mut proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load ViewSpec proposal for ratification.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };
    if !matches!(proposal.status, ViewSpecProposalStatus::Approved) {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_PROPOSAL_RATIFY_INVALID_STATE",
            "Proposal can only be ratified from approved state.",
            Some(json!({
                "proposalId": proposal_id,
                "status": viewspec_proposal_status_name(&proposal.status)
            })),
        );
    }
    if proposal
        .proposed_by
        .trim()
        .eq_ignore_ascii_case(request.decided_by.trim())
    {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "VIEWSPEC_PROPOSAL_SELF_RATIFY_BLOCKED",
            "Proposer cannot self-ratify a ViewSpec proposal.",
            Some(json!({ "proposalId": proposal_id, "proposedBy": proposal.proposed_by })),
        );
    }

    let Some(spec) = (match load_viewspec(proposal.view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for proposal ratification.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found for proposal ratification.",
            Some(json!({ "viewSpecId": proposal.view_spec_id })),
        );
    };
    let space_id = spec
        .scope
        .space_id
        .clone()
        .unwrap_or_else(|| "space-default".to_string());
    let gate = match resolve_viewspec_governance_gate(
        &headers,
        proposal.proposal_id.as_str(),
        &space_id,
        "governance:viewspec:ratify",
        "release_blocker",
        Some(request.decided_by.as_str()),
        true,
    )
    .await
    {
        Ok(value) => value,
        Err(response) => return response,
    };

    proposal.decision = Some(ViewSpecProposalDecisionRecord {
        decided_by: request.decided_by.clone(),
        decided_at: viewspec_now_iso(),
        decision: "ratified".to_string(),
        rationale: request.rationale.clone(),
    });
    proposal.status = ViewSpecProposalStatus::Ratified;
    proposal.governance_ref = Some(governance_ref_from_gate(&gate));

    if let Err(err) = store_viewspec_proposal(&spec.scope, &proposal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_STORE_FAILED",
            "Failed to persist proposal ratification.",
            Some(json!({ "reason": err })),
        );
    }

    let mut all_proposals = match list_viewspec_proposals().await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to enumerate proposals for supersede handling.",
                Some(json!({ "reason": err })),
            );
        }
    };
    for existing in all_proposals.iter_mut() {
        if existing.proposal_id == proposal.proposal_id {
            continue;
        }
        if existing.scope_key != proposal.scope_key {
            continue;
        }
        if !matches!(existing.status, ViewSpecProposalStatus::Ratified) {
            continue;
        }
        if let Some(existing_scope) = load_viewspec(existing.view_spec_id.as_str(), None)
            .await
            .ok()
            .flatten()
            .map(|value| value.scope)
        {
            existing.status = ViewSpecProposalStatus::Superseded;
            existing.decision = Some(ViewSpecProposalDecisionRecord {
                decided_by: request.decided_by.clone(),
                decided_at: viewspec_now_iso(),
                decision: "superseded".to_string(),
                rationale: format!("Superseded by {}", proposal.proposal_id),
            });
            existing.governance_ref = Some(governance_ref_from_gate(&gate));
            let _ = store_viewspec_proposal(&existing_scope, existing).await;
        }
    }

    let adoption = ViewSpecScopeAdoptionRecord {
        scope_key: proposal.scope_key.clone(),
        active_view_spec_id: proposal.view_spec_id.clone(),
        adopted_from_proposal_id: proposal.proposal_id.clone(),
        adopted_at: viewspec_now_iso(),
        adopted_by: request.decided_by.clone(),
    };
    if let Err(err) = store_viewspec_scope_adoption(&adoption).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_ACTIVE_SCOPE_STORE_FAILED",
            "Failed to persist active scope adoption pointer.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = append_viewspec_proposal_event(
        "viewspec_proposal_ratified",
        &proposal,
        &request.decided_by,
        json!({
            "rationale": request.rationale,
            "activeScope": adoption.scope_key,
            "activeViewSpecId": adoption.active_view_spec_id
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_EVENT_STORE_FAILED",
            "Failed to append proposal ratification event.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_governance_event(
        "governance:viewspec:ratify",
        &proposal,
        &request.decided_by,
        json!({
            "gateLevel": gate.gate_level,
            "gateStatus": gate.gate_status,
            "decisionGateId": gate.decision_gate_id,
            "sourceOfTruth": gate.source_of_truth,
            "degradedReason": gate.degraded_reason
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_GOVERNANCE_EVENT_STORE_FAILED",
            "Failed to append governance ratification evidence.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = emit_viewspec_learning_signal(
        "proposal_ratified",
        &spec,
        &request.decided_by,
        json!({
            "proposalId": proposal.proposal_id,
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append ratification learning signal.",
            Some(json!({ "reason": err })),
        );
    }
    let (replay, digest) = match build_viewspec_replay_and_digest(&proposal).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_REPLAY_BUILD_FAILED",
                "Failed to build replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };
    if let Err(err) =
        store_viewspec_replay_artifact(proposal.proposal_id.as_str(), &replay, &digest).await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_REPLAY_STORE_FAILED",
            "Failed to persist replay and digest artifacts.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecProposalActionResponse {
        accepted: true,
        proposal,
        gate_level: gate.gate_level,
        gate_status: gate.gate_status,
        decision_gate_id: gate.decision_gate_id,
        replay_contract_ref: gate.replay_contract_ref,
        source_of_truth: gate.source_of_truth,
        degraded_reason: gate.degraded_reason,
    })
    .into_response()
}

async fn post_cortex_viewspec_proposal_reject(
    headers: HeaderMap,
    Path(proposal_id): Path<String>,
    Json(request): Json<ViewSpecProposalDecisionRequest>,
) -> axum::response::Response {
    if request.decided_by.trim().is_empty() || request.rationale.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_PROPOSAL_REJECT_REQUEST",
            "decidedBy and rationale are required.",
            None,
        );
    }
    let Some(mut proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load ViewSpec proposal for rejection.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };
    if !matches!(
        proposal.status,
        ViewSpecProposalStatus::Staged
            | ViewSpecProposalStatus::UnderReview
            | ViewSpecProposalStatus::Approved
    ) {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_PROPOSAL_REJECT_INVALID_STATE",
            "Proposal reject is only allowed from staged, under_review, or approved states.",
            Some(json!({
                "proposalId": proposal_id,
                "status": viewspec_proposal_status_name(&proposal.status)
            })),
        );
    }

    let Some(spec) = (match load_viewspec(proposal.view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for proposal rejection.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found for proposal rejection.",
            Some(json!({ "viewSpecId": proposal.view_spec_id })),
        );
    };
    let space_id = spec
        .scope
        .space_id
        .clone()
        .unwrap_or_else(|| "space-default".to_string());
    let gate = match resolve_viewspec_governance_gate(
        &headers,
        proposal.proposal_id.as_str(),
        &space_id,
        "governance:viewspec:reject",
        "informational",
        Some(request.decided_by.as_str()),
        false,
    )
    .await
    {
        Ok(value) => value,
        Err(response) => return response,
    };

    proposal.decision = Some(ViewSpecProposalDecisionRecord {
        decided_by: request.decided_by.clone(),
        decided_at: viewspec_now_iso(),
        decision: "rejected".to_string(),
        rationale: request.rationale.clone(),
    });
    proposal.status = ViewSpecProposalStatus::Rejected;
    proposal.governance_ref = Some(governance_ref_from_gate(&gate));

    if let Err(err) = store_viewspec_proposal(&spec.scope, &proposal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_STORE_FAILED",
            "Failed to persist proposal rejection.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_proposal_event(
        "viewspec_proposal_rejected",
        &proposal,
        &request.decided_by,
        json!({ "rationale": request.rationale }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_EVENT_STORE_FAILED",
            "Failed to append proposal rejection event.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_governance_event(
        "governance:viewspec:reject",
        &proposal,
        &request.decided_by,
        json!({
            "gateLevel": gate.gate_level,
            "gateStatus": gate.gate_status,
            "decisionGateId": gate.decision_gate_id,
            "sourceOfTruth": gate.source_of_truth,
            "degradedReason": gate.degraded_reason
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_GOVERNANCE_EVENT_STORE_FAILED",
            "Failed to append governance rejection evidence.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = emit_viewspec_learning_signal(
        "proposal_rejected",
        &spec,
        &request.decided_by,
        json!({
            "proposalId": proposal.proposal_id,
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_LEARNING_SIGNAL_STORE_FAILED",
            "Failed to append proposal rejection learning signal.",
            Some(json!({ "reason": err })),
        );
    }
    let (replay, digest) = match build_viewspec_replay_and_digest(&proposal).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_REPLAY_BUILD_FAILED",
                "Failed to build replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };
    if let Err(err) =
        store_viewspec_replay_artifact(proposal.proposal_id.as_str(), &replay, &digest).await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_REPLAY_STORE_FAILED",
            "Failed to persist replay and digest artifacts.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecProposalActionResponse {
        accepted: true,
        proposal,
        gate_level: gate.gate_level,
        gate_status: gate.gate_status,
        decision_gate_id: gate.decision_gate_id,
        replay_contract_ref: gate.replay_contract_ref,
        source_of_truth: gate.source_of_truth,
        degraded_reason: gate.degraded_reason,
    })
    .into_response()
}

async fn post_cortex_viewspec_proposal_merge(
    headers: HeaderMap,
    Path(proposal_id): Path<String>,
    Json(request): Json<ViewSpecProposalMergeRequest>,
) -> axum::response::Response {
    if request.merged_by.trim().is_empty()
        || request.merged_with_proposal_id.trim().is_empty()
        || request.rationale.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_VIEWSPEC_PROPOSAL_MERGE_REQUEST",
            "mergedBy, mergedWithProposalId, and rationale are required.",
            None,
        );
    }
    if request
        .merged_with_proposal_id
        .trim()
        .eq_ignore_ascii_case(proposal_id.trim())
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "VIEWSPEC_PROPOSAL_MERGE_SELF",
            "Proposal cannot merge with itself.",
            Some(json!({ "proposalId": proposal_id })),
        );
    }

    let Some(mut proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load source proposal for merge.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "Source ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };
    let Some(target) = (match load_viewspec_proposal(request.merged_with_proposal_id.as_str()).await
    {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load target proposal for merge.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "Merge target proposal not found.",
            Some(json!({ "proposalId": request.merged_with_proposal_id })),
        );
    };
    if proposal.scope_key != target.scope_key {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_PROPOSAL_MERGE_SCOPE_MISMATCH",
            "Proposals can only merge within the same scope.",
            Some(json!({
                "sourceScope": proposal.scope_key,
                "targetScope": target.scope_key
            })),
        );
    }
    if !matches!(
        proposal.status,
        ViewSpecProposalStatus::Approved | ViewSpecProposalStatus::Ratified
    ) {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "VIEWSPEC_PROPOSAL_MERGE_INVALID_STATE",
            "Source proposal must be approved or ratified before merge.",
            Some(json!({
                "proposalId": proposal_id,
                "status": viewspec_proposal_status_name(&proposal.status)
            })),
        );
    }

    let Some(spec) = (match load_viewspec(proposal.view_spec_id.as_str(), None).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_LOAD_FAILED",
                "Failed to load ViewSpec for proposal merge.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_NOT_FOUND",
            "ViewSpec not found for proposal merge.",
            Some(json!({ "viewSpecId": proposal.view_spec_id })),
        );
    };
    let space_id = spec
        .scope
        .space_id
        .clone()
        .unwrap_or_else(|| "space-default".to_string());
    let gate = match resolve_viewspec_governance_gate(
        &headers,
        proposal.proposal_id.as_str(),
        &space_id,
        "governance:viewspec:merge",
        "release_blocker",
        Some(request.merged_by.as_str()),
        true,
    )
    .await
    {
        Ok(value) => value,
        Err(response) => return response,
    };

    proposal.merge = Some(ViewSpecProposalMergeRecord {
        merged_by: request.merged_by.clone(),
        merged_at: viewspec_now_iso(),
        merged_with_proposal_id: request.merged_with_proposal_id.clone(),
        rationale: request.rationale.clone(),
    });
    proposal.status = ViewSpecProposalStatus::Merged;
    proposal.governance_ref = Some(governance_ref_from_gate(&gate));

    if let Err(err) = store_viewspec_proposal(&spec.scope, &proposal).await {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_STORE_FAILED",
            "Failed to persist proposal merge.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_proposal_event(
        "viewspec_proposal_merged",
        &proposal,
        &request.merged_by,
        json!({
            "mergedWithProposalId": request.merged_with_proposal_id,
            "rationale": request.rationale
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_PROPOSAL_EVENT_STORE_FAILED",
            "Failed to append proposal merge event.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = append_viewspec_governance_event(
        "governance:viewspec:merge",
        &proposal,
        &request.merged_by,
        json!({
            "gateLevel": gate.gate_level,
            "gateStatus": gate.gate_status,
            "decisionGateId": gate.decision_gate_id,
            "sourceOfTruth": gate.source_of_truth,
            "degradedReason": gate.degraded_reason
        }),
    )
    .await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_GOVERNANCE_EVENT_STORE_FAILED",
            "Failed to append governance merge evidence.",
            Some(json!({ "reason": err })),
        );
    }
    let (replay, digest) = match build_viewspec_replay_and_digest(&proposal).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_REPLAY_BUILD_FAILED",
                "Failed to build replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };
    if let Err(err) =
        store_viewspec_replay_artifact(proposal.proposal_id.as_str(), &replay, &digest).await
    {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_REPLAY_STORE_FAILED",
            "Failed to persist replay and digest artifacts.",
            Some(json!({ "reason": err })),
        );
    }

    Json(ViewSpecProposalActionResponse {
        accepted: true,
        proposal,
        gate_level: gate.gate_level,
        gate_status: gate.gate_status,
        decision_gate_id: gate.decision_gate_id,
        replay_contract_ref: gate.replay_contract_ref,
        source_of_truth: gate.source_of_truth,
        degraded_reason: gate.degraded_reason,
    })
    .into_response()
}

async fn get_cortex_viewspec_active(
    Query(query): Query<ViewSpecActiveQuery>,
) -> axum::response::Response {
    if let Some(scope_key_value) = query
        .scope_key
        .as_ref()
        .map(|value| value.trim().to_string())
    {
        if scope_key_value.is_empty() {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "INVALID_VIEWSPEC_ACTIVE_QUERY",
                "scopeKey cannot be blank.",
                None,
            );
        }
        return match load_viewspec_active_scope(scope_key_value.as_str()).await {
            Ok(Some(record)) => Json(ViewSpecActiveResponse {
                schema_version: "1.0.0".to_string(),
                generated_at: viewspec_now_iso(),
                active: vec![record],
            })
            .into_response(),
            Ok(None) => Json(ViewSpecActiveResponse {
                schema_version: "1.0.0".to_string(),
                generated_at: viewspec_now_iso(),
                active: Vec::new(),
            })
            .into_response(),
            Err(err) => cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_ACTIVE_LOAD_FAILED",
                "Failed to load active scope record.",
                Some(json!({ "reason": err })),
            ),
        };
    }

    match list_viewspec_active_scopes().await {
        Ok(active) => Json(ViewSpecActiveResponse {
            schema_version: "1.0.0".to_string(),
            generated_at: viewspec_now_iso(),
            active,
        })
        .into_response(),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "VIEWSPEC_ACTIVE_LOAD_FAILED",
            "Failed to load active scope records.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_cortex_viewspec_proposal_replay(
    Path(proposal_id): Path<String>,
) -> axum::response::Response {
    let Some(proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load proposal replay.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };

    let replay = match load_viewspec_latest_replay_artifact(proposal.proposal_id.as_str()).await {
        Ok(Some(value)) => value,
        Ok(None) => {
            let (replay, digest) = match build_viewspec_replay_and_digest(&proposal).await {
                Ok(value) => value,
                Err(err) => {
                    return cortex_ux_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "VIEWSPEC_REPLAY_BUILD_FAILED",
                        "Failed to build replay artifact.",
                        Some(json!({ "reason": err })),
                    );
                }
            };
            if let Err(err) =
                store_viewspec_replay_artifact(proposal.proposal_id.as_str(), &replay, &digest)
                    .await
            {
                return cortex_ux_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "VIEWSPEC_REPLAY_STORE_FAILED",
                    "Failed to persist replay artifact.",
                    Some(json!({ "reason": err })),
                );
            }
            replay
        }
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_REPLAY_LOAD_FAILED",
                "Failed to load replay artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };

    Json(ViewSpecReplayResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        replay,
    })
    .into_response()
}

async fn get_cortex_viewspec_proposal_digest(
    Path(proposal_id): Path<String>,
) -> axum::response::Response {
    let Some(proposal) = (match load_viewspec_proposal(proposal_id.as_str()).await {
        Ok(value) => value,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_PROPOSAL_LOAD_FAILED",
                "Failed to load proposal digest.",
                Some(json!({ "reason": err })),
            );
        }
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "VIEWSPEC_PROPOSAL_NOT_FOUND",
            "ViewSpec proposal not found.",
            Some(json!({ "proposalId": proposal_id })),
        );
    };

    let key = viewspec_replay_digest_latest_key(proposal.proposal_id.as_str());
    let digest = match store_read_json::<ViewSpecDigestArtifact>(key.as_str()).await {
        Ok(Some(value)) => value,
        Ok(None) => {
            let (replay, digest) = match build_viewspec_replay_and_digest(&proposal).await {
                Ok(value) => value,
                Err(err) => {
                    return cortex_ux_error(
                        StatusCode::INTERNAL_SERVER_ERROR,
                        "VIEWSPEC_REPLAY_BUILD_FAILED",
                        "Failed to build replay digest.",
                        Some(json!({ "reason": err })),
                    );
                }
            };
            if let Err(err) =
                store_viewspec_replay_artifact(proposal.proposal_id.as_str(), &replay, &digest)
                    .await
            {
                return cortex_ux_error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "VIEWSPEC_REPLAY_STORE_FAILED",
                    "Failed to persist replay digest.",
                    Some(json!({ "reason": err })),
                );
            }
            digest
        }
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "VIEWSPEC_DIGEST_LOAD_FAILED",
                "Failed to load digest artifact.",
                Some(json!({ "reason": err })),
            );
        }
    };

    Json(ViewSpecDigestResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: viewspec_now_iso(),
        digest,
    })
    .into_response()
}

async fn get_cortex_capability_matrix() -> Json<CortexCapabilityMatrixResponse> {
    let persisted =
        load_persisted_shell_contract().unwrap_or_else(|_| default_persisted_shell_contract());
    Json(CortexCapabilityMatrixResponse {
        schema_version: persisted.schema_version,
        generated_at: persisted.updated_at,
        layout_spec: resolve_shell_layout_spec(),
        view_capabilities: resolve_view_capability_manifests(),
        patterns: resolve_pattern_contracts(),
        matrix: resolve_capability_matrix(),
    })
}

async fn post_cortex_layout_evaluate(
    Json(request): Json<UxLayoutEvaluationRequest>,
) -> axum::response::Response {
    if request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_UX_EVALUATION_REQUEST",
            "Candidate ID, route ID, and view capability ID are required.",
            Some(json!({
                "candidateId": request.candidate_id,
                "routeId": request.route_id,
                "viewCapabilityId": request.view_capability_id
            })),
        );
    }

    let evaluation = evaluate_cuqs(request);
    if let Err(err) = append_json_line(&cortex_ux_evaluation_log_path(), &evaluation) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_EVALUATION_PERSIST_FAILED",
            "Failed to persist UX evaluation event.",
            Some(json!({ "reason": err })),
        );
    }

    if evaluation.promotion_status == "eligible_auto" {
        let _ = update_feedback_status_for_candidate(
            &evaluation.route_id,
            &evaluation.view_capability_id,
            UX_STATUS_CANDIDATE,
            None,
            None,
        );
    }

    let promotion_decision = if evaluation.promotion_status == "eligible_hitl_approved" {
        match (
            evaluation.candidate_id.clone(),
            evaluation.route_id.clone(),
            evaluation.view_capability_id.clone(),
            evaluation.approved_by.clone(),
            evaluation.approval_rationale.clone(),
            evaluation.approved_at.clone(),
        ) {
            (
                candidate_id,
                route_id,
                view_capability_id,
                Some(approved_by),
                Some(rationale),
                Some(timestamp),
            ) => {
                let decision = UxPromotionDecision {
                    decision_id: format!("ux_promotion_{}", Utc::now().timestamp_millis()),
                    candidate_id,
                    route_id: route_id.clone(),
                    view_capability_id: view_capability_id.clone(),
                    promotion_action: "promote_structural_candidate".to_string(),
                    approved_by,
                    rationale,
                    timestamp,
                };
                let _ = append_json_line(&cortex_ux_promotion_log_path(), &decision);
                let _ = update_feedback_status_for_candidate(
                    &route_id,
                    &view_capability_id,
                    UX_STATUS_APPROVED,
                    None,
                    None,
                );
                Some(decision)
            }
            _ => None,
        }
    } else {
        None
    };

    Json(CortexLayoutEvaluationResponse {
        evaluation,
        promotion_decision,
    })
    .into_response()
}

async fn post_cortex_feedback_ux(
    Json(mut event): Json<UxFeedbackEvent>,
) -> axum::response::Response {
    if event.event_id.trim().is_empty() {
        event.event_id = format!("ux_feedback_{}", Utc::now().timestamp_millis());
    }
    if event.timestamp.trim().is_empty() {
        event.timestamp = now_iso();
    }
    if event.route_id.trim().is_empty()
        || event.view_id.trim().is_empty()
        || event.friction_tag.trim().is_empty()
        || event.severity.trim().is_empty()
        || event.session_id.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_UX_FEEDBACK_EVENT",
            "routeId, viewId, frictionTag, severity, and sessionId are required.",
            Some(json!({
                "routeId": event.route_id,
                "viewId": event.view_id,
                "frictionTag": event.friction_tag,
                "severity": event.severity,
                "sessionId": event.session_id
            })),
        );
    }

    if let Err(err) = append_json_line(&cortex_ux_feedback_log_path(), &event) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_FEEDBACK_PERSIST_FAILED",
            "Failed to persist UX feedback event.",
            Some(json!({ "reason": err })),
        );
    }

    if let Err(err) = upsert_feedback_queue_item(&event) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_FEEDBACK_QUEUE_PERSIST_FAILED",
            "Failed to persist UX feedback queue state.",
            Some(json!({ "reason": err })),
        );
    }

    Json(CortexFeedbackAck {
        accepted: true,
        event_id: event.event_id,
        stored_at: event.timestamp,
    })
    .into_response()
}

async fn get_cortex_feedback_ux(
    Query(query): Query<CortexFeedbackQuery>,
) -> Json<CortexFeedbackQueueResponse> {
    let mut items = read_feedback_queue_items();
    if let Some(route_id) = query.route_id {
        items.retain(|item| item.route_id == route_id);
    }
    if let Some(status) = query.status {
        items.retain(|item| item.status == status);
    }
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    if let Some(limit) = query.limit {
        if items.len() > limit {
            items.truncate(limit);
        }
    }
    Json(CortexFeedbackQueueResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        items,
    })
}

async fn post_cortex_feedback_triage(
    Json(request): Json<CortexFeedbackTriageRequest>,
) -> axum::response::Response {
    if request.queue_id.trim().is_empty() || request.status.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_UX_TRIAGE_REQUEST",
            "queueId and status are required.",
            None,
        );
    }
    if !valid_feedback_status(request.status.as_str()) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_UX_TRIAGE_STATUS",
            "Unsupported UX triage status.",
            Some(json!({ "status": request.status })),
        );
    }
    if let Some(date) = request.baseline_metric_date.as_deref() {
        if !parse_metric_date(date) {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "INVALID_BASELINE_METRIC_DATE",
                "baselineMetricDate must be RFC3339.",
                Some(json!({ "baselineMetricDate": date })),
            );
        }
    }
    if let Some(date) = request.post_release_metric_date.as_deref() {
        if !parse_metric_date(date) {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "INVALID_POST_RELEASE_METRIC_DATE",
                "postReleaseMetricDate must be RFC3339.",
                Some(json!({ "postReleaseMetricDate": date })),
            );
        }
    }

    match update_feedback_queue_item(&request) {
        Ok(Some(item)) => Json(CortexFeedbackTriageResponse {
            accepted: true,
            item,
        })
        .into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "UX_QUEUE_ITEM_NOT_FOUND",
            "No queue item found for queueId.",
            Some(json!({ "queueId": request.queue_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_TRIAGE_PERSIST_FAILED",
            "Failed to persist triage update.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_feedback_promote_candidate(
    headers: HeaderMap,
    Json(request): Json<CortexPromoteCandidateRequest>,
) -> axum::response::Response {
    if request.queue_id.trim().is_empty()
        || request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
        || request.approved_by.trim().is_empty()
        || request.rationale.trim().is_empty()
        || request.approved_at.trim().is_empty()
        || request.baseline_metric_date.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_PROMOTE_CANDIDATE_REQUEST",
            "queueId, candidateId, routeId, viewCapabilityId, approvedBy, rationale, approvedAt, and baselineMetricDate are required.",
            None,
        );
    }
    if !parse_metric_date(&request.approved_at) || !parse_metric_date(&request.baseline_metric_date)
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_PROMOTE_CANDIDATE_DATES",
            "approvedAt and baselineMetricDate must be RFC3339.",
            None,
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    let updated = match transition_feedback_queue_item(
        &request.queue_id,
        UX_STATUS_APPROVED,
        &actor_id,
        Some(format!("Promoted candidate {}", request.candidate_id)),
        Some(&request.baseline_metric_date),
        None,
    ) {
        Ok(Some(item)) => item,
        Ok(None) => {
            return cortex_ux_error(
                StatusCode::NOT_FOUND,
                "UX_QUEUE_ITEM_NOT_FOUND",
                "No queue item found for queueId.",
                Some(json!({ "queueId": request.queue_id })),
            );
        }
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UX_PROMOTE_CANDIDATE_PERSIST_FAILED",
                "Failed to persist candidate promotion state.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let decision = UxPromotionDecision {
        decision_id: format!("ux_promotion_{}", Utc::now().timestamp_millis()),
        candidate_id: request.candidate_id,
        route_id: request.route_id,
        view_capability_id: request.view_capability_id,
        promotion_action: "approve_structural_promotion".to_string(),
        approved_by: request.approved_by,
        rationale: request.rationale,
        timestamp: request.approved_at,
    };
    if let Err(err) = append_json_line(&cortex_ux_promotion_log_path(), &decision) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROMOTION_APPROVAL_PERSIST_FAILED",
            "Failed to persist promotion approval.",
            Some(json!({ "reason": err })),
        );
    }
    Json(json!({
        "accepted": true,
        "item": updated,
        "decision": decision
    }))
    .into_response()
}

async fn post_cortex_feedback_mark_shipped(
    headers: HeaderMap,
    Json(request): Json<CortexMarkShippedRequest>,
) -> axum::response::Response {
    if request.queue_id.trim().is_empty()
        || request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
        || request.shipped_at.trim().is_empty()
        || request.post_release_metric_date.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MARK_SHIPPED_REQUEST",
            "queueId, candidateId, routeId, viewCapabilityId, shippedAt, and postReleaseMetricDate are required.",
            None,
        );
    }
    if !parse_metric_date(&request.shipped_at)
        || !parse_metric_date(&request.post_release_metric_date)
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MARK_SHIPPED_DATES",
            "shippedAt and postReleaseMetricDate must be RFC3339.",
            None,
        );
    }

    let actor_id = actor_id_from_headers(&headers);
    match transition_feedback_queue_item(
        &request.queue_id,
        UX_STATUS_SHIPPED,
        &actor_id,
        request.note,
        None,
        Some(&request.post_release_metric_date),
    ) {
        Ok(Some(item)) => Json(json!({ "accepted": true, "item": item })).into_response(),
        Ok(None) => cortex_ux_error(
            StatusCode::NOT_FOUND,
            "UX_QUEUE_ITEM_NOT_FOUND",
            "No queue item found for queueId.",
            Some(json!({ "queueId": request.queue_id })),
        ),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_MARK_SHIPPED_PERSIST_FAILED",
            "Failed to persist shipped status.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_feedback_mark_remeasured(
    headers: HeaderMap,
    Json(request): Json<CortexMarkRemeasuredRequest>,
) -> axum::response::Response {
    if request.queue_id.trim().is_empty()
        || request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
        || request.remeasured_at.trim().is_empty()
        || request.post_release_metric_date.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MARK_REMEASURED_REQUEST",
            "queueId, candidateId, routeId, viewCapabilityId, remeasuredAt, and postReleaseMetricDate are required.",
            None,
        );
    }
    if !parse_metric_date(&request.remeasured_at)
        || !parse_metric_date(&request.post_release_metric_date)
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MARK_REMEASURED_DATES",
            "remeasuredAt and postReleaseMetricDate must be RFC3339.",
            None,
        );
    }

    let actor_id = actor_id_from_headers(&headers);
    let item = match transition_feedback_queue_item(
        &request.queue_id,
        UX_STATUS_REMEASURED,
        &actor_id,
        request.summary.clone(),
        None,
        Some(&request.post_release_metric_date),
    ) {
        Ok(Some(item)) => item,
        Ok(None) => {
            return cortex_ux_error(
                StatusCode::NOT_FOUND,
                "UX_QUEUE_ITEM_NOT_FOUND",
                "No queue item found for queueId.",
                Some(json!({ "queueId": request.queue_id })),
            );
        }
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "UX_MARK_REMEASURED_PERSIST_FAILED",
                "Failed to persist remeasured status.",
                Some(json!({ "reason": err })),
            );
        }
    };

    let baseline_metric_date = item.baseline_metric_date.clone().unwrap_or_default();
    if baseline_metric_date.is_empty() {
        let _ = transition_feedback_queue_item(
            &item.queue_id,
            UX_STATUS_BLOCKED_MISSING_BASELINE,
            &actor_id,
            Some("baseline metric missing before remeasurement closeout".to_string()),
            None,
            None,
        );
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "UX_BASELINE_METRIC_REQUIRED",
            "baselineMetricDate must exist before remeasure closeout.",
            Some(json!({ "queueId": request.queue_id })),
        );
    }

    let mut records = read_remeasurements();
    records.push(UxRemeasurementRecord {
        queue_id: request.queue_id,
        route_id: request.route_id,
        view_id: request.view_capability_id,
        candidate_id: request.candidate_id,
        baseline_metric_date,
        post_release_metric_date: request.post_release_metric_date,
        remeasured_at: request.remeasured_at,
        summary: request.summary,
    });
    if let Err(err) = write_remeasurements(&records) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "UX_REMEASUREMENT_WRITE_FAILED",
            "Failed to persist remeasurement record.",
            Some(json!({ "reason": err })),
        );
    }
    Json(json!({
        "accepted": true,
        "item": item
    }))
    .into_response()
}

async fn get_cortex_feedback_overdue(
    Query(query): Query<CortexFeedbackOverdueQuery>,
) -> Json<CortexFeedbackOverdueResponse> {
    let threshold_days = query.days.unwrap_or(14).max(1);
    let now = Utc::now();
    let mut items = read_feedback_queue_items()
        .into_iter()
        .filter(|item| item.status == UX_STATUS_SHIPPED)
        .filter(|item| {
            DateTime::parse_from_rfc3339(&item.updated_at)
                .map(|ts| {
                    now.signed_duration_since(ts.with_timezone(&Utc)).num_days() >= threshold_days
                })
                .unwrap_or(true)
        })
        .collect::<Vec<_>>();
    for item in &mut items {
        item.status = UX_STATUS_OVERDUE_REMEASUREMENT.to_string();
    }
    items.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
    Json(CortexFeedbackOverdueResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        threshold_days,
        items,
    })
}

async fn post_cortex_promotion_approve(
    Json(request): Json<UxPromotionApproval>,
) -> axum::response::Response {
    if request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
        || request.approved_by.trim().is_empty()
        || request.rationale.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_PROMOTION_APPROVAL",
            "candidateId, routeId, viewCapabilityId, approvedBy, and rationale are required.",
            None,
        );
    }
    if !parse_metric_date(&request.baseline_metric_date)
        || !parse_metric_date(&request.post_release_metric_date)
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_METRIC_DATES",
            "baselineMetricDate and postReleaseMetricDate must be RFC3339.",
            None,
        );
    }

    let decision = UxPromotionDecision {
        decision_id: format!("ux_promotion_{}", Utc::now().timestamp_millis()),
        candidate_id: request.candidate_id,
        route_id: request.route_id.clone(),
        view_capability_id: request.view_capability_id.clone(),
        promotion_action: "approve_structural_promotion".to_string(),
        approved_by: request.approved_by,
        rationale: request.rationale,
        timestamp: request.approved_at,
    };
    if let Err(err) = append_json_line(&cortex_ux_promotion_log_path(), &decision) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROMOTION_APPROVAL_PERSIST_FAILED",
            "Failed to persist promotion approval.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = update_feedback_status_for_candidate(
        &request.route_id,
        &request.view_capability_id,
        UX_STATUS_APPROVED,
        Some(&request.baseline_metric_date),
        Some(&request.post_release_metric_date),
    );
    Json(decision).into_response()
}

async fn post_cortex_promotion_reject(
    Json(request): Json<UxPromotionRejection>,
) -> axum::response::Response {
    if request.candidate_id.trim().is_empty()
        || request.route_id.trim().is_empty()
        || request.view_capability_id.trim().is_empty()
        || request.rejected_by.trim().is_empty()
        || request.rationale.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_PROMOTION_REJECTION",
            "candidateId, routeId, viewCapabilityId, rejectedBy, and rationale are required.",
            None,
        );
    }

    let decision = UxPromotionDecision {
        decision_id: format!("ux_promotion_{}", Utc::now().timestamp_millis()),
        candidate_id: request.candidate_id,
        route_id: request.route_id.clone(),
        view_capability_id: request.view_capability_id.clone(),
        promotion_action: "reject_structural_promotion".to_string(),
        approved_by: request.rejected_by,
        rationale: request.rationale,
        timestamp: request.rejected_at,
    };
    if let Err(err) = append_json_line(&cortex_ux_promotion_log_path(), &decision) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROMOTION_REJECTION_PERSIST_FAILED",
            "Failed to persist promotion rejection.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = update_feedback_status_for_candidate(
        &request.route_id,
        &request.view_capability_id,
        UX_STATUS_REJECTED,
        None,
        None,
    );
    Json(decision).into_response()
}

async fn get_cortex_promotion_history(
    Query(query): Query<CortexPromotionHistoryQuery>,
) -> Json<CortexPromotionHistoryResponse> {
    let mut decisions: Vec<UxPromotionDecision> = read_jsonl_vec(&cortex_ux_promotion_log_path());
    decisions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
    if let Some(limit) = query.limit {
        if decisions.len() > limit {
            decisions.truncate(limit);
        }
    }
    Json(CortexPromotionHistoryResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        decisions,
    })
}

fn heap_emit_error_status(code: &str) -> StatusCode {
    match code {
        "HEAP_SCHEMA_UNSUPPORTED_VERSION" => StatusCode::UNPROCESSABLE_ENTITY,
        "HEAP_CANONICALIZATION_ERROR" => StatusCode::UNPROCESSABLE_ENTITY,
        _ => StatusCode::BAD_REQUEST,
    }
}

fn heap_cursor_key(updated_at: &str, artifact_id: &str) -> String {
    format!("{updated_at}|{artifact_id}")
}

fn parse_heap_cursor(cursor: &str) -> Option<(String, String)> {
    let (updated_at, artifact_id) = cursor.split_once('|')?;
    if updated_at.trim().is_empty() || artifact_id.trim().is_empty() {
        return None;
    }
    Some((updated_at.to_string(), artifact_id.to_string()))
}

fn stable_heap_emit_op_id(
    artifact_id: &str,
    request: &EmitHeapBlockRequest,
    canonical_surface_json: &Value,
) -> String {
    if let Some(request_id) = request
        .source
        .request_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let normalized = request_id
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>();
        return format!("heap_emit_{artifact_id}_{normalized}");
    }

    let fingerprint_material = json!({
        "workspaceId": request.workspace_id,
        "emittedAt": request.source.emitted_at,
        "surfaceId": request.content.a2ui.as_ref().map(|a| a.surface_id.clone()).unwrap_or_else(|| "fallback_surface".to_string()),
        "tree": canonical_surface_json
    });
    let encoded = serde_json::to_string(&fingerprint_material).unwrap_or_default();
    let hash = hash_markdown(&encoded);
    let suffix = hash.get(0..24).unwrap_or(hash.as_str());
    format!("heap_emit_{artifact_id}_{suffix}")
}

fn append_heap_emit_rejection_safe(
    actor_id: &str,
    actor_role: &str,
    request_id: Option<String>,
    code: &str,
    message: &str,
    details: Option<Value>,
) {
    let _ = append_heap_emit_rejection(&HeapEmitRejectionEvent {
        timestamp: now_iso(),
        actor_id: actor_id.to_string(),
        actor_role: actor_role.to_string(),
        request_id,
        code: code.to_string(),
        message: message.to_string(),
        details,
    });
}

async fn post_cortex_heap_emit(
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> axum::response::Response {
    let actor_role = actor_role_from_headers(&headers);
    let actor_id = actor_id_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("operator") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "HEAP_EMIT_FORBIDDEN",
            "Operator role or higher is required to emit heap blocks.",
            Some(json!({ "actorRole": actor_role })),
        );
    }

    let request_id_hint = payload
        .get("source")
        .and_then(|source| source.get("request_id"))
        .and_then(|value| value.as_str())
        .map(str::to_string);

    let request = match parse_emit_heap_block(payload) {
        Ok(request) => request,
        Err(err) => {
            append_heap_emit_rejection_safe(
                &actor_id,
                &actor_role,
                request_id_hint,
                &err.code,
                &err.message,
                err.details.clone(),
            );
            return cortex_ux_error(
                heap_emit_error_status(&err.code),
                &err.code,
                &err.message,
                err.details,
            );
        }
    };

    if let Err(err) = validate_emit_heap_block(&request) {
        append_heap_emit_rejection_safe(
            &actor_id,
            &actor_role,
            request.source.request_id.clone(),
            &err.code,
            &err.message,
            err.details.clone(),
        );
        return cortex_ux_error(
            heap_emit_error_status(&err.code),
            &err.code,
            &err.message,
            err.details,
        );
    }

    let canonical = match canonicalize_emit_heap_block(&request) {
        Ok(canonical) => canonical,
        Err(err) => {
            append_heap_emit_rejection_safe(
                &actor_id,
                &actor_role,
                request.source.request_id.clone(),
                &err.code,
                &err.message,
                err.details.clone(),
            );
            return cortex_ux_error(
                heap_emit_error_status(&err.code),
                &err.code,
                &err.message,
                err.details,
            );
        }
    };

    let agui_mutations = match map_emit_heap_block_to_agui_mutations(&request, &canonical) {
        Ok(mutations) => mutations,
        Err(err) => {
            append_heap_emit_rejection_safe(
                &actor_id,
                &actor_role,
                request.source.request_id.clone(),
                &err.code,
                &err.message,
                err.details.clone(),
            );
            return cortex_ux_error(
                heap_emit_error_status(&err.code),
                &err.code,
                &err.message,
                err.details,
            );
        }
    };

    let mut items = read_artifacts_store();
    let mut revisions = read_artifact_revisions();
    let mut revisions_dirty = false;

    let artifact_id = request
        .crdt_projection
        .artifact_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .or_else(|| request.block.id.clone())
        .unwrap_or_else(|| format!("artifact_heap_{}", Utc::now().timestamp_millis()));
    let source_state = cortex_ux_source_state();

    let mut artifact_idx = items
        .iter()
        .position(|item| item.artifact_id == artifact_id);
    if artifact_idx.is_none() {
        let now = now_iso();
        let seed_markdown = match &request.content.payload_type {
            crate::services::heap_mapper::HeapPayloadType::RichText => request
                .content
                .rich_text
                .as_ref()
                .map(|rt| rt.plain_text.clone())
                .filter(|value| !value.trim().is_empty())
                .unwrap_or_else(|| format!("# {}", request.block.title)),
            _ => format!("# {}", request.block.title),
        };
        let content_hash = hash_markdown(&seed_markdown);
        let seed_revision = ArtifactRevision {
            artifact_id: artifact_id.clone(),
            revision_id: format!("rev_{}_{}", artifact_id, Utc::now().timestamp_millis()),
            revision_number: revisions
                .iter()
                .filter(|rev| rev.artifact_id == artifact_id)
                .map(|rev| rev.revision_number)
                .max()
                .unwrap_or(0)
                + 1,
            markdown_source: seed_markdown.clone(),
            content_hash: content_hash.clone(),
            created_at: now.clone(),
            created_by: actor_id.clone(),
            parent_revision_id: None,
            published: false,
        };
        revisions.push(seed_revision.clone());
        revisions_dirty = true;
        items.push(ArtifactDocumentV2 {
            artifact_id: artifact_id.clone(),
            title: request.block.title.clone(),
            markdown_source: seed_markdown.clone(),
            rich_content: ArtifactRichContentProjection {
                hash: content_hash.clone(),
                block_count: estimate_markdown_blocks(&seed_markdown),
            },
            content_hash,
            status: "draft".to_string(),
            created_at: now.clone(),
            updated_at: now,
            published_at: None,
            head_revision_id: seed_revision.revision_id,
            version: 1,
            route_id: "/heap".to_string(),
            owner_role: actor_role.clone(),
            source_of_truth: source_state.source_of_truth.clone(),
            fallback_active: source_state.fallback_active,
            agui_initial_ui_json: None,
            agui_tags: None,
            agui_mentions: None,
            heap_workspace_id: None,
            heap_block_type: None,
            heap_emitted_at: None,
            heap_file_keys: None,
            heap_mirror_mentions_to_relations: None,
            heap_relation_map_version: None,
            heap_files_key_format: None,
        });
        artifact_idx = items
            .iter()
            .position(|item| item.artifact_id == artifact_id);
    }

    let Some(artifact_idx) = artifact_idx else {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_ARTIFACT_RESOLUTION_FAILED",
            "Failed to resolve heap artifact record.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };

    let seed_markdown = items[artifact_idx].markdown_source.clone();
    let mut state = read_artifact_crdt_state(&artifact_id, &seed_markdown);
    let target_markdown = match &request.content.payload_type {
        crate::services::heap_mapper::HeapPayloadType::RichText => request
            .content
            .rich_text
            .as_ref()
            .map(|rt| rt.plain_text.clone())
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| seed_markdown.clone()),
        _ => seed_markdown.clone(),
    };
    let op_id = stable_heap_emit_op_id(&artifact_id, &request, &canonical.surface_json);
    let sequence = state.last_sequence.saturating_add(1);
    let lamport = state.last_lamport.saturating_add(1);
    let created_at = now_iso();
    let session_id = request
        .source
        .session_id
        .clone()
        .unwrap_or_else(|| format!("heap:{}", request.workspace_id));
    let mut envelope = build_replace_markdown_update(
        &state,
        &artifact_id,
        &session_id,
        &actor_id,
        &op_id,
        sequence,
        lamport,
        &target_markdown,
        Some(artifact_realtime_channel(&artifact_id)),
        created_at.clone(),
    );
    envelope.agui_mutations = agui_mutations;

    let apply_result = match apply_crdt_update(&mut state, &envelope, now_iso()) {
        Ok(result) => result,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::CONFLICT,
                "HEAP_CRDT_APPLY_FAILED",
                "Failed to apply heap CRDT update envelope.",
                Some(json!({
                    "artifactId": artifact_id,
                    "opId": op_id,
                    "reason": err
                })),
            );
        }
    };

    if let Err(err) = write_artifact_crdt_state(&artifact_id, &state) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_CRDT_STATE_WRITE_FAILED",
            "Failed to persist heap CRDT snapshot.",
            Some(json!({ "artifactId": artifact_id, "reason": err })),
        );
    }

    let mut existing_ops = read_artifact_crdt_ops(&artifact_id);
    if !existing_ops
        .iter()
        .any(|entry| entry.op_id == envelope.op_id)
    {
        existing_ops.push(envelope.clone());
        if let Err(err) = write_artifact_crdt_ops(&artifact_id, &existing_ops) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "HEAP_CRDT_OPS_WRITE_FAILED",
                "Failed to persist heap CRDT operation log.",
                Some(json!({ "artifactId": artifact_id, "reason": err })),
            );
        }
    }

    let previous_markdown = items[artifact_idx].markdown_source.clone();
    if !apply_result.idempotent && apply_result.materialized_markdown != previous_markdown {
        let revision = upsert_artifact_revision(
            &artifact_id,
            &apply_result.materialized_markdown,
            &actor_id,
            Some(items[artifact_idx].head_revision_id.clone()),
            false,
        );
        items[artifact_idx].head_revision_id = revision.revision_id.clone();
        items[artifact_idx].version = items[artifact_idx].version.saturating_add(1);
        revisions.push(revision);
        revisions_dirty = true;
    }

    if !apply_result.idempotent {
        items[artifact_idx].updated_at = now_iso();
    }
    items[artifact_idx].title = request.block.title.clone();
    items[artifact_idx].markdown_source = apply_result.materialized_markdown.clone();
    items[artifact_idx].content_hash = hash_markdown(&apply_result.materialized_markdown);
    items[artifact_idx].rich_content = ArtifactRichContentProjection {
        hash: items[artifact_idx].content_hash.clone(),
        block_count: estimate_markdown_blocks(&apply_result.materialized_markdown),
    };
    items[artifact_idx].agui_initial_ui_json = Some(canonical.surface_json.to_string());
    items[artifact_idx].agui_tags = Some(canonical.tags.clone());
    items[artifact_idx].agui_mentions = Some(canonical.mentions_inline.clone());
    items[artifact_idx].heap_workspace_id = Some(request.workspace_id.clone());
    items[artifact_idx].heap_block_type = Some(request.block.r#type.clone());
    items[artifact_idx].heap_emitted_at = Some(request.source.emitted_at.clone());
    items[artifact_idx].heap_file_keys = Some(
        canonical
            .files
            .iter()
            .map(|file| file.key.clone())
            .collect::<Vec<_>>(),
    );
    items[artifact_idx].heap_mirror_mentions_to_relations =
        Some(canonical.mirror_mentions_to_relations);
    items[artifact_idx].heap_relation_map_version = Some(canonical.relation_map_version.clone());
    items[artifact_idx].heap_files_key_format = Some(canonical.files_key_format.clone());
    let payload_deleted_at = request
        .meta
        .as_ref()
        .and_then(|meta| meta.deleted_at.clone())
        .or_else(|| {
            request
                .meta
                .as_ref()
                .and_then(|meta| meta.permanently_deleted_at.clone())
        });
    if payload_deleted_at.is_some() {
        items[artifact_idx].status = "deleted".to_string();
    }

    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist heap artifact state.",
            Some(json!({ "artifactId": artifact_id, "reason": err })),
        );
    }
    if revisions_dirty {
        if let Err(err) = write_artifact_revisions(&revisions) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "HEAP_ARTIFACT_REVISION_WRITE_FAILED",
                "Failed to persist heap artifact revisions.",
                Some(json!({ "artifactId": artifact_id, "reason": err })),
            );
        }
    }

    let projection = project_heap_block(
        &request,
        &canonical,
        &artifact_id,
        &actor_id,
        &actor_role,
        &items[artifact_idx].created_at,
        &items[artifact_idx].updated_at,
    );
    let mut projections = read_heap_projection_store();
    if let Some(existing) = projections
        .iter_mut()
        .find(|entry| entry.projection.artifact_id == artifact_id)
    {
        existing.projection = projection.clone();
        existing.surface_json = canonical.surface_json.clone();
        existing.warnings = canonical.warnings.clone();
        if payload_deleted_at.is_some() {
            existing.deleted_at = payload_deleted_at.clone();
        }
    } else {
        projections.push(HeapProjectionRecord {
            projection: projection.clone(),
            surface_json: canonical.surface_json.clone(),
            warnings: canonical.warnings.clone(),
            pinned_at: None,
            deleted_at: payload_deleted_at.clone(),
        });
    }

    if let Err(err) = write_heap_projection_store(&projections) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_PROJECTION_WRITE_FAILED",
            "Failed to persist heap projection store.",
            Some(json!({ "artifactId": artifact_id, "reason": err })),
        );
    }

    let _ = append_artifact_audit(
        &artifact_id,
        "heap_emit",
        &actor_role,
        &actor_id,
        "/heap",
        request.source.request_id.clone(),
    );

    Json(EmitHeapBlockResponse {
        schema_version: "1.0.0".to_string(),
        accepted: true,
        artifact_id: artifact_id.clone(),
        block_id: projection.block_id.clone(),
        op_id,
        idempotent: apply_result.idempotent,
        warnings: canonical.warnings,
        projection,
        source_of_truth: source_state.source_of_truth,
        fallback_active: source_state.fallback_active,
    })
    .into_response()
}

async fn get_cortex_heap_blocks(Query(query): Query<HeapBlocksQuery>) -> axum::response::Response {
    let from_ts = if let Some(from_ts_raw) = query.from_ts.as_deref() {
        let parsed = parse_heap_iso_timestamp(from_ts_raw);
        if parsed.is_none() {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "HEAP_QUERY_INVALID_FROM_TS",
                "fromTs must be RFC3339.",
                Some(json!({ "fromTs": from_ts_raw })),
            );
        }
        parsed
    } else if let Some(changed_since_raw) = query.changed_since.as_deref() {
        let parsed = parse_heap_iso_timestamp(changed_since_raw);
        if parsed.is_none() {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "HEAP_QUERY_INVALID_CHANGED_SINCE",
                "changedSince must be RFC3339.",
                Some(json!({ "changedSince": changed_since_raw })),
            );
        }
        record_heap_blocks_changed_since_alias_usage();
        tracing::info!("heap blocks query used changedSince alias");
        parsed
    } else {
        None
    };

    let to_ts = query.to_ts.as_deref().and_then(parse_heap_iso_timestamp);
    if query.to_ts.is_some() && to_ts.is_none() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "HEAP_QUERY_INVALID_TO_TS",
            "toTs must be RFC3339.",
            query.to_ts.as_ref().map(|value| json!({ "toTs": value })),
        );
    }

    let cursor = if let Some(cursor) = query.cursor.as_deref() {
        match parse_heap_cursor(cursor) {
            Some(value) => Some(value),
            None => {
                return cortex_ux_error(
                    StatusCode::BAD_REQUEST,
                    "HEAP_QUERY_INVALID_CURSOR",
                    "cursor must be encoded as '<updatedAt>|<artifactId>'.",
                    Some(json!({ "cursor": cursor })),
                );
            }
        }
    } else {
        None
    };

    let mut rows = read_heap_projection_store();
    let include_deleted = query.include_deleted.unwrap_or(false);
    if !include_deleted {
        rows.retain(|entry| entry.deleted_at.is_none());
    }
    if let Some(space_id) = query.space_id.as_deref() {
        rows.retain(|entry| entry.projection.workspace_id == space_id);
    }
    if let Some(tag) = query.tag.as_deref() {
        let target = tag.trim().to_ascii_lowercase();
        rows.retain(|entry| {
            entry
                .projection
                .tags
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(mention) = query.mention.as_deref() {
        let target = mention.trim().to_ascii_lowercase();
        rows.retain(|entry| {
            entry
                .projection
                .mentions_query
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(page_link) = query.page_link.as_deref() {
        let target = page_link.trim().to_ascii_lowercase();
        record_heap_blocks_page_link_filter_usage();
        tracing::info!("heap blocks query applied pageLink filter");
        rows.retain(|entry| {
            entry
                .projection
                .page_links
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(attribute) = query.attribute.as_deref() {
        let raw = attribute.trim();
        if !raw.is_empty() {
            let normalized = raw.to_ascii_lowercase();
            let split = normalized
                .split_once(':')
                .or_else(|| normalized.split_once('='))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .filter(|(key, value)| !key.is_empty() && !value.is_empty());

            rows.retain(|entry| {
                let attrs = entry.projection.attributes.as_ref();
                let Some(attrs) = attrs else {
                    return false;
                };

                if let Some((key_filter, value_filter)) = split.as_ref() {
                    return attrs.iter().any(|(key, value)| {
                        key.to_ascii_lowercase() == *key_filter
                            && value.to_ascii_lowercase() == *value_filter
                    });
                }

                attrs.iter().any(|(key, value)| {
                    key.to_ascii_lowercase() == normalized
                        || value.to_ascii_lowercase() == normalized
                })
            });
        }
    }
    if let Some(block_type) = query.block_type.as_deref() {
        let target = block_type.trim().to_ascii_lowercase();
        rows.retain(|entry| entry.projection.block_type.to_ascii_lowercase() == target);
    }
    if let Some(has_files) = query.has_files {
        rows.retain(|entry| entry.projection.has_files == has_files);
    }
    if from_ts.is_some() || to_ts.is_some() {
        rows.retain(|entry| {
            let Some(ts) = parse_heap_iso_timestamp(&entry.projection.emitted_at) else {
                return false;
            };
            let after_from = from_ts
                .as_ref()
                .map(|from| ts >= from.clone())
                .unwrap_or(true);
            let before_to = to_ts.as_ref().map(|to| ts <= to.clone()).unwrap_or(true);
            after_from && before_to
        });
    }

    rows.sort_by(|left, right| {
        right
            .projection
            .updated_at
            .cmp(&left.projection.updated_at)
            .then_with(|| {
                right
                    .projection
                    .artifact_id
                    .cmp(&left.projection.artifact_id)
            })
    });

    if let Some((cursor_updated_at, cursor_artifact_id)) = cursor {
        rows.retain(|entry| {
            let entry_key = (
                entry.projection.updated_at.clone(),
                entry.projection.artifact_id.clone(),
            );
            entry_key < (cursor_updated_at.clone(), cursor_artifact_id.clone())
        });
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let has_more = rows.len() > limit;
    if rows.len() > limit {
        rows.truncate(limit);
    }
    let next_cursor = if has_more {
        rows.last().map(|entry| {
            heap_cursor_key(&entry.projection.updated_at, &entry.projection.artifact_id)
        })
    } else {
        None
    };

    let items = rows
        .into_iter()
        .map(|entry| HeapBlockListItem {
            projection: entry.projection,
            surface_json: entry.surface_json,
            warnings: entry.warnings,
            pinned_at: entry.pinned_at,
            deleted_at: entry.deleted_at,
        })
        .collect::<Vec<_>>();

    Json(HeapBlocksResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        count: items.len(),
        has_more,
        next_cursor,
        items,
    })
    .into_response()
}

async fn get_cortex_heap_changed_blocks(
    Query(query): Query<HeapBlocksQuery>,
) -> axum::response::Response {
    record_heap_changed_blocks_endpoint_usage();
    tracing::info!("heap changed_blocks query invoked");

    let from_ts = if let Some(from_ts_raw) = query.from_ts.as_deref() {
        let parsed = parse_heap_iso_timestamp(from_ts_raw);
        if parsed.is_none() {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "HEAP_QUERY_INVALID_FROM_TS",
                "fromTs must be RFC3339.",
                Some(json!({ "fromTs": from_ts_raw })),
            );
        }
        parsed
    } else if let Some(changed_since_raw) = query.changed_since.as_deref() {
        let parsed = parse_heap_iso_timestamp(changed_since_raw);
        if parsed.is_none() {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "HEAP_QUERY_INVALID_CHANGED_SINCE",
                "changedSince must be RFC3339.",
                Some(json!({ "changedSince": changed_since_raw })),
            );
        }
        record_heap_changed_blocks_changed_since_alias_usage();
        tracing::info!("heap changed_blocks query used changedSince alias");
        parsed
    } else {
        None
    };

    let to_ts = query.to_ts.as_deref().and_then(parse_heap_iso_timestamp);
    if query.to_ts.is_some() && to_ts.is_none() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "HEAP_QUERY_INVALID_TO_TS",
            "toTs must be RFC3339.",
            query.to_ts.as_ref().map(|value| json!({ "toTs": value })),
        );
    }

    let cursor = if let Some(cursor) = query.cursor.as_deref() {
        match parse_heap_cursor(cursor) {
            Some(value) => Some(value),
            None => {
                return cortex_ux_error(
                    StatusCode::BAD_REQUEST,
                    "HEAP_QUERY_INVALID_CURSOR",
                    "cursor must be encoded as '<updatedAt>|<artifactId>'.",
                    Some(json!({ "cursor": cursor })),
                );
            }
        }
    } else {
        None
    };

    let mut rows = read_heap_projection_store();
    let include_deleted = query.include_deleted.unwrap_or(true);
    if !include_deleted {
        rows.retain(|entry| entry.deleted_at.is_none());
    }
    if let Some(space_id) = query.space_id.as_deref() {
        rows.retain(|entry| entry.projection.workspace_id == space_id);
    }
    if let Some(tag) = query.tag.as_deref() {
        let target = tag.trim().to_ascii_lowercase();
        rows.retain(|entry| {
            entry
                .projection
                .tags
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(mention) = query.mention.as_deref() {
        let target = mention.trim().to_ascii_lowercase();
        rows.retain(|entry| {
            entry
                .projection
                .mentions_query
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(page_link) = query.page_link.as_deref() {
        let target = page_link.trim().to_ascii_lowercase();
        record_heap_changed_blocks_page_link_filter_usage();
        tracing::info!("heap changed_blocks query applied pageLink filter");
        rows.retain(|entry| {
            entry
                .projection
                .page_links
                .iter()
                .any(|value| value.to_ascii_lowercase() == target)
        });
    }
    if let Some(attribute) = query.attribute.as_deref() {
        let raw = attribute.trim();
        if !raw.is_empty() {
            let normalized = raw.to_ascii_lowercase();
            let split = normalized
                .split_once(':')
                .or_else(|| normalized.split_once('='))
                .map(|(key, value)| (key.trim().to_string(), value.trim().to_string()))
                .filter(|(key, value)| !key.is_empty() && !value.is_empty());

            rows.retain(|entry| {
                let attrs = entry.projection.attributes.as_ref();
                let Some(attrs) = attrs else {
                    return false;
                };

                if let Some((key_filter, value_filter)) = split.as_ref() {
                    return attrs.iter().any(|(key, value)| {
                        key.to_ascii_lowercase() == *key_filter
                            && value.to_ascii_lowercase() == *value_filter
                    });
                }

                attrs.iter().any(|(key, value)| {
                    key.to_ascii_lowercase() == normalized
                        || value.to_ascii_lowercase() == normalized
                })
            });
        }
    }
    if let Some(block_type) = query.block_type.as_deref() {
        let target = block_type.trim().to_ascii_lowercase();
        rows.retain(|entry| entry.projection.block_type.to_ascii_lowercase() == target);
    }
    if let Some(has_files) = query.has_files {
        rows.retain(|entry| entry.projection.has_files == has_files);
    }
    if from_ts.is_some() || to_ts.is_some() {
        rows.retain(|entry| {
            let Some(ts) = parse_heap_iso_timestamp(&entry.projection.updated_at) else {
                return false;
            };
            let after_from = from_ts
                .as_ref()
                .map(|from| ts >= from.clone())
                .unwrap_or(true);
            let before_to = to_ts.as_ref().map(|to| ts <= to.clone()).unwrap_or(true);
            after_from && before_to
        });
    }

    rows.sort_by(|left, right| {
        right
            .projection
            .updated_at
            .cmp(&left.projection.updated_at)
            .then_with(|| {
                right
                    .projection
                    .artifact_id
                    .cmp(&left.projection.artifact_id)
            })
    });

    if let Some((cursor_updated_at, cursor_artifact_id)) = cursor {
        rows.retain(|entry| {
            let entry_key = (
                entry.projection.updated_at.clone(),
                entry.projection.artifact_id.clone(),
            );
            entry_key < (cursor_updated_at.clone(), cursor_artifact_id.clone())
        });
    }

    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let has_more = rows.len() > limit;
    if rows.len() > limit {
        rows.truncate(limit);
    }
    let next_cursor = if has_more {
        rows.last().map(|entry| {
            heap_cursor_key(&entry.projection.updated_at, &entry.projection.artifact_id)
        })
    } else {
        None
    };

    let count = rows.len();
    let changed = rows
        .iter()
        .filter(|entry| entry.deleted_at.is_none())
        .map(|entry| HeapBlockListItem {
            projection: entry.projection.clone(),
            surface_json: entry.surface_json.clone(),
            warnings: entry.warnings.clone(),
            pinned_at: entry.pinned_at.clone(),
            deleted_at: entry.deleted_at.clone(),
        })
        .collect::<Vec<_>>();
    let deleted = rows
        .into_iter()
        .filter_map(|entry| {
            entry.deleted_at.map(|deleted_at| HeapDeletedListItem {
                artifact_id: entry.projection.artifact_id,
                deleted_at,
            })
        })
        .collect::<Vec<_>>();

    Json(HeapChangedBlocksResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: now_iso(),
        count,
        has_more,
        next_cursor,
        changed,
        deleted,
    })
    .into_response()
}

async fn post_cortex_heap_block_pin(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);

    let mut projections = read_heap_projection_store();
    let Some(entry) = projections
        .iter_mut()
        .find(|entry| entry.projection.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "HEAP_BLOCK_NOT_FOUND",
            "Heap block does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let updated_at = now_iso();
    entry.pinned_at = Some(updated_at.clone());
    entry.projection.updated_at = updated_at.clone();
    if let Err(err) = write_heap_projection_store(&projections) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_PROJECTION_WRITE_FAILED",
            "Failed to persist heap pin action.",
            Some(json!({ "artifactId": artifact_id, "reason": err })),
        );
    }

    let mut artifacts = read_artifacts_store();
    if let Some(artifact) = artifacts
        .iter_mut()
        .find(|item| item.artifact_id == artifact_id)
    {
        artifact.updated_at = updated_at.clone();
        let _ = write_artifacts_store(&artifacts);
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "heap_pin",
        &actor_role,
        &actor_id,
        "/heap",
        None,
    );

    Json(HeapBlockActionResponse {
        accepted: true,
        artifact_id,
        action: "pin".to_string(),
        updated_at,
    })
    .into_response()
}

async fn post_cortex_heap_block_delete(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let deleted_at = now_iso();

    let mut projections = read_heap_projection_store();
    let Some(entry) = projections
        .iter_mut()
        .find(|entry| entry.projection.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "HEAP_BLOCK_NOT_FOUND",
            "Heap block does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    entry.deleted_at = Some(deleted_at.clone());
    entry.projection.updated_at = deleted_at.clone();
    if let Err(err) = write_heap_projection_store(&projections) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "HEAP_PROJECTION_WRITE_FAILED",
            "Failed to persist heap delete action.",
            Some(json!({ "artifactId": artifact_id, "reason": err })),
        );
    }

    let mut artifacts = read_artifacts_store();
    if let Some(artifact) = artifacts
        .iter_mut()
        .find(|item| item.artifact_id == artifact_id)
    {
        artifact.status = "deleted".to_string();
        artifact.updated_at = deleted_at.clone();
        let _ = write_artifacts_store(&artifacts);
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "heap_delete",
        &actor_role,
        &actor_id,
        "/heap",
        None,
    );

    Json(HeapBlockActionResponse {
        accepted: true,
        artifact_id,
        action: "delete".to_string(),
        updated_at: deleted_at,
    })
    .into_response()
}

#[derive(Deserialize)]
struct HeapBlocksContextRequest {
    block_ids: Vec<String>,
}

#[derive(Serialize)]
struct HeapBlocksContextResponse {
    context_bundle: HeapBlocksContextBundle,
}

#[derive(Serialize)]
struct HeapBlocksContextBundle {
    blocks: Vec<serde_json::Value>,
    block_count: usize,
    prepared_at: String,
}

async fn post_cortex_heap_blocks_context(
    headers: HeaderMap,
    Json(request): Json<HeapBlocksContextRequest>,
) -> axum::response::Response {
    let _actor_id = actor_id_from_headers(&headers);

    if request.block_ids.is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "HEAP_CONTEXT_EMPTY",
            "No block IDs provided for context bundling.",
            None,
        );
    }

    let projections = read_heap_projection_store();
    let requested_set: std::collections::HashSet<&str> =
        request.block_ids.iter().map(|id| id.as_str()).collect();

    let matched_blocks: Vec<serde_json::Value> = projections
        .iter()
        .filter(|entry| {
            requested_set.contains(entry.projection.artifact_id.as_str())
                && entry.deleted_at.is_none()
        })
        .map(|entry| {
            json!({
                "artifact_id": entry.projection.artifact_id,
                "title": entry.projection.title,
                "block_type": entry.projection.block_type,
                "tags": entry.projection.tags,
                "mentions": entry.projection.mentions_inline,
                "surface_json": entry.surface_json,
                "updated_at": entry.projection.updated_at,
            })
        })
        .collect();

    let block_count = matched_blocks.len();
    Json(HeapBlocksContextResponse {
        context_bundle: HeapBlocksContextBundle {
            blocks: matched_blocks,
            block_count,
            prepared_at: now_iso(),
        },
    })
    .into_response()
}

#[derive(Deserialize)]
struct ExportQuery {
    format: Option<String>,
}

async fn get_cortex_heap_block_export(
    Path(artifact_id): Path<String>,
    Query(query): Query<ExportQuery>,
) -> axum::response::Response {
    let projections = read_heap_projection_store();
    let Some(entry) = projections
        .iter()
        .find(|entry| entry.projection.artifact_id == artifact_id && entry.deleted_at.is_none())
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "HEAP_BLOCK_NOT_FOUND",
            "Heap block does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };

    let format = query.format.as_deref().unwrap_or("markdown");
    match format {
        "json" => {
            let payload = json!({
                "artifact_id": entry.projection.artifact_id,
                "title": entry.projection.title,
                "block_type": entry.projection.block_type,
                "tags": entry.projection.tags,
                "mentions": entry.projection.mentions_inline,
                "surface_json": entry.surface_json,
                "updated_at": entry.projection.updated_at,
            });
            Json(payload).into_response()
        }
        _ => {
            // Markdown export with YAML frontmatter
            let mut md = String::new();
            md.push_str("---\n");
            md.push_str(&format!("title: \"{}\"\n", entry.projection.title));
            md.push_str(&format!(
                "artifact_id: \"{}\"\n",
                entry.projection.artifact_id
            ));
            md.push_str(&format!(
                "block_type: \"{}\"\n",
                entry.projection.block_type
            ));
            if !entry.projection.tags.is_empty() {
                md.push_str("tags:\n");
                for tag in &entry.projection.tags {
                    md.push_str(&format!("  - \"{}\"\n", tag));
                }
            }
            if !entry.projection.mentions_inline.is_empty() {
                md.push_str("mentions:\n");
                for mention in &entry.projection.mentions_inline {
                    md.push_str(&format!("  - \"[[{}]]\"\n", mention));
                }
            }
            md.push_str(&format!(
                "updated_at: \"{}\"\n",
                entry.projection.updated_at
            ));
            md.push_str("---\n\n");
            md.push_str(&format!("# {}\n\n", entry.projection.title));
            md.push_str(&format!(
                "```json\n{}\n```\n",
                serde_json::to_string_pretty(&entry.surface_json).unwrap_or_default()
            ));

            axum::response::Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "text/markdown; charset=utf-8")
                .header(
                    "content-disposition",
                    format!(
                        "attachment; filename=\"{}.md\"",
                        entry.projection.artifact_id
                    ),
                )
                .body(axum::body::Body::from(md))
                .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response())
        }
    }
}

#[derive(Serialize)]
struct HeapBlockHistoryResponse {
    artifact_id: String,
    versions: Vec<HeapBlockVersion>,
}

#[derive(Serialize)]
struct HeapBlockVersion {
    version: usize,
    timestamp: String,
    mutation_type: String,
    actor: String,
}

async fn get_cortex_heap_block_history(
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let projections = read_heap_projection_store();
    let exists = projections
        .iter()
        .any(|entry| entry.projection.artifact_id == artifact_id);
    if !exists {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "HEAP_BLOCK_NOT_FOUND",
            "Heap block does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    }

    // Read audit log and filter for this artifact
    let all_audit: Vec<serde_json::Value> = {
        let path = cortex_ux_artifact_audit_log_path();
        let raw = match std::fs::read_to_string(&path) {
            Ok(content) => content,
            Err(_) => String::new(),
        };
        raw.lines()
            .filter_map(|line| serde_json::from_str::<serde_json::Value>(line).ok())
            .filter(|entry| {
                entry.get("artifact_id").and_then(|v| v.as_str()) == Some(artifact_id.as_str())
            })
            .collect()
    };

    let versions: Vec<HeapBlockVersion> = all_audit
        .iter()
        .enumerate()
        .map(|(idx, entry)| HeapBlockVersion {
            version: idx + 1,
            timestamp: entry
                .get("timestamp")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            mutation_type: entry
                .get("action")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
            actor: entry
                .get("actor_id")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown")
                .to_string(),
        })
        .collect();

    Json(HeapBlockHistoryResponse {
        artifact_id,
        versions,
    })
    .into_response()
}

async fn post_cortex_artifact_create(
    headers: HeaderMap,
    Json(request): Json<ArtifactCreateRequest>,
) -> axum::response::Response {
    if request.title.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_CREATE_REQUEST",
            "title is required.",
            None,
        );
    }
    let actor_role = actor_role_from_headers(&headers);
    if !has_route_access(
        &default_artifact_capability_manifest().required_role_create,
        &actor_role,
    ) {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_CREATE_FORBIDDEN",
            "Role is not permitted to create artifacts.",
            Some(json!({ "role": actor_role })),
        );
    }
    let actor_id = actor_id_from_headers(&headers);

    let mut items = read_artifacts_store();
    let artifact_id = request
        .artifact_id
        .unwrap_or_else(|| format!("artifact_{}", Utc::now().timestamp_millis()));
    let now = now_iso();
    let markdown_source = request
        .markdown_source
        .or(request.content)
        .unwrap_or_default();
    let content_hash = hash_markdown(&markdown_source);
    let seed_revision = ArtifactRevision {
        artifact_id: artifact_id.clone(),
        revision_id: format!("rev_{}_{}", artifact_id, Utc::now().timestamp_millis()),
        revision_number: 1,
        markdown_source: markdown_source.clone(),
        content_hash: content_hash.clone(),
        created_at: now.clone(),
        created_by: actor_id.clone(),
        parent_revision_id: None,
        published: false,
    };
    let source_state = cortex_ux_source_state();
    let record = ArtifactDocumentV2 {
        artifact_id: artifact_id.clone(),
        title: request.title,
        markdown_source: markdown_source.clone(),
        rich_content: ArtifactRichContentProjection {
            hash: content_hash.clone(),
            block_count: estimate_markdown_blocks(&markdown_source),
        },
        content_hash,
        status: "draft".to_string(),
        created_at: now.clone(),
        updated_at: now,
        published_at: None,
        head_revision_id: seed_revision.revision_id.clone(),
        version: 1,
        route_id: "/artifacts".to_string(),
        owner_role: actor_role.clone(),
        source_of_truth: source_state.source_of_truth,
        fallback_active: source_state.fallback_active,
        agui_initial_ui_json: None,
        agui_tags: None,
        agui_mentions: None,
        heap_workspace_id: None,
        heap_block_type: None,
        heap_emitted_at: None,
        heap_file_keys: None,
        heap_mirror_mentions_to_relations: None,
        heap_relation_map_version: None,
        heap_files_key_format: None,
    };
    items.retain(|item| item.artifact_id != artifact_id);
    items.push(record.clone());
    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist artifact.",
            Some(json!({ "reason": err })),
        );
    }
    let mut revisions = read_artifact_revisions();
    revisions.retain(|rev| rev.artifact_id != artifact_id);
    revisions.push(seed_revision);
    if let Err(err) = write_artifact_revisions(&revisions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REVISION_STORE_WRITE_FAILED",
            "Failed to persist artifact revision.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &record.artifact_id,
        "create",
        &actor_role,
        &actor_id,
        "/artifacts",
        idempotency_key_from_headers(&headers),
    );
    Json(record).into_response()
}

async fn get_cortex_artifact(Path(artifact_id): Path<String>) -> axum::response::Response {
    let items = read_artifacts_store();
    if let Some(record) = items
        .into_iter()
        .find(|item| item.artifact_id == artifact_id)
    {
        return Json(record).into_response();
    }
    cortex_ux_error(
        StatusCode::NOT_FOUND,
        "ARTIFACT_NOT_FOUND",
        "Artifact does not exist.",
        Some(json!({ "artifactId": artifact_id })),
    )
}

async fn post_cortex_artifact_publish(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactPublishRequest>,
) -> axum::response::Response {
    let actor_role = actor_role_from_headers(&headers);
    let actor_id = actor_id_from_headers(&headers);
    let required_role = default_artifact_capability_manifest().required_role_publish;
    if role_rank(&actor_role) < role_rank(&required_role) {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_PUBLISH_FORBIDDEN",
            "Role is not permitted to publish artifacts.",
            Some(json!({ "role": actor_role, "requiredRole": required_role })),
        );
    }
    if let Err(response) = require_governance_envelope(&actor_id, request.governance.as_ref()) {
        return response;
    }

    if let Some(lease_id) = request.lease_id.as_deref() {
        if let Err(response) = require_active_lease(&artifact_id, lease_id, &actor_id) {
            return response;
        }
    }

    let mut items = read_artifacts_store();
    let mut revisions = read_artifact_revisions();
    let mut published = None;
    for item in &mut items {
        if item.artifact_id != artifact_id {
            continue;
        }
        if let Some(expected_revision_id) = request.expected_revision_id.as_deref() {
            if item.head_revision_id != expected_revision_id {
                return cortex_ux_error(
                    StatusCode::CONFLICT,
                    "ARTIFACT_REVISION_CONFLICT",
                    "expectedRevisionId does not match current head revision.",
                    Some(json!({
                        "artifactId": artifact_id,
                        "expectedRevisionId": expected_revision_id,
                        "headRevisionId": item.head_revision_id
                    })),
                );
            }
        }

        let new_revision = upsert_artifact_revision(
            &item.artifact_id,
            &item.markdown_source,
            &actor_id,
            Some(item.head_revision_id.clone()),
            true,
        );
        item.head_revision_id = new_revision.revision_id.clone();
        item.status = "published".to_string();
        item.updated_at = now_iso();
        item.published_at = Some(item.updated_at.clone());
        item.version = item.version.saturating_add(1);
        revisions.push(new_revision);
        published = Some(item.clone());
        break;
    }
    let Some(record) = published else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist artifact publish update.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifact_revisions(&revisions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REVISION_STORE_WRITE_FAILED",
            "Failed to persist artifact publish revision.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &record.artifact_id,
        "publish",
        &actor_role,
        &actor_id,
        "/artifacts",
        request.notes.or_else(|| {
            request
                .governance
                .as_ref()
                .map(|env| {
                    format!(
                        "decision:{} signer:{}",
                        env.decision_proof.decision_id, env.decision_proof.signer
                    )
                })
                .or_else(|| {
                    idempotency_key_from_headers(&headers).map(|key| format!("idempotency:{key}"))
                })
        }),
    );
    Json(record).into_response()
}

async fn post_cortex_artifact_checkout(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCheckoutRequest>,
) -> axum::response::Response {
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let ttl = request.lease_ttl_secs.unwrap_or(900).clamp(60, 3600);
    let acquired_at = Utc::now();
    let expires_at = acquired_at + chrono::Duration::seconds(ttl as i64);
    let lease = ArtifactLease {
        artifact_id: artifact_id.clone(),
        lease_id: format!("lease_{}_{}", artifact_id, Utc::now().timestamp_millis()),
        holder_id: actor_id.clone(),
        holder_role: actor_role.clone(),
        acquired_at: acquired_at.to_rfc3339(),
        renewed_at: acquired_at.to_rfc3339(),
        expires_at: expires_at.to_rfc3339(),
    };

    let mut leases = read_artifact_leases();
    leases.retain(|existing| {
        existing.artifact_id != artifact_id
            || DateTime::parse_from_rfc3339(&existing.expires_at)
                .map(|ts| ts.with_timezone(&Utc) < Utc::now())
                .unwrap_or(true)
    });
    if leases
        .iter()
        .any(|existing| existing.artifact_id == artifact_id)
    {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_LEASE_ALREADY_HELD",
            "Artifact is already leased by another actor.",
            Some(json!({ "artifactId": artifact_id })),
        );
    }
    leases.push(lease.clone());
    if let Err(err) = write_artifact_leases(&leases) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_LEASE_WRITE_FAILED",
            "Failed to persist artifact lease.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "checkout",
        &actor_role,
        &actor_id,
        "/artifacts",
        idempotency_key_from_headers(&headers),
    );
    Json(lease).into_response()
}

async fn post_cortex_artifact_lease_renew(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactLeaseRenewRequest>,
) -> axum::response::Response {
    if request.lease_id.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_LEASE_RENEW_REQUEST",
            "leaseId is required.",
            None,
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let ttl = request.lease_ttl_secs.unwrap_or(900).clamp(60, 3600);
    let mut leases = read_artifact_leases();
    let mut renewed = None;
    for lease in &mut leases {
        if lease.artifact_id != artifact_id || lease.lease_id != request.lease_id {
            continue;
        }
        if lease.holder_id != actor_id {
            return cortex_ux_error(
                StatusCode::FORBIDDEN,
                "ARTIFACT_LEASE_OWNERSHIP_MISMATCH",
                "Only lease holder may renew lease.",
                Some(
                    json!({ "artifactId": artifact_id, "holderId": lease.holder_id, "actorId": actor_id }),
                ),
            );
        }
        let renewed_at = Utc::now();
        lease.renewed_at = renewed_at.to_rfc3339();
        lease.expires_at = (renewed_at + chrono::Duration::seconds(ttl as i64)).to_rfc3339();
        renewed = Some(lease.clone());
        break;
    }
    let Some(lease) = renewed else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_LEASE_NOT_FOUND",
            "Artifact lease does not exist.",
            Some(json!({ "artifactId": artifact_id, "leaseId": request.lease_id })),
        );
    };
    if let Err(err) = write_artifact_leases(&leases) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_LEASE_WRITE_FAILED",
            "Failed to persist artifact lease renewal.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "lease_renew",
        &actor_role,
        &actor_id,
        "/artifacts",
        idempotency_key_from_headers(&headers),
    );
    Json(lease).into_response()
}

async fn post_cortex_artifact_lease_release(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactLeaseReleaseRequest>,
) -> axum::response::Response {
    if request.lease_id.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_LEASE_RELEASE_REQUEST",
            "leaseId is required.",
            None,
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let mut leases = read_artifact_leases();
    let before = leases.len();
    leases.retain(|lease| {
        !(lease.artifact_id == artifact_id
            && lease.lease_id == request.lease_id
            && lease.holder_id == actor_id)
    });
    if leases.len() == before {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_LEASE_NOT_FOUND",
            "Artifact lease does not exist or actor is not lease holder.",
            Some(json!({ "artifactId": artifact_id, "leaseId": request.lease_id })),
        );
    }
    if let Err(err) = write_artifact_leases(&leases) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_LEASE_WRITE_FAILED",
            "Failed to persist artifact lease release.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "lease_release",
        &actor_role,
        &actor_id,
        "/artifacts",
        idempotency_key_from_headers(&headers),
    );
    Json(json!({
        "accepted": true,
        "artifactId": artifact_id,
        "leaseId": request.lease_id
    }))
    .into_response()
}

async fn post_cortex_artifact_save(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactSaveRequest>,
) -> axum::response::Response {
    if request.lease_id.trim().is_empty()
        || request.expected_revision_id.trim().is_empty()
        || request.markdown_source.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_SAVE_REQUEST",
            "leaseId, expectedRevisionId, and markdownSource are required.",
            None,
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    if let Err(response) = require_active_lease(&artifact_id, &request.lease_id, &actor_id) {
        return response;
    }

    let mut items = read_artifacts_store();
    let mut revisions = read_artifact_revisions();
    let mut saved = None;
    for item in &mut items {
        if item.artifact_id != artifact_id {
            continue;
        }
        if item.head_revision_id != request.expected_revision_id {
            return cortex_ux_error(
                StatusCode::CONFLICT,
                "ARTIFACT_REVISION_CONFLICT",
                "expectedRevisionId does not match current head revision.",
                Some(json!({
                    "artifactId": artifact_id,
                    "expectedRevisionId": request.expected_revision_id,
                    "headRevisionId": item.head_revision_id
                })),
            );
        }
        let revision = upsert_artifact_revision(
            &item.artifact_id,
            &request.markdown_source,
            &actor_id,
            Some(item.head_revision_id.clone()),
            false,
        );
        item.head_revision_id = revision.revision_id.clone();
        item.version = item.version.saturating_add(1);
        item.updated_at = revision.created_at.clone();
        item.markdown_source = request.markdown_source.clone();
        item.content_hash = revision.content_hash.clone();
        item.rich_content = ArtifactRichContentProjection {
            hash: revision.content_hash.clone(),
            block_count: estimate_markdown_blocks(&request.markdown_source),
        };
        if let Some(title) = request.title.clone() {
            if !title.trim().is_empty() {
                item.title = title;
            }
        }
        revisions.push(revision);
        saved = Some(item.clone());
        break;
    }
    let Some(record) = saved else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };

    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist artifact save update.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifact_revisions(&revisions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REVISION_STORE_WRITE_FAILED",
            "Failed to persist artifact save revision.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &record.artifact_id,
        "save",
        &actor_role,
        &actor_id,
        "/artifacts",
        request.notes.or_else(|| {
            idempotency_key_from_headers(&headers).map(|key| format!("idempotency:{key}"))
        }),
    );
    Json(record).into_response()
}

async fn get_cortex_artifact_revisions(
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let items = read_artifacts_store();
    let Some(record) = items
        .into_iter()
        .find(|item| item.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let mut revisions = read_artifact_revisions()
        .into_iter()
        .filter(|revision| revision.artifact_id == record.artifact_id)
        .collect::<Vec<_>>();
    revisions.sort_by(|a, b| b.revision_number.cmp(&a.revision_number));
    Json(ArtifactRevisionListResponse {
        artifact_id: record.artifact_id,
        head_revision_id: record.head_revision_id,
        revisions,
    })
    .into_response()
}

async fn get_cortex_artifact_revision(
    Path((artifact_id, revision_id)): Path<(String, String)>,
) -> axum::response::Response {
    let revisions = read_artifact_revisions();
    if let Some(revision) = revisions
        .into_iter()
        .find(|item| item.artifact_id == artifact_id && item.revision_id == revision_id)
    {
        return Json(revision).into_response();
    }
    cortex_ux_error(
        StatusCode::NOT_FOUND,
        "ARTIFACT_REVISION_NOT_FOUND",
        "Artifact revision does not exist.",
        Some(json!({ "artifactId": artifact_id, "revisionId": revision_id })),
    )
}

async fn post_cortex_artifact_collab_session_open(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabSessionOpenRequest>,
) -> axum::response::Response {
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let ttl_secs = request.lease_ttl_secs.unwrap_or(900).clamp(60, 3600);

    let items = read_artifacts_store();
    let Some(artifact) = items.iter().find(|item| item.artifact_id == artifact_id) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };

    let mut sessions = read_collab_sessions();
    sessions.retain(|session| !(session.active && iso_timestamp_expired(&session.expires_at)));
    if sessions.iter().any(|session| {
        session.artifact_id == artifact_id && session.active && session.holder_id != actor_id
    }) {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SESSION_ACTIVE",
            "Another actor holds an active collaboration session for this artifact.",
            Some(json!({ "artifactId": artifact_id })),
        );
    }

    let now = Utc::now();
    let session_id = format!("collab_{}_{}", artifact_id, now.timestamp_millis());
    let lease_id = format!("lease_{}_{}", artifact_id, now.timestamp_millis());
    let session = ArtifactCollabSession {
        artifact_id: artifact_id.clone(),
        session_id: session_id.clone(),
        lease_id: lease_id.clone(),
        holder_id: actor_id.clone(),
        holder_role: actor_role.clone(),
        base_revision_id: artifact.head_revision_id.clone(),
        opened_at: now.to_rfc3339(),
        expires_at: (now + chrono::Duration::seconds(ttl_secs as i64)).to_rfc3339(),
        last_sequence: 0,
        active: true,
    };
    sessions.retain(|entry| !(entry.artifact_id == artifact_id && entry.holder_id == actor_id));
    sessions.push(session.clone());
    if let Err(err) = write_collab_sessions(&sessions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_COLLAB_SESSION_WRITE_FAILED",
            "Failed to persist collaboration session.",
            Some(json!({ "reason": err })),
        );
    }

    let mut leases = read_artifact_leases();
    leases.retain(|lease| {
        !(lease.artifact_id == artifact_id
            && (lease.holder_id == actor_id || iso_timestamp_expired(&lease.expires_at)))
    });
    if leases
        .iter()
        .any(|lease| lease.artifact_id == artifact_id && lease.holder_id != actor_id)
    {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_LEASE_ALREADY_HELD",
            "Artifact is already leased by another actor.",
            Some(json!({ "artifactId": artifact_id })),
        );
    }
    leases.push(ArtifactLease {
        artifact_id: artifact_id.clone(),
        lease_id: lease_id.clone(),
        holder_id: actor_id.clone(),
        holder_role: actor_role.clone(),
        acquired_at: session.opened_at.clone(),
        renewed_at: session.opened_at.clone(),
        expires_at: session.expires_at.clone(),
    });
    if let Err(err) = write_artifact_leases(&leases) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_LEASE_WRITE_FAILED",
            "Failed to persist collaboration lease.",
            Some(json!({ "reason": err })),
        );
    }

    let _ = append_artifact_audit(
        &artifact_id,
        "collab_session_open",
        &actor_role,
        &actor_id,
        "/artifacts",
        idempotency_key_from_headers(&headers),
    );
    let _ = upsert_artifact_presence(
        &artifact_id,
        &session.session_id,
        &actor_id,
        &actor_role,
        None,
        ttl_secs as i64,
    );
    Json(session).into_response()
}

async fn post_cortex_artifact_collab_op(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabOpRequest>,
) -> axum::response::Response {
    if request.session_id.trim().is_empty()
        || request.expected_head_revision_id.trim().is_empty()
        || request.op_type.trim().is_empty()
        || request.payload_markdown.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_OP_REQUEST",
            "sessionId, expectedHeadRevisionId, opType, and payloadMarkdown are required.",
            None,
        );
    }

    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let mut sessions = read_collab_sessions();
    let Some(session) = sessions.iter_mut().find(|session| {
        session.artifact_id == artifact_id && session.session_id == request.session_id
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_COLLAB_SESSION_NOT_FOUND",
            "Collaboration session does not exist.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    };
    if !session.active {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SESSION_CLOSED",
            "Collaboration session is closed.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    }
    if session.holder_id != actor_id {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_COLLAB_SESSION_OWNERSHIP_MISMATCH",
            "Only session holder may submit collaboration operations.",
            Some(
                json!({ "artifactId": artifact_id, "holderId": session.holder_id, "actorId": actor_id }),
            ),
        );
    }
    if iso_timestamp_expired(&session.expires_at) {
        session.active = false;
        let _ = write_collab_sessions(&sessions);
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SESSION_EXPIRED",
            "Collaboration session has expired.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    }
    if let Err(response) = require_active_lease(&artifact_id, &session.lease_id, &actor_id) {
        return response;
    }

    let mut items = read_artifacts_store();
    let mut revisions = read_artifact_revisions();
    let mut ops = read_collab_ops();
    let Some(index) = items
        .iter()
        .position(|item| item.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let current_head = items[index].head_revision_id.clone();
    let current_markdown = items[index].markdown_source.clone();
    let mut merge_result = ArtifactMergeResult {
        artifact_id: artifact_id.clone(),
        session_id: request.session_id.clone(),
        merge_status: "applied_head".to_string(),
        head_revision_id: current_head.clone(),
        merged_markdown: request.payload_markdown.clone(),
        conflict_summary: None,
    };

    if request.expected_head_revision_id != current_head {
        let proposed_base = request
            .proposed_base_revision_id
            .clone()
            .unwrap_or_else(|| request.expected_head_revision_id.clone());
        let Some(base_markdown) = revision_markdown(&revisions, &proposed_base) else {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "ARTIFACT_COLLAB_BASE_REVISION_NOT_FOUND",
                "proposedBaseRevisionId was not found in revision history.",
                Some(json!({ "artifactId": artifact_id, "proposedBaseRevisionId": proposed_base })),
            );
        };
        let mut merged =
            three_way_merge(&base_markdown, &current_markdown, &request.payload_markdown);
        merged.artifact_id = artifact_id.clone();
        merged.session_id = request.session_id.clone();
        merged.head_revision_id = current_head.clone();
        merge_result = merged;
    }

    if merge_result.merge_status != "merge_required"
        && merge_result.merged_markdown != current_markdown
    {
        let revision = upsert_artifact_revision(
            &artifact_id,
            &merge_result.merged_markdown,
            &actor_id,
            Some(current_head.clone()),
            false,
        );
        items[index].head_revision_id = revision.revision_id.clone();
        items[index].version = items[index].version.saturating_add(1);
        items[index].updated_at = revision.created_at.clone();
        items[index].markdown_source = merge_result.merged_markdown.clone();
        items[index].content_hash = revision.content_hash.clone();
        items[index].rich_content = ArtifactRichContentProjection {
            hash: revision.content_hash.clone(),
            block_count: estimate_markdown_blocks(&merge_result.merged_markdown),
        };
        merge_result.head_revision_id = revision.revision_id.clone();
        revisions.push(revision);
    }

    let next_sequence = ops
        .iter()
        .filter(|op| op.artifact_id == artifact_id && op.session_id == request.session_id)
        .map(|op| op.sequence)
        .max()
        .unwrap_or(0)
        + 1;

    let op = ArtifactCollabOp {
        artifact_id: artifact_id.clone(),
        session_id: request.session_id.clone(),
        op_id: format!(
            "collab_op_{}_{}",
            artifact_id,
            Utc::now().timestamp_millis()
        ),
        sequence: next_sequence,
        op_type: request.op_type,
        actor_id: actor_id.clone(),
        proposed_base_revision_id: request
            .proposed_base_revision_id
            .unwrap_or_else(|| request.expected_head_revision_id.clone()),
        expected_head_revision_id: request.expected_head_revision_id,
        applied_head_revision_id: merge_result.head_revision_id.clone(),
        created_at: now_iso(),
        merge_status: merge_result.merge_status.clone(),
    };
    ops.push(op.clone());

    if let Err(err) = write_collab_ops(&ops) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_COLLAB_OP_WRITE_FAILED",
            "Failed to persist collaboration operation log.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist artifact state for collaboration operation.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifact_revisions(&revisions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REVISION_STORE_WRITE_FAILED",
            "Failed to persist collaboration revision state.",
            Some(json!({ "reason": err })),
        );
    }

    let _ = append_artifact_audit(
        &artifact_id,
        "collab_op",
        &actor_role,
        &actor_id,
        "/artifacts",
        Some(format!(
            "session:{} seq:{} status:{}",
            request.session_id, op.sequence, merge_result.merge_status
        )),
    );
    Json(json!({
        "op": op,
        "mergeResult": merge_result
    }))
    .into_response()
}

async fn post_cortex_artifact_collab_session_close(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabSessionCloseRequest>,
) -> axum::response::Response {
    if request.session_id.trim().is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_SESSION_CLOSE_REQUEST",
            "sessionId is required.",
            None,
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    let mut sessions = read_collab_sessions();
    let mut closed = None;
    for session in &mut sessions {
        if session.artifact_id != artifact_id || session.session_id != request.session_id {
            continue;
        }
        if session.holder_id != actor_id {
            return cortex_ux_error(
                StatusCode::FORBIDDEN,
                "ARTIFACT_COLLAB_SESSION_OWNERSHIP_MISMATCH",
                "Only session holder may close collaboration session.",
                Some(
                    json!({ "artifactId": artifact_id, "holderId": session.holder_id, "actorId": actor_id }),
                ),
            );
        }
        session.active = false;
        closed = Some(session.clone());
        break;
    }
    let Some(session) = closed else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_COLLAB_SESSION_NOT_FOUND",
            "Collaboration session does not exist.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    };

    if let Err(err) = write_collab_sessions(&sessions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_COLLAB_SESSION_WRITE_FAILED",
            "Failed to persist collaboration session close.",
            Some(json!({ "reason": err })),
        );
    }
    let mut leases = read_artifact_leases();
    leases
        .retain(|lease| !(lease.artifact_id == artifact_id && lease.lease_id == session.lease_id));
    let _ = write_artifact_leases(&leases);
    let mut presence = clean_expired_presence(read_artifact_crdt_presence(&artifact_id));
    presence
        .retain(|entry| !(entry.session_id == request.session_id && entry.actor_id == actor_id));
    let _ = write_artifact_crdt_presence(&artifact_id, &presence);

    let _ = append_artifact_audit(
        &artifact_id,
        "collab_session_close",
        &actor_role,
        &actor_id,
        "/artifacts",
        Some(format!("session:{}", request.session_id)),
    );
    Json(session).into_response()
}

async fn get_cortex_artifact_collab_session(
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let mut sessions = read_collab_sessions()
        .into_iter()
        .filter(|session| session.artifact_id == artifact_id && session.active)
        .collect::<Vec<_>>();
    sessions.sort_by(|a, b| b.opened_at.cmp(&a.opened_at));
    Json(json!({
        "artifactId": artifact_id,
        "sessions": sessions
    }))
    .into_response()
}

async fn get_cortex_artifact_collab_state(
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let items = read_artifacts_store();
    let Some(artifact) = items
        .into_iter()
        .find(|item| item.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };

    let state = read_artifact_crdt_state(&artifact_id, &artifact.markdown_source);
    let materialized = materialize_crdt_markdown(&state);
    let mut conflicts = Vec::new();
    if materialized != artifact.markdown_source {
        conflicts.push(ArtifactCrdtConflict {
            code: "CRDT_PROJECTION_DRIFT".to_string(),
            message: "CRDT materialized markdown differs from artifact head projection."
                .to_string(),
            blocking: false,
            details: Some(format!("headRevisionId={}", artifact.head_revision_id)),
        });
    }

    let mut sessions = read_collab_sessions()
        .into_iter()
        .filter(|session| session.artifact_id == artifact_id && session.active)
        .filter(|session| !iso_timestamp_expired(&session.expires_at))
        .collect::<Vec<_>>();
    sessions.sort_by(|a, b| b.opened_at.cmp(&a.opened_at));

    let mut presence = read_artifact_crdt_presence(&artifact_id);
    let original_presence_len = presence.len();
    presence = clean_expired_presence(presence);
    if presence.len() != original_presence_len {
        let _ = write_artifact_crdt_presence(&artifact_id, &presence);
    }

    let source_state = cortex_ux_source_state();
    Json(ArtifactCrdtStateResponse {
        schema_version: "1.0.0".to_string(),
        artifact_id: artifact_id.clone(),
        head_revision_id: artifact.head_revision_id,
        materialized_markdown: materialized,
        op_count: state.applied_op_ids.len() as u64,
        source_of_truth: source_state.source_of_truth,
        fallback_active: source_state.fallback_active,
        sessions,
        presence,
        conflicts,
    })
    .into_response()
}

async fn post_cortex_artifact_collab_op_batch(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabOpBatchRequest>,
) -> axum::response::Response {
    if request.session_id.trim().is_empty() || request.operations.is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_BATCH_REQUEST",
            "sessionId and operations are required.",
            None,
        );
    }
    if request.batch_sequence == 0 {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_BATCH_SEQUENCE",
            "batchSequence must be >= 1.",
            None,
        );
    }

    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("operator") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_COLLAB_ROLE_DENIED",
            "Operator role or higher is required for collaboration operations.",
            Some(json!({ "actorRole": actor_role })),
        );
    }

    let mut sessions = read_collab_sessions();
    let Some(session_idx) = sessions.iter().position(|session| {
        session.artifact_id == artifact_id && session.session_id == request.session_id
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_COLLAB_SESSION_NOT_FOUND",
            "Collaboration session does not exist.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    };
    if !sessions[session_idx].active {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SESSION_CLOSED",
            "Collaboration session is closed.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    }
    if iso_timestamp_expired(&sessions[session_idx].expires_at) {
        sessions[session_idx].active = false;
        let _ = write_collab_sessions(&sessions);
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SESSION_EXPIRED",
            "Collaboration session has expired.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    }
    if request.batch_sequence <= sessions[session_idx].last_sequence {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_COLLAB_SEQUENCE_CONFLICT",
            "batchSequence must be strictly monotonic per session.",
            Some(json!({
                "artifactId": artifact_id,
                "sessionId": request.session_id,
                "lastSequence": sessions[session_idx].last_sequence
            })),
        );
    }

    let mut items = read_artifacts_store();
    let Some(artifact_idx) = items
        .iter()
        .position(|item| item.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let mut revisions = read_artifact_revisions();
    let current_head = items[artifact_idx].head_revision_id.clone();
    if let Some(expected_head) = request.expected_head_revision_id.as_ref() {
        if expected_head != &current_head {
            return cortex_ux_error(
                StatusCode::CONFLICT,
                "ARTIFACT_COLLAB_HEAD_CONFLICT",
                "expectedHeadRevisionId does not match current head.",
                Some(json!({
                    "artifactId": artifact_id,
                    "expectedHeadRevisionId": expected_head,
                    "currentHeadRevisionId": current_head
                })),
            );
        }
    }

    let mut state = read_artifact_crdt_state(&artifact_id, &items[artifact_idx].markdown_source);
    let mut existing_ops = read_artifact_crdt_ops(&artifact_id);
    let mut existing_ids = existing_ops
        .iter()
        .map(|entry| entry.op_id.clone())
        .collect::<HashSet<_>>();

    let mut operations = request.operations.clone();
    operations.sort_by(|a, b| {
        a.lamport
            .cmp(&b.lamport)
            .then_with(|| a.op_id.cmp(&b.op_id))
    });

    let mut new_ops = Vec::new();
    let mut applied = 0usize;
    let mut idempotent = 0usize;
    for operation in operations {
        if existing_ids.contains(&operation.op_id) {
            idempotent = idempotent.saturating_add(1);
            continue;
        }
        let sequence = state.last_sequence.saturating_add(1);
        let lamport = state.last_lamport.max(operation.lamport).saturating_add(1);
        let envelope = build_replace_markdown_update(
            &state,
            &artifact_id,
            &request.session_id,
            &actor_id,
            &operation.op_id,
            sequence,
            lamport,
            &operation.markdown_source,
            operation.stream_channel.clone(),
            now_iso(),
        );
        if let Err(err) = apply_crdt_update(&mut state, &envelope, now_iso()) {
            return cortex_ux_error(
                StatusCode::CONFLICT,
                "ARTIFACT_CRDT_APPLY_FAILED",
                "Failed to apply CRDT update envelope.",
                Some(json!({ "artifactId": artifact_id, "opId": operation.op_id, "reason": err })),
            );
        }
        existing_ids.insert(envelope.op_id.clone());
        new_ops.push(envelope);
        applied = applied.saturating_add(1);
    }

    let materialized = materialize_crdt_markdown(&state);
    let mut revision_created = false;
    if materialized != items[artifact_idx].markdown_source {
        let parent_head = items[artifact_idx].head_revision_id.clone();
        let revision = upsert_artifact_revision(
            &artifact_id,
            &materialized,
            &actor_id,
            Some(parent_head),
            false,
        );
        items[artifact_idx].head_revision_id = revision.revision_id.clone();
        items[artifact_idx].version = items[artifact_idx].version.saturating_add(1);
        items[artifact_idx].updated_at = revision.created_at.clone();
        items[artifact_idx].markdown_source = materialized.clone();
        items[artifact_idx].content_hash = revision.content_hash.clone();
        items[artifact_idx].rich_content = ArtifactRichContentProjection {
            hash: revision.content_hash.clone(),
            block_count: estimate_markdown_blocks(&materialized),
        };
        revisions.push(revision);
        revision_created = true;
    }

    if let Err(err) = write_artifact_crdt_state(&artifact_id, &state) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_CRDT_STATE_WRITE_FAILED",
            "Failed to persist CRDT snapshot.",
            Some(json!({ "reason": err })),
        );
    }
    let published_ops = new_ops.clone();
    if !new_ops.is_empty() {
        existing_ops.extend(new_ops);
        if let Err(err) = write_artifact_crdt_ops(&artifact_id, &existing_ops) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ARTIFACT_CRDT_OPS_WRITE_FAILED",
                "Failed to persist CRDT operation log.",
                Some(json!({ "reason": err })),
            );
        }
    }
    if revision_created {
        if let Err(err) = write_artifacts_store(&items) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ARTIFACT_STORE_WRITE_FAILED",
                "Failed to persist artifact state.",
                Some(json!({ "reason": err })),
            );
        }
        if let Err(err) = write_artifact_revisions(&revisions) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ARTIFACT_REVISION_STORE_WRITE_FAILED",
                "Failed to persist artifact revisions.",
                Some(json!({ "reason": err })),
            );
        }
    }

    sessions[session_idx].last_sequence = request.batch_sequence;
    sessions[session_idx].expires_at = (Utc::now() + chrono::Duration::seconds(900)).to_rfc3339();
    if let Err(err) = write_collab_sessions(&sessions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_COLLAB_SESSION_WRITE_FAILED",
            "Failed to persist collaboration session sequence state.",
            Some(json!({ "reason": err })),
        );
    }

    let presence = match upsert_artifact_presence(
        &artifact_id,
        &request.session_id,
        &actor_id,
        &actor_role,
        request.cursor.clone(),
        120,
    ) {
        Ok(items) => items,
        Err(err) => {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ARTIFACT_COLLAB_PRESENCE_WRITE_FAILED",
                "Failed to persist collaboration presence.",
                Some(json!({ "reason": err })),
            );
        }
    };

    if realtime_feature_enabled() {
        for op in published_ops {
            let envelope = ArtifactRealtimeEnvelope {
                schema_version: "1.0.0".to_string(),
                channel: artifact_realtime_channel(&artifact_id),
                artifact_id: artifact_id.clone(),
                session_id: request.session_id.clone(),
                actor_id: actor_id.clone(),
                op_id: op.op_id.clone(),
                sequence: op.sequence,
                lamport: op.lamport,
                event_type: "op_applied".to_string(),
                timestamp: now_iso(),
                payload: json!({
                    "headRevisionId": items[artifact_idx].head_revision_id,
                    "materializedMarkdown": materialized,
                    "batchSequence": request.batch_sequence
                }),
            };
            let _ = streaming_transport_manager().publish(envelope).await;
        }
        let presence_event = ArtifactRealtimeEnvelope {
            schema_version: "1.0.0".to_string(),
            channel: artifact_realtime_channel(&artifact_id),
            artifact_id: artifact_id.clone(),
            session_id: request.session_id.clone(),
            actor_id: actor_id.clone(),
            op_id: format!("presence_{}_{}", artifact_id, Utc::now().timestamp_millis()),
            sequence: request.batch_sequence,
            lamport: request.batch_sequence,
            event_type: "presence_update".to_string(),
            timestamp: now_iso(),
            payload: json!({
                "presenceCount": presence.len(),
            }),
        };
        let _ = streaming_transport_manager().publish(presence_event).await;
    }

    let _ = append_artifact_audit(
        &artifact_id,
        "collab_op_batch",
        &actor_role,
        &actor_id,
        "/artifacts",
        Some(format!(
            "session:{} batchSequence:{} applied:{} idempotent:{}",
            request.session_id, request.batch_sequence, applied, idempotent
        )),
    );

    let source_state = cortex_ux_source_state();
    let realtime_status = streaming_transport_manager().status().await;
    Json(json!({
        "artifactId": artifact_id,
        "sessionId": request.session_id,
        "batchSequence": request.batch_sequence,
        "applied": applied,
        "idempotent": idempotent,
        "headRevisionId": items[artifact_idx].head_revision_id,
        "materializedMarkdown": materialized,
        "opCount": state.applied_op_ids.len(),
        "presenceCount": presence.len(),
        "sourceOfTruth": source_state.source_of_truth,
        "fallbackActive": source_state.fallback_active,
        "realtime": realtime_status
    }))
    .into_response()
}

async fn get_cortex_artifact_collab_ops(
    Path(artifact_id): Path<String>,
    Query(query): Query<ArtifactCollabOpsQuery>,
) -> axum::response::Response {
    let mut ops = read_artifact_crdt_ops(&artifact_id);
    ops.sort_by(|a, b| a.sequence.cmp(&b.sequence));
    if let Some(since) = query.since_sequence {
        ops.retain(|op| op.sequence > since);
    }
    let limit = query.limit.unwrap_or(250).clamp(1, 2000);
    if ops.len() > limit {
        let drop = ops.len() - limit;
        ops.drain(0..drop);
    }
    Json(json!({
        "artifactId": artifact_id,
        "count": ops.len(),
        "ops": ops
    }))
    .into_response()
}

async fn get_cortex_artifact_collab_presence(
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let before = read_artifact_crdt_presence(&artifact_id);
    let after = clean_expired_presence(before.clone());
    if after.len() != before.len() {
        let _ = write_artifact_crdt_presence(&artifact_id, &after);
    }
    Json(json!({
        "artifactId": artifact_id,
        "presence": after
    }))
    .into_response()
}

async fn post_cortex_artifact_collab_checkpoint(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabCheckpointRequest>,
) -> axum::response::Response {
    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("operator") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_COLLAB_CHECKPOINT_ROLE_DENIED",
            "Operator role or higher is required for checkpoint compaction.",
            Some(json!({ "actorRole": actor_role })),
        );
    }

    let items = read_artifacts_store();
    let Some(artifact) = items.iter().find(|item| item.artifact_id == artifact_id) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let state = read_artifact_crdt_state(&artifact_id, &artifact.markdown_source);
    if let Err(err) = write_artifact_crdt_state(&artifact_id, &state) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_CRDT_CHECKPOINT_WRITE_FAILED",
            "Failed to persist CRDT checkpoint snapshot.",
            Some(json!({ "reason": err })),
        );
    }

    let max_ops = request
        .max_ops_after_compaction
        .unwrap_or(500)
        .clamp(25, 5000);
    let mut ops = read_artifact_crdt_ops(&artifact_id);
    ops.sort_by(|a, b| a.sequence.cmp(&b.sequence));
    if ops.len() > max_ops {
        let retained = ops.split_off(ops.len() - max_ops);
        if let Err(err) = write_artifact_crdt_ops(&artifact_id, &retained) {
            return cortex_ux_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "ARTIFACT_CRDT_COMPACTION_WRITE_FAILED",
                "Failed to persist compacted CRDT op log.",
                Some(json!({ "reason": err })),
            );
        }
    }

    let snapshot_path = cortex_ux_artifact_crdt_snapshot_path(&artifact_id);
    let checkpoint = ArtifactCollabCheckpoint {
        checkpoint_id: format!(
            "checkpoint_{}_{}",
            artifact_id,
            Utc::now().timestamp_millis()
        ),
        artifact_id: artifact_id.clone(),
        created_at: now_iso(),
        op_count: state.applied_op_ids.len() as u64,
        state_hash: crdt_state_hash(&state),
        snapshot_key: to_cortex_vfs_key(&snapshot_path)
            .unwrap_or_else(|| snapshot_path.display().to_string()),
    };
    let _ = append_artifact_audit(
        &artifact_id,
        "collab_checkpoint",
        &actor_role,
        &actor_id,
        "/artifacts",
        Some(format!(
            "checkpoint:{} maxOpsAfterCompaction:{}",
            checkpoint.checkpoint_id, max_ops
        )),
    );
    Json(checkpoint).into_response()
}

async fn post_cortex_artifact_collab_force_resolve(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactCollabForceResolveRequest>,
) -> axum::response::Response {
    if request.session_id.trim().is_empty()
        || request.markdown_source.trim().is_empty()
        || request.approved_by.trim().is_empty()
        || request.rationale.trim().is_empty()
        || request.approved_at.trim().is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_FORCE_RESOLVE_REQUEST",
            "sessionId, markdownSource, approvedBy, rationale, and approvedAt are required.",
            None,
        );
    }
    if !parse_metric_date(&request.approved_at) {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "INVALID_ARTIFACT_COLLAB_FORCE_RESOLVE_DATE",
            "approvedAt must be RFC3339.",
            None,
        );
    }

    let actor_id = actor_id_from_headers(&headers);
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("steward") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_COLLAB_FORCE_RESOLVE_STEWARD_REQUIRED",
            "Steward role is required for force-resolve actions.",
            Some(json!({ "actorRole": actor_role })),
        );
    }
    if let Err(response) = require_governance_envelope(&actor_id, request.governance.as_ref()) {
        return response;
    }
    if let Some(governance) = request.governance.as_ref() {
        if governance.approved_by != request.approved_by
            || governance.rationale != request.rationale
            || governance.approved_at != request.approved_at
        {
            return cortex_ux_error(
                StatusCode::BAD_REQUEST,
                "ARTIFACT_GOVERNANCE_FIELDS_MISMATCH",
                "governance envelope fields must match approvedBy/rationale/approvedAt.",
                None,
            );
        }
    }

    let mut sessions = read_collab_sessions();
    let Some(session_idx) = sessions.iter().position(|session| {
        session.artifact_id == artifact_id && session.session_id == request.session_id
    }) else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_COLLAB_SESSION_NOT_FOUND",
            "Collaboration session does not exist.",
            Some(json!({ "artifactId": artifact_id, "sessionId": request.session_id })),
        );
    };

    let mut items = read_artifacts_store();
    let Some(artifact_idx) = items
        .iter()
        .position(|item| item.artifact_id == artifact_id)
    else {
        return cortex_ux_error(
            StatusCode::NOT_FOUND,
            "ARTIFACT_NOT_FOUND",
            "Artifact does not exist.",
            Some(json!({ "artifactId": artifact_id })),
        );
    };
    let mut revisions = read_artifact_revisions();
    let parent_head = items[artifact_idx].head_revision_id.clone();
    let revision = upsert_artifact_revision(
        &artifact_id,
        &request.markdown_source,
        &actor_id,
        Some(parent_head),
        false,
    );
    items[artifact_idx].head_revision_id = revision.revision_id.clone();
    items[artifact_idx].version = items[artifact_idx].version.saturating_add(1);
    items[artifact_idx].updated_at = revision.created_at.clone();
    items[artifact_idx].markdown_source = request.markdown_source.clone();
    items[artifact_idx].content_hash = revision.content_hash.clone();
    items[artifact_idx].rich_content = ArtifactRichContentProjection {
        hash: revision.content_hash.clone(),
        block_count: estimate_markdown_blocks(&request.markdown_source),
    };
    revisions.push(revision);

    let state = init_crdt_state(&artifact_id, &request.markdown_source, now_iso());
    if let Err(err) = write_artifact_crdt_state(&artifact_id, &state) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_CRDT_STATE_WRITE_FAILED",
            "Failed to persist CRDT force-resolve snapshot.",
            Some(json!({ "reason": err })),
        );
    }
    let synthetic = ArtifactCrdtUpdateEnvelope {
        op_id: format!(
            "force_resolve_{}_{}",
            artifact_id,
            Utc::now().timestamp_millis()
        ),
        artifact_id: artifact_id.clone(),
        session_id: request.session_id.clone(),
        actor_id: actor_id.clone(),
        sequence: sessions[session_idx].last_sequence.saturating_add(1),
        lamport: 1,
        created_at: now_iso(),
        stream_channel: None,
        mutations: Vec::new(),
        agui_mutations: Vec::new(),
    };
    let mut ops = read_artifact_crdt_ops(&artifact_id);
    ops.push(synthetic);
    if let Err(err) = write_artifact_crdt_ops(&artifact_id, &ops) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_CRDT_OPS_WRITE_FAILED",
            "Failed to persist force-resolve CRDT operation.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifacts_store(&items) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_STORE_WRITE_FAILED",
            "Failed to persist force-resolve artifact state.",
            Some(json!({ "reason": err })),
        );
    }
    if let Err(err) = write_artifact_revisions(&revisions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REVISION_STORE_WRITE_FAILED",
            "Failed to persist force-resolve revision.",
            Some(json!({ "reason": err })),
        );
    }

    sessions[session_idx].last_sequence = sessions[session_idx].last_sequence.saturating_add(1);
    if let Err(err) = write_collab_sessions(&sessions) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_COLLAB_SESSION_WRITE_FAILED",
            "Failed to persist collaboration session state.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = upsert_artifact_presence(
        &artifact_id,
        &request.session_id,
        &actor_id,
        &actor_role,
        request.cursor.clone(),
        120,
    );

    let decision = UxPromotionDecision {
        decision_id: format!("ux_promotion_{}", Utc::now().timestamp_millis()),
        candidate_id: artifact_id.clone(),
        route_id: "/artifacts".to_string(),
        view_capability_id: "view.artifacts".to_string(),
        promotion_action: "force_resolve_conflict".to_string(),
        approved_by: request.approved_by,
        rationale: request.rationale,
        timestamp: request.approved_at,
    };
    if let Err(err) = append_json_line(&cortex_ux_promotion_log_path(), &decision) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PROMOTION_APPROVAL_PERSIST_FAILED",
            "Failed to persist force-resolve governance decision.",
            Some(json!({ "reason": err })),
        );
    }
    let _ = append_artifact_audit(
        &artifact_id,
        "collab_force_resolve",
        &actor_role,
        &actor_id,
        "/artifacts",
        Some(format!(
            "session:{} decision:{} proof:{}",
            request.session_id,
            decision.decision_id,
            request
                .governance
                .as_ref()
                .map(|value| value.decision_proof.decision_id.clone())
                .unwrap_or_default()
        )),
    );

    Json(json!({
        "accepted": true,
        "artifactId": artifact_id,
        "headRevisionId": items[artifact_idx].head_revision_id,
        "stateHash": crdt_state_hash(&state),
        "promotionDecision": decision
    }))
    .into_response()
}

async fn get_cortex_artifact_collab_realtime_status(
    Path(_artifact_id): Path<String>,
) -> Json<ArtifactRealtimeTransportStatus> {
    Json(streaming_transport_manager().status().await)
}

async fn post_cortex_artifact_collab_realtime_connect(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactRealtimeConnectRequest>,
) -> axum::response::Response {
    if !realtime_feature_enabled() {
        return cortex_ux_error(
            StatusCode::CONFLICT,
            "ARTIFACT_REALTIME_DISABLED",
            "Realtime collaboration feature flag is disabled.",
            None,
        );
    }
    let actor_id = request
        .actor_id
        .unwrap_or_else(|| actor_id_from_headers(&headers));
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("operator") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_REALTIME_CONNECT_ROLE_DENIED",
            "Operator role or higher is required for realtime connect.",
            Some(json!({ "actorRole": actor_role })),
        );
    }
    match streaming_transport_manager()
        .connect(&actor_id, &artifact_id)
        .await
    {
        Ok(ack) => Json(ack).into_response(),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REALTIME_CONNECT_FAILED",
            "Failed to connect realtime collaboration transport.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn post_cortex_artifact_collab_realtime_disconnect(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactRealtimeDisconnectRequest>,
) -> axum::response::Response {
    let actor_id = request
        .actor_id
        .unwrap_or_else(|| actor_id_from_headers(&headers));
    match streaming_transport_manager()
        .disconnect(&actor_id, &artifact_id)
        .await
    {
        Ok(ack) => Json(ack).into_response(),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REALTIME_DISCONNECT_FAILED",
            "Failed to disconnect realtime collaboration transport.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_cortex_artifact_collab_realtime_backlog(
    Path(artifact_id): Path<String>,
    Query(query): Query<ArtifactRealtimeBacklogQuery>,
) -> Json<Value> {
    let mut backlog: Vec<ArtifactRealtimeBacklogItem> = streaming_transport_manager()
        .backlog(Some(&artifact_id))
        .await;
    backlog.sort_by(|a, b| b.enqueued_at.cmp(&a.enqueued_at));
    let limit = query.limit.unwrap_or(250).clamp(1, 2000);
    if backlog.len() > limit {
        backlog.truncate(limit);
    }
    Json(json!({
        "artifactId": artifact_id,
        "count": backlog.len(),
        "items": backlog,
    }))
}

async fn get_cortex_artifact_collab_realtime_integrity(
    Path(artifact_id): Path<String>,
) -> Json<ArtifactRealtimeIntegrityReport> {
    Json(
        streaming_transport_manager()
            .integrity_report(&artifact_id)
            .await,
    )
}

async fn post_cortex_artifact_collab_realtime_resync(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
) -> axum::response::Response {
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("operator") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_REALTIME_RESYNC_ROLE_DENIED",
            "Operator role or higher is required for realtime resync.",
            Some(json!({ "actorRole": actor_role })),
        );
    }
    match streaming_transport_manager()
        .resync_channel(&artifact_id)
        .await
    {
        Ok(report) => Json::<ArtifactRealtimeResyncResult>(report).into_response(),
        Err(err) => cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "ARTIFACT_REALTIME_RESYNC_FAILED",
            "Failed to replay/resync realtime channel.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_cortex_artifact_collab_realtime_ack(Path(artifact_id): Path<String>) -> Json<Value> {
    let cursor: Option<ArtifactRealtimeAckCursor> =
        streaming_transport_manager().ack_cursor(&artifact_id).await;
    Json(json!({
        "artifactId": artifact_id,
        "ackCursor": cursor,
    }))
}

async fn post_cortex_artifact_collab_realtime_ack_reset(
    headers: HeaderMap,
    Path(artifact_id): Path<String>,
    Json(request): Json<ArtifactRealtimeAckResetRequest>,
) -> axum::response::Response {
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("steward") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "ARTIFACT_REALTIME_ACK_RESET_ROLE_DENIED",
            "Steward role is required to reset realtime ack cursor.",
            Some(json!({ "actorRole": actor_role })),
        );
    }
    let actor_id = actor_id_from_headers(&headers);
    if let Err(response) = require_governance_envelope(&actor_id, request.governance.as_ref()) {
        return response;
    }
    let reset = streaming_transport_manager()
        .reset_ack_cursor(&artifact_id)
        .await;
    Json(json!({
        "artifactId": artifact_id,
        "reset": reset,
    }))
    .into_response()
}

async fn ws_collab_handler(
    ws: WebSocketUpgrade,
    State(_state): State<GatewayState>,
) -> impl IntoResponse {
    ws.on_upgrade(handle_collab_socket)
}

async fn handle_collab_socket(socket: WebSocket) {
    let (mut sender, mut receiver) = socket.split();
    let mut subscribed = HashSet::<String>::new();
    let mut since_nonce = 0u64;
    let mut heartbeat = tokio::time::interval(std::time::Duration::from_millis(700));
    let mut actor_id = "cortex-desktop".to_string();

    loop {
        tokio::select! {
            _ = heartbeat.tick() => {
                let ArtifactRealtimePollResult { next_nonce, events } = streaming_transport_manager()
                    .poll(since_nonce, 200, Some(&subscribed))
                    .await;
                since_nonce = next_nonce;
                if !events.is_empty() {
                    let payload = json!({
                        "type": "events",
                        "count": events.len(),
                        "events": events,
                        "nonce": since_nonce
                    });
                    if sender.send(Message::Text(payload.to_string())).await.is_err() {
                        break;
                    }
                }
                let status = streaming_transport_manager().status().await;
                let status_payload = json!({
                    "type": "status",
                    "status": status
                });
                if sender.send(Message::Text(status_payload.to_string())).await.is_err() {
                    break;
                }
            }
            incoming = receiver.next() => {
                let Some(Ok(message)) = incoming else {
                    break;
                };
                match message {
                    Message::Text(text) => {
                        let Ok(command) = serde_json::from_str::<ArtifactRealtimeSubscribe>(&text) else {
                            let _ = sender.send(Message::Text(json!({
                                "type": "error",
                                "code": "INVALID_WS_COMMAND",
                                "message": "Expected JSON command payload."
                            }).to_string())).await;
                            continue;
                        };
                        match command.action.as_str() {
                            "subscribe" => {
                                if let Some(artifact_id) = command.artifact_id.as_ref() {
                                    subscribed.insert(artifact_id.clone());
                                }
                                if let Some(nonce) = command.nonce {
                                    since_nonce = nonce;
                                }
                                if let Some(value) = command.actor_id {
                                    actor_id = value;
                                }
                                let _ = streaming_transport_manager().connect(
                                    &actor_id,
                                    command.artifact_id.as_deref().unwrap_or_default(),
                                ).await;
                                let _ = sender.send(Message::Text(json!({
                                    "type": "subscribed",
                                    "actorId": actor_id,
                                    "artifacts": subscribed,
                                    "nonce": since_nonce
                                }).to_string())).await;
                            }
                            "unsubscribe" => {
                                if let Some(artifact_id) = command.artifact_id.as_ref() {
                                    subscribed.remove(artifact_id);
                                    let _ = streaming_transport_manager()
                                        .disconnect(&actor_id, artifact_id)
                                        .await;
                                }
                                let _ = sender.send(Message::Text(json!({
                                    "type": "unsubscribed",
                                    "actorId": actor_id,
                                    "artifacts": subscribed
                                }).to_string())).await;
                            }
                            "replay" => {
                                let pending = streaming_transport_manager().replay_pending().await.unwrap_or(0);
                                let _ = sender.send(Message::Text(json!({
                                    "type": "replay",
                                    "pending": pending
                                }).to_string())).await;
                            }
                            "ping" => {
                                let _ = sender.send(Message::Text(json!({
                                    "type": "pong",
                                    "actorId": actor_id,
                                    "timestamp": now_iso()
                                }).to_string())).await;
                            }
                            _ => {
                                let _ = sender.send(Message::Text(json!({
                                    "type": "error",
                                    "code": "UNKNOWN_WS_ACTION",
                                    "message": "Unknown action. Expected subscribe/unsubscribe/replay/ping."
                                }).to_string())).await;
                            }
                        }
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    }
}

fn motoko_graph_log_dir() -> PathBuf {
    std::env::var("NOSTRA_MOTOKO_GRAPH_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace_logs_dir()
                .join("knowledge_graphs")
                .join("motoko_graph")
        })
}

fn motoko_graph_history_dir() -> PathBuf {
    motoko_graph_log_dir().join("history")
}

fn motoko_graph_pending_dir() -> PathBuf {
    motoko_graph_log_dir().join("decisions").join("pending")
}

fn motoko_graph_snapshot_path() -> PathBuf {
    motoko_graph_log_dir().join("snapshot_latest.json")
}

fn motoko_graph_monitoring_runs_dir() -> PathBuf {
    motoko_graph_log_dir().join("monitoring_runs")
}

fn motoko_graph_monitoring_trend_path() -> PathBuf {
    motoko_graph_log_dir().join("monitoring_trend_latest.json")
}

fn dpub_managed_workspace_root(space_id: &str) -> PathBuf {
    workspace_root().join("_spaces").join(space_id)
}

fn dpub_workspace_root(space_id: Option<&str>) -> PathBuf {
    if let Some(sid) = space_id {
        let registry_path = workspace_root().join("_spaces").join("registry.json");
        if let Ok(reg) = cortex_domain::spaces::SpaceRegistry::load_from_path(&registry_path) {
            if let Some(space) = reg.get(sid) {
                if let Some(uri) = &space.reference_uri {
                    return PathBuf::from(uri);
                }
            }
        }
        return dpub_managed_workspace_root(sid);
    }

    std::env::var("NOSTRA_DPUB_WORKSPACE_ROOT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_root())
}

fn dpub_graph_workspace_dir(space_id: Option<&str>) -> PathBuf {
    // If we have a space_id, the metadata (graph/sims) goes to the MANAGED VFS state,
    // not into the Reference_URI folder, keeping the physical source pristine.
    let base = if let Some(sid) = space_id {
        dpub_managed_workspace_root(sid)
    } else {
        dpub_workspace_root(None)
    };
    base.join("research").join("000-contribution-graph")
}

fn dpub_graph_path(space_id: Option<&str>) -> PathBuf {
    dpub_graph_workspace_dir(space_id).join("contribution_graph.json")
}

fn dpub_path_assessment_path(space_id: Option<&str>) -> PathBuf {
    dpub_graph_workspace_dir(space_id).join("path_assessment.json")
}

fn dpub_doctor_path(space_id: Option<&str>) -> PathBuf {
    dpub_graph_workspace_dir(space_id).join("doctor_report.json")
}

fn dpub_simulations_dir(space_id: Option<&str>) -> PathBuf {
    dpub_graph_workspace_dir(space_id).join("simulations")
}

fn dpub_editions_dir(space_id: Option<&str>) -> PathBuf {
    dpub_graph_workspace_dir(space_id).join("editions")
}

fn dpub_run_log_dir(space_id: Option<&str>) -> PathBuf {
    if let Some(sid) = space_id {
        return dpub_managed_workspace_root(sid)
            .join(".cortex")
            .join("logs")
            .join("contribution_graph")
            .join("runs");
    }
    std::env::var("NOSTRA_DPUB_RUN_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_logs_dir().join("contribution_graph").join("runs"))
}

fn dpub_steward_packet_dir(space_id: Option<&str>) -> PathBuf {
    if let Some(sid) = space_id {
        return dpub_managed_workspace_root(sid)
            .join(".cortex")
            .join("logs")
            .join("contribution_graph")
            .join("steward_packets");
    }
    std::env::var("NOSTRA_DPUB_STEWARD_PACKET_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace_logs_dir()
                .join("contribution_graph")
                .join("steward_packets")
        })
}

fn dpub_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn dpub_mode_is_mutating(mode: &str) -> bool {
    matches!(
        mode.trim().to_ascii_lowercase().as_str(),
        "full" | "ingest" | "doctor" | "simulate" | "publish"
    )
}

fn dpub_require_approval(
    approval: &Option<DpubApprovalEnvelope>,
) -> Result<DpubApprovalEnvelope, axum::response::Response> {
    let Some(envelope) = approval else {
        return Err(dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_APPROVAL_REQUIRED",
            "Mutating pipeline modes require approval metadata.",
            None,
        ));
    };

    if envelope.approved_by.trim().is_empty()
        || envelope.rationale.trim().is_empty()
        || envelope.approved_at.trim().is_empty()
        || envelope.decision_ref.trim().is_empty()
    {
        return Err(dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_APPROVAL_INVALID",
            "approvedBy, rationale, approvedAt, and decisionRef are required.",
            None,
        ));
    }

    if !parse_metric_date(&envelope.approved_at) {
        return Err(dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_APPROVAL_DATE_INVALID",
            "approval.approvedAt must be RFC3339.",
            None,
        ));
    }

    Ok(envelope.clone())
}

fn dpub_read_json<T: DeserializeOwned>(path: &FsPath) -> Result<T, axum::response::Response> {
    let raw = fs::read_to_string(path).map_err(|err| {
        dpub_error(
            StatusCode::NOT_FOUND,
            "DPUB_ARTIFACT_NOT_FOUND",
            "DPub artifact not found.",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })?;

    serde_json::from_str::<T>(&raw).map_err(|err| {
        dpub_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "DPUB_ARTIFACT_INVALID",
            "DPub artifact could not be parsed.",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })
}

fn dpub_load_graph(space_id: Option<&str>) -> Result<ContributionGraphV1, axum::response::Response> {
    dpub_read_json(&dpub_graph_path(space_id))
}

fn dpub_load_path_bundle(
    space_id: Option<&str>,
) -> Result<PathAssessmentBundleV1, axum::response::Response> {
    dpub_read_json(&dpub_path_assessment_path(space_id))
}

fn dpub_load_doctor(space_id: Option<&str>) -> Result<DoctorReport, axum::response::Response> {
    dpub_read_json(&dpub_doctor_path(space_id))
}

fn normalize_siq_projection(projection: &mut SiqGraphProjection) {
    if let Some(entities) = projection.entities.as_object_mut() {
        for bucket in [
            "contributions",
            "rules",
            "gate_runs",
            "violations",
            "evidence",
            "waivers",
        ] {
            if let Some(rows) = entities.get_mut(bucket).and_then(Value::as_array_mut) {
                rows.sort_by(|left, right| {
                    let left_id = left
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    let right_id = right
                        .get("id")
                        .and_then(Value::as_str)
                        .unwrap_or_default()
                        .to_string();
                    left_id.cmp(&right_id)
                });
            }
        }
    }

    projection.edges.sort_by(|left, right| {
        let left_id = left
            .get("edge_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        let right_id = right
            .get("edge_id")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        left_id.cmp(&right_id)
    });
}

fn siq_load_projection_optional(space_id: Option<&str>) -> Option<SiqGraphProjection> {
    let raw = fs::read_to_string(siq_graph_projection_path(space_id)).ok()?;
    let mut projection = serde_json::from_str::<SiqGraphProjection>(&raw).ok()?;
    normalize_siq_projection(&mut projection);
    Some(projection)
}

fn siq_load_gate_summary_optional(space_id: Option<&str>) -> Option<SiqGateSummary> {
    let raw = fs::read_to_string(siq_gate_summary_path(space_id)).ok()?;
    serde_json::from_str::<SiqGateSummary>(&raw).ok()
}

fn dpub_scenario_path_from_template(root: &FsPath, template_id: Option<&str>) -> PathBuf {
    let key = template_id
        .unwrap_or("accelerate-118")
        .trim()
        .to_ascii_lowercase();
    let file = match key.as_str() {
        "accelerate-118" | "accelerate_118" => "accelerate_118.yaml".to_string(),
        "delay-013-workflow-bridge" | "delay_013_bridge" => "delay_013_bridge.yaml".to_string(),
        "governance-first" | "derisk_governance_first" => {
            "derisk_governance_first.yaml".to_string()
        }
        other => format!("{}.yaml", other.replace('-', "_")),
    };
    root.join("research")
        .join("000-contribution-graph")
        .join("scenarios")
        .join(file)
}

fn dpub_graph_hash_at(path: &FsPath) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let value: Value = serde_json::from_str(&raw).ok()?;
    value
        .get("graph_root_hash")
        .and_then(|v| v.as_str())
        .map(str::to_string)
}

fn dpub_run_id(mode: &str) -> String {
    let safe_mode = sanitize_fs_component(mode);
    format!("dpub_{}_{}", safe_mode, Utc::now().format("%Y%m%dT%H%M%SZ"))
}

fn dpub_phase_result(
    phase: &str,
    status: &str,
    started: Instant,
    message: Option<String>,
) -> DpubPhaseResult {
    DpubPhaseResult {
        phase: phase.to_string(),
        status: status.to_string(),
        message,
        duration_ms: Some(started.elapsed().as_millis() as u64),
    }
}

fn dpub_read_run_records(space_id: Option<&str>) -> Vec<DpubRunRecord> {
    let mut out = Vec::new();
    let dir = dpub_run_log_dir(space_id);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let record = match serde_json::from_str::<DpubRunRecord>(&raw) {
            Ok(record) => record,
            Err(_) => continue,
        };
        out.push(record);
    }

    out.sort_by(|a, b| b.started_at.cmp(&a.started_at));
    out
}

fn dpub_run_history_item(record: &DpubRunRecord) -> DpubRunHistoryItem {
    DpubRunHistoryItem {
        run_id: record.run_id.clone(),
        mode: record.mode.clone(),
        actor_role: record.actor_role.clone(),
        status: record.status.clone(),
        started_at: record.started_at.clone(),
        duration_ms: record.duration_ms,
        graph_root_hash_after: record.graph_root_hash_after.clone(),
    }
}

fn dpub_to_report(record: &DpubRunRecord) -> DpubPipelineRunReport {
    DpubPipelineRunReport {
        run_id: record.run_id.clone(),
        mode: record.mode.clone(),
        status: record.status.clone(),
        started_at: record.started_at.clone(),
        finished_at: record.finished_at.clone(),
        graph_root_hash_before: record.graph_root_hash_before.clone(),
        graph_root_hash_after: record.graph_root_hash_after.clone(),
        phase_results: record.phase_results.clone(),
        artifacts: record.artifacts.clone(),
        error: record.error.clone(),
    }
}

fn dpub_persist_run_record(record: &DpubRunRecord, space_id: Option<&str>) -> Result<(), String> {
    let path = dpub_run_log_dir(space_id).join(format!("{}.json", record.run_id));
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let value = serde_json::to_value(record).map_err(|err| err.to_string())?;
    persist_json(&path, &value)
}

fn dpub_research_index_path(root: &FsPath) -> PathBuf {
    root.join("research").join("RESEARCH_INITIATIVES_STATUS.md")
}

fn dpub_copy_recursive(src: &FsPath, dst: &FsPath) -> Result<(), String> {
    if !src.exists() {
        return Ok(());
    }
    if src.is_file() {
        if let Some(parent) = dst.parent() {
            fs::create_dir_all(parent).map_err(|err| err.to_string())?;
        }
        fs::copy(src, dst).map_err(|err| err.to_string())?;
        return Ok(());
    }

    fs::create_dir_all(dst).map_err(|err| err.to_string())?;
    for entry in fs::read_dir(src).map_err(|err| err.to_string())? {
        let entry = entry.map_err(|err| err.to_string())?;
        let child_src = entry.path();
        let child_dst = dst.join(entry.file_name());
        if child_src.is_dir() {
            dpub_copy_recursive(&child_src, &child_dst)?;
        } else {
            if let Some(parent) = child_dst.parent() {
                fs::create_dir_all(parent).map_err(|err| err.to_string())?;
            }
            fs::copy(&child_src, &child_dst).map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

fn dpub_sync_graph_outputs_to_space(source_root: &FsPath, space_id: &str) -> Result<(), String> {
    let src = source_root.join("research").join("000-contribution-graph");
    if !src.exists() {
        return Err(format!("source graph workspace missing: {}", src.display()));
    }
    let dst = dpub_graph_workspace_dir(Some(space_id));
    dpub_copy_recursive(&src, &dst)?;
    Ok(())
}

fn dpub_edition_entries(space_id: Option<&str>) -> Vec<DpubEditionEntry> {
    let mut out = Vec::new();
    let dir = dpub_editions_dir(space_id);
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(_) => return out,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_dir() {
            continue;
        }
        let version = path
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::to_string)
            .unwrap_or_else(|| "unknown".to_string());
        let manifest_path = path.join("edition_manifest.json");
        if manifest_path.exists() {
            if let Ok(raw) = fs::read_to_string(&manifest_path) {
                if let Ok(value) = serde_json::from_str::<Value>(&raw) {
                    out.push(DpubEditionEntry {
                        version,
                        generated_at: value
                            .get("generated_at")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                        graph_root_hash: value
                            .get("graph_root_hash")
                            .and_then(|v| v.as_str())
                            .map(str::to_string),
                    });
                    continue;
                }
            }
        }
        out.push(DpubEditionEntry {
            version,
            generated_at: None,
            graph_root_hash: None,
        });
    }

    out.sort_by(|a, b| b.version.cmp(&a.version));
    out
}

fn dpub_lens_counts(graph: &ContributionGraphV1) -> BTreeMap<String, usize> {
    let critical_nodes = graph
        .integrity_report
        .violations
        .iter()
        .filter(|violation| format!("{:?}", violation.severity).to_ascii_lowercase() == "critical")
        .flat_map(|violation| violation.affected_nodes.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let violation_nodes = graph
        .integrity_report
        .violations
        .iter()
        .filter(|violation| format!("{:?}", violation.severity).to_ascii_lowercase() == "violation")
        .flat_map(|violation| violation.affected_nodes.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let cycle_members = graph
        .integrity_report
        .cycles
        .iter()
        .flatten()
        .cloned()
        .collect::<BTreeSet<_>>()
        .len();
    let orphan_active = graph
        .integrity_report
        .violations
        .iter()
        .filter(|violation| violation.rule_id.to_ascii_lowercase().contains("orphan"))
        .flat_map(|violation| violation.affected_nodes.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let depends_on = graph
        .edges
        .iter()
        .filter(|edge| edge.edge_kind.eq_ignore_ascii_case("depends_on"))
        .count();
    let blocks = graph
        .edges
        .iter()
        .filter(|edge| edge.edge_kind.eq_ignore_ascii_case("blocks"))
        .count();
    let invalidates_supersedes = graph
        .edges
        .iter()
        .filter(|edge| {
            matches!(
                edge.edge_kind.as_str(),
                "invalidated_by" | "invalidates" | "supersedes" | "superseded_by"
            )
        })
        .count();
    let cross_layer_jumps = graph
        .edges
        .iter()
        .filter(|edge| {
            let from = graph
                .nodes
                .iter()
                .find(|node| node.id == edge.from)
                .map(|node| node.layer.to_ascii_lowercase());
            let to = graph
                .nodes
                .iter()
                .find(|node| node.id == edge.to)
                .map(|node| node.layer.to_ascii_lowercase());
            from.is_some() && to.is_some() && from != to
        })
        .count();
    let constitutional_basis = graph
        .edges
        .iter()
        .filter(|edge| edge.edge_kind.eq_ignore_ascii_case("constitutional_basis"))
        .count();
    let recommended_nodes = graph.path_assessment.recommended_path.node_ids.len();
    let blocking_nodes = graph.path_assessment.recommended_path.blocking_nodes.len();
    let path_rule_violations = graph.path_assessment.recommended_path.rule_violations.len();
    let candidate_compare = graph.path_assessment.candidate_paths.len().min(3);
    let risk_contributors = [
        graph.path_assessment.score_breakdown.critical,
        graph.path_assessment.score_breakdown.violation,
        graph.path_assessment.score_breakdown.warning,
        graph.path_assessment.score_breakdown.superseded_edges,
        graph.path_assessment.score_breakdown.cross_layer_jumps,
        graph.path_assessment.score_breakdown.unresolved_ref_penalty,
    ]
    .iter()
    .filter(|count| **count > 0)
    .count();
    let governance_sensitive = graph
        .nodes
        .iter()
        .filter(|node| {
            node.tags
                .iter()
                .any(|tag| tag.to_ascii_lowercase().contains("governance"))
                || node
                    .stewardship
                    .domain
                    .to_ascii_lowercase()
                    .contains("governance")
        })
        .count();
    let superseded_on_path = graph
        .path_assessment
        .candidate_paths
        .iter()
        .flat_map(|path| path.node_ids.iter().cloned())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .filter(|node_id| {
            graph
                .nodes
                .iter()
                .any(|node| node.id == *node_id && node.status.eq_ignore_ascii_case("superseded"))
        })
        .count();
    let missing_constitutional_basis = graph
        .integrity_report
        .violations
        .iter()
        .filter(|violation| {
            violation
                .rule_id
                .to_ascii_lowercase()
                .contains("constitutional")
        })
        .flat_map(|violation| violation.affected_nodes.iter().cloned())
        .collect::<BTreeSet<_>>()
        .len();
    let stewardship_domain = graph.nodes.len();
    let explicit_only = graph.edges.iter().filter(|edge| edge.is_explicit).count();
    let inferred_only = graph.edges.iter().filter(|edge| !edge.is_explicit).count();
    let low_confidence = graph
        .edges
        .iter()
        .filter(|edge| edge.confidence < 0.70)
        .count();
    let confidence_buckets = graph.edges.len();
    let missing_lineage = graph
        .edges
        .iter()
        .filter(|edge| edge.evidence_lines.is_empty() || edge.evidence_ref.trim().is_empty())
        .count();

    BTreeMap::from([
        ("integrity.critical_nodes".to_string(), critical_nodes),
        ("integrity.violation_nodes".to_string(), violation_nodes),
        ("integrity.cycle_members".to_string(), cycle_members),
        (
            "integrity.unresolved_refs".to_string(),
            graph.build_report.unresolved_references.len(),
        ),
        ("integrity.orphan_active".to_string(), orphan_active),
        ("topology.depends_on".to_string(), depends_on),
        ("topology.blocks".to_string(), blocks),
        (
            "topology.invalidates_supersedes".to_string(),
            invalidates_supersedes,
        ),
        ("topology.cross_layer_jumps".to_string(), cross_layer_jumps),
        (
            "topology.constitutional_basis".to_string(),
            constitutional_basis,
        ),
        ("path.recommended".to_string(), recommended_nodes),
        ("path.blocking_nodes".to_string(), blocking_nodes),
        ("path.rule_violations".to_string(), path_rule_violations),
        ("path.candidate_compare".to_string(), candidate_compare),
        ("path.risk_contributors".to_string(), risk_contributors),
        ("gov.governance_sensitive".to_string(), governance_sensitive),
        ("gov.superseded_on_path".to_string(), superseded_on_path),
        (
            "gov.missing_constitutional_basis".to_string(),
            missing_constitutional_basis,
        ),
        ("gov.stewardship_domain".to_string(), stewardship_domain),
        ("evidence.explicit_only".to_string(), explicit_only),
        ("evidence.inferred_only".to_string(), inferred_only),
        ("evidence.low_confidence".to_string(), low_confidence),
        (
            "evidence.confidence_buckets".to_string(),
            confidence_buckets,
        ),
        ("evidence.missing_lineage".to_string(), missing_lineage),
        ("temporal.edition_added_removed".to_string(), 0),
        ("temporal.risk_trend".to_string(), 0),
        ("temporal.goal_drift".to_string(), 0),
        ("temporal.run_recency".to_string(), 0),
    ])
}

fn dpub_lens_summary(graph: &ContributionGraphV1) -> DpubLensSummaryResponse {
    let counts = dpub_lens_counts(graph);
    let categories = vec![
        DpubLensSummaryCategory {
            id: "integrity".to_string(),
            label: "Integrity".to_string(),
            total: 5,
            active: 3,
        },
        DpubLensSummaryCategory {
            id: "topology".to_string(),
            label: "Dependency Topology".to_string(),
            total: 5,
            active: 0,
        },
        DpubLensSummaryCategory {
            id: "path".to_string(),
            label: "Path / Risk".to_string(),
            total: 5,
            active: 2,
        },
        DpubLensSummaryCategory {
            id: "gov".to_string(),
            label: "Governance / Constitution".to_string(),
            total: 4,
            active: 0,
        },
        DpubLensSummaryCategory {
            id: "evidence".to_string(),
            label: "Evidence Confidence".to_string(),
            total: 5,
            active: 1,
        },
        DpubLensSummaryCategory {
            id: "temporal".to_string(),
            label: "Temporal / Edition Drift".to_string(),
            total: 4,
            active: 0,
        },
    ];
    let definitions = vec![
        (
            "integrity.critical_nodes",
            "integrity",
            "Critical nodes",
            true,
        ),
        (
            "integrity.violation_nodes",
            "integrity",
            "Violation nodes",
            true,
        ),
        (
            "integrity.cycle_members",
            "integrity",
            "Cycle members",
            false,
        ),
        (
            "integrity.unresolved_refs",
            "integrity",
            "Unresolved refs",
            true,
        ),
        (
            "integrity.orphan_active",
            "integrity",
            "Orphan active",
            false,
        ),
        ("topology.depends_on", "topology", "Depends on", false),
        ("topology.blocks", "topology", "Blocks", false),
        (
            "topology.invalidates_supersedes",
            "topology",
            "Invalidates/supersedes",
            false,
        ),
        (
            "topology.cross_layer_jumps",
            "topology",
            "Cross-layer jumps",
            false,
        ),
        (
            "topology.constitutional_basis",
            "topology",
            "Constitutional basis",
            false,
        ),
        ("path.recommended", "path", "Recommended path", true),
        ("path.blocking_nodes", "path", "Blocking nodes", true),
        (
            "path.rule_violations",
            "path",
            "Path rule violations",
            false,
        ),
        ("path.candidate_compare", "path", "Candidate compare", false),
        ("path.risk_contributors", "path", "Risk contributors", false),
        (
            "gov.governance_sensitive",
            "gov",
            "Governance sensitive",
            false,
        ),
        ("gov.superseded_on_path", "gov", "Superseded on path", false),
        (
            "gov.missing_constitutional_basis",
            "gov",
            "Missing constitutional basis",
            false,
        ),
        ("gov.stewardship_domain", "gov", "Stewardship domain", false),
        ("evidence.explicit_only", "evidence", "Explicit only", false),
        ("evidence.inferred_only", "evidence", "Inferred only", false),
        (
            "evidence.low_confidence",
            "evidence",
            "Low confidence",
            true,
        ),
        (
            "evidence.confidence_buckets",
            "evidence",
            "Confidence buckets",
            false,
        ),
        (
            "evidence.missing_lineage",
            "evidence",
            "Missing lineage",
            false,
        ),
        (
            "temporal.edition_added_removed",
            "temporal",
            "Edition added/removed",
            false,
        ),
        ("temporal.risk_trend", "temporal", "Risk trend", false),
        ("temporal.goal_drift", "temporal", "Goal drift", false),
        ("temporal.run_recency", "temporal", "Run recency", false),
    ];
    let lenses = definitions
        .into_iter()
        .map(|(id, category, label, default_on)| DpubLensSummaryLens {
            id: id.to_string(),
            category: category.to_string(),
            label: label.to_string(),
            count: counts.get(id).copied().unwrap_or_default(),
            default_on,
        })
        .collect::<Vec<_>>();

    DpubLensSummaryResponse {
        graph_root_hash: graph.graph_root_hash.clone(),
        categories,
        lenses,
    }
}

fn dpub_edition_trends(
    space_id: Option<&str>,
    goal: &str,
    window: usize,
) -> Result<DpubEditionTrendResponse, String> {
    let mut entries = dpub_edition_entries(space_id);
    entries.sort_by(|a, b| a.version.cmp(&b.version));
    if window > 0 && entries.len() > window {
        entries = entries[entries.len() - window..].to_vec();
    }

    let mut points = Vec::<DpubEditionTrendPoint>::new();
    let mut recommendation_changes = Vec::<String>::new();
    let mut previous_recommended = None::<String>;
    for entry in entries {
        let snapshot_path = dpub_editions_dir(None)
            .join(&entry.version)
            .join("snapshot.json");
        let graph = match dpub_read_json::<ContributionGraphV1>(&snapshot_path) {
            Ok(graph) => graph,
            Err(_) => continue,
        };
        let recommended = if graph.path_assessment.goal == goal {
            Some(graph.path_assessment.recommended_path.name.clone())
        } else {
            Some(graph.path_assessment.recommended_path.name.clone())
        };
        if let Some(current) = recommended.clone() {
            if let Some(previous) = previous_recommended.clone() {
                if current != previous {
                    recommendation_changes
                        .push(format!("{}: {} -> {}", entry.version, previous, current));
                }
            }
            previous_recommended = Some(current);
        }
        points.push(DpubEditionTrendPoint {
            version: entry.version,
            risk_score: graph.path_assessment.risk_score,
            critical: graph.integrity_report.counts.critical,
            violation: graph.integrity_report.counts.violation,
            warning: graph.integrity_report.counts.warning,
            recommended_path: recommended,
        });
    }

    Ok(DpubEditionTrendResponse {
        goal: goal.to_string(),
        points,
        recommendation_changes,
    })
}

fn dpub_markdown_packet(
    goal: &str,
    from_version: &str,
    to_version: &str,
    diff: &EditionDiffReport,
    path_bundle: &PathAssessmentBundleV1,
    graph: &ContributionGraphV1,
) -> String {
    let assessment = path_bundle
        .assessments
        .iter()
        .find(|item| item.goal == goal)
        .or_else(|| path_bundle.assessments.first());
    let (recommended, risk, blockers) = if let Some(item) = assessment {
        (
            item.recommended_path.name.clone(),
            item.recommended_path.risk_score,
            if item.recommended_path.blocking_nodes.is_empty() {
                "none".to_string()
            } else {
                item.recommended_path.blocking_nodes.join(", ")
            },
        )
    } else {
        ("unavailable".to_string(), 0, "none".to_string())
    };

    format!(
        "# Steward Packet: DPub Contribution Graph\n\
Generated: {generated}\n\
\n\
## Graph Snapshot\n\
- Hash: `{hash}`\n\
- Nodes: `{nodes}`\n\
- Edges: `{edges}`\n\
- Integrity: critical `{critical}`, violation `{violation}`, warning `{warning}`\n\
\n\
## Path Recommendation ({goal})\n\
- Recommended: `{recommended}`\n\
- Risk Score: `{risk}`\n\
- Blocking Nodes: `{blockers}`\n\
\n\
## Edition Diff\n\
- From: `{from_version}` ({from_hash})\n\
- To: `{to_version}` ({to_hash})\n\
- Structural diff summary emitted in JSON artifact.\n",
        generated = now_iso(),
        hash = graph.graph_root_hash,
        nodes = graph.nodes.len(),
        edges = graph.edges.len(),
        critical = graph.integrity_report.counts.critical,
        violation = graph.integrity_report.counts.violation,
        warning = graph.integrity_report.counts.warning,
        goal = goal,
        recommended = recommended,
        risk = risk,
        blockers = blockers,
        from_version = from_version,
        to_version = to_version,
        from_hash = diff.from_hash,
        to_hash = diff.to_hash
    )
}

fn motoko_graph_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn count_json_files(dir: &FsPath) -> usize {
    let mut count = 0usize;
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("json") {
                count += 1;
            }
        }
    }
    count
}

fn decision_event_id_from_payload(payload: &DecisionCaptureRequest) -> String {
    let canonical = json!({
        "schema_version": payload.schema_version,
        "contribution": payload.contribution,
        "decision_date": payload.decision_date,
        "selected_option": payload.selected_option,
        "rationale": payload.rationale,
        "posture_before": payload.posture_before,
        "posture_after": payload.posture_after,
        "authority_mode": payload.authority_mode,
        "evidence_refs": payload.evidence_refs,
        "steward": payload.steward,
        "owner": payload.owner,
        "follow_up_actions": payload.follow_up_actions,
        "source": payload.source
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    let date_key = payload.decision_date.replace('-', "");
    format!("kg_decision_{}_{}", date_key, &digest[..12])
}

fn validate_decision_capture_request(
    payload: &DecisionCaptureRequest,
) -> Result<(), axum::response::Response> {
    let selected_allowed = [
        "Hold Deferred",
        "Conditional Progression",
        "Request Additional Evidence",
    ]
    .iter()
    .any(|allowed| *allowed == payload.selected_option);

    let missing_scalar = [
        payload.schema_version.trim().is_empty(),
        payload.contribution.trim().is_empty(),
        payload.decision_date.trim().is_empty(),
        payload.rationale.trim().is_empty(),
        payload.posture_before.trim().is_empty(),
        payload.posture_after.trim().is_empty(),
        payload.authority_mode.trim().is_empty(),
        payload.steward.trim().is_empty(),
        payload.owner.trim().is_empty(),
        payload.source.trim().is_empty(),
    ]
    .iter()
    .any(|is_empty| *is_empty);

    let invalid_array = payload.evidence_refs.is_empty()
        || payload.follow_up_actions.is_empty()
        || payload
            .evidence_refs
            .iter()
            .any(|item| item.trim().is_empty())
        || payload
            .follow_up_actions
            .iter()
            .any(|item| item.trim().is_empty());

    if payload.schema_version != MOTOKO_GRAPH_SCHEMA_VERSION
        || payload.contribution != "078"
        || !selected_allowed
        || missing_scalar
        || invalid_array
    {
        return Err(motoko_graph_error(
            StatusCode::BAD_REQUEST,
            "INVALID_DECISION_CAPTURE",
            "Decision capture payload failed validation",
            Some(json!({
                "schema_version": payload.schema_version,
                "contribution": payload.contribution,
                "selected_option": payload.selected_option
            })),
        ));
    }

    Ok(())
}

fn read_decision_events_from_dir(
    dir: &FsPath,
    default_status: Option<&str>,
) -> Vec<MotokoGraphDecisionEvent> {
    let mut events = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return events,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let value: Value = match serde_json::from_str(&raw) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if value.get("decision_event_id").is_none() {
            continue;
        }
        let mut event: MotokoGraphDecisionEvent = match serde_json::from_value(value) {
            Ok(event) => event,
            Err(_) => continue,
        };
        if event.status.is_none() {
            event.status = default_status.map(str::to_string);
        }
        events.push(event);
    }

    events.sort_by(|a, b| {
        b.captured_at
            .cmp(&a.captured_at)
            .then_with(|| b.decision_event_id.cmp(&a.decision_event_id))
    });

    events
}

fn read_monitoring_runs_from_dir(dir: &FsPath) -> Vec<MotokoGraphMonitoringRun> {
    let mut runs = Vec::new();
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return runs,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let run: MotokoGraphMonitoringRun = match serde_json::from_str(&raw) {
            Ok(run) => run,
            Err(_) => continue,
        };
        runs.push(run);
    }

    runs.sort_by(|a, b| {
        b.finished_at
            .cmp(&a.finished_at)
            .then_with(|| b.run_id.cmp(&a.run_id))
    });
    runs
}

fn testing_log_dir() -> PathBuf {
    std::env::var("NOSTRA_TESTING_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_logs_dir().join("testing"))
}

fn testing_runs_dir() -> PathBuf {
    testing_log_dir().join("runs")
}

fn testing_catalog_path() -> PathBuf {
    testing_log_dir().join("test_catalog_latest.json")
}

fn testing_gate_summary_path() -> PathBuf {
    testing_log_dir().join("test_gate_summary_latest.json")
}

fn siq_log_dir() -> PathBuf {
    std::env::var("NOSTRA_SIQ_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_logs_dir().join("siq"))
}

fn siq_runs_dir() -> PathBuf {
    siq_log_dir().join("runs")
}

fn siq_coverage_path() -> PathBuf {
    siq_log_dir().join("siq_coverage_latest.json")
}

fn siq_dependency_closure_path() -> PathBuf {
    siq_log_dir().join("siq_dependency_closure_latest.json")
}

fn siq_gate_summary_path(space_id: Option<&str>) -> PathBuf {
    let base = if let Some(sid) = space_id {
        workspace_root().join("_spaces").join(sid)
    } else {
        workspace_root()
    };
    base.join(".cortex")
        .join("logs")
        .join("siq")
        .join("siq_gate_summary_latest.json")
}

fn siq_graph_projection_path(space_id: Option<&str>) -> PathBuf {
    let base = if let Some(sid) = space_id {
        workspace_root().join("_spaces").join(sid)
    } else {
        workspace_root()
    };
    base.join(".cortex")
        .join("logs")
        .join("siq")
        .join("graph_projection_latest.json")
}

fn read_json_artifact<T: DeserializeOwned>(path: &FsPath) -> Result<T, axum::response::Response> {
    let raw = fs::read_to_string(path).map_err(|err| {
        testing_error(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "Testing artifact not found",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })?;

    serde_json::from_str::<T>(&raw).map_err(|err| {
        testing_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_ARTIFACT",
            "Testing artifact cannot be parsed",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })
}

fn read_siq_json_artifact<T: DeserializeOwned>(
    path: &FsPath,
) -> Result<T, axum::response::Response> {
    let raw = fs::read_to_string(path).map_err(|err| {
        siq_error(
            StatusCode::NOT_FOUND,
            "NOT_FOUND",
            "SIQ artifact not found",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })?;

    serde_json::from_str::<T>(&raw).map_err(|err| {
        siq_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "INVALID_ARTIFACT",
            "SIQ artifact cannot be parsed",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        )
    })
}

fn testing_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn siq_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn file_last_modified_secs(path: &FsPath) -> Option<u64> {
    let modified = fs::metadata(path).ok()?.modified().ok()?;
    modified
        .duration_since(UNIX_EPOCH)
        .ok()
        .map(|d| d.as_secs())
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn is_fresh(last_modified: Option<u64>) -> bool {
    last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= TESTING_STALE_AFTER_SECS)
        .unwrap_or(false)
}

fn should_emit_testing_surface() -> bool {
    std::env::var("NOSTRA_TESTING_SURFACE_ENABLE")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn decision_surface_log_dir() -> PathBuf {
    std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            workspace_logs_dir()
                .join("system")
                .join("decision_surfaces")
        })
}

fn decision_canonical_only_enabled() -> bool {
    std::env::var("NOSTRA_DECISION_CANONICAL_ONLY")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn decision_projection_cache_dir() -> PathBuf {
    decision_surface_log_dir().join("cache")
}

fn decision_actions_dir() -> PathBuf {
    decision_surface_log_dir().join("actions")
}

fn sanitize_fs_component(raw: &str) -> String {
    raw.trim()
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-' | ':') {
                ch
            } else {
                '_'
            }
        })
        .collect::<String>()
}

fn decision_surface_error(
    status: StatusCode,
    code: &str,
    message: &str,
    details: Option<Value>,
) -> axum::response::Response {
    (
        status,
        Json(ErrorResponse {
            error: message.to_string(),
            error_code: code.to_string(),
            details,
        }),
    )
        .into_response()
}

fn persist_json(path: &FsPath, value: &Value) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let bytes = serde_json::to_vec_pretty(value).map_err(|err| err.to_string())?;
    fs::write(path, bytes).map_err(|err| err.to_string())
}

fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

fn decision_telemetry_path() -> PathBuf {
    decision_surface_log_dir()
        .join("metrics")
        .join("decision_gate_telemetry_latest.json")
}

fn decision_telemetry_lock() -> &'static Mutex<()> {
    static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
    &LOCK
}

fn percentile_p95(samples: &[u64]) -> Option<u64> {
    if samples.is_empty() {
        return None;
    }
    let mut sorted = samples.to_vec();
    sorted.sort_unstable();
    let idx = ((sorted.len().saturating_sub(1)) as f64 * 0.95).round() as usize;
    sorted.get(idx).copied()
}

fn load_decision_telemetry_state() -> DecisionTelemetryState {
    let path = decision_telemetry_path();
    let raw = fs::read_to_string(path).ok();
    raw.and_then(|body| serde_json::from_str::<DecisionTelemetryState>(&body).ok())
        .unwrap_or_default()
}

fn save_decision_telemetry_state(state: &DecisionTelemetryState) {
    let path = decision_telemetry_path();
    if let Ok(value) = serde_json::to_value(state) {
        let _ = persist_json(&path, &value);
    }
}

fn decision_telemetry_snapshot_for_scope(
    scope: &DecisionTelemetryScopeState,
    schema_version: &str,
    updated_at: &str,
) -> DecisionTelemetrySnapshot {
    DecisionTelemetrySnapshot {
        schema_version: schema_version.to_string(),
        updated_at: updated_at.to_string(),
        decision_gate_samples: scope.decision_gate_samples,
        latency_ms_p95: percentile_p95(&scope.latency_ms_samples),
        gate_status_counts: scope.gate_status_counts.clone(),
        source_of_truth_counts: scope.source_of_truth_counts.clone(),
        degraded_reason_counts: scope.degraded_reason_counts.clone(),
        fallback_usage_total: scope.fallback_usage_total,
        cache_usage_total: scope.cache_usage_total,
        space_id: None,
        scope_space_id: None,
        global_decision_gate_samples: None,
        global_fallback_usage_total: None,
        global_cache_usage_total: None,
    }
}

fn decision_telemetry_snapshot() -> DecisionTelemetrySnapshot {
    let state = load_decision_telemetry_state();
    decision_telemetry_snapshot_for_scope(&state.global, &state.schema_version, &state.updated_at)
}

fn decision_telemetry_snapshot_by_space(space_id: &str) -> DecisionTelemetrySnapshot {
    let state = load_decision_telemetry_state();
    let scope = state.by_space.get(space_id).cloned().unwrap_or_default();
    let mut snapshot =
        decision_telemetry_snapshot_for_scope(&scope, &state.schema_version, &state.updated_at);
    snapshot.space_id = Some(space_id.to_string());
    snapshot.scope_space_id = Some(space_id.to_string());
    snapshot.global_decision_gate_samples = Some(state.global.decision_gate_samples);
    snapshot.global_fallback_usage_total = Some(state.global.fallback_usage_total);
    snapshot.global_cache_usage_total = Some(state.global.cache_usage_total);
    snapshot
}

fn increment_counter(map: &mut BTreeMap<String, u64>, key: &str) {
    let normalized = key.trim();
    if normalized.is_empty() {
        return;
    }
    let entry = map.entry(normalized.to_string()).or_insert(0);
    *entry = entry.saturating_add(1);
}

fn update_telemetry_scope(
    scope: &mut DecisionTelemetryScopeState,
    status: &str,
    source_of_truth: Option<&str>,
    degraded_reason: Option<&str>,
    latency_ms: u64,
) {
    scope.decision_gate_samples = scope.decision_gate_samples.saturating_add(1);
    scope.latency_ms_samples.push(latency_ms);
    if scope.latency_ms_samples.len() > 2048 {
        let drain = scope.latency_ms_samples.len() - 2048;
        scope.latency_ms_samples.drain(0..drain);
    }
    increment_counter(&mut scope.gate_status_counts, status);

    if let Some(source) = source_of_truth {
        increment_counter(&mut scope.source_of_truth_counts, source);
        if source.eq_ignore_ascii_case("fallback") {
            scope.fallback_usage_total = scope.fallback_usage_total.saturating_add(1);
        } else if source.eq_ignore_ascii_case("cache") {
            scope.cache_usage_total = scope.cache_usage_total.saturating_add(1);
        }
    }

    if let Some(reason) = degraded_reason {
        for item in reason.split(';').map(str::trim).filter(|v| !v.is_empty()) {
            increment_counter(&mut scope.degraded_reason_counts, item);
        }
    }
}

fn canonicalize_source_of_truth(
    source_of_truth: Option<String>,
    degraded_reason: Option<&str>,
) -> Option<String> {
    match source_of_truth {
        Some(source) => {
            let normalized = source.trim().to_ascii_lowercase();
            match normalized.as_str() {
                "canister" | "cache" | "fallback" => Some(normalized),
                _ => Some(
                    if degraded_reason
                        .map(str::trim)
                        .filter(|entry| !entry.is_empty())
                        .is_some()
                    {
                        "fallback".to_string()
                    } else {
                        "cache".to_string()
                    },
                ),
            }
        }
        None => {
            if degraded_reason
                .map(str::trim)
                .filter(|entry| !entry.is_empty())
                .is_some()
            {
                Some("fallback".to_string())
            } else {
                None
            }
        }
    }
}

fn record_decision_gate_telemetry(
    space_id: &str,
    status: &str,
    source_of_truth: Option<&str>,
    degraded_reason: Option<&str>,
    latency_ms: u64,
) {
    let guard = decision_telemetry_lock().lock();
    if guard.is_err() {
        return;
    }

    let mut state = load_decision_telemetry_state();
    state.updated_at = now_iso();
    update_telemetry_scope(
        &mut state.global,
        status,
        source_of_truth,
        degraded_reason,
        latency_ms,
    );

    let normalized_space_id = space_id.trim();
    if !normalized_space_id.is_empty() {
        let space_scope = state
            .by_space
            .entry(normalized_space_id.to_string())
            .or_default();
        update_telemetry_scope(
            space_scope,
            status,
            source_of_truth,
            degraded_reason,
            latency_ms,
        );
    }

    save_decision_telemetry_state(&state);
}

fn runtime_dispatch_telemetry_path() -> PathBuf {
    decision_surface_log_dir()
        .join("metrics")
        .join("runtime_gateway_dispatch_telemetry_latest.json")
}

fn runtime_dispatch_telemetry_lock() -> &'static Mutex<()> {
    static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
    &LOCK
}

fn load_runtime_dispatch_telemetry_state() -> RuntimeDispatchTelemetryState {
    let path = runtime_dispatch_telemetry_path();
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<RuntimeDispatchTelemetryState>(&raw).ok())
        .unwrap_or_default()
}

fn save_runtime_dispatch_telemetry_state(state: &RuntimeDispatchTelemetryState) {
    let path = runtime_dispatch_telemetry_path();
    if let Ok(value) = serde_json::to_value(state) {
        let _ = persist_json(&path, &value);
    }
}

fn status_class(status: u16) -> &'static str {
    match status {
        100..=199 => "1xx",
        200..=299 => "2xx",
        300..=399 => "3xx",
        400..=499 => "4xx",
        500..=599 => "5xx",
        _ => "unknown",
    }
}

fn transaction_boundary_key(
    boundary: cortex_runtime::gateway::types::GatewayTransactionBoundary,
) -> &'static str {
    match boundary {
        cortex_runtime::gateway::types::GatewayTransactionBoundary::ReadOnly => "read_only",
        cortex_runtime::gateway::types::GatewayTransactionBoundary::SingleRequestMutation => {
            "single_request_mutation"
        }
        cortex_runtime::gateway::types::GatewayTransactionBoundary::MultiStepBestEffort => {
            "multi_step_best_effort"
        }
        cortex_runtime::gateway::types::GatewayTransactionBoundary::HostManaged => "host_managed",
        cortex_runtime::gateway::types::GatewayTransactionBoundary::StreamingSession => {
            "streaming_session"
        }
    }
}

fn classify_runtime_dispatch_error(err: &str) -> &'static str {
    if err.contains("legacy_upstream_timeout") {
        "upstream_timeout"
    } else if err.contains("legacy_upstream_network") {
        "upstream_network"
    } else if err.contains("legacy_upstream_invalid_json") {
        "upstream_invalid_body"
    } else if err.contains("route_not_found") {
        "route_not_found"
    } else {
        "runtime_internal"
    }
}

fn update_runtime_dispatch_route(
    route: &mut RuntimeDispatchTelemetryRoute,
    latency_ms: u64,
    status: Option<u16>,
    replay_hit: bool,
    transaction_boundary: Option<cortex_runtime::gateway::types::GatewayTransactionBoundary>,
    error_class: Option<&str>,
) {
    route.request_count = route.request_count.saturating_add(1);
    route.latency_ms_samples.push(latency_ms);
    if route.latency_ms_samples.len() > 2048 {
        let overflow = route.latency_ms_samples.len() - 2048;
        route.latency_ms_samples.drain(0..overflow);
    }
    if replay_hit {
        route.replay_hit_count = route.replay_hit_count.saturating_add(1);
    }
    if let Some(status) = status {
        increment_counter(&mut route.status_class_counts, status_class(status));
    }
    if let Some(error_class) = error_class {
        increment_counter(&mut route.error_class_counts, error_class);
    }
    if let Some(boundary) = transaction_boundary {
        increment_counter(
            &mut route.transaction_boundary_counts,
            transaction_boundary_key(boundary),
        );
    }
}

fn record_runtime_dispatch_telemetry_success(
    request_method: &str,
    request_path: &str,
    response: &cortex_runtime::gateway::types::GatewayResponseEnvelope,
    latency_ms: u64,
) {
    let _guard = match runtime_dispatch_telemetry_lock().lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let mut state = load_runtime_dispatch_telemetry_state();
    state.updated_at = now_iso();
    state.total_requests = state.total_requests.saturating_add(1);
    if response.idempotency.replayed {
        state.total_replay_hits = state.total_replay_hits.saturating_add(1);
    }
    let route_key = response
        .route_template
        .clone()
        .unwrap_or_else(|| format!("{} {}", request_method, request_path));
    let route = state.routes.entry(route_key).or_default();
    let error_class = response
        .dispatch_error
        .as_ref()
        .map(|entry| format!("{:?}", entry.class).to_ascii_lowercase())
        .or_else(|| response.error.as_ref().map(|entry| entry.code.clone()));
    update_runtime_dispatch_route(
        route,
        latency_ms,
        Some(response.status),
        response.idempotency.replayed,
        Some(response.transaction_boundary),
        error_class.as_deref(),
    );
    save_runtime_dispatch_telemetry_state(&state);
}

fn record_runtime_dispatch_telemetry_failure(
    request_method: &str,
    request_path: &str,
    error_class: &str,
    latency_ms: u64,
) {
    let _guard = match runtime_dispatch_telemetry_lock().lock() {
        Ok(guard) => guard,
        Err(_) => return,
    };

    let mut state = load_runtime_dispatch_telemetry_state();
    state.updated_at = now_iso();
    state.total_requests = state.total_requests.saturating_add(1);
    let route_key = format!("{} {}", request_method, request_path);
    let route = state.routes.entry(route_key).or_default();
    update_runtime_dispatch_route(route, latency_ms, None, false, None, Some(error_class));
    save_runtime_dispatch_telemetry_state(&state);
}

fn build_decision_envelope(
    surface_id: String,
    workflow_id: String,
    mutation_id: String,
    status: String,
    required_actions: Vec<String>,
    evidence_refs: Vec<String>,
    source_of_truth: Option<String>,
    lineage_id: Option<String>,
    policy_ref: Option<String>,
    policy_version: Option<u64>,
    degraded_reason: Option<String>,
    auth_status: Option<String>,
    auth_reason: Option<String>,
    payload: Option<Value>,
) -> DecisionSurfaceEnvelope {
    let normalized_source =
        canonicalize_source_of_truth(source_of_truth, degraded_reason.as_deref());
    DecisionSurfaceEnvelope {
        surface_id,
        workflow_id,
        mutation_id,
        status,
        required_actions,
        evidence_refs,
        last_updated_at: now_iso(),
        source_of_truth: normalized_source,
        lineage_id,
        policy_ref,
        policy_version,
        degraded_reason,
        auth_status,
        auth_reason,
        payload,
    }
}

fn validate_quality_payload(request: &DecisionActionRequest) -> Result<(), String> {
    let risk = request.risk_statement.as_deref().unwrap_or("").trim();
    let rollback = request.rollback_path.as_deref().unwrap_or("").trim();
    if risk.is_empty() {
        return Err("risk_statement is required for this gate class".to_string());
    }
    if rollback.is_empty() {
        return Err("rollback_path is required for this gate class".to_string());
    }
    if request.evidence_refs.is_empty()
        || request
            .evidence_refs
            .iter()
            .any(|entry| entry.trim().is_empty())
    {
        return Err("evidence_refs must contain at least one non-empty entry".to_string());
    }
    Ok(())
}

fn parse_mutation_id_from_gate_id(gate_id: &str) -> Option<String> {
    gate_id
        .rsplit_once(':')
        .map(|(_, id)| id.trim().to_string())
        .filter(|id| !id.is_empty())
}

fn normalize_decision_required_action(raw: &str) -> Option<&'static str> {
    let normalized = raw.trim().to_ascii_lowercase();
    if normalized == "decision_ack" || normalized.starts_with("decision_ack:") {
        Some("ack")
    } else if normalized == "decision_escalate" || normalized.starts_with("decision_escalate:") {
        Some("escalate")
    } else {
        None
    }
}

fn decision_actions_from_requirements(required_actions: &[String]) -> Vec<&'static str> {
    let mut normalized = BTreeSet::<&'static str>::new();
    for required in required_actions {
        if let Some(action) = normalize_decision_required_action(required.as_str()) {
            normalized.insert(action);
        }
    }
    normalized.into_iter().collect()
}

fn load_decision_action_records_by_mutation(
    mutation_id: &str,
) -> Result<Vec<DecisionActionRecord>, String> {
    let actions_dir = decision_actions_dir();
    if !actions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut records = Vec::<DecisionActionRecord>::new();
    let entries = fs::read_dir(&actions_dir).map_err(|err| err.to_string())?;
    for entry in entries {
        let entry = entry.map_err(|err| err.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let bytes = fs::read(&path).map_err(|err| err.to_string())?;
        let record = match serde_json::from_slice::<DecisionActionRecord>(&bytes) {
            Ok(value) => value,
            Err(_) => continue,
        };
        if record.mutation_id == mutation_id {
            records.push(record);
        }
    }
    records.sort_by(|left, right| left.created_at.cmp(&right.created_at));
    Ok(records)
}

fn missing_viewspec_required_actions(
    mutation_id: &str,
    action_target: &str,
    actor_principal: &str,
    actor_role: &str,
    required_actions: &[String],
) -> Result<Vec<String>, String> {
    let required = decision_actions_from_requirements(required_actions);
    if required.is_empty() {
        return Ok(Vec::new());
    }

    let actor_ref = format!("{}#{}", actor_principal, actor_role);
    let records = load_decision_action_records_by_mutation(mutation_id)?;
    let mut satisfied = BTreeSet::<String>::new();
    for record in records {
        if !record.action_target.eq_ignore_ascii_case(action_target) {
            continue;
        }
        if let Some(record_actor_ref) = record.actor_ref.as_ref() {
            if !record_actor_ref.eq_ignore_ascii_case(&actor_ref) {
                continue;
            }
        }
        let normalized = record.action.trim().to_ascii_lowercase();
        if normalized == "ack" && required.contains(&"ack") {
            satisfied.insert("decision_ack".to_string());
        } else if normalized == "escalate" && required.contains(&"escalate") {
            satisfied.insert("decision_escalate".to_string());
        }
    }

    let mut missing = Vec::<String>::new();
    if required.contains(&"ack") && !satisfied.contains("decision_ack") {
        missing.push("decision_ack".to_string());
    }
    if required.contains(&"escalate") && !satisfied.contains("decision_escalate") {
        missing.push("decision_escalate".to_string());
    }
    Ok(missing)
}

fn can_bridge_viewspec_governance_block(evaluation: &ActionScopeEvaluation) -> bool {
    evaluation
        .reason
        .to_ascii_lowercase()
        .contains("mandatory review")
        && !decision_actions_from_requirements(&evaluation.required_actions).is_empty()
}

fn normalize_role(raw: &str) -> Option<String> {
    let role = raw.trim().to_ascii_lowercase();
    if role.is_empty() { None } else { Some(role) }
}

fn resolve_requested_role(headers: &HeaderMap, actor_ref: Option<&str>) -> Option<String> {
    headers
        .get("x-cortex-role")
        .and_then(|value| value.to_str().ok())
        .and_then(normalize_role)
        .or_else(|| {
            headers
                .get("x-cortex-actor-role")
                .and_then(|value| value.to_str().ok())
                .and_then(normalize_role)
        })
        .or_else(|| actor_ref.and_then(normalize_role))
}

fn resolve_actor_principal(headers: &HeaderMap, actor_ref: Option<&str>) -> Option<String> {
    let header_principal = headers
        .get("x-ic-principal")
        .and_then(|value| value.to_str().ok())
        .map(str::trim)
        .filter(|value| !value.is_empty());

    if let Some(value) = header_principal {
        if Principal::from_text(value).is_ok() {
            return Some(value.to_string());
        }
    }

    actor_ref
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .and_then(|value| {
            if Principal::from_text(value).is_ok() {
                Some(value.to_string())
            } else {
                None
            }
        })
}

fn decision_signature_secret() -> Option<String> {
    std::env::var("NOSTRA_DECISION_SIGNING_SECRET")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DecisionSignedMode {
    Off,
    Warn,
    RequiredP0P1,
    RequiredAll,
}

fn decision_signed_mode() -> DecisionSignedMode {
    if let Ok(value) = std::env::var("NOSTRA_DECISION_SIGNED_MODE") {
        match value.trim().to_ascii_lowercase().as_str() {
            "off" => return DecisionSignedMode::Off,
            "warn" => return DecisionSignedMode::Warn,
            "required_p0_p1" => return DecisionSignedMode::RequiredP0P1,
            "required_all" => return DecisionSignedMode::RequiredAll,
            _ => {}
        }
    }
    if let Ok(value) = std::env::var("NOSTRA_DECISION_REQUIRE_SIGNED_PRINCIPAL") {
        if matches!(
            value.trim().to_ascii_lowercase().as_str(),
            "1" | "true" | "yes" | "on"
        ) {
            return DecisionSignedMode::RequiredAll;
        }
    }
    if decision_signature_secret().is_some() {
        DecisionSignedMode::Warn
    } else {
        DecisionSignedMode::Off
    }
}

fn signature_required_for_gate(
    mode: DecisionSignedMode,
    gate_level: &str,
    risky_gate: bool,
) -> bool {
    match mode {
        DecisionSignedMode::Off => false,
        DecisionSignedMode::Warn => false,
        DecisionSignedMode::RequiredAll => true,
        DecisionSignedMode::RequiredP0P1 => {
            let normalized = gate_level.trim().to_ascii_lowercase();
            risky_gate
                || normalized.contains("release")
                || normalized.contains("hard")
                || normalized.contains("p0")
                || normalized.contains("p1")
        }
    }
}

fn decision_signature_max_skew_secs() -> u64 {
    std::env::var("NOSTRA_DECISION_SIGNATURE_MAX_SKEW_SECS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(5 * 60)
}

fn principal_role_bindings() -> HashMap<String, String> {
    let raw = match std::env::var("NOSTRA_DECISION_PRINCIPAL_ROLE_BINDINGS") {
        Ok(value) => value,
        Err(_) => return HashMap::new(),
    };
    let parsed = serde_json::from_str::<Value>(&raw).ok();
    let mut bindings = HashMap::new();
    if let Some(Value::Object(map)) = parsed {
        for (principal, role) in map {
            if Principal::from_text(&principal).is_err() {
                continue;
            }
            if let Some(role_text) = role.as_str().and_then(normalize_role) {
                bindings.insert(principal, role_text);
            }
        }
    }
    bindings
}

fn env_role_fallback_allowed() -> bool {
    std::env::var("NOSTRA_DECISION_ALLOW_ENV_ROLE_FALLBACK")
        .ok()
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(false)
}

#[cfg(test)]
fn test_override_actor_role_binding() -> Option<Result<Option<String>, String>> {
    let raw = std::env::var("NOSTRA_TEST_DECISION_ROLE_BINDING").ok()?;
    let normalized = raw.trim();
    if normalized.is_empty() {
        return None;
    }
    if normalized.eq_ignore_ascii_case("__missing__") {
        return Some(Ok(None));
    }
    if let Some(rest) = normalized.strip_prefix("__error__:") {
        let reason = rest.trim();
        return Some(Err(if reason.is_empty() {
            "mock role binding failure".to_string()
        } else {
            reason.to_string()
        }));
    }
    Some(Ok(Some(normalized.to_ascii_lowercase())))
}

#[cfg(test)]
fn test_override_policy_evaluation() -> Option<Result<ActionScopeEvaluation, String>> {
    let raw = std::env::var("NOSTRA_TEST_DECISION_POLICY_EVAL").ok()?;
    let normalized = raw.trim();
    if normalized.is_empty() {
        return None;
    }
    if let Some(rest) = normalized.strip_prefix("__error__:") {
        let reason = rest.trim();
        return Some(Err(if reason.is_empty() {
            "mock governance evaluation failure".to_string()
        } else {
            reason.to_string()
        }));
    }

    let (allowed, requires_review, gate_decision) = match normalized.to_ascii_lowercase().as_str() {
        "allow" => (true, false, "allow".to_string()),
        "review" => (true, true, "review".to_string()),
        "block" => (false, false, "block".to_string()),
        _ => {
            return Some(Err(format!(
                "unsupported mock policy decision: {}",
                normalized
            )));
        }
    };
    Some(Ok(ActionScopeEvaluation {
        allowed,
        reason: format!("mock policy decision: {gate_decision}"),
        effective_weight: 1.0,
        requires_review,
        gate_decision,
        required_actions: if allowed {
            vec!["decision_ack:mock".to_string()]
        } else {
            vec!["decision_escalate:mock".to_string()]
        },
        policy_ref: Some("policy:mock".to_string()),
        policy_version: 1,
    }))
}

#[cfg(test)]
fn test_override_agent_l2_scope_evaluation() -> Option<Result<ActionScopeEvaluation, String>> {
    let raw = std::env::var("NOSTRA_TEST_AGENT_L2_SCOPE_EVAL").ok()?;
    let normalized = raw.trim();
    if normalized.is_empty() {
        return None;
    }
    if let Some(rest) = normalized.strip_prefix("__error__:") {
        let reason = rest.trim();
        return Some(Err(if reason.is_empty() {
            "mock l2 governance scope failure".to_string()
        } else {
            reason.to_string()
        }));
    }

    let (allowed, requires_review, gate_decision) = match normalized.to_ascii_lowercase().as_str() {
        "allow" => (true, false, "allow".to_string()),
        "review" => (true, true, "review".to_string()),
        "block" => (false, false, "block".to_string()),
        _ => {
            return Some(Err(format!(
                "unsupported mock l2 governance scope decision: {}",
                normalized
            )));
        }
    };
    Some(Ok(ActionScopeEvaluation {
        allowed,
        reason: format!("mock l2 governance scope decision: {gate_decision}"),
        effective_weight: 1.0,
        requires_review,
        gate_decision,
        required_actions: if allowed {
            vec!["decision_ack:l2_mock".to_string()]
        } else {
            vec!["decision_escalate:l2_mock".to_string()]
        },
        policy_ref: Some("policy:l2_mock".to_string()),
        policy_version: 1,
    }))
}

fn signature_material(
    principal: &str,
    role: &str,
    decision_gate_id: &str,
    mutation_id: &str,
    action_target: &str,
    signed_at: u64,
) -> String {
    format!(
        "{}|{}|{}|{}|{}|{}",
        principal, role, decision_gate_id, mutation_id, action_target, signed_at
    )
}

fn signature_hash(secret: &str, material: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret.as_bytes());
    hasher.update(b"|");
    hasher.update(material.as_bytes());
    hex::encode(hasher.finalize())
}

fn constant_time_eq(left: &str, right: &str) -> bool {
    if left.len() != right.len() {
        return false;
    }
    left.as_bytes()
        .iter()
        .zip(right.as_bytes().iter())
        .fold(0u8, |acc, (l, r)| acc | (l ^ r))
        == 0
}

fn resolve_actor_identity(
    headers: &HeaderMap,
    request: &DecisionActionRequest,
    decision_gate_id: &str,
    mutation_id: &str,
    action_target: &str,
    require_principal: bool,
    signature_required: bool,
) -> Result<VerifiedDecisionActor, axum::response::Response> {
    let requested_role = resolve_requested_role(headers, request.actor_ref.as_deref())
        .unwrap_or_else(|| "operator".to_string());
    if !role_is_authorized(&requested_role) {
        return Err(decision_surface_error(
            StatusCode::FORBIDDEN,
            "UNAUTHORIZED_DECISION_ROLE",
            "Decision action rejected: caller role is not authorized",
            Some(json!({ "role": requested_role })),
        ));
    }

    let principal = resolve_actor_principal(headers, request.actor_ref.as_deref());
    if require_principal && principal.is_none() {
        return Err(decision_surface_error(
            StatusCode::FORBIDDEN,
            "MISSING_ACTOR_PRINCIPAL",
            "Decision action rejected: principal is required for this gate class",
            Some(json!({ "decisionGateId": decision_gate_id, "mutationId": mutation_id })),
        ));
    }
    let principal = principal.unwrap_or_else(|| "2vxsx-fae".to_string());

    let resolved_role = if env_role_fallback_allowed() {
        let bindings = principal_role_bindings();
        if let Some(bound_role) = bindings.get(&principal) {
            if &requested_role != bound_role {
                return Err(decision_surface_error(
                    StatusCode::FORBIDDEN,
                    "ROLE_BINDING_MISMATCH",
                    "Decision action rejected: principal role binding mismatch",
                    Some(json!({
                        "principal": principal,
                        "requestedRole": requested_role,
                        "boundRole": bound_role
                    })),
                ));
            }
            bound_role.clone()
        } else {
            requested_role
        }
    } else {
        requested_role
    };

    let signed_at = headers
        .get("x-cortex-signed-at")
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.trim().parse::<u64>().ok());
    let signature = headers
        .get("x-cortex-signature")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty());

    let mut signature_validated = false;
    let mut auth_reason = None::<String>;
    if let Some(secret) = decision_signature_secret() {
        match (signed_at, signature) {
            (Some(signed_at), Some(signature)) => {
                let now = now_secs();
                let skew = now.abs_diff(signed_at);
                if skew > decision_signature_max_skew_secs() {
                    return Err(decision_surface_error(
                        StatusCode::FORBIDDEN,
                        "STALE_DECISION_SIGNATURE",
                        "Decision signature timestamp is outside accepted skew window",
                        Some(
                            json!({ "skewSecs": skew, "maxSkewSecs": decision_signature_max_skew_secs() }),
                        ),
                    ));
                }
                let material = signature_material(
                    &principal,
                    &resolved_role,
                    decision_gate_id,
                    mutation_id,
                    action_target,
                    signed_at,
                );
                let expected = signature_hash(&secret, &material);
                if !constant_time_eq(&expected, &signature.to_ascii_lowercase()) {
                    return Err(decision_surface_error(
                        StatusCode::FORBIDDEN,
                        "INVALID_DECISION_SIGNATURE",
                        "Decision action rejected: signature verification failed",
                        Some(json!({ "principal": principal, "role": resolved_role })),
                    ));
                }
                signature_validated = true;
            }
            _ if signature_required => {
                return Err(decision_surface_error(
                    StatusCode::FORBIDDEN,
                    "MISSING_DECISION_SIGNATURE",
                    "Decision action rejected: signed principal/role proof is required",
                    Some(json!({
                        "principal": principal,
                        "role": resolved_role,
                        "decisionGateId": decision_gate_id
                    })),
                ));
            }
            _ => {
                auth_reason = Some("signature_missing_warn_only".to_string());
            }
        }
    } else if signature_required {
        return Err(decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "SIGNATURE_SECRET_UNAVAILABLE",
            "Signed principal/role verification required but signing secret is missing",
            None,
        ));
    } else {
        auth_reason = Some("signature_secret_missing_mode_non_required".to_string());
    }

    Ok(VerifiedDecisionActor {
        principal,
        role: resolved_role,
        signed: signature_validated,
        auth_status: if signature_validated {
            "verified".to_string()
        } else {
            "warn".to_string()
        },
        auth_reason,
    })
}

fn role_is_authorized(role: &str) -> bool {
    matches!(role, "admin" | "steward" | "operator")
}

fn viewspec_proposal_status_matches(value: &str, status: &ViewSpecProposalStatus) -> bool {
    matches!(
        (value.trim().to_ascii_lowercase().as_str(), status),
        ("staged", ViewSpecProposalStatus::Staged)
            | ("under_review", ViewSpecProposalStatus::UnderReview)
            | ("approved", ViewSpecProposalStatus::Approved)
            | ("ratified", ViewSpecProposalStatus::Ratified)
            | ("rejected", ViewSpecProposalStatus::Rejected)
            | ("superseded", ViewSpecProposalStatus::Superseded)
            | ("merged", ViewSpecProposalStatus::Merged)
    )
}

fn viewspec_proposal_status_name(status: &ViewSpecProposalStatus) -> &'static str {
    match status {
        ViewSpecProposalStatus::Staged => "staged",
        ViewSpecProposalStatus::UnderReview => "under_review",
        ViewSpecProposalStatus::Approved => "approved",
        ViewSpecProposalStatus::Ratified => "ratified",
        ViewSpecProposalStatus::Rejected => "rejected",
        ViewSpecProposalStatus::Superseded => "superseded",
        ViewSpecProposalStatus::Merged => "merged",
    }
}

fn viewspec_digest_hex(value: &Value) -> String {
    cortex_runtime::viewspec::viewspec_digest_hex(value)
}

async fn resolve_viewspec_governance_gate(
    headers: &HeaderMap,
    proposal_id: &str,
    space_id: &str,
    action_target: &str,
    gate_level: &str,
    actor_ref: Option<&str>,
    block_on_degraded: bool,
) -> Result<ViewSpecGovernanceDecisionGate, axum::response::Response> {
    let decision_gate_id = format!(
        "viewspec_gate:{}:{}",
        sanitize_viewspec_candidate_set_token(action_target),
        sanitize_viewspec_candidate_set_token(proposal_id)
    );
    let replay_contract_ref = format!(
        "viewspec_replay_contract:{}",
        sanitize_viewspec_candidate_set_token(proposal_id)
    );
    let signed_mode = decision_signed_mode();
    let signature_required = signature_required_for_gate(signed_mode, gate_level, true);
    let actor_request = DecisionActionRequest {
        space_id: None,
        decision_gate_id: None,
        workflow_id: None,
        mutation_id: None,
        action_target: None,
        domain_mode: None,
        gate_level: None,
        actor_ref: actor_ref.map(|value| value.to_string()),
        risk_statement: None,
        rollback_path: None,
        evidence_refs: Vec::new(),
        note: None,
    };
    let actor = resolve_actor_identity(
        headers,
        &actor_request,
        &decision_gate_id,
        proposal_id,
        action_target,
        true,
        signature_required,
    )?;
    let actor_principal = Principal::from_text(&actor.principal).map_err(|_| {
        cortex_ux_error(
            StatusCode::FORBIDDEN,
            "INVALID_ACTOR_PRINCIPAL",
            "Governance decision rejected: actor principal is invalid.",
            Some(json!({ "principal": actor.principal })),
        )
    })?;

    let mut degraded_reason = None::<String>;
    let mut gate_status: Option<String>;
    let mut source_of_truth = "fallback".to_string();

    #[cfg(test)]
    let mock_role_binding = test_override_actor_role_binding();
    #[cfg(not(test))]
    let mock_role_binding: Option<Result<Option<String>, String>> = None;
    #[cfg(test)]
    let mock_policy_eval = test_override_policy_evaluation();
    #[cfg(not(test))]
    let mock_policy_eval: Option<Result<ActionScopeEvaluation, String>> = None;

    if mock_role_binding.is_some() || mock_policy_eval.is_some() {
        if let Some(binding_result) = mock_role_binding {
            match binding_result {
                Ok(Some(bound_role)) => {
                    if bound_role != actor.role {
                        return Err(cortex_ux_error(
                            StatusCode::FORBIDDEN,
                            "VIEWSPEC_GOVERNANCE_ROLE_BINDING_MISMATCH",
                            "Governance decision rejected: canonical role binding mismatch.",
                            Some(json!({
                                "spaceId": space_id,
                                "principal": actor.principal,
                                "requestedRole": actor.role,
                                "boundRole": bound_role
                            })),
                        ));
                    }
                }
                Ok(None) => {
                    degraded_reason = Some("missing_canister_actor_role_binding".to_string());
                    if block_on_degraded {
                        return Err(cortex_ux_error(
                            StatusCode::FORBIDDEN,
                            "VIEWSPEC_GOVERNANCE_ROLE_BINDING_MISSING",
                            "Canonical governance role binding is required for this action.",
                            Some(json!({
                                "spaceId": space_id,
                                "principal": actor.principal,
                                "actionTarget": action_target
                            })),
                        ));
                    }
                }
                Err(err) => {
                    degraded_reason = Some(format!("actor_role_binding_query_failed:{err}"));
                    if block_on_degraded {
                        return Err(cortex_ux_error(
                            StatusCode::SERVICE_UNAVAILABLE,
                            "VIEWSPEC_GOVERNANCE_CANONICAL_REQUIRED",
                            "Canonical governance authority is required for this action.",
                            Some(json!({
                                "spaceId": space_id,
                                "actionTarget": action_target,
                                "reason": degraded_reason
                            })),
                        ));
                    }
                }
            }
        }

        match mock_policy_eval {
            Some(Ok(evaluation)) => {
                gate_status = Some(evaluation.gate_decision.clone());
                if evaluation.gate_decision.eq_ignore_ascii_case("block") {
                    if can_bridge_viewspec_governance_block(&evaluation) {
                        match missing_viewspec_required_actions(
                            proposal_id,
                            action_target,
                            &actor.principal,
                            &actor.role,
                            &evaluation.required_actions,
                        ) {
                            Ok(missing) if missing.is_empty() => {
                                gate_status = Some("allow".to_string());
                            }
                            Ok(missing) => {
                                return Err(cortex_ux_error(
                                    StatusCode::FORBIDDEN,
                                    "VIEWSPEC_GOVERNANCE_REQUIRED_ACTIONS_MISSING",
                                    "Required decision actions must be recorded before ratify/merge can proceed.",
                                    Some(json!({
                                        "spaceId": space_id,
                                        "actionTarget": action_target,
                                        "reason": evaluation.reason,
                                        "requiredActions": evaluation.required_actions,
                                        "missingActions": missing,
                                        "mutationId": proposal_id,
                                        "policyRef": evaluation.policy_ref,
                                        "policyVersion": evaluation.policy_version
                                    })),
                                ));
                            }
                            Err(err) => {
                                return Err(cortex_ux_error(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "VIEWSPEC_GOVERNANCE_ACTION_LOAD_FAILED",
                                    "Failed to load decision action evidence for governance bridge.",
                                    Some(json!({
                                        "mutationId": proposal_id,
                                        "reason": err
                                    })),
                                ));
                            }
                        }
                    } else {
                        return Err(cortex_ux_error(
                            StatusCode::FORBIDDEN,
                            "VIEWSPEC_GOVERNANCE_DENIED",
                            "Governance policy denied this proposal action.",
                            Some(json!({
                                "spaceId": space_id,
                                "actionTarget": action_target,
                                "reason": evaluation.reason,
                                "requiredActions": evaluation.required_actions,
                                "policyRef": evaluation.policy_ref,
                                "policyVersion": evaluation.policy_version
                            })),
                        ));
                    }
                }
                source_of_truth = if degraded_reason.is_some() {
                    "fallback".to_string()
                } else {
                    "canister".to_string()
                };
            }
            Some(Err(err)) => {
                degraded_reason = Some(format!("governance_scope_evaluation_failed:{err}"));
                gate_status = Some("degraded".to_string());
                if block_on_degraded {
                    return Err(cortex_ux_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "VIEWSPEC_GOVERNANCE_CANONICAL_REQUIRED",
                        "Canonical governance authority is required for this action.",
                        Some(json!({
                            "spaceId": space_id,
                            "actionTarget": action_target,
                            "reason": degraded_reason
                        })),
                    ));
                }
            }
            None => {
                gate_status = Some(if degraded_reason.is_some() {
                    "degraded".to_string()
                } else {
                    "review".to_string()
                });
                source_of_truth = if degraded_reason.is_some() {
                    "fallback".to_string()
                } else {
                    "canister".to_string()
                };
            }
        }
    } else if let Ok(client) = GovernanceClient::from_env() {
        match client
            .get_actor_role_binding(space_id, &actor_principal)
            .await
        {
            Ok(Some(binding)) => {
                if binding.role != actor.role {
                    return Err(cortex_ux_error(
                        StatusCode::FORBIDDEN,
                        "VIEWSPEC_GOVERNANCE_ROLE_BINDING_MISMATCH",
                        "Governance decision rejected: canonical role binding mismatch.",
                        Some(json!({
                            "spaceId": space_id,
                            "principal": actor.principal,
                            "requestedRole": actor.role,
                            "boundRole": binding.role
                        })),
                    ));
                }
            }
            Ok(None) => {
                degraded_reason = Some("missing_canister_actor_role_binding".to_string());
                if block_on_degraded {
                    return Err(cortex_ux_error(
                        StatusCode::FORBIDDEN,
                        "VIEWSPEC_GOVERNANCE_ROLE_BINDING_MISSING",
                        "Canonical governance role binding is required for this action.",
                        Some(json!({
                            "spaceId": space_id,
                            "principal": actor.principal,
                            "actionTarget": action_target
                        })),
                    ));
                }
            }
            Err(err) => {
                degraded_reason = Some(format!("actor_role_binding_query_failed:{err}"));
                if block_on_degraded {
                    return Err(cortex_ux_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "VIEWSPEC_GOVERNANCE_CANONICAL_REQUIRED",
                        "Canonical governance authority is required for this action.",
                        Some(json!({
                            "spaceId": space_id,
                            "actionTarget": action_target,
                            "reason": degraded_reason
                        })),
                    ));
                }
            }
        }

        match client
            .evaluate_action_scope_with_actor(
                space_id,
                action_target,
                "attributed",
                gate_level,
                &actor_principal,
            )
            .await
        {
            Ok(evaluation) => {
                gate_status = Some(evaluation.gate_decision.clone());
                if evaluation.gate_decision.eq_ignore_ascii_case("block") {
                    if can_bridge_viewspec_governance_block(&evaluation) {
                        match missing_viewspec_required_actions(
                            proposal_id,
                            action_target,
                            &actor.principal,
                            &actor.role,
                            &evaluation.required_actions,
                        ) {
                            Ok(missing) if missing.is_empty() => {
                                gate_status = Some("allow".to_string());
                            }
                            Ok(missing) => {
                                return Err(cortex_ux_error(
                                    StatusCode::FORBIDDEN,
                                    "VIEWSPEC_GOVERNANCE_REQUIRED_ACTIONS_MISSING",
                                    "Required decision actions must be recorded before ratify/merge can proceed.",
                                    Some(json!({
                                        "spaceId": space_id,
                                        "actionTarget": action_target,
                                        "reason": evaluation.reason,
                                        "requiredActions": evaluation.required_actions,
                                        "missingActions": missing,
                                        "mutationId": proposal_id,
                                        "policyRef": evaluation.policy_ref,
                                        "policyVersion": evaluation.policy_version
                                    })),
                                ));
                            }
                            Err(err) => {
                                return Err(cortex_ux_error(
                                    StatusCode::INTERNAL_SERVER_ERROR,
                                    "VIEWSPEC_GOVERNANCE_ACTION_LOAD_FAILED",
                                    "Failed to load decision action evidence for governance bridge.",
                                    Some(json!({
                                        "mutationId": proposal_id,
                                        "reason": err
                                    })),
                                ));
                            }
                        }
                    } else {
                        return Err(cortex_ux_error(
                            StatusCode::FORBIDDEN,
                            "VIEWSPEC_GOVERNANCE_DENIED",
                            "Governance policy denied this proposal action.",
                            Some(json!({
                                "spaceId": space_id,
                                "actionTarget": action_target,
                                "reason": evaluation.reason,
                                "requiredActions": evaluation.required_actions,
                                "policyRef": evaluation.policy_ref,
                                "policyVersion": evaluation.policy_version
                            })),
                        ));
                    }
                }
                if degraded_reason.is_none() {
                    source_of_truth = "canister".to_string();
                }
            }
            Err(err) => {
                degraded_reason = Some(format!("governance_scope_evaluation_failed:{err}"));
                gate_status = Some("degraded".to_string());
                if block_on_degraded {
                    return Err(cortex_ux_error(
                        StatusCode::SERVICE_UNAVAILABLE,
                        "VIEWSPEC_GOVERNANCE_CANONICAL_REQUIRED",
                        "Canonical governance authority is required for this action.",
                        Some(json!({
                            "spaceId": space_id,
                            "actionTarget": action_target,
                            "reason": degraded_reason
                        })),
                    ));
                }
            }
        }
    } else {
        degraded_reason = Some("governance_client_unavailable".to_string());
        gate_status = Some("degraded".to_string());
        if block_on_degraded {
            return Err(cortex_ux_error(
                StatusCode::SERVICE_UNAVAILABLE,
                "VIEWSPEC_GOVERNANCE_CANONICAL_REQUIRED",
                "Canonical governance authority is required for this action.",
                Some(json!({
                    "spaceId": space_id,
                    "actionTarget": action_target,
                    "reason": degraded_reason
                })),
            ));
        }
    }

    Ok(ViewSpecGovernanceDecisionGate {
        gate_level: gate_level.to_string(),
        gate_status: gate_status.unwrap_or_else(|| "review".to_string()),
        decision_gate_id,
        replay_contract_ref,
        source_of_truth,
        degraded_reason,
        actor_principal: actor.principal,
        actor_role: actor.role,
    })
}

async fn build_viewspec_replay_and_digest(
    proposal: &ViewSpecProposalEnvelope,
) -> Result<(ViewSpecReplayArtifact, ViewSpecDigestArtifact), String> {
    let active_adoption = load_viewspec_active_scope(proposal.scope_key.as_str()).await?;
    let mut signal_count = 0u64;
    if let Some(spec) = load_viewspec(proposal.view_spec_id.as_str(), None).await? {
        if let Some(space_id) = spec.scope.space_id.as_deref() {
            let signals = load_viewspec_learning_signals(space_id).await?;
            signal_count = signals
                .iter()
                .filter(|signal| signal.view_spec_id == proposal.view_spec_id)
                .count() as u64;
        }
    }

    let lineage = json!({
        "proposalId": proposal.proposal_id.clone(),
        "viewSpecId": proposal.view_spec_id.clone(),
        "scopeKey": proposal.scope_key.clone(),
        "status": proposal.status.clone(),
        "review": proposal.review.clone(),
        "decision": proposal.decision.clone(),
        "merge": proposal.merge.clone()
    });
    let gate_metadata = json!({
        "governanceRef": proposal.governance_ref.clone(),
        "activeAdoption": active_adoption
    });
    let run_id = format!("viewspec_replay_{}", Utc::now().timestamp_millis());
    let replay = ViewSpecReplayArtifact {
        schema_version: "1.0.0".to_string(),
        run_id,
        proposal_id: proposal.proposal_id.clone(),
        scope_key: proposal.scope_key.clone(),
        generated_at: viewspec_now_iso(),
        proposal: proposal.clone(),
        lineage: lineage.clone(),
        gate_metadata: gate_metadata.clone(),
        signal_count,
    };
    let payload = json!({
        "lineage": lineage,
        "gateMetadata": gate_metadata,
        "signalCount": signal_count
    });
    let digest = ViewSpecDigestArtifact {
        schema_version: "1.0.0".to_string(),
        proposal_id: proposal.proposal_id.clone(),
        digest: viewspec_digest_hex(&payload),
        generated_at: viewspec_now_iso(),
        scope_key: proposal.scope_key.clone(),
        status: viewspec_proposal_status_name(&proposal.status).to_string(),
        payload,
    };
    Ok((replay, digest))
}

fn deterministic_action_id(action: &str, record: &DecisionActionRecord) -> String {
    let canonical = json!({
        "action": action,
        "decision_gate_id": record.decision_gate_id,
        "workflow_id": record.workflow_id,
        "mutation_id": record.mutation_id,
        "action_target": record.action_target,
        "risk_statement": record.risk_statement,
        "rollback_path": record.rollback_path,
        "evidence_refs": record.evidence_refs,
        "lineage_id": record.lineage_id,
        "policy_ref": record.policy_ref,
        "actor_ref": record.actor_ref,
        "note": record.note
    });
    let bytes = serde_json::to_vec(&canonical).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(bytes);
    let digest = hex::encode(hasher.finalize());
    format!("decision_{}_{}", action, &digest[..16])
}

async fn record_decision_action(
    action: &str,
    headers: &HeaderMap,
    request: DecisionActionRequest,
) -> Result<DecisionSurfaceEnvelope, axum::response::Response> {
    fn resolve_action_space(request: &DecisionActionRequest) -> (String, Option<String>) {
        if let Some(space_id) = request
            .space_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            return (space_id.to_string(), None);
        }
        if let Ok(space_id) = std::env::var("NOSTRA_ACTIVE_SPACE_ID") {
            let normalized = space_id.trim().to_string();
            if !normalized.is_empty() {
                return (
                    normalized,
                    Some("space_id_missing_request_used_env".to_string()),
                );
            }
        }
        (
            "space-default".to_string(),
            Some("space_id_missing_defaulted_space_default".to_string()),
        )
    }

    let decision_gate_id = request
        .decision_gate_id
        .clone()
        .or_else(|| {
            request
                .mutation_id
                .as_ref()
                .map(|id| format!("blackwell_gate:{id}"))
        })
        .ok_or_else(|| {
            decision_surface_error(
                StatusCode::BAD_REQUEST,
                "MISSING_DECISION_GATE_ID",
                "decision_gate_id or mutation_id is required",
                None,
            )
        })?;
    let mutation_id = request
        .mutation_id
        .clone()
        .or_else(|| parse_mutation_id_from_gate_id(&decision_gate_id))
        .ok_or_else(|| {
            decision_surface_error(
                StatusCode::BAD_REQUEST,
                "MISSING_MUTATION_ID",
                "mutation_id is required",
                Some(json!({ "decisionGateId": decision_gate_id })),
            )
        })?;
    let workflow_id = request
        .workflow_id
        .clone()
        .unwrap_or_else(|| "workflow:unknown".to_string());
    let (space_id, space_degraded_reason) = resolve_action_space(&request);
    let mut degraded_reasons = Vec::<String>::new();
    if let Some(reason) = space_degraded_reason {
        degraded_reasons.push(reason);
    }
    let action_target = request
        .action_target
        .clone()
        .unwrap_or_else(|| "decision_target:unknown".to_string());
    let risky_gate = decision_gate_id.starts_with("blackwell_gate:")
        || decision_gate_id.starts_with("system_test_gate:")
        || action_target.to_ascii_lowercase().contains("governance")
        || action_target.to_ascii_lowercase().contains("release");
    let gate_level = request.gate_level.clone().unwrap_or_else(|| {
        if risky_gate {
            "release_blocker"
        } else {
            "informational"
        }
        .to_string()
    });
    let signed_mode = decision_signed_mode();
    let signature_required = signature_required_for_gate(signed_mode, &gate_level, risky_gate);
    let actor = resolve_actor_identity(
        headers,
        &request,
        &decision_gate_id,
        &mutation_id,
        &action_target,
        signature_required,
        signature_required,
    )?;
    if actor.auth_status.eq_ignore_ascii_case("warn") {
        if let Some(reason) = actor.auth_reason.as_deref() {
            degraded_reasons.push(format!("auth:{reason}"));
        }
    }
    let actor_ref = format!("{}#{}", actor.principal, actor.role);
    if risky_gate {
        validate_quality_payload(&request).map_err(|err| {
            decision_surface_error(
                StatusCode::BAD_REQUEST,
                "INVALID_OVERRIDE_PAYLOAD",
                "Decision action missing required quality fields",
                Some(json!({ "reason": err, "decisionGateId": decision_gate_id })),
            )
        })?;
    }

    let domain_mode = request
        .domain_mode
        .clone()
        .unwrap_or_else(|| "attributed".to_string());
    let policy_action_target = format!(
        "{}|role:{}|principal:{}",
        action_target, actor.role, actor.principal
    );
    let actor_principal = Principal::from_text(&actor.principal).map_err(|_| {
        decision_surface_error(
            StatusCode::FORBIDDEN,
            "INVALID_ACTOR_PRINCIPAL",
            "Decision action rejected: actor principal is invalid",
            Some(json!({ "principal": actor.principal })),
        )
    })?;
    let governance_client = GovernanceClient::from_env().ok();
    #[cfg(test)]
    let mock_role_binding = test_override_actor_role_binding();
    #[cfg(not(test))]
    let mock_role_binding: Option<Result<Option<String>, String>> = None;
    #[cfg(test)]
    let mock_policy_eval = test_override_policy_evaluation();
    #[cfg(not(test))]
    let mock_policy_eval: Option<Result<ActionScopeEvaluation, String>> = None;

    let policy_eval = if mock_role_binding.is_some() || mock_policy_eval.is_some() {
        if let Some(binding_result) = mock_role_binding {
            match binding_result {
                Ok(Some(bound_role)) => {
                    if bound_role != actor.role {
                        return Err(decision_surface_error(
                            StatusCode::FORBIDDEN,
                            "ROLE_BINDING_MISMATCH",
                            "Decision action rejected: canister role binding mismatch",
                            Some(json!({
                                "spaceId": space_id,
                                "principal": actor.principal,
                                "requestedRole": actor.role,
                                "boundRole": bound_role
                            })),
                        ));
                    }
                }
                Ok(None) => {
                    if env_role_fallback_allowed() {
                        degraded_reasons
                            .push("role_binding_missing_canister_env_fallback".to_string());
                    } else if risky_gate {
                        return Err(decision_surface_error(
                            StatusCode::FORBIDDEN,
                            "MISSING_CANISTER_ROLE_BINDING",
                            "Decision action rejected: missing canister actor-role binding",
                            Some(json!({
                                "spaceId": space_id,
                                "principal": actor.principal,
                                "requiredAction": "upsert_actor_role_binding"
                            })),
                        ));
                    } else {
                        degraded_reasons
                            .push("role_binding_missing_canister_non_risky".to_string());
                    }
                }
                Err(err) => {
                    degraded_reasons.push(format!("role_binding_query_failed:{err}"));
                }
            }
        }

        match mock_policy_eval {
            Some(Ok(value)) => Some(value),
            Some(Err(err)) => {
                degraded_reasons.push(format!("governance_eval_failed:{err}"));
                None
            }
            None => None,
        }
    } else if let Some(client) = governance_client.as_ref() {
        match client
            .get_actor_role_binding(&space_id, &actor_principal)
            .await
        {
            Ok(Some(binding)) => {
                if binding.role != actor.role {
                    return Err(decision_surface_error(
                        StatusCode::FORBIDDEN,
                        "ROLE_BINDING_MISMATCH",
                        "Decision action rejected: canister role binding mismatch",
                        Some(json!({
                            "spaceId": space_id,
                            "principal": actor.principal,
                            "requestedRole": actor.role,
                            "boundRole": binding.role
                        })),
                    ));
                }
            }
            Ok(None) => {
                if env_role_fallback_allowed() {
                    degraded_reasons.push("role_binding_missing_canister_env_fallback".to_string());
                } else if risky_gate {
                    return Err(decision_surface_error(
                        StatusCode::FORBIDDEN,
                        "MISSING_CANISTER_ROLE_BINDING",
                        "Decision action rejected: missing canister actor-role binding",
                        Some(json!({
                            "spaceId": space_id,
                            "principal": actor.principal,
                            "requiredAction": "upsert_actor_role_binding"
                        })),
                    ));
                } else {
                    degraded_reasons.push("role_binding_missing_canister_non_risky".to_string());
                }
            }
            Err(err) => {
                degraded_reasons.push(format!("role_binding_query_failed:{err}"));
            }
        }
        match client
            .evaluate_action_scope_with_actor(
                &space_id,
                &policy_action_target,
                &domain_mode,
                &gate_level,
                &actor_principal,
            )
            .await
        {
            Ok(value) => Some(value),
            Err(err) => {
                degraded_reasons.push(format!("governance_eval_failed:{err}"));
                None
            }
        }
    } else {
        degraded_reasons.push("governance_client_unavailable".to_string());
        None
    };
    if let Some(evaluation) = policy_eval.as_ref() {
        if evaluation.gate_decision.eq_ignore_ascii_case("block") {
            let normalized_action = action.trim().to_ascii_lowercase();
            let required_actions = decision_actions_from_requirements(&evaluation.required_actions);
            let action_is_required = (normalized_action == "ack"
                && required_actions.contains(&"ack"))
                || (normalized_action == "escalate" && required_actions.contains(&"escalate"));
            if !action_is_required {
                return Err(decision_surface_error(
                    StatusCode::FORBIDDEN,
                    "POLICY_GATE_BLOCKED",
                    "Decision action blocked by governance policy gate",
                    Some(json!({
                        "reason": evaluation.reason,
                        "requiredActions": evaluation.required_actions,
                        "policyRef": evaluation.policy_ref,
                        "policyVersion": evaluation.policy_version
                    })),
                ));
            }
        }
    }

    let lineage_id = format!(
        "lineage:{}",
        deterministic_action_id(
            action,
            &DecisionActionRecord {
                schema_version: "1.0.0".to_string(),
                action_id: String::new(),
                action: action.to_string(),
                decision_gate_id: decision_gate_id.clone(),
                workflow_id: workflow_id.clone(),
                mutation_id: mutation_id.clone(),
                action_target: action_target.clone(),
                risk_statement: request.risk_statement.clone().unwrap_or_default(),
                rollback_path: request.rollback_path.clone().unwrap_or_default(),
                evidence_refs: request.evidence_refs.clone(),
                lineage_id: String::new(),
                policy_ref: policy_eval
                    .as_ref()
                    .and_then(|entry| entry.policy_ref.clone()),
                actor_ref: Some(actor_ref.clone()),
                note: request.note.clone(),
                created_at: now_iso(),
            }
        )
    );
    let mut record = DecisionActionRecord {
        schema_version: "1.0.0".to_string(),
        action_id: String::new(),
        action: action.to_string(),
        decision_gate_id: decision_gate_id.clone(),
        workflow_id: workflow_id.clone(),
        mutation_id: mutation_id.clone(),
        action_target,
        risk_statement: request.risk_statement.unwrap_or_default(),
        rollback_path: request.rollback_path.unwrap_or_default(),
        evidence_refs: request.evidence_refs.clone(),
        lineage_id: lineage_id.clone(),
        policy_ref: policy_eval
            .as_ref()
            .and_then(|entry| entry.policy_ref.clone()),
        actor_ref: Some(actor_ref.clone()),
        note: request.note.clone(),
        created_at: now_iso(),
    };
    record.action_id = deterministic_action_id(action, &record);

    let action_path = decision_actions_dir().join(format!("{}.json", record.action_id));
    let action_value = serde_json::to_value(&record).map_err(|err| {
        decision_surface_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERIALIZE_ACTION_FAILED",
            "Unable to serialize decision action",
            Some(json!({ "reason": err.to_string() })),
        )
    })?;
    persist_json(&action_path, &action_value).map_err(|err| {
        decision_surface_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PERSIST_ACTION_FAILED",
            "Unable to persist decision action",
            Some(json!({ "path": action_path.display().to_string(), "reason": err })),
        )
    })?;

    let envelope = build_decision_envelope(
        if action == "ack" {
            format!("blackwell_override_ack:{}", mutation_id)
        } else {
            format!("blackwell_gate:{}", mutation_id)
        },
        workflow_id,
        mutation_id.clone(),
        action.to_string(),
        if action == "ack" {
            Vec::new()
        } else {
            vec![format!("decision_ack:{mutation_id}")]
        },
        request.evidence_refs.clone(),
        Some(if policy_eval.is_some() {
            "canister".to_string()
        } else if env_role_fallback_allowed() {
            "fallback".to_string()
        } else {
            "gateway".to_string()
        }),
        Some(lineage_id.clone()),
        policy_eval
            .as_ref()
            .and_then(|entry| entry.policy_ref.clone()),
        policy_eval.as_ref().map(|entry| entry.policy_version),
        if degraded_reasons.is_empty() {
            None
        } else {
            Some(degraded_reasons.join(";"))
        },
        Some(actor.auth_status.clone()),
        actor.auth_reason.clone(),
        Some(json!({
            "spaceId": space_id,
            "decisionGateId": decision_gate_id,
            "actionId": record.action_id,
            "lineageId": lineage_id,
            "actorPrincipal": actor.principal,
            "actorRole": actor.role,
            "signatureValidated": actor.signed,
            "authStatus": actor.auth_status,
            "authReason": actor.auth_reason,
            "signedMode": format!("{:?}", signed_mode),
            "policyEvaluation": policy_eval,
            "recordPath": action_path.display().to_string()
        })),
    );

    let cache_path = decision_projection_cache_dir().join(format!(
        "decision_gate_{}.json",
        sanitize_fs_component(&mutation_id)
    ));
    let cache_value = serde_json::to_value(&envelope).map_err(|err| {
        decision_surface_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "SERIALIZE_ENVELOPE_FAILED",
            "Unable to serialize decision projection envelope",
            Some(json!({ "reason": err.to_string() })),
        )
    })?;
    persist_json(&cache_path, &cache_value).map_err(|err| {
        decision_surface_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PERSIST_ENVELOPE_FAILED",
            "Unable to persist decision projection envelope",
            Some(json!({ "path": cache_path.display().to_string(), "reason": err })),
        )
    })?;

    Ok(envelope)
}

fn synthesize_testing_gate_surface(summary: &TestGateSummaryArtifact) -> Value {
    let run_id = summary
        .latest_run_id
        .clone()
        .unwrap_or_else(|| "latest".to_string());

    let failure_lines = if summary.failures.is_empty() {
        "No gate failures detected.".to_string()
    } else {
        summary
            .failures
            .iter()
            .map(|f| format!("- {}: {}", f.code, f.message))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let priority = if summary.required_blockers_pass {
        "p2"
    } else {
        "p0"
    };
    let tone = if summary.required_blockers_pass {
        "info"
    } else {
        "critical"
    };
    let verdict = summary.overall_verdict.to_uppercase();

    json!({
      "type": "RenderSurface",
      "surfaceId": format!("system_test_gate:{}", run_id),
      "meta": {
        "context": "system",
        "priority": priority,
        "tone": tone,
        "severity": "system",
        "source": "cortex-testing-gateway",
        "timestamp": summary.generated_at
      },
      "components": [
        {
          "id": "root",
          "type": "Card",
          "props": { "title": "Test Gate Summary" },
          "children": ["summary", "verdict", "failures"]
        },
        {
          "id": "summary",
          "type": "Text",
          "props": {
            "text": format!("Mode: {} | Catalog valid: {} | Run artifacts valid: {}", summary.mode, summary.catalog_valid, summary.run_artifacts_valid)
          }
        },
        {
          "id": "verdict",
          "type": "StatusBadge",
          "props": {
            "status": if summary.overall_verdict == "ready" { "Success" } else { "Error" },
            "label": verdict
          }
        },
        {
          "id": "failures",
          "type": "Markdown",
          "props": { "content": failure_lines }
        }
      ]
    })
}

fn build_acp_adapter() -> Result<AcpAdapter, AcpPolicyError> {
    // Spike scope: constrain ACP file and terminal operations to the existing workflow root.
    let workflow_root = FileSystemService::get_root_path();
    let cfg = AcpPolicyConfig::baseline(vec![workflow_root]);
    AcpAdapter::new(cfg)
}

fn acp_error_response(err: AcpPolicyError) -> axum::response::Response {
    let status = match err {
        AcpPolicyError::Io(_) => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        _ => axum::http::StatusCode::BAD_REQUEST,
    };
    (
        status,
        Json(serde_json::json!({ "error": err.to_string() })),
    )
        .into_response()
}

fn acp_pilot_disabled_rest_response() -> axum::response::Response {
    (
        axum::http::StatusCode::SERVICE_UNAVAILABLE,
        Json(serde_json::json!({
            "error": "ACP pilot is disabled",
            "errorCode": "ACP_PILOT_DISABLED",
        })),
    )
        .into_response()
}

async fn acp_read_text_file(Json(payload): Json<FsReadTextFileRequest>) -> impl IntoResponse {
    let adapter = match build_acp_adapter() {
        Ok(adapter) => adapter,
        Err(err) => return acp_error_response(err),
    };

    match adapter.read_text_file(payload) {
        Ok(result) => Json(result).into_response(),
        Err(err) => acp_error_response(err),
    }
}

async fn acp_write_text_file(Json(payload): Json<FsWriteTextFileRequest>) -> impl IntoResponse {
    let adapter = match build_acp_adapter() {
        Ok(adapter) => adapter,
        Err(err) => return acp_error_response(err),
    };

    match adapter.write_text_file(payload) {
        Ok(()) => Json(serde_json::json!({ "status": "ok" })).into_response(),
        Err(err) => acp_error_response(err),
    }
}

async fn acp_rpc(Json(payload): Json<JsonRpcRequest>) -> impl IntoResponse {
    Json(handle_rpc_request(payload).await).into_response()
}

async fn acp_terminal_create(Json(payload): Json<TerminalCreateRequest>) -> impl IntoResponse {
    if !is_acp_pilot_enabled() {
        return acp_pilot_disabled_rest_response();
    }
    let adapter = match build_acp_adapter() {
        Ok(adapter) => adapter,
        Err(err) => return acp_error_response(err),
    };

    match adapter.validate_terminal_create(payload) {
        Ok(validated) => match TerminalService::acp_terminal_create(validated).await {
            Ok(created) => Json(serde_json::json!({
                "status": "created",
                "terminalId": created.terminal_id,
                "command": created.command,
                "args": created.args,
                "cwd": created.cwd,
                "outputByteLimit": created.output_byte_limit
            }))
            .into_response(),
            Err(err) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": err })),
            )
                .into_response(),
        },
        Err(err) => acp_error_response(err),
    }
}

async fn acp_terminal_output(Json(payload): Json<AcpTerminalOutputRequest>) -> impl IntoResponse {
    if !is_acp_pilot_enabled() {
        return acp_pilot_disabled_rest_response();
    }
    match TerminalService::acp_terminal_output(payload).await {
        Ok(result) => Json(result).into_response(),
        Err(err) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn acp_terminal_wait_for_exit(
    Json(payload): Json<AcpTerminalWaitRequest>,
) -> impl IntoResponse {
    if !is_acp_pilot_enabled() {
        return acp_pilot_disabled_rest_response();
    }
    match TerminalService::acp_terminal_wait_for_exit(payload).await {
        Ok(result) => Json(result).into_response(),
        Err(err) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct TerminalIdRequest {
    terminal_id: String,
}

async fn acp_terminal_kill(Json(payload): Json<TerminalIdRequest>) -> impl IntoResponse {
    if !is_acp_pilot_enabled() {
        return acp_pilot_disabled_rest_response();
    }
    match TerminalService::acp_terminal_kill(payload.terminal_id).await {
        Ok(result) => Json(result).into_response(),
        Err(err) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn acp_terminal_release(Json(payload): Json<TerminalIdRequest>) -> impl IntoResponse {
    if !is_acp_pilot_enabled() {
        return acp_pilot_disabled_rest_response();
    }
    match TerminalService::acp_terminal_release(payload.terminal_id) {
        Ok(result) => Json(result).into_response(),
        Err(err) => (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn read_workflow(Json(payload): Json<WorkflowReadRequest>) -> impl IntoResponse {
    match crate::services::file_system_service::FileSystemService::read_file(&payload.path) {
        Some(content) => Json(serde_json::json!({ "content": content })).into_response(),
        None => (axum::http::StatusCode::NOT_FOUND, "File not found").into_response(),
    }
}

async fn save_workflow(Json(payload): Json<WorkflowSaveRequest>) -> impl IntoResponse {
    match crate::services::file_system_service::FileSystemService::save_file(
        &payload.path,
        &payload.content,
    ) {
        Ok(_) => Json(serde_json::json!({ "status": "success" })).into_response(),
        Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

async fn run_workflow(
    State(state): State<GatewayState>,
    Json(payload): Json<WorkflowReadRequest>, // Reusing ReadRequest as it has 'path'
) -> impl IntoResponse {
    let path = payload.path.clone();

    // Read content
    if let Some(content) = crate::services::file_system_service::FileSystemService::read_file(&path)
    {
        let tx = state.broadcast_tx.clone();
        tokio::spawn(async move {
            crate::services::workflow_executor::WorkflowExecutor::run_workflow(
                content,
                Arc::new(tx),
            )
            .await;
        });
        Json(serde_json::json!({ "status": "started", "message": "Execution started in background" })).into_response()
    } else {
        (axum::http::StatusCode::NOT_FOUND, "Workflow file not found").into_response()
    }
}

#[derive(Deserialize)]
struct IngestDoc {
    id: String,
    content: String,
    #[serde(rename = "metadata")]
    _metadata: Option<serde_json::Value>,
    modality: Option<String>,
}

async fn ingest_document(Json(payload): Json<IngestDoc>) -> impl IntoResponse {
    tracing::info!("Received document for ingestion: {}", payload.id);

    let modality = match payload.modality.as_deref() {
        Some("Image") | Some("image") => crate::services::agent_service::Modality::Image,
        Some("Audio") | Some("audio") => crate::services::agent_service::Modality::Audio,
        Some("Video") | Some("video") => crate::services::agent_service::Modality::Video,
        _ => crate::services::agent_service::Modality::Text,
    };

    // Call AgentService to index
    crate::services::agent_service::AgentService::index(
        payload.id.clone(),
        payload.content.clone(),
        modality,
    )
    .await;

    // Broadcast "Thinking" / "Ingesting" event to UI
    // let _ = state.broadcast_tx.send(Message::Text(format!("Ingesting: {}", payload.id)));

    Json(
        serde_json::json!({ "status": "queued", "message": format!("Ingestion queued for {}", payload.id) }),
    )
}

#[derive(Deserialize)]
struct SearchFilters {
    modality: Option<String>,
}

#[derive(Deserialize)]
struct SearchRequest {
    query: String,
    filters: Option<SearchFilters>,
}

async fn search_vector(Json(payload): Json<SearchRequest>) -> impl IntoResponse {
    let modality_filter = if let Some(filters) = payload.filters {
        match filters.modality.as_deref() {
            Some("Image") | Some("image") => Some(crate::services::agent_service::Modality::Image),
            Some("Audio") | Some("audio") => Some(crate::services::agent_service::Modality::Audio),
            Some("Video") | Some("video") => Some(crate::services::agent_service::Modality::Video),
            Some("Text") | Some("text") => Some(crate::services::agent_service::Modality::Text),
            _ => None,
        }
    } else {
        None
    };

    let results =
        crate::services::agent_service::AgentService::search(payload.query, modality_filter).await;
    Json(serde_json::json!({ "results": results }))
}

async fn get_resilience_metrics() -> impl IntoResponse {
    let svc = crate::services::resilience_service::ResilienceService::new();
    let report = svc.calculate_scores().await;
    Json(report)
}

async fn get_acp_metrics() -> impl IntoResponse {
    Json(get_acp_metrics_snapshot())
}

async fn get_testing_catalog() -> impl IntoResponse {
    let path = testing_catalog_path();
    match read_json_artifact::<TestCatalogArtifact>(&path) {
        Ok(catalog) => Json(catalog).into_response(),
        Err(err) => err,
    }
}

async fn get_testing_runs(Query(query): Query<TestingRunsQuery>) -> impl IntoResponse {
    let runs_dir = testing_runs_dir();
    let entries = match fs::read_dir(&runs_dir) {
        Ok(entries) => entries,
        Err(err) => {
            return testing_error(
                StatusCode::NOT_FOUND,
                "RUNS_NOT_FOUND",
                "Testing runs directory does not exist",
                Some(json!({
                    "path": runs_dir.display().to_string(),
                    "reason": err.to_string()
                })),
            );
        }
    };

    let mut runs = Vec::<TestRunArtifact>::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Ok(run) = read_json_artifact::<TestRunArtifact>(&path) {
            runs.push(run);
        }
    }

    runs.sort_by(|a, b| b.finished_at.cmp(&a.finished_at));

    if let Some(status) = query.status.as_ref() {
        let status = status.to_ascii_lowercase();
        runs.retain(|run| {
            run.results
                .iter()
                .any(|result| result.status.to_ascii_lowercase() == status)
        });
    }

    let limit = query.limit.unwrap_or(20).min(500);
    if runs.len() > limit {
        runs.truncate(limit);
    }

    Json(runs).into_response()
}

async fn get_testing_run(Path(run_id): Path<String>) -> impl IntoResponse {
    if run_id.contains('/') || run_id.contains('\\') || run_id.contains("..") {
        return testing_error(
            StatusCode::BAD_REQUEST,
            "INVALID_RUN_ID",
            "run_id contains invalid path characters",
            Some(json!({ "run_id": run_id })),
        );
    }

    let path = testing_runs_dir().join(format!("{}.json", run_id));
    match read_json_artifact::<TestRunArtifact>(&path) {
        Ok(run) => Json(run).into_response(),
        Err(err) => err,
    }
}

async fn get_testing_gates_latest() -> impl IntoResponse {
    let path = testing_gate_summary_path();
    let summary = match read_json_artifact::<TestGateSummaryArtifact>(&path) {
        Ok(summary) => summary,
        Err(err) => return err,
    };

    let response = TestGateLatestResponse {
        surface: if should_emit_testing_surface() {
            Some(synthesize_testing_gate_surface(&summary))
        } else {
            None
        },
        summary,
    };

    Json(response).into_response()
}

async fn get_testing_health() -> impl IntoResponse {
    let log_dir = testing_log_dir();
    let catalog_path = testing_catalog_path();
    let gate_path = testing_gate_summary_path();
    let runs_dir = testing_runs_dir();

    let catalog_last_modified = file_last_modified_secs(&catalog_path);
    let gate_last_modified = file_last_modified_secs(&gate_path);

    let mut latest_run_last_modified: Option<u64> = None;
    let mut runs_count = 0usize;
    if let Ok(entries) = fs::read_dir(&runs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            runs_count += 1;
            if let Some(modified) = file_last_modified_secs(&path) {
                latest_run_last_modified =
                    Some(latest_run_last_modified.unwrap_or(0).max(modified));
            }
        }
    }

    let catalog_exists = catalog_path.exists();
    let gate_exists = gate_path.exists();
    let catalog_fresh = is_fresh(catalog_last_modified);
    let gate_fresh = is_fresh(gate_last_modified);

    let status = if catalog_exists && gate_exists && catalog_fresh && gate_fresh {
        "ok".to_string()
    } else if catalog_exists || gate_exists {
        "degraded".to_string()
    } else {
        "missing".to_string()
    };

    Json(TestingHealthResponse {
        status,
        testing_log_dir: log_dir.display().to_string(),
        schema_version: TESTING_SCHEMA_VERSION.to_string(),
        catalog_exists,
        gate_exists,
        runs_count,
        catalog_last_modified,
        gate_last_modified,
        latest_run_last_modified,
        catalog_fresh,
        gate_fresh,
    })
    .into_response()
}

async fn get_siq_coverage() -> impl IntoResponse {
    let path = siq_coverage_path();
    match read_siq_json_artifact::<SiqCoverage>(&path) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => err,
    }
}

async fn get_siq_dependency_closure() -> impl IntoResponse {
    let path = siq_dependency_closure_path();
    match read_siq_json_artifact::<SiqDependencyClosure>(&path) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => err,
    }
}

async fn get_siq_gates_latest() -> impl IntoResponse {
    let path = siq_gate_summary_path(None);
    match read_siq_json_artifact::<SiqGateSummary>(&path) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => err,
    }
}

async fn get_siq_graph_projection() -> impl IntoResponse {
    let path = siq_graph_projection_path(None);
    match read_siq_json_artifact::<SiqGraphProjection>(&path) {
        Ok(payload) => Json(payload).into_response(),
        Err(err) => err,
    }
}

async fn get_siq_runs(Query(query): Query<SiqRunsQuery>) -> impl IntoResponse {
    let runs_dir = siq_runs_dir();
    let entries = match fs::read_dir(&runs_dir) {
        Ok(entries) => entries,
        Err(err) => {
            return siq_error(
                StatusCode::NOT_FOUND,
                "RUNS_NOT_FOUND",
                "SIQ runs directory does not exist",
                Some(json!({
                    "path": runs_dir.display().to_string(),
                    "reason": err.to_string()
                })),
            );
        }
    };

    let mut runs = Vec::<SiqRunArtifact>::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        if let Ok(run) = read_siq_json_artifact::<SiqRunArtifact>(&path) {
            runs.push(run);
        }
    }

    runs.sort_by(|a, b| b.generated_at.cmp(&a.generated_at));
    let limit = query.limit.unwrap_or(20).min(500);
    if runs.len() > limit {
        runs.truncate(limit);
    }
    Json(runs).into_response()
}

async fn get_siq_run(Path(run_id): Path<String>) -> impl IntoResponse {
    if run_id.contains('/') || run_id.contains('\\') || run_id.contains("..") {
        return siq_error(
            StatusCode::BAD_REQUEST,
            "INVALID_RUN_ID",
            "run_id contains invalid path characters",
            Some(json!({ "run_id": run_id })),
        );
    }

    let path = siq_runs_dir().join(format!("{}.json", run_id));
    match read_siq_json_artifact::<SiqRunArtifact>(&path) {
        Ok(run) => Json(run).into_response(),
        Err(err) => err,
    }
}

async fn get_siq_health() -> impl IntoResponse {
    let log_dir = siq_log_dir();
    let coverage_path = siq_coverage_path();
    let dependency_path = siq_dependency_closure_path();
    let gate_path = siq_gate_summary_path(None);
    let projection_path = siq_graph_projection_path(None);
    let runs_dir = siq_runs_dir();

    let coverage_last_modified = file_last_modified_secs(&coverage_path);
    let dependency_last_modified = file_last_modified_secs(&dependency_path);
    let gate_last_modified = file_last_modified_secs(&gate_path);
    let projection_last_modified = file_last_modified_secs(&projection_path);

    let mut latest_run_last_modified: Option<u64> = None;
    let mut runs_count = 0usize;
    if let Ok(entries) = fs::read_dir(&runs_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            runs_count += 1;
            if let Some(modified) = file_last_modified_secs(&path) {
                latest_run_last_modified =
                    Some(latest_run_last_modified.unwrap_or(0).max(modified));
            }
        }
    }

    let coverage_exists = coverage_path.exists();
    let dependency_exists = dependency_path.exists();
    let gate_exists = gate_path.exists();
    let projection_exists = projection_path.exists();
    let coverage_fresh = coverage_last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= SIQ_STALE_AFTER_SECS)
        .unwrap_or(false);
    let dependency_fresh = dependency_last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= SIQ_STALE_AFTER_SECS)
        .unwrap_or(false);
    let gate_fresh = gate_last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= SIQ_STALE_AFTER_SECS)
        .unwrap_or(false);
    let projection_fresh = projection_last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= SIQ_STALE_AFTER_SECS)
        .unwrap_or(false);

    let all_exist = coverage_exists && dependency_exists && gate_exists && projection_exists;
    let all_fresh = coverage_fresh && dependency_fresh && gate_fresh && projection_fresh;
    let any_exists = coverage_exists || dependency_exists || gate_exists || projection_exists;
    let status = if all_exist && all_fresh {
        "ok".to_string()
    } else if any_exists {
        "degraded".to_string()
    } else {
        "missing".to_string()
    };

    Json(SiqHealth {
        status,
        siq_log_dir: log_dir.display().to_string(),
        schema_version: SIQ_SCHEMA_VERSION.to_string(),
        coverage_exists,
        dependency_exists,
        gate_exists,
        projection_exists,
        runs_count,
        latest_run_last_modified,
        coverage_fresh,
        dependency_fresh,
        gate_fresh,
        projection_fresh,
    })
    .into_response()
}

async fn get_motoko_graph_snapshot() -> impl IntoResponse {
    let path = motoko_graph_snapshot_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            return motoko_graph_error(
                StatusCode::NOT_FOUND,
                "SNAPSHOT_NOT_FOUND",
                "Motoko-graph snapshot not found",
                Some(json!({
                    "path": path.display().to_string(),
                    "reason": err.to_string()
                })),
            );
        }
    };

    match serde_json::from_str::<MotokoGraphSnapshot>(&raw) {
        Ok(snapshot) => Json(snapshot).into_response(),
        Err(err) => motoko_graph_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "SNAPSHOT_INVALID",
            "Motoko-graph snapshot cannot be parsed",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        ),
    }
}

async fn get_motoko_graph_decision_history() -> impl IntoResponse {
    let mut events = read_decision_events_from_dir(&motoko_graph_history_dir(), Some("applied"));
    events.extend(read_decision_events_from_dir(
        &motoko_graph_pending_dir(),
        Some("pending"),
    ));
    events.sort_by(|a, b| {
        b.captured_at
            .cmp(&a.captured_at)
            .then_with(|| b.decision_event_id.cmp(&a.decision_event_id))
    });
    Json(events).into_response()
}

async fn get_motoko_graph_health() -> impl IntoResponse {
    let log_dir = motoko_graph_log_dir();
    let snapshot_path = motoko_graph_snapshot_path();
    let history_dir = motoko_graph_history_dir();
    let pending_dir = motoko_graph_pending_dir();

    let snapshot_exists = snapshot_path.exists();
    let snapshot_last_modified = file_last_modified_secs(&snapshot_path);
    let snapshot_fresh = snapshot_last_modified
        .map(|ts| now_secs().saturating_sub(ts) <= MOTOKO_GRAPH_STALE_AFTER_SECS)
        .unwrap_or(false);
    let history_count = count_json_files(&history_dir);
    let pending_count = count_json_files(&pending_dir);

    let status = if snapshot_exists && snapshot_fresh {
        "ok".to_string()
    } else if snapshot_exists {
        "degraded".to_string()
    } else {
        "missing".to_string()
    };

    Json(MotokoGraphHealthResponse {
        status,
        schema_version: MOTOKO_GRAPH_SCHEMA_VERSION.to_string(),
        kg_log_dir: log_dir.display().to_string(),
        snapshot_exists,
        history_count,
        pending_count,
        snapshot_last_modified,
        snapshot_fresh,
    })
    .into_response()
}

async fn get_motoko_graph_monitoring_trends() -> impl IntoResponse {
    let path = motoko_graph_monitoring_trend_path();
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(err) => {
            return motoko_graph_error(
                StatusCode::NOT_FOUND,
                "MONITORING_TREND_NOT_FOUND",
                "Motoko-graph monitoring trend artifact not found",
                Some(json!({
                    "path": path.display().to_string(),
                    "reason": err.to_string()
                })),
            );
        }
    };

    match serde_json::from_str::<MotokoGraphMonitoringTrend>(&raw) {
        Ok(trend) => Json(trend).into_response(),
        Err(err) => motoko_graph_error(
            StatusCode::UNPROCESSABLE_ENTITY,
            "MONITORING_TREND_INVALID",
            "Motoko-graph monitoring trend artifact cannot be parsed",
            Some(json!({
                "path": path.display().to_string(),
                "reason": err.to_string()
            })),
        ),
    }
}

async fn get_motoko_graph_monitoring_runs(
    Query(query): Query<MotokoGraphMonitoringRunsQuery>,
) -> impl IntoResponse {
    let mut runs = read_monitoring_runs_from_dir(&motoko_graph_monitoring_runs_dir());
    let limit = query.limit.unwrap_or(20).min(200);
    if runs.len() > limit {
        runs.truncate(limit);
    }
    Json(runs).into_response()
}

async fn capture_motoko_graph_decision(
    Json(payload): Json<DecisionCaptureRequest>,
) -> impl IntoResponse {
    if let Err(err) = validate_decision_capture_request(&payload) {
        return err;
    }

    let decision_event_id = decision_event_id_from_payload(&payload);
    let captured_at = Utc::now().to_rfc3339();
    let pending_dir = motoko_graph_pending_dir();
    if let Err(err) = fs::create_dir_all(&pending_dir) {
        return motoko_graph_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PENDING_DIR_CREATE_FAILED",
            "Unable to create pending decision directory",
            Some(json!({
                "path": pending_dir.display().to_string(),
                "reason": err.to_string()
            })),
        );
    }

    let event = MotokoGraphDecisionEvent {
        schema_version: MOTOKO_GRAPH_SCHEMA_VERSION.to_string(),
        decision_event_id: decision_event_id.clone(),
        captured_at,
        contribution: payload.contribution,
        decision_date: payload.decision_date,
        selected_option: payload.selected_option,
        rationale: payload.rationale,
        posture_before: payload.posture_before,
        posture_after: payload.posture_after,
        authority_mode: payload.authority_mode,
        evidence_refs: payload.evidence_refs,
        steward: payload.steward,
        owner: payload.owner,
        follow_up_actions: payload.follow_up_actions,
        source: payload.source,
        status: Some("pending".to_string()),
        applied_at: None,
    };

    let target = pending_dir.join(format!("{}.json", decision_event_id));
    let json_bytes = match serde_json::to_vec_pretty(&event) {
        Ok(bytes) => bytes,
        Err(err) => {
            return motoko_graph_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "SERIALIZATION_FAILED",
                "Failed to serialize decision event",
                Some(json!({ "reason": err.to_string() })),
            );
        }
    };
    if let Err(err) = fs::write(&target, json_bytes) {
        return motoko_graph_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "PENDING_WRITE_FAILED",
            "Unable to persist pending decision event",
            Some(json!({
                "path": target.display().to_string(),
                "reason": err.to_string()
            })),
        );
    }

    Json(DecisionCaptureResponse {
        decision_event_id,
        status: "pending".to_string(),
        path: target.display().to_string(),
    })
    .into_response()
}

fn dpub_duration_ms(started_at: &str, finished_at: Option<&String>) -> Option<u64> {
    let finished_at = finished_at?;
    let start = DateTime::parse_from_rfc3339(started_at).ok()?;
    let finish = DateTime::parse_from_rfc3339(finished_at).ok()?;
    let delta = finish
        .with_timezone(&Utc)
        .signed_duration_since(start.with_timezone(&Utc));
    if delta.num_milliseconds() < 0 {
        return None;
    }
    Some(delta.num_milliseconds() as u64)
}

fn dpub_execute_pipeline(
    request: DpubPipelineRunRequest,
    root: PathBuf,
    run_id: String,
) -> DpubPipelineRunReport {
    let mode = request.mode.trim().to_ascii_lowercase();
    let started_at = now_iso();
    let graph_path = root
        .join("research")
        .join("000-contribution-graph")
        .join("contribution_graph.json");
    let mut phase_results = Vec::<DpubPhaseResult>::new();
    let mut artifacts = serde_json::Map::<String, Value>::new();
    let mut report = DpubPipelineRunReport {
        run_id,
        mode: mode.clone(),
        status: "running".to_string(),
        started_at,
        finished_at: None,
        graph_root_hash_before: dpub_graph_hash_at(&graph_path),
        graph_root_hash_after: None,
        phase_results: Vec::new(),
        artifacts: Value::Object(serde_json::Map::new()),
        error: None,
    };

    let execution: Result<(), String> = (|| {
        if mode.is_empty() {
            return Err("mode is required".to_string());
        }

        let run_validate = |phase_results: &mut Vec<DpubPhaseResult>| -> Result<(), String> {
            let started = Instant::now();
            match validate_research_portfolio(&root) {
                Ok(_) => {
                    phase_results.push(dpub_phase_result(
                        "validate",
                        "ok",
                        started,
                        Some("metadata and portfolio checks passed".to_string()),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "validate",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("validate failed: {err:#}"))
                }
            }
        };

        let run_ingest = |phase_results: &mut Vec<DpubPhaseResult>,
                          artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let started = Instant::now();
            let graph = match ingest_and_write(&root) {
                Ok(graph) => graph,
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "ingest",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    return Err(format!("ingest failed: {err:#}"));
                }
            };
            artifacts.insert(
                "ingest".to_string(),
                json!({
                    "graphRootHash": graph.graph_root_hash,
                    "nodes": graph.nodes.len(),
                    "edges": graph.edges.len()
                }),
            );
            phase_results.push(dpub_phase_result(
                "ingest",
                "ok",
                started,
                Some("graph artifacts generated".to_string()),
            ));

            let started_determinism = Instant::now();
            let rerun = match ingest_and_write(&root) {
                Ok(graph) => graph,
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "determinism",
                        "error",
                        started_determinism,
                        Some(err.to_string()),
                    ));
                    return Err(format!("determinism rerun failed: {err:#}"));
                }
            };
            let first_hash = artifacts
                .get("ingest")
                .and_then(|v| v.get("graphRootHash"))
                .and_then(|v| v.as_str())
                .unwrap_or_default()
                .to_string();
            if first_hash != rerun.graph_root_hash {
                phase_results.push(dpub_phase_result(
                    "determinism",
                    "error",
                    started_determinism,
                    Some(format!(
                        "graph hash drift detected: first={} rerun={}",
                        first_hash, rerun.graph_root_hash
                    )),
                ));
                return Err(
                    "determinism check failed: graph hash changed on unchanged corpus".to_string(),
                );
            }
            phase_results.push(dpub_phase_result(
                "determinism",
                "ok",
                started_determinism,
                Some(format!("stable hash {}", rerun.graph_root_hash)),
            ));

            // Optional SIQ bridge intake (read-only): enrich pipeline artifacts without mutating SIQ state.
            if let Some(projection) = siq_load_projection_optional(None) {
                let gate_summary = siq_load_gate_summary_optional(None);
                artifacts.insert(
                    "siq_projection".to_string(),
                    json!({
                        "runId": projection.run_id,
                        "graphFingerprint": projection.graph_fingerprint,
                        "integritySetCount": projection.integrity_set.len(),
                        "edgeCount": projection.edges.len(),
                        "overallVerdict": gate_summary.map(|row| row.overall_verdict).unwrap_or_else(|| "unknown".to_string())
                    }),
                );
                phase_results.push(dpub_phase_result(
                    "siq_bridge",
                    "ok",
                    Instant::now(),
                    Some("optional SIQ projection consumed".to_string()),
                ));
            } else {
                phase_results.push(dpub_phase_result(
                    "siq_bridge",
                    "ok",
                    Instant::now(),
                    Some("optional SIQ projection unavailable; skipped".to_string()),
                ));
            }
            Ok(())
        };

        let run_doctor = |phase_results: &mut Vec<DpubPhaseResult>,
                          artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let started = Instant::now();
            match doctor(&root) {
                Ok(report) => {
                    artifacts.insert("doctor".to_string(), json!(report));
                    phase_results.push(dpub_phase_result(
                        "doctor",
                        "ok",
                        started,
                        Some("doctor report generated".to_string()),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "doctor",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("doctor failed: {err:#}"))
                }
            }
        };

        let run_path = |goal: &str,
                        phase_results: &mut Vec<DpubPhaseResult>,
                        artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let started = Instant::now();
            match assess_path(&root, goal) {
                Ok(path_assessment) => {
                    artifacts.insert(
                        format!("path:{}", goal),
                        json!({
                            "goal": goal,
                            "recommendedPath": path_assessment.recommended_path.name,
                            "riskScore": path_assessment.risk_score
                        }),
                    );
                    phase_results.push(dpub_phase_result(
                        &format!("path({goal})"),
                        "ok",
                        started,
                        Some("path assessment computed".to_string()),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        &format!("path({goal})"),
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("path assessment failed for {goal}: {err:#}"))
                }
            }
        };

        let run_simulate = |template_id: Option<&str>,
                            phase_results: &mut Vec<DpubPhaseResult>,
                            artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let scenario_path = dpub_scenario_path_from_template(&root, template_id);
            if !scenario_path.exists() {
                return Err(format!(
                    "scenario template not found: {}",
                    scenario_path.display()
                ));
            }
            let started = Instant::now();
            match simulate(&root, &scenario_path) {
                Ok(session) => {
                    artifacts.insert(
                        "simulate".to_string(),
                        json!({
                            "scenarioPath": scenario_path.display().to_string(),
                            "sessionId": session.session_id
                        }),
                    );
                    phase_results.push(dpub_phase_result(
                        "simulate",
                        "ok",
                        started,
                        Some(format!(
                            "scenario executed from {}",
                            scenario_path.display()
                        )),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "simulate",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("simulate failed: {err:#}"))
                }
            }
        };

        let run_publish = |version: Option<&str>,
                           phase_results: &mut Vec<DpubPhaseResult>,
                           artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let edition_version = version.unwrap_or("v0.2.0");
            let started = Instant::now();
            match publish_edition(&root, edition_version) {
                Ok(result) => {
                    artifacts.insert("publish".to_string(), json!(result));
                    phase_results.push(dpub_phase_result(
                        "publish",
                        "ok",
                        started,
                        Some(format!("published {}", edition_version)),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "publish",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("publish failed: {err:#}"))
                }
            }
        };

        let run_diff = |from: Option<&str>,
                        to: Option<&str>,
                        phase_results: &mut Vec<DpubPhaseResult>,
                        artifacts: &mut serde_json::Map<String, Value>|
         -> Result<(), String> {
            let from = from.ok_or_else(|| "fromVersion is required for diff mode".to_string())?;
            let to = to.ok_or_else(|| "toVersion is required for diff mode".to_string())?;
            let started = Instant::now();
            match diff_editions(&root, from, to) {
                Ok(result) => {
                    artifacts.insert("diff".to_string(), json!(result));
                    phase_results.push(dpub_phase_result(
                        "diff",
                        "ok",
                        started,
                        Some(format!("diffed {} -> {}", from, to)),
                    ));
                    Ok(())
                }
                Err(err) => {
                    phase_results.push(dpub_phase_result(
                        "diff",
                        "error",
                        started,
                        Some(err.to_string()),
                    ));
                    Err(format!("diff failed: {err:#}"))
                }
            }
        };

        match mode.as_str() {
            "validate" => run_validate(&mut phase_results)?,
            "ingest" => run_ingest(&mut phase_results, &mut artifacts)?,
            "doctor" => run_doctor(&mut phase_results, &mut artifacts)?,
            "path" => {
                let goal = request.goal.as_deref().unwrap_or("stable-cortex-domain");
                run_path(goal, &mut phase_results, &mut artifacts)?
            }
            "simulate" => run_simulate(
                request.scenario_template_id.as_deref(),
                &mut phase_results,
                &mut artifacts,
            )?,
            "publish" => run_publish(
                request.publish_version.as_deref(),
                &mut phase_results,
                &mut artifacts,
            )?,
            "diff" => run_diff(
                request.from_version.as_deref(),
                request.to_version.as_deref(),
                &mut phase_results,
                &mut artifacts,
            )?,
            "full" => {
                run_validate(&mut phase_results)?;
                run_ingest(&mut phase_results, &mut artifacts)?;
                run_doctor(&mut phase_results, &mut artifacts)?;
                run_path("stable-cortex-domain", &mut phase_results, &mut artifacts)?;
                run_path("accelerate-118", &mut phase_results, &mut artifacts)?;
                run_simulate(
                    request.scenario_template_id.as_deref(),
                    &mut phase_results,
                    &mut artifacts,
                )?;
                if request.publish_version.is_some() {
                    run_publish(
                        request.publish_version.as_deref(),
                        &mut phase_results,
                        &mut artifacts,
                    )?;
                }
                if request.from_version.is_some() || request.to_version.is_some() {
                    run_diff(
                        request.from_version.as_deref(),
                        request.to_version.as_deref(),
                        &mut phase_results,
                        &mut artifacts,
                    )?;
                }
            }
            other => return Err(format!("unsupported mode: {}", other)),
        }
        Ok(())
    })();

    report.finished_at = Some(now_iso());
    report.graph_root_hash_after = dpub_graph_hash_at(&graph_path);
    report.phase_results = phase_results;
    report.artifacts = Value::Object(artifacts);
    if let Err(err) = execution {
        report.status = "failed".to_string();
        report.error = Some(err);
    } else {
        report.status = "success".to_string();
    }
    report
}

async fn get_contribution_graph_overview(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let graph = dpub_load_graph(Some(&space_id)).ok();
    let path_bundle = dpub_load_path_bundle(Some(&space_id)).ok();
    let runs = dpub_read_run_records(Some(&space_id));
    let latest_run_summary = runs.first().map(dpub_run_history_item);
    let siq_projection = siq_load_projection_optional(Some(&space_id));
    let siq_summary = siq_load_gate_summary_optional(Some(&space_id));

    let health = json!({
        "status": if graph.is_some() { "ok" } else { "missing" },
        "workspaceRoot": dpub_workspace_root(Some(&space_id)).display().to_string(),
        "graphPath": dpub_graph_path(Some(&space_id)).display().to_string(),
        "pathAssessmentPath": dpub_path_assessment_path(Some(&space_id)).display().to_string(),
        "doctorPath": dpub_doctor_path(Some(&space_id)).display().to_string(),
        "doctorExists": dpub_doctor_path(Some(&space_id)).exists(),
        "simulationsDir": dpub_simulations_dir(Some(&space_id)).display().to_string(),
        "editionsDir": dpub_editions_dir(Some(&space_id)).display().to_string(),
        "runLogDir": dpub_run_log_dir(Some(&space_id)).display().to_string()
    });

    let latest_graph_metrics = if let Some(graph) = &graph {
        json!({
            "hash": graph.graph_root_hash,
            "nodes": graph.nodes.len(),
            "edges": graph.edges.len(),
            "critical": graph.integrity_report.counts.critical,
            "violation": graph.integrity_report.counts.violation,
            "warning": graph.integrity_report.counts.warning,
            "unresolvedRefs": graph.build_report.unresolved_references.len()
        })
    } else {
        json!({})
    };

    let latest_path_summary = if let Some(bundle) = &path_bundle {
        let goals = bundle
            .assessments
            .iter()
            .map(|a| {
                json!({
                    "goal": a.goal,
                    "recommendedPath": a.recommended_path.name,
                    "riskScore": a.risk_score
                })
            })
            .collect::<Vec<_>>();
        json!({ "goals": goals })
    } else {
        json!({})
    };

    let overview = DpubWorkbenchOverview {
        health,
        latest_graph_metrics,
        latest_path_summary,
        latest_run_summary,
        siq_run_id: siq_projection.as_ref().map(|row| row.run_id.clone()),
        siq_graph_fingerprint: siq_projection
            .as_ref()
            .map(|row| row.graph_fingerprint.clone()),
        siq_overall_verdict: siq_summary.as_ref().map(|row| row.overall_verdict.clone()),
        artifact_paths: json!({
            "graph": dpub_graph_path(Some(&space_id)).display().to_string(),
            "pathAssessment": dpub_path_assessment_path(Some(&space_id)).display().to_string(),
            "doctor": dpub_doctor_path(Some(&space_id)).display().to_string(),
            "simulationsDir": dpub_simulations_dir(Some(&space_id)).display().to_string(),
            "editionsDir": dpub_editions_dir(Some(&space_id)).display().to_string(),
            "siqProjection": siq_graph_projection_path(Some(&space_id)).display().to_string(),
            "siqGateSummary": siq_gate_summary_path(Some(&space_id)).display().to_string()
        }),
    };
    Json(overview).into_response()
}

async fn get_contribution_graph_graph(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => Json(graph).into_response(),
        Err(err) => err,
    }
}

async fn get_contribution_graph_path_assessment(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match dpub_load_path_bundle(Some(&space_id)) {
        Ok(bundle) => Json(bundle).into_response(),
        Err(err) => err,
    }
}

async fn get_contribution_graph_lens_summary(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => Json(dpub_lens_summary(&graph)).into_response(),
        Err(err) => err,
    }
}

async fn get_contribution_graph_edition_trends(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Query(query): Query<DpubEditionTrendQuery>,
) -> impl IntoResponse {
    let goal = query
        .goal
        .as_deref()
        .unwrap_or("stable-cortex-domain")
        .trim()
        .to_string();
    let window = query.window.unwrap_or(12).min(104);
    match dpub_edition_trends(Some(&space_id), &goal, window) {
        Ok(trends) => Json(trends).into_response(),
        Err(err) => dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_TREND_FAILED",
            "Unable to compute edition trends.",
            Some(json!({ "reason": err })),
        ),
    }
}

async fn get_contribution_graph_doctor(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    match dpub_load_doctor(Some(&space_id)) {
        Ok(report) => Json(report).into_response(),
        Err(err) => err,
    }
}

async fn get_contribution_graph_simulations(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let dir = dpub_simulations_dir(Some(&space_id));
    if !dir.exists() {
        return Json(Vec::<DpubSimulationArtifact>::new()).into_response();
    }

    let mut out = Vec::<DpubSimulationArtifact>::new();
    let entries = match fs::read_dir(&dir) {
        Ok(entries) => entries,
        Err(err) => {
            return dpub_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DPUB_SIMULATION_READ_FAILED",
                "Unable to list simulation artifacts.",
                Some(json!({ "path": dir.display().to_string(), "reason": err.to_string() })),
            );
        }
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let metadata = match entry.metadata() {
            Ok(meta) => meta,
            Err(_) => continue,
        };
        let modified_at = metadata
            .modified()
            .ok()
            .and_then(|ts| ts.duration_since(UNIX_EPOCH).ok())
            .map(|dur| timestamp_iso(dur.as_secs()))
            .flatten();
        out.push(DpubSimulationArtifact {
            file_name: path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or_default()
                .to_string(),
            bytes: metadata.len(),
            modified_at,
        });
    }

    out.sort_by(|a, b| a.file_name.cmp(&b.file_name));
    Json(out).into_response()
}

async fn get_contribution_graph_editions(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    Json(dpub_edition_entries(Some(&space_id))).into_response()
}

async fn get_contribution_graph_edition_diff(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Query(query): Query<DpubEditionDiffQuery>,
) -> impl IntoResponse {
    if query.from.trim().is_empty() || query.to.trim().is_empty() {
        return dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_DIFF_VERSION_REQUIRED",
            "from and to versions are required.",
            None,
        );
    }
    match diff_editions(
        &dpub_workspace_root(Some(&space_id)),
        &query.from,
        &query.to,
    ) {
        Ok(diff) => Json(diff).into_response(),
        Err(err) => dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_DIFF_FAILED",
            "Unable to compute edition diff.",
            Some(json!({ "reason": err.to_string() })),
        ),
    }
}

async fn get_contribution_graph_runs(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Query(query): Query<DpubRunHistoryQuery>,
) -> impl IntoResponse {
    let mut items = dpub_read_run_records(Some(&space_id))
        .iter()
        .map(dpub_run_history_item)
        .collect::<Vec<_>>();
    let limit = query.limit.unwrap_or(25).min(500);
    if items.len() > limit {
        items.truncate(limit);
    }
    Json(items).into_response()
}

async fn get_contribution_graph_run(
    Path((space_id, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    if run_id.contains('/') || run_id.contains('\\') || run_id.contains("..") {
        return dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_RUN_ID_INVALID",
            "run_id contains invalid path characters.",
            Some(json!({ "runId": run_id })),
        );
    }
    let path = dpub_run_log_dir(Some(&space_id)).join(format!("{}.json", run_id));
    match dpub_read_json::<DpubRunRecord>(&path) {
        Ok(record) => Json(dpub_to_report(&record)).into_response(),
        Err(err) => err,
    }
}

async fn post_contribution_graph_pipeline_query(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Json(payload): Json<DpubPipelineQueryRequest>,
) -> impl IntoResponse {
    if payload.kind.trim().is_empty() || payload.id.trim().is_empty() {
        return dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_QUERY_INVALID",
            "kind and id are required.",
            None,
        );
    }

    match query_graph(&dpub_managed_workspace_root(&space_id), &payload.kind, &payload.id) {
        Ok(result) => Json(result).into_response(),
        Err(err) => dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_QUERY_FAILED",
            "Unable to execute query.",
            Some(json!({ "reason": err.to_string() })),
        ),
    }
}

async fn post_contribution_graph_lens_evaluate(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Json(payload): Json<DpubLensEvaluateRequest>,
) -> impl IntoResponse {
    let graph = match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => graph,
        Err(err) => return err,
    };
    let summary = dpub_lens_summary(&graph);
    let lens_state = json!({
        "activeLenses": payload.active_lenses,
        "goal": payload.goal
    });
    let response = DpubLensOverlayResponse {
        graph_root_hash: graph.graph_root_hash.clone(),
        lens_state,
        node_flags: json!({}),
        edge_flags: json!({}),
        counts: serde_json::to_value(summary.lenses).unwrap_or_else(|_| json!([])),
    };
    Json(response).into_response()
}

async fn get_contribution_graph_violations_by_node(
    axum::extract::Path(space_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let graph = match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => graph,
        Err(err) => return err,
    };
    let mut by_node = BTreeMap::<String, Vec<Value>>::new();
    for violation in graph.integrity_report.violations {
        let severity = format!("{:?}", violation.severity).to_ascii_lowercase();
        for node_id in violation.affected_nodes {
            by_node.entry(node_id).or_default().push(json!({
                "ruleId": violation.rule_id,
                "severity": severity,
                "explanation": violation.explanation
            }));
        }
    }
    Json(json!({
        "graphRootHash": graph.graph_root_hash,
        "violationsByNode": by_node
    }))
    .into_response()
}

async fn post_contribution_graph_pipeline_run(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    headers: HeaderMap,
    Json(payload): Json<DpubPipelineRunRequest>,
) -> impl IntoResponse {
    if payload.mode.trim().is_empty() {
        return dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_MODE_REQUIRED",
            "mode is required.",
            None,
        );
    }

    let actor_role = actor_role_from_headers(&headers);
    let actor_id = actor_id_from_headers(&headers);
    let mutating = dpub_mode_is_mutating(&payload.mode);
    let approval = if mutating {
        if role_rank(&actor_role) < role_rank("steward") {
            return dpub_error(
                StatusCode::FORBIDDEN,
                "DPUB_MUTATION_STEWARD_REQUIRED",
                "Steward role is required for mutating pipeline modes.",
                Some(json!({ "actorRole": actor_role })),
            );
        }
        match dpub_require_approval(&payload.approval) {
            Ok(approval) => Some(approval),
            Err(err) => return err,
        }
    } else {
        None
    };

    let preferred_root = dpub_workspace_root(Some(&space_id));
    let root = if dpub_research_index_path(&preferred_root).exists() {
        preferred_root
    } else {
        dpub_workspace_root(None)
    };
    let run_id = dpub_run_id(&payload.mode);
    let request_for_exec = payload.clone();
    let root_for_exec = root.clone();
    let run_id_for_exec = run_id.clone();
    let report = match tokio::task::spawn_blocking(move || {
        dpub_execute_pipeline(request_for_exec, root_for_exec, run_id_for_exec)
    })
    .await
    {
        Ok(report) => report,
        Err(err) => {
            return dpub_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DPUB_PIPELINE_EXECUTION_JOIN_FAILED",
                "Pipeline worker join failed.",
                Some(json!({ "reason": err.to_string() })),
            );
        }
    };

    if report.status == "success" {
        if let Err(err) = dpub_sync_graph_outputs_to_space(&root, &space_id) {
            return dpub_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DPUB_SPACE_SYNC_FAILED",
                "Unable to sync contribution graph outputs to managed space.",
                Some(json!({ "reason": err, "spaceId": space_id })),
            );
        }
    }

    let run_record = DpubRunRecord {
        schema_version: "nostra.dpub.pipeline_run.v1".to_string(),
        run_id: report.run_id.clone(),
        mode: report.mode.clone(),
        actor_role: actor_role.clone(),
        actor_id: actor_id.clone(),
        started_at: report.started_at.clone(),
        finished_at: report.finished_at.clone(),
        status: report.status.clone(),
        duration_ms: dpub_duration_ms(&report.started_at, report.finished_at.as_ref()),
        graph_root_hash_before: report.graph_root_hash_before.clone(),
        graph_root_hash_after: report.graph_root_hash_after.clone(),
        phase_results: report.phase_results.clone(),
        artifacts: report.artifacts.clone(),
        error: report.error.clone(),
        approval,
    };

    if let Err(err) = dpub_persist_run_record(&run_record, Some(&space_id)) {
        return dpub_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DPUB_RUN_LOG_PERSIST_FAILED",
            "Unable to persist pipeline run record.",
            Some(json!({ "reason": err })),
        );
    }

    Json(report).into_response()
}

async fn get_contribution_graph_blast_radius(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    Query(query): Query<DpubBlastRadiusQuery>,
) -> impl IntoResponse {
    if query.contribution_id.trim().is_empty() {
        return dpub_error(
            StatusCode::BAD_REQUEST,
            "DPUB_BLAST_RADIUS_ID_REQUIRED",
            "contributionId is required.",
            None,
        );
    }

    let graph = match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => graph,
        Err(err) => return err,
    };
    let id = query.contribution_id.trim();
    let mut depends_on = BTreeSet::new();
    let mut depended_by = BTreeSet::new();
    let mut invalidates = BTreeSet::new();
    let mut invalidated_by = BTreeSet::new();
    let mut supersedes = BTreeSet::new();
    let mut superseded_by = BTreeSet::new();
    let mut references = BTreeSet::new();
    let mut referenced_by = BTreeSet::new();

    for edge in &graph.edges {
        match edge.edge_kind.as_str() {
            "depends_on" => {
                if edge.from == id {
                    depends_on.insert(edge.to.clone());
                }
                if edge.to == id {
                    depended_by.insert(edge.from.clone());
                }
            }
            "invalidates" => {
                if edge.from == id {
                    invalidates.insert(edge.to.clone());
                }
                if edge.to == id {
                    invalidated_by.insert(edge.from.clone());
                }
            }
            "supersedes" => {
                if edge.from == id {
                    supersedes.insert(edge.to.clone());
                }
                if edge.to == id {
                    superseded_by.insert(edge.from.clone());
                }
            }
            "references" => {
                if edge.from == id {
                    references.insert(edge.to.clone());
                }
                if edge.to == id {
                    referenced_by.insert(edge.from.clone());
                }
            }
            _ => {}
        }
    }

    Json(DpubBlastRadiusResponse {
        contribution_id: id.to_string(),
        depends_on: depends_on.into_iter().collect(),
        depended_by: depended_by.into_iter().collect(),
        invalidates: invalidates.into_iter().collect(),
        invalidated_by: invalidated_by.into_iter().collect(),
        supersedes: supersedes.into_iter().collect(),
        superseded_by: superseded_by.into_iter().collect(),
        references: references.into_iter().collect(),
        referenced_by: referenced_by.into_iter().collect(),
    })
    .into_response()
}

async fn post_contribution_graph_steward_packet_export(
    axum::extract::Path(space_id): axum::extract::Path<String>,
    headers: HeaderMap,
    Json(payload): Json<DpubStewardPacketExportRequest>,
) -> impl IntoResponse {
    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("steward") {
        return dpub_error(
            StatusCode::FORBIDDEN,
            "DPUB_PACKET_STEWARD_REQUIRED",
            "Steward role is required for steward packet export.",
            Some(json!({ "actorRole": actor_role })),
        );
    }
    let _approval = match dpub_require_approval(&payload.approval) {
        Ok(approval) => approval,
        Err(err) => return err,
    };

    let graph = match dpub_load_graph(Some(&space_id)) {
        Ok(graph) => graph,
        Err(err) => return err,
    };
    let path_bundle = match dpub_load_path_bundle(Some(&space_id)) {
        Ok(bundle) => bundle,
        Err(err) => return err,
    };

    let editions = dpub_edition_entries(Some(&space_id));
    let (from_version, to_version) = if let (Some(from), Some(to)) =
        (payload.from_version.clone(), payload.to_version.clone())
    {
        (from, to)
    } else if editions.len() >= 2 {
        (editions[1].version.clone(), editions[0].version.clone())
    } else {
        ("v0.1.0".to_string(), "v0.2.0".to_string())
    };
    let goal = payload
        .goal
        .as_deref()
        .unwrap_or("stable-cortex-domain")
        .to_string();

    let diff = match diff_editions(
        &dpub_managed_workspace_root(&space_id),
        &from_version,
        &to_version,
    ) {
        Ok(diff) => diff,
        Err(err) => {
            return dpub_error(
                StatusCode::BAD_REQUEST,
                "DPUB_PACKET_DIFF_FAILED",
                "Unable to compute edition diff for packet.",
                Some(json!({ "reason": err.to_string() })),
            );
        }
    };

    let markdown = dpub_markdown_packet(
        &goal,
        &from_version,
        &to_version,
        &diff,
        &path_bundle,
        &graph,
    );
    let file_name = format!(
        "packet_{}_{}.md",
        Utc::now().format("%Y%m%dT%H%M%SZ"),
        sanitize_fs_component(&goal)
    );
    let packet_path = dpub_steward_packet_dir(Some(&space_id)).join(file_name);
    if let Some(parent) = packet_path.parent() {
        if let Err(err) = fs::create_dir_all(parent) {
            return dpub_error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "DPUB_PACKET_DIR_CREATE_FAILED",
                "Unable to create steward packet directory.",
                Some(json!({ "reason": err.to_string() })),
            );
        }
    }
    if let Err(err) = fs::write(&packet_path, markdown.as_bytes()) {
        return dpub_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "DPUB_PACKET_WRITE_FAILED",
            "Unable to write steward packet.",
            Some(json!({ "reason": err.to_string() })),
        );
    }

    Json(DpubStewardPacketExportResponse {
        packet_path: packet_path.display().to_string(),
        goal,
        from_version,
        to_version,
    })
    .into_response()
}

async fn health_check() -> impl IntoResponse {
    Json(json!({ "status": "ok" })).into_response()
}

fn dfx_command() -> Command {
    let mut command = Command::new("dfx");
    let term = std::env::var("TERM").unwrap_or_default();
    if term.trim().is_empty() || term.eq_ignore_ascii_case("dumb") {
        command.env("TERM", "xterm-256color");
    }
    command.env("CLICOLOR", "1");
    command.env_remove("NO_COLOR");
    command
}

fn dfx_port_healthy() -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], 4943));
    std::net::TcpStream::connect_timeout(&addr, Duration::from_millis(350)).is_ok()
}

async fn get_system_ready() -> Json<SystemReady> {
    let mut notes = Vec::new();
    let (gateway_port, gateway_port_note) =
        crate::services::gateway_config::gateway_port_with_note();
    if let Some(note) = gateway_port_note {
        notes.push(note);
    }

    let dfx_port_healthy = dfx_port_healthy();
    if !dfx_port_healthy {
        notes.push("Local replica TCP probe failed on port 4943".to_string());
    }

    Json(SystemReady {
        ready: dfx_port_healthy,
        gateway_port,
        dfx_port_healthy,
        notes,
    })
}

fn system_build_id() -> String {
    option_env!("GIT_SHA")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn system_build_time() -> String {
    option_env!("BUILD_TIME_UTC")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn system_gateway_dispatch_mode() -> String {
    match crate::services::gateway_config::gateway_legacy_dispatch_mode() {
        cortex_runtime::GatewayLegacyDispatchMode::HttpLoopback => "http_loopback".to_string(),
        cortex_runtime::GatewayLegacyDispatchMode::InProcess => "in_process".to_string(),
    }
}

async fn get_system_build() -> Json<SystemBuild> {
    let (gateway_port, _) = crate::services::gateway_config::gateway_port_with_note();
    Json(SystemBuild {
        build_id: system_build_id(),
        build_time_utc: system_build_time(),
        gateway_dispatch_mode: system_gateway_dispatch_mode(),
        gateway_port,
        workspace_root: dpub_workspace_root(None).display().to_string(),
    })
}

fn route_node_id(route_id: &str) -> String {
    let token: String = route_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() {
                c.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();
    format!("route:{}", token.trim_matches('_'))
}

fn pattern_node_id(pattern_id: &str) -> String {
    format!("pattern:{pattern_id}")
}

fn intent_type_for_pattern(pattern_id: Option<&str>) -> String {
    match pattern_id {
        Some("pattern.workflow") => "execute".to_string(),
        Some("pattern.studio") | Some("pattern.artifacts") => "mutate".to_string(),
        Some("pattern.testing") | Some("pattern.system") => "monitor".to_string(),
        Some("pattern.spaces") => "configure".to_string(),
        Some(_) => "unspecified".to_string(),
        None => "navigate".to_string(),
    }
}

fn domain_color(domain: &str) -> String {
    match domain {
        "system" => "#38bdf8".to_string(),
        "pattern" => "#a78bfa".to_string(),
        "studio" => "#f59e0b".to_string(),
        "systems" | "system_ops" => "#0ea5e9".to_string(),
        "spaces" => "#14b8a6".to_string(),
        "artifacts" => "#f97316".to_string(),
        "workflows" => "#22c55e".to_string(),
        "testing" => "#ef4444".to_string(),
        _ => {
            let palette = [
                "#60a5fa", "#f472b6", "#34d399", "#facc15", "#fb7185", "#22d3ee",
            ];
            let idx = domain.bytes().fold(0usize, |acc, b| acc + usize::from(b)) % palette.len();
            palette[idx].to_string()
        }
    }
}

fn relationship_label(relationship: &str) -> String {
    match relationship {
        "contains" => "Contains".to_string(),
        "drill_down" => "Route Drill-Down".to_string(),
        "follows" => "Navigation Sequence".to_string(),
        _ => relationship.to_string(),
    }
}

fn relationship_rationale(relationship: &str) -> String {
    match relationship {
        "contains" => "Pattern-level grouping emitted by runtime shell contract.".to_string(),
        "drill_down" => {
            "Route binding emitted from matrix + navigation graph contracts.".to_string()
        }
        "follows" => "Deterministic ordering from canonical navigation entry sequence.".to_string(),
        _ => "Relationship emitted by runtime graph synthesis.".to_string(),
    }
}

fn relationship_policy_ref(relationship: &str) -> String {
    match relationship {
        "contains" => "policy:capability_graph.contains".to_string(),
        "drill_down" => "policy:capability_graph.drill_down".to_string(),
        "follows" => "policy:capability_graph.sequence".to_string(),
        _ => "policy:capability_graph.generic".to_string(),
    }
}

fn relationship_confidence(relationship: &str) -> u8 {
    match relationship {
        "contains" => 100,
        "drill_down" => 98,
        "follows" => 95,
        _ => 90,
    }
}

fn visibility_state(required_role: Option<&str>) -> String {
    if required_role.map(role_rank).unwrap_or(0) > role_rank("viewer") {
        "role_gated".to_string()
    } else {
        "visible".to_string()
    }
}

fn locked_reason(required_role: Option<&str>) -> Option<String> {
    let role = required_role.unwrap_or("viewer");
    if role_rank(role) > role_rank("viewer") {
        Some(format!("Requires {} role", role))
    } else {
        None
    }
}

fn priority_from_metadata(operator_critical: bool, promotion_status: Option<&str>) -> String {
    if operator_critical {
        "high".to_string()
    } else if matches!(promotion_status, Some("production")) {
        "high".to_string()
    } else if matches!(promotion_status, Some("candidate")) {
        "medium".to_string()
    } else {
        "normal".to_string()
    }
}

fn domain_intent_type_for_pattern(pattern_id: Option<&str>) -> DomainIntentType {
    let intent = intent_type_for_pattern(pattern_id);
    match intent.as_str() {
        "monitor" => DomainIntentType::Monitor,
        "execute" => DomainIntentType::Execute,
        "mutate" => DomainIntentType::Mutate,
        "configure" => DomainIntentType::Configure,
        "navigate" => DomainIntentType::Visualize,
        _ => DomainIntentType::Unspecified,
    }
}

fn default_surfacing_for_category(category: &str) -> SurfacingHeuristic {
    match category {
        "core" => SurfacingHeuristic::PrimaryCore,
        "secondary" => SurfacingHeuristic::Secondary,
        "bridge" => SurfacingHeuristic::Secondary,
        _ => SurfacingHeuristic::Secondary,
    }
}

fn default_operational_frequency_for_route(route: &str) -> OperationalFrequency {
    match route {
        "/heap" | "/logs" | "/metrics" | "/inbox" => OperationalFrequency::Continuous,
        "/spaces" | "/workflows" | "/agents" | "/discovery" | "/memory" => {
            OperationalFrequency::Daily
        }
        "/settings" | "/labs" | "/system" => OperationalFrequency::Rare,
        _ => OperationalFrequency::AdHoc,
    }
}

fn build_platform_capability_catalog() -> PlatformCapabilityCatalog {
    let layout_spec = resolve_shell_layout_spec();
    let capabilities = resolve_view_capability_manifests();
    let capability_by_route: BTreeMap<String, ViewCapabilityManifest> = capabilities
        .iter()
        .map(|item| (item.route_id.clone(), item.clone()))
        .collect();

    let mut catalog = PlatformCapabilityCatalog::new();
    let root_id = DomainCapabilityId("cortex.workbench.root".to_string());
    catalog.unverified_add_node(DomainCapabilityNode {
        id: root_id.clone(),
        name: "Cortex Workbench".to_string(),
        description: "Canonical Cortex capability catalog root".to_string(),
        intent_type: DomainIntentType::Monitor,
        route_id: None,
        category: Some("system".to_string()),
        required_role: Some("viewer".to_string()),
        icon: None,
        surfacing_heuristic: SurfacingHeuristic::Hidden,
        operational_frequency: OperationalFrequency::Continuous,
        domain_entities: vec![],
        placement_constraint: None,
        root_path: None,
        invariant_violations: vec![],
    });

    for entry in layout_spec.navigation_graph.entries.iter() {
        let capability = capability_by_route.get(&entry.route_id);
        let pattern_id = capability.map(|item| item.pattern_id.as_str());
        let node_id = DomainCapabilityId(format!("route:{}", entry.route_id));
        catalog.unverified_add_node(DomainCapabilityNode {
            id: node_id.clone(),
            name: capability
                .map(|item| item.route_label.clone())
                .unwrap_or_else(|| entry.label.clone()),
            description: capability
                .map(|item| item.description.clone())
                .unwrap_or_else(|| format!("Capability route {}", entry.route_id)),
            intent_type: domain_intent_type_for_pattern(pattern_id),
            route_id: Some(entry.route_id.clone()),
            category: Some(entry.category.clone()),
            required_role: Some(entry.required_role.clone()),
            icon: Some(entry.icon.clone()),
            surfacing_heuristic: default_surfacing_for_category(&entry.category),
            operational_frequency: default_operational_frequency_for_route(&entry.route_id),
            domain_entities: vec![],
            placement_constraint: None,
            root_path: None,
            invariant_violations: vec![],
        });
        catalog.unverified_add_edge(DomainCapabilityEdge {
            source: root_id.clone(),
            target: node_id,
            relationship: DomainEdgeRelationship::ChildOf,
        });
    }

    let catalog_hash = hash_json_hex(&json!({
        "schemaVersion": catalog.schema_version,
        "nodes": &catalog.nodes,
        "edges": &catalog.edges,
    }));
    catalog.catalog_version = format!(
        "v1-{}",
        catalog_hash.chars().take(12).collect::<String>()
    );
    catalog.catalog_hash = Some(catalog_hash);
    catalog
}

fn space_capability_graph_path(space_id: &str) -> PathBuf {
    workspace_root()
        .join("_spaces")
        .join(space_id)
        .join("capability_graph.json")
}

fn space_capability_graph_uri(space_id: &str) -> String {
    format!("_spaces/{space_id}/capability_graph.json")
}

fn default_space_capability_graph(
    space_id: &str,
    catalog: &PlatformCapabilityCatalog,
) -> SpaceCapabilityGraph {
    let base_catalog_hash = catalog
        .catalog_hash
        .clone()
        .unwrap_or_else(|| hash_json_hex(catalog));
    SpaceCapabilityGraph {
        schema_version: "1.0.0".to_string(),
        space_id: space_id.to_string(),
        base_catalog_version: catalog.catalog_version.clone(),
        base_catalog_hash,
        nodes: catalog
            .nodes
            .iter()
            .map(|node| SpaceCapabilityNodeOverride {
                capability_id: node.id.clone(),
                local_alias: None,
                is_active: true,
                local_required_role: None,
                surfacing_heuristic: None,
                operational_frequency: None,
                placement_constraint: None,
            })
            .collect(),
        edges: catalog.edges.clone(),
        updated_at: now_iso(),
        updated_by: "system".to_string(),
        lineage_ref: Some("bootstrap:space-capability-graph".to_string()),
    }
}

fn write_space_capability_graph(space_id: &str, graph: &SpaceCapabilityGraph) -> Result<(), String> {
    let path = space_capability_graph_path(space_id);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|err| format!("failed to create capability graph directory: {err}"))?;
    }
    let encoded = serde_json::to_string_pretty(graph)
        .map_err(|err| format!("failed to serialize capability graph: {err}"))?;
    fs::write(path, encoded).map_err(|err| format!("failed to write capability graph: {err}"))
}

fn read_space_capability_graph(space_id: &str) -> Result<Option<SpaceCapabilityGraph>, String> {
    let path = space_capability_graph_path(space_id);
    if !path.exists() {
        return Ok(None);
    }
    let raw = fs::read_to_string(&path).map_err(|err| format!("failed to read capability graph: {err}"))?;
    let parsed = serde_json::from_str::<SpaceCapabilityGraph>(&raw)
        .map_err(|err| format!("failed to parse capability graph: {err}"))?;
    Ok(Some(parsed))
}

fn load_or_initialize_space_capability_graph(
    space_id: &str,
    catalog: &PlatformCapabilityCatalog,
) -> Result<SpaceCapabilityGraph, String> {
    if let Some(graph) = read_space_capability_graph(space_id)? {
        return Ok(graph);
    }
    let graph = default_space_capability_graph(space_id, catalog);
    write_space_capability_graph(space_id, &graph)?;
    Ok(graph)
}

async fn get_system_capability_catalog() -> impl IntoResponse {
    Json(build_platform_capability_catalog())
}

async fn get_space_capability_graph(Path(space_id): Path<String>) -> impl IntoResponse {
    let normalized = space_id.trim();
    if normalized.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "space_id is required" })),
        )
            .into_response();
    }
    let catalog = build_platform_capability_catalog();
    match load_or_initialize_space_capability_graph(normalized, &catalog) {
        Ok(graph) => (StatusCode::OK, Json(graph)).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": err })),
        )
            .into_response(),
    }
}

async fn put_space_capability_graph(
    Path(space_id): Path<String>,
    headers: HeaderMap,
    Json(mut payload): Json<SpaceCapabilityGraph>,
) -> impl IntoResponse {
    let normalized = space_id.trim();
    if normalized.is_empty() {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SPACE_ID_REQUIRED",
            "space_id is required",
            None,
        );
    }

    let actor_role = actor_role_from_headers(&headers);
    if role_rank(&actor_role) < role_rank("steward") {
        return cortex_ux_error(
            StatusCode::FORBIDDEN,
            "STEWARD_ROLE_REQUIRED",
            "Steward role is required for structural capability graph updates.",
            Some(json!({ "actorRole": actor_role })),
        );
    }

    if payload.space_id.trim().is_empty() {
        payload.space_id = normalized.to_string();
    }
    if payload.space_id != normalized {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "SPACE_ID_MISMATCH",
            "space_id path and payload must match",
            Some(json!({ "pathSpaceId": normalized, "payloadSpaceId": payload.space_id })),
        );
    }

    if payload
        .lineage_ref
        .as_deref()
        .map(str::trim)
        .unwrap_or("")
        .is_empty()
    {
        return cortex_ux_error(
            StatusCode::BAD_REQUEST,
            "LINEAGE_REF_REQUIRED",
            "lineage_ref is required for steward structural updates.",
            None,
        );
    }

    let catalog = build_platform_capability_catalog();
    if payload.base_catalog_version.trim().is_empty() {
        payload.base_catalog_version = catalog.catalog_version.clone();
    }
    if payload.base_catalog_hash.trim().is_empty() {
        payload.base_catalog_hash = catalog
            .catalog_hash
            .clone()
            .unwrap_or_else(|| hash_json_hex(&catalog));
    }
    payload.updated_at = now_iso();
    payload.updated_by = actor_id_from_headers(&headers);

    if let Err(err) = write_space_capability_graph(normalized, &payload) {
        return cortex_ux_error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "CAPABILITY_GRAPH_WRITE_FAILED",
            "Failed to persist capability graph.",
            Some(json!({ "reason": err })),
        );
    }

    let capability_graph_hash = hash_json_hex(&payload);
    let registry_path = workspace_root().join("_spaces").join("registry.json");
    let mut registry =
        cortex_domain::spaces::SpaceRegistry::load_from_path(&registry_path).unwrap_or_default();
    if let Some(record) = registry.spaces.get_mut(normalized) {
        record.capability_graph_uri = Some(space_capability_graph_uri(normalized));
        record.capability_graph_version = Some(payload.base_catalog_version.clone());
        record.capability_graph_hash = Some(capability_graph_hash.clone());
        let _ = registry.save_to_path(&registry_path);
    }

    (
        StatusCode::OK,
        Json(SpaceCapabilityGraphUpsertResponse {
            accepted: true,
            space_id: normalized.to_string(),
            capability_graph_hash,
            capability_graph_version: payload.base_catalog_version,
            stored_at: payload.updated_at,
        }),
    )
        .into_response()
}

async fn get_space_navigation_plan(
    Path(space_id): Path<String>,
    Query(query): Query<SpaceNavigationPlanQuery>,
) -> impl IntoResponse {
    let normalized = space_id.trim();
    if normalized.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "space_id is required" })),
        )
            .into_response();
    }
    let catalog = build_platform_capability_catalog();
    let graph = match load_or_initialize_space_capability_graph(normalized, &catalog) {
        Ok(graph) => graph,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": err })),
            )
                .into_response();
        }
    };
    let context = CompilationContext {
        space_id: normalized.to_string(),
        actor_role: query.actor_role.unwrap_or_else(|| "operator".to_string()),
        intent: query.intent,
        density: query.density,
    };
    let layout_spec = resolve_shell_layout_spec();
    let plan = compile_navigation_plan(&catalog, &graph, &layout_spec, &context);
    (StatusCode::OK, Json(plan)).into_response()
}

async fn get_system_capability_graph() -> impl IntoResponse {
    let layout_spec = resolve_shell_layout_spec();
    let capabilities = resolve_view_capability_manifests();
    let pattern_contracts = resolve_pattern_contracts();
    let matrix = resolve_capability_matrix();
    let source_state = cortex_ux_source_state();

    let capability_by_route: BTreeMap<String, ViewCapabilityManifest> = capabilities
        .iter()
        .map(|item| (item.route_id.clone(), item.clone()))
        .collect();
    let pattern_by_id: BTreeMap<String, crate::services::cortex_ux::PatternContract> =
        pattern_contracts
            .iter()
            .map(|item| (item.pattern_id.clone(), item.clone()))
            .collect();
    let matrix_by_route: BTreeMap<String, ViewCapabilityMatrixRow> = matrix
        .iter()
        .map(|item| (item.route_id.clone(), item.clone()))
        .collect();
    let nav_entry_by_route: BTreeMap<String, cortex_domain::ux::types::NavigationEntrySpec> =
        layout_spec
            .navigation_graph
            .entries
            .iter()
            .map(|item| (item.route_id.clone(), item.clone()))
            .collect();
    let capability_signature = hash_json_hex(&json!({
        "layout_entries": &layout_spec.navigation_graph.entries,
        "capabilities": &capabilities,
        "patterns": &pattern_contracts,
        "matrix": &matrix,
    }));
    let capabilities_version = format!(
        "v1-{}",
        capability_signature.chars().take(12).collect::<String>()
    );

    let mut nodes = vec![SystemCapabilityGraphNode {
        id: "cortex.workbench.root".to_string(),
        title: "Cortex Workbench".to_string(),
        description: "Canonical Cortex web navigation root".to_string(),
        intent_type: "monitor".to_string(),
        route_id: None,
        required_role: Some("viewer".to_string()),
        pattern_id: None,
        promotion_status: None,
        invariant_violations: None,
        cluster_key: Some("domain:system".to_string()),
        domain: Some("system".to_string()),
        locked_reason: None,
        visibility_state: Some("visible".to_string()),
        health: Some("healthy".to_string()),
        priority: Some("high".to_string()),
        inspector: Some(SystemCapabilityNodeInspector {
            route_id: None,
            category: Some("system".to_string()),
            pattern_label: None,
            required_role: Some("viewer".to_string()),
            required_role_rank: Some(role_rank("viewer") as u8),
            operator_critical: Some(true),
            approval_required: Some(false),
            promotion_status: Some("production".to_string()),
        }),
    }];

    for pattern in pattern_contracts.iter() {
        let required_role = Some(pattern.required_role.clone());
        nodes.push(SystemCapabilityGraphNode {
            id: pattern_node_id(&pattern.pattern_id),
            title: pattern.label.clone(),
            description: pattern.description.clone(),
            intent_type: "configure".to_string(),
            route_id: None,
            required_role: required_role.clone(),
            pattern_id: Some(pattern.pattern_id.clone()),
            promotion_status: None,
            invariant_violations: None,
            cluster_key: Some("domain:pattern".to_string()),
            domain: Some("pattern".to_string()),
            locked_reason: locked_reason(required_role.as_deref()),
            visibility_state: Some(visibility_state(required_role.as_deref())),
            health: Some("healthy".to_string()),
            priority: Some("normal".to_string()),
            inspector: Some(SystemCapabilityNodeInspector {
                route_id: None,
                category: Some("pattern".to_string()),
                pattern_label: Some(pattern.label.clone()),
                required_role: required_role.clone(),
                required_role_rank: Some(role_rank(&pattern.required_role) as u8),
                operator_critical: Some(false),
                approval_required: Some(false),
                promotion_status: None,
            }),
        });
    }

    let mut route_ids_seen = BTreeSet::new();
    for entry in layout_spec.navigation_graph.entries.iter() {
        let row = matrix_by_route.get(&entry.route_id);
        let capability = capability_by_route.get(&entry.route_id);
        let pattern_id = row
            .map(|item| item.pattern_id.clone())
            .or_else(|| capability.map(|item| item.pattern_id.clone()));
        let promotion_status = row
            .map(|item| item.promotion_status.clone())
            .or_else(|| capability.map(|item| item.promotion_status.clone()));
        let required_role = Some(
            row.map(|item| item.required_role.clone())
                .unwrap_or_else(|| entry.required_role.clone()),
        );
        let operator_critical = capability
            .map(|item| item.operator_critical)
            .unwrap_or(false);
        let approval_required = row
            .map(|item| item.approval_required)
            .or_else(|| capability.map(|item| item.approval_required))
            .unwrap_or(false);
        let pattern_label = pattern_id
            .as_ref()
            .and_then(|id| pattern_by_id.get(id))
            .map(|pattern| pattern.label.clone());
        nodes.push(SystemCapabilityGraphNode {
            id: route_node_id(&entry.route_id),
            title: capability
                .map(|item| item.route_label.clone())
                .unwrap_or_else(|| entry.label.clone()),
            description: capability
                .map(|item| item.description.clone())
                .unwrap_or_else(|| format!("Navigation route {}", entry.route_id)),
            intent_type: intent_type_for_pattern(
                row.map(|item| item.pattern_id.as_str())
                    .or_else(|| capability.map(|item| item.pattern_id.as_str())),
            ),
            route_id: Some(entry.route_id.clone()),
            required_role: required_role.clone(),
            pattern_id: pattern_id.clone(),
            promotion_status: promotion_status.clone(),
            invariant_violations: None,
            cluster_key: Some(format!("domain:{}", entry.category)),
            domain: Some(entry.category.clone()),
            locked_reason: locked_reason(required_role.as_deref()),
            visibility_state: Some(visibility_state(required_role.as_deref())),
            health: Some("healthy".to_string()),
            priority: Some(priority_from_metadata(
                operator_critical,
                promotion_status.as_deref(),
            )),
            inspector: Some(SystemCapabilityNodeInspector {
                route_id: Some(entry.route_id.clone()),
                category: Some(entry.category.clone()),
                pattern_label,
                required_role: required_role.clone(),
                required_role_rank: required_role.as_ref().map(|role| role_rank(role) as u8),
                operator_critical: Some(operator_critical),
                approval_required: Some(approval_required),
                promotion_status,
            }),
        });
        route_ids_seen.insert(entry.route_id.clone());
    }

    for row in matrix.iter() {
        if route_ids_seen.contains(&row.route_id) {
            continue;
        }
        let capability = capability_by_route.get(&row.route_id);
        let domain = nav_entry_by_route
            .get(&row.route_id)
            .map(|entry| entry.category.clone())
            .unwrap_or_else(|| "matrix_only".to_string());
        let required_role = Some(row.required_role.clone());
        let operator_critical = capability
            .map(|item| item.operator_critical)
            .unwrap_or(false);
        let pattern_label = pattern_by_id
            .get(&row.pattern_id)
            .map(|pattern| pattern.label.clone());
        nodes.push(SystemCapabilityGraphNode {
            id: route_node_id(&row.route_id),
            title: capability
                .map(|item| item.route_label.clone())
                .unwrap_or_else(|| row.route_id.clone()),
            description: capability
                .map(|item| item.description.clone())
                .unwrap_or_else(|| format!("Capability route {}", row.route_id)),
            intent_type: intent_type_for_pattern(Some(row.pattern_id.as_str())),
            route_id: Some(row.route_id.clone()),
            required_role: required_role.clone(),
            pattern_id: Some(row.pattern_id.clone()),
            promotion_status: Some(row.promotion_status.clone()),
            invariant_violations: None,
            cluster_key: Some(format!("domain:{domain}")),
            domain: Some(domain.clone()),
            locked_reason: locked_reason(required_role.as_deref()),
            visibility_state: Some(visibility_state(required_role.as_deref())),
            health: Some("healthy".to_string()),
            priority: Some(priority_from_metadata(
                operator_critical,
                Some(row.promotion_status.as_str()),
            )),
            inspector: Some(SystemCapabilityNodeInspector {
                route_id: Some(row.route_id.clone()),
                category: Some(domain),
                pattern_label,
                required_role: required_role.clone(),
                required_role_rank: Some(role_rank(&row.required_role) as u8),
                operator_critical: Some(operator_critical),
                approval_required: Some(row.approval_required),
                promotion_status: Some(row.promotion_status.clone()),
            }),
        });
    }

    let mut edge_set: BTreeSet<SystemCapabilityGraphEdge> = BTreeSet::new();
    for pattern in pattern_contracts.iter() {
        edge_set.insert(SystemCapabilityGraphEdge {
            from: "cortex.workbench.root".to_string(),
            to: pattern_node_id(&pattern.pattern_id),
            relationship: "contains".to_string(),
            relationship_label: Some(relationship_label("contains")),
            confidence: Some(relationship_confidence("contains")),
            policy_ref: Some(relationship_policy_ref("contains")),
            rationale: Some(relationship_rationale("contains")),
            directionality: Some("directed".to_string()),
        });
    }

    for route_node in nodes.iter().filter(|node| node.route_id.is_some()) {
        let route_id = route_node.route_id.as_ref().expect("route id missing");
        let pattern_id = route_node
            .pattern_id
            .as_ref()
            .filter(|pattern_id| pattern_by_id.contains_key(*pattern_id));
        let from = pattern_id
            .map(|pattern| pattern_node_id(pattern))
            .unwrap_or_else(|| "cortex.workbench.root".to_string());
        edge_set.insert(SystemCapabilityGraphEdge {
            from,
            to: route_node_id(route_id),
            relationship: "drill_down".to_string(),
            relationship_label: Some(relationship_label("drill_down")),
            confidence: Some(relationship_confidence("drill_down")),
            policy_ref: Some(relationship_policy_ref("drill_down")),
            rationale: Some(relationship_rationale("drill_down")),
            directionality: Some("directed".to_string()),
        });
    }

    let nav_entries = &layout_spec.navigation_graph.entries;
    for pair in nav_entries.windows(2) {
        if let [left, right] = pair {
            edge_set.insert(SystemCapabilityGraphEdge {
                from: route_node_id(&left.route_id),
                to: route_node_id(&right.route_id),
                relationship: "follows".to_string(),
                relationship_label: Some(relationship_label("follows")),
                confidence: Some(relationship_confidence("follows")),
                policy_ref: Some(relationship_policy_ref("follows")),
                rationale: Some(relationship_rationale("follows")),
                directionality: Some("directed".to_string()),
            });
        }
    }

    nodes.sort_by(|left, right| left.id.cmp(&right.id));
    let edges: Vec<SystemCapabilityGraphEdge> = edge_set.into_iter().collect();
    let mut seen_domains = BTreeSet::new();
    let mut ordered_domains: Vec<String> = vec!["system".to_string(), "pattern".to_string()];
    for domain in ordered_domains.iter() {
        seen_domains.insert(domain.clone());
    }
    for entry in layout_spec.navigation_graph.entries.iter() {
        if seen_domains.insert(entry.category.clone()) {
            ordered_domains.push(entry.category.clone());
        }
    }
    for node in nodes.iter() {
        if let Some(domain) = node.domain.as_ref() {
            if seen_domains.insert(domain.clone()) {
                ordered_domains.push(domain.clone());
            }
        }
    }
    let groups: Vec<SystemCapabilityGraphLayoutGroup> = ordered_domains
        .iter()
        .enumerate()
        .map(|(index, domain)| SystemCapabilityGraphLayoutGroup {
            key: format!("domain:{domain}"),
            label: domain.replace('_', " "),
            order: index,
            color: domain_color(domain),
        })
        .collect();
    let layout_hints = SystemCapabilityGraphLayoutHints {
        engine: "react_flow".to_string(),
        seed: "capability-graph-v2".to_string(),
        cluster_by: "domain".to_string(),
        groups,
    };
    let mut intent_type_colors = BTreeMap::new();
    intent_type_colors.insert("configure".to_string(), "#f59e0b".to_string());
    intent_type_colors.insert("execute".to_string(), "#22c55e".to_string());
    intent_type_colors.insert("monitor".to_string(), "#0ea5e9".to_string());
    intent_type_colors.insert("mutate".to_string(), "#f97316".to_string());
    intent_type_colors.insert("navigate".to_string(), "#a78bfa".to_string());
    intent_type_colors.insert("unspecified".to_string(), "#94a3b8".to_string());
    let mut relationship_styles = BTreeMap::new();
    relationship_styles.insert("contains".to_string(), "solid".to_string());
    relationship_styles.insert("drill_down".to_string(), "solid-arrow".to_string());
    relationship_styles.insert("follows".to_string(), "dashed-arrow".to_string());
    let legend = SystemCapabilityGraphLegend {
        intent_type_colors,
        relationship_styles,
        lock_semantics: "role_rank(required_role) > actor_role_rank".to_string(),
    };
    let graph_hash = hash_json_hex(&json!({
        "schema_version": "1.1.0",
        "source_of_truth": &source_state.source_of_truth,
        "capabilities_version": &capabilities_version,
        "layout_hints": &layout_hints,
        "legend": &legend,
        "nodes": &nodes,
        "edges": &edges,
    }));

    let response = SystemCapabilityGraphResponse {
        schema_version: "1.1.0".to_string(),
        generated_at: now_iso(),
        source_of_truth: source_state.source_of_truth,
        graph_hash: Some(graph_hash),
        layout_hints: Some(layout_hints),
        legend: Some(legend),
        capabilities_version: Some(capabilities_version),
        nodes,
        edges,
    };

    (StatusCode::OK, Json(response)).into_response()
}

async fn get_system_brand_policy() -> axum::response::Response {
    let server_now = Utc::now();
    let cache_path = brand_policy_cache_path();
    let canonical = fetch_canonical_brand_policy_bundle().await;

    if brand_policy_canonical_only_enabled() && canonical.is_none() {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({
                "error": "Canonical-only mode is enabled and brand policy could not be loaded from canister",
                "errorCode": "CANONICAL_SOURCE_REQUIRED",
                "details": {
                    "canister": "brand_policy_registry"
                }
            })),
        )
            .into_response();
    }

    let (bundle, source_of_truth, degraded_reason) = if let Some(bundle) = canonical {
        (bundle, "canister".to_string(), None)
    } else if let Some(cached) = load_brand_policy_cache(&cache_path) {
        (
            cached,
            "cache".to_string(),
            Some("brand_policy_registry_unavailable".to_string()),
        )
    } else {
        let fallback_policy =
            match serde_json::from_str::<BrandPolicyDocument>(EMBEDDED_BRAND_POLICY_JSON) {
                Ok(policy) => policy,
                Err(err) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({
                            "error": "Failed to load fallback brand policy artifact",
                            "errorCode": "BRAND_POLICY_FALLBACK_INVALID",
                            "details": {
                                "reason": err.to_string()
                            }
                        })),
                    )
                        .into_response();
                }
            };

        (
            BrandPolicyCacheRecord {
                policy_version: fallback_policy.policy_version,
                policy_digest: hash_json_hex(&fallback_policy),
                policy: fallback_policy,
            },
            "fallback".to_string(),
            Some("no_canonical_or_cached_brand_policy".to_string()),
        )
    };

    let (normalized_policy, defaults_applied) = normalize_brand_policy_document(&bundle.policy);
    let normalized_bundle = BrandPolicyCacheRecord {
        policy: normalized_policy.clone(),
        policy_version: bundle.policy_version,
        policy_digest: bundle.policy_digest.clone(),
    };

    if source_of_truth == "canister" || source_of_truth == "cache" {
        if let Ok(value) = serde_json::to_value(&normalized_bundle) {
            let _ = persist_json(&cache_path, &value);
        }
    }

    let response = SystemBrandPolicyResponse {
        active_temporal_state: resolve_active_temporal_state(&normalized_policy, &server_now),
        server_time_utc: server_now.to_rfc3339(),
        policy: normalized_policy,
        policy_version: normalized_bundle.policy_version,
        policy_digest: normalized_bundle.policy_digest,
        source_of_truth,
        degraded_reason,
        policy_normalization: Some(if defaults_applied {
            "legacy_defaults_applied".to_string()
        } else {
            "none".to_string()
        }),
    };
    Json(response).into_response()
}

async fn post_create_space(Json(payload): Json<serde_json::Value>) -> impl IntoResponse {
    let space_id = payload
        .get("space_id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let creation_mode_str = payload
        .get("creation_mode")
        .and_then(|v| v.as_str())
        .unwrap_or("blank");
    let owner = payload
        .get("owner")
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let reference_uri = payload
        .get("reference_uri")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    if space_id.is_empty() {
        return (
            axum::http::StatusCode::BAD_REQUEST,
            Json(serde_json::json!({ "error": "space_id is required" })),
        )
            .into_response();
    }

    let creation_mode = match creation_mode_str {
        "import" => cortex_domain::spaces::CreationMode::Import,
        "template" => cortex_domain::spaces::CreationMode::Template,
        _ => cortex_domain::spaces::CreationMode::Blank,
    };

    let initial_status = if matches!(creation_mode, cortex_domain::spaces::CreationMode::Import) {
        cortex_domain::spaces::SpaceStatus::Quarantine
    } else {
        cortex_domain::spaces::SpaceStatus::Active
    };

    let template_id = payload
        .get("template_id")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string());

    let catalog = build_platform_capability_catalog();
    let initial_graph = default_space_capability_graph(&space_id, &catalog);
    let graph_hash = hash_json_hex(&initial_graph);
    let graph_uri = space_capability_graph_uri(&space_id);

    let record = cortex_domain::spaces::SpaceRecord {
        space_id: space_id.clone(),
        creation_mode,
        status: initial_status,
        reference_uri,
        template_id,
        capability_graph_uri: Some(graph_uri.clone()),
        capability_graph_version: Some(initial_graph.base_catalog_version.clone()),
        capability_graph_hash: Some(graph_hash.clone()),
        owner,
        created_at: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| format!("{}", d.as_secs()))
            .unwrap_or_else(|_| "0".to_string()),
    };

    let registry_path = workspace_root().join("_spaces").join("registry.json");
    let mut registry =
        cortex_domain::spaces::SpaceRegistry::load_from_path(&registry_path).unwrap_or_default();
    registry.upsert(record.clone());

    // Create the managed VFS directory structure
    let space_dir = workspace_root().join("_spaces").join(&space_id);
    if let Err(err) = std::fs::create_dir_all(&space_dir) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to create space directory: {}", err) })),
        ).into_response();
    }

    if let Err(err) = write_space_capability_graph(&space_id, &initial_graph) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to write space capability graph: {}", err) })),
        )
            .into_response();
    }

    if let Err(err) = registry.save_to_path(&registry_path) {
        return (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({ "error": format!("Failed to save registry: {}", err) })),
        )
            .into_response();
    }

    tracing::info!("Space '{}' provisioned successfully", space_id);
    (
        axum::http::StatusCode::CREATED,
        Json(serde_json::json!({
            "space_id": space_id,
            "status": format!("{:?}", record.status),
            "message": format!("Space '{}' provisioned successfully", space_id),
        })),
    )
        .into_response()
}

async fn get_system_status() -> Json<SystemStatus> {
    let dfx_running = dfx_port_healthy();

    let version_output = dfx_command()
        .arg("--version")
        .output()
        .ok()
        .map(|o| String::from_utf8_lossy(&o.stdout).to_string())
        .unwrap_or_else(|| "Unknown".to_string());

    Json(SystemStatus {
        dfx_running,
        version: version_output.trim().to_string(),
        replica_port: 4943, // Default local port
    })
}

async fn fetch_canonical_brand_policy_bundle() -> Option<BrandPolicyCacheRecord> {
    let service = BrandPolicyRegistryService::from_env().ok()?;
    let BrandPolicyBundle {
        policy,
        policy_version,
        policy_digest,
    } = service.get_brand_policy_bundle().await.ok()?;
    Some(BrandPolicyCacheRecord {
        policy,
        policy_version,
        policy_digest,
    })
}

fn brand_policy_cache_path() -> PathBuf {
    if let Ok(path) = std::env::var("CORTEX_BRAND_POLICY_CACHE_PATH") {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return PathBuf::from(trimmed);
        }
    }
    decision_projection_cache_dir().join(DEFAULT_BRAND_POLICY_CACHE_FILE)
}

fn brand_policy_canonical_only_enabled() -> bool {
    std::env::var("NOSTRA_BRAND_POLICY_CANONICAL_ONLY")
        .map(|v| matches!(v.as_str(), "1" | "true" | "TRUE" | "yes" | "YES"))
        .unwrap_or(false)
}

fn load_brand_policy_cache(path: &FsPath) -> Option<BrandPolicyCacheRecord> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<BrandPolicyCacheRecord>(&raw).ok()
}

fn resolve_active_temporal_state(policy: &BrandPolicyDocument, now: &DateTime<Utc>) -> String {
    policy
        .temporal_windows
        .iter()
        .find(|window| is_temporal_window_active(window, now))
        .map(|window| window.state.to_ascii_lowercase())
        .unwrap_or_else(|| "none".to_string())
}

fn is_temporal_window_active(window: &TemporalWindow, now: &DateTime<Utc>) -> bool {
    if !window.recurrence.eq_ignore_ascii_case("annual") {
        return false;
    }

    let start = match parse_month_day_time(&window.start_month_day, &window.start_time_utc) {
        Some(value) => value,
        None => return false,
    };
    let end = match parse_month_day_time(&window.end_month_day, &window.end_time_utc) {
        Some(value) => value,
        None => return false,
    };
    let current = (
        now.month(),
        now.day(),
        now.hour(),
        now.minute(),
        now.second(),
    );

    if start <= end {
        current >= start && current <= end
    } else {
        current >= start || current <= end
    }
}

fn parse_month_day_time(month_day: &str, time: &str) -> Option<(u32, u32, u32, u32, u32)> {
    let (month, day) = parse_month_day(month_day)?;
    let (hour, minute, second) = parse_hms(time)?;
    Some((month, day, hour, minute, second))
}

fn parse_month_day(value: &str) -> Option<(u32, u32)> {
    let (month, day) = value.split_once('-')?;
    let month = month.parse::<u32>().ok()?;
    let day = day.parse::<u32>().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some((month, day))
}

fn parse_hms(value: &str) -> Option<(u32, u32, u32)> {
    let mut parts = value.split(':');
    let hour = parts.next()?.parse::<u32>().ok()?;
    let minute = parts.next()?.parse::<u32>().ok()?;
    let second = parts.next()?.parse::<u32>().ok()?;
    if parts.next().is_some() || hour > 23 || minute > 59 || second > 59 {
        return None;
    }
    Some((hour, minute, second))
}

async fn get_local_gateway_queue() -> impl IntoResponse {
    let queue = crate::gateway::runtime_host::local_gateway_queue_snapshot();
    let mut oldest_timestamp: Option<u64> = None;
    let mut newest_timestamp: Option<u64> = None;
    let mut conflict_count = 0usize;

    let items = queue
        .into_iter()
        .map(|mutation| {
            oldest_timestamp = Some(
                oldest_timestamp
                    .unwrap_or(mutation.timestamp)
                    .min(mutation.timestamp),
            );
            newest_timestamp = Some(
                newest_timestamp
                    .unwrap_or(mutation.timestamp)
                    .max(mutation.timestamp),
            );
            if mutation.conflict_state {
                conflict_count += 1;
            }

            LocalGatewayQueueMutationRecord {
                mutation_id: mutation.mutation_id,
                idempotency_key: mutation.idempotency_key,
                space_id: mutation.space_id,
                kip_command: mutation.kip_command,
                timestamp: mutation.timestamp,
                timestamp_iso: timestamp_iso(mutation.timestamp),
                attempts: mutation.attempts,
                last_error: mutation.last_error,
                last_attempt_at: mutation.last_attempt_at,
                last_attempt_at_iso: mutation.last_attempt_at.and_then(timestamp_iso),
                conflict_state: mutation.conflict_state,
            }
        })
        .collect::<Vec<_>>();

    Json(LocalGatewayQueueSnapshot {
        queue_size: items.len(),
        conflict_count,
        oldest_timestamp,
        oldest_timestamp_iso: oldest_timestamp.and_then(timestamp_iso),
        newest_timestamp,
        newest_timestamp_iso: newest_timestamp.and_then(timestamp_iso),
        items,
    })
}

async fn get_local_gateway_queue_export() -> impl IntoResponse {
    match crate::gateway::runtime_host::local_gateway_export_queue_json() {
        Ok(export_raw) => match serde_json::from_str::<Value>(&export_raw) {
            Ok(payload) => Json(payload).into_response(),
            Err(err) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "Failed to parse queue export payload",
                    "reason": err.to_string()
                })),
            )
                .into_response(),
        },
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to export local gateway queue",
                "reason": err
            })),
        )
            .into_response(),
    }
}

async fn post_local_gateway_queue_retry(Path(mutation_id): Path<String>) -> impl IntoResponse {
    apply_queue_mutation_action(&mutation_id, "retry")
}

async fn post_local_gateway_queue_discard(Path(mutation_id): Path<String>) -> impl IntoResponse {
    apply_queue_mutation_action(&mutation_id, "discard")
}

async fn post_local_gateway_queue_fork(Path(mutation_id): Path<String>) -> impl IntoResponse {
    apply_queue_mutation_action(&mutation_id, "fork")
}

fn apply_queue_mutation_action(mutation_id: &str, action: &str) -> axum::response::Response {
    let mutation_id = mutation_id.trim();
    if mutation_id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "mutation_id is required"
            })),
        )
            .into_response();
    }

    match crate::gateway::runtime_host::local_gateway_apply_queue_action(mutation_id, action) {
        Ok(()) => Json(LocalGatewayQueueActionResponse {
            accepted: true,
            mutation_id: mutation_id.to_string(),
            action: action.to_string(),
        })
        .into_response(),
        Err(err) if err.contains("not found") => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": err,
                "mutationId": mutation_id,
                "action": action,
            })),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": err,
                "mutationId": mutation_id,
                "action": action,
            })),
        )
            .into_response(),
    }
}

async fn get_cortex_theme_policy() -> impl IntoResponse {
    Json(current_theme_policy())
}

async fn put_cortex_theme_policy(Json(request): Json<ThemePolicyPreferences>) -> impl IntoResponse {
    match persist_theme_policy(request) {
        Ok(saved) => Json(saved).into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "Failed to persist theme policy preferences",
                "reason": err
            })),
        )
            .into_response(),
    }
}

#[cfg(test)]
fn mutation_conflict_state(last_error: Option<&str>) -> bool {
    last_error
        .as_ref()
        .map(|value| {
            let normalized = value.to_ascii_lowercase();
            normalized.contains("conflict")
                || normalized.contains("reject")
                || normalized.contains("fork")
                || normalized.contains("unauthorized")
                || normalized.contains("invalid")
        })
        .unwrap_or(false)
}

fn timestamp_iso(ts: u64) -> Option<String> {
    DateTime::<Utc>::from_timestamp(ts as i64, 0).map(|value| value.to_rfc3339())
}

fn load_cached_value(path: &FsPath) -> Option<Value> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<Value>(&raw).ok()
}

fn load_cached_envelope(path: &FsPath) -> Option<DecisionSurfaceEnvelope> {
    let raw = fs::read_to_string(path).ok()?;
    serde_json::from_str::<DecisionSurfaceEnvelope>(&raw).ok()
}

fn execution_profile_payload(space_id: &str, profile: &ExecutionProfile) -> Value {
    json!({
        "spaceId": space_id,
        "executionProfile": {
            "executionTopology": format!("{:?}", profile.execution_topology),
            "consensusMode": format!("{:?}", profile.consensus_mode),
            "trustBoundary": format!("{:?}", profile.trust_boundary),
            "updatedBy": profile.updated_by.clone(),
            "updatedAt": profile.updated_at
        }
    })
}

fn attribution_domains_payload(space_id: &str, domains: &[AttributionDomain]) -> Value {
    let rows = domains
        .iter()
        .map(|domain| {
            json!({
                "id": domain.id,
                "attributionMode": format!("{:?}", domain.mode).to_ascii_lowercase(),
                "reattachmentPolicy": domain.reattachment_policy,
                "governanceVisibility": domain.governance_visibility,
                "auditabilityLevel": domain.auditability_level,
                "weightPolicyRef": domain.weight_policy_ref,
                "updatedBy": domain.updated_by.clone(),
                "updatedAt": domain.updated_at
            })
        })
        .collect::<Vec<_>>();
    json!({ "spaceId": space_id, "domains": rows })
}

fn governance_scope_payload(space_id: &str, evaluation: &ActionScopeEvaluation) -> Value {
    json!({
        "spaceId": space_id,
        "scope": {
            "allowed": evaluation.allowed,
            "reason": evaluation.reason,
            "effectiveWeight": evaluation.effective_weight,
            "requiresReview": evaluation.requires_review,
            "gateDecision": evaluation.gate_decision,
            "requiredActions": evaluation.required_actions,
            "policyRef": evaluation.policy_ref,
            "policyVersion": evaluation.policy_version
        }
    })
}

fn replay_contract_payload(contract: &ReplayContract) -> Value {
    json!({
        "mutationId": contract.mutation_id,
        "workflowId": contract.workflow_id,
        "actionTarget": contract.action_target,
        "adapterSetRef": contract.adapter_set_ref,
        "executionProfileRef": contract.execution_profile_ref,
        "attributionDomainRef": contract.attribution_domain_ref,
        "deterministicInputHash": contract.deterministic_input_hash,
        "lineageId": contract.lineage_id,
        "policyRef": contract.policy_ref,
        "policySnapshotRef": contract.policy_snapshot_ref,
        "evidenceRefs": contract.evidence_refs,
        "decisionDigest": contract.decision_digest,
        "capturedAt": contract.captured_at
    })
}

fn infer_domain_mode_from_replay(contract: &ReplayContract) -> String {
    let normalized = contract.attribution_domain_ref.to_ascii_lowercase();
    if normalized.contains("anonymous") {
        "anonymous".to_string()
    } else if normalized.contains("pseudonymous") {
        "pseudonymous".to_string()
    } else if normalized.contains("delayed") {
        "delayed".to_string()
    } else {
        "attributed".to_string()
    }
}

fn assessment_gate_status(assessment: &EpistemicAssessment) -> String {
    match assessment.gate_outcome {
        crate::services::workflow_engine_client::GateOutcome::Pass => "ready".to_string(),
        crate::services::workflow_engine_client::GateOutcome::Warn => "review".to_string(),
        crate::services::workflow_engine_client::GateOutcome::RequireReview => "review".to_string(),
        crate::services::workflow_engine_client::GateOutcome::RequireSimulation => {
            "require_simulation".to_string()
        }
        crate::services::workflow_engine_client::GateOutcome::Block => "blocked".to_string(),
    }
}

async fn get_system_execution_profile(Path(space_id): Path<String>) -> impl IntoResponse {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPACE_ID",
            "space_id is required",
            None,
        );
    }
    let cache_path = decision_projection_cache_dir().join(format!(
        "execution_profile_{}.json",
        sanitize_fs_component(space_id)
    ));
    let canonical = match WorkflowEngineClient::from_env() {
        Ok(client) => client
            .get_space_execution_profile(space_id)
            .await
            .ok()
            .flatten(),
        Err(_) => None,
    };
    if decision_canonical_only_enabled() && canonical.is_none() {
        return decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "CANONICAL_SOURCE_REQUIRED",
            "Canonical-only mode is enabled and execution profile could not be loaded from canister",
            Some(json!({ "spaceId": space_id })),
        );
    }
    let (payload, source_of_truth, degraded_reason) = if let Some(profile) = canonical {
        (
            execution_profile_payload(space_id, &profile),
            Some("canister".to_string()),
            None,
        )
    } else if let Some(cached) = load_cached_value(&cache_path) {
        (
            cached,
            Some("cache".to_string()),
            Some("workflow_engine_unreachable_or_missing_profile".to_string()),
        )
    } else {
        (
            json!({
                "spaceId": space_id,
                "executionProfile": {
                    "executionTopology": "LocalOnly",
                    "consensusMode": "NoneLocal",
                    "trustBoundary": "AttributedDefault"
                }
            }),
            Some("fallback".to_string()),
            Some("no_canonical_or_cached_execution_profile".to_string()),
        )
    };
    let envelope = build_decision_envelope(
        format!("system_execution_profile:{space_id}"),
        format!("workflow:execution_profile:{space_id}"),
        format!("execution_profile_{space_id}"),
        "ok".to_string(),
        vec![format!("decision_ack:execution_profile_{space_id}")],
        Vec::new(),
        source_of_truth,
        None,
        None,
        None,
        degraded_reason,
        None,
        None,
        Some(payload),
    );
    let projection_path = decision_projection_cache_dir().join(format!(
        "projection_execution_profile_{}.json",
        sanitize_fs_component(space_id)
    ));
    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&projection_path, &value);
    }
    Json(envelope).into_response()
}

async fn get_system_attribution_domains(Path(space_id): Path<String>) -> impl IntoResponse {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPACE_ID",
            "space_id is required",
            None,
        );
    }
    let cache_path = decision_projection_cache_dir().join(format!(
        "attribution_domains_{}.json",
        sanitize_fs_component(space_id)
    ));
    let canonical = match WorkflowEngineClient::from_env() {
        Ok(client) => client.get_attribution_domains(space_id).await.ok(),
        Err(_) => None,
    };
    if decision_canonical_only_enabled() && canonical.is_none() {
        return decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "CANONICAL_SOURCE_REQUIRED",
            "Canonical-only mode is enabled and attribution domains could not be loaded from canister",
            Some(json!({ "spaceId": space_id })),
        );
    }
    let (payload, source_of_truth, degraded_reason) = if let Some(domains) = canonical {
        (
            attribution_domains_payload(space_id, &domains),
            Some("canister".to_string()),
            None,
        )
    } else if let Some(cached) = load_cached_value(&cache_path) {
        (
            cached,
            Some("cache".to_string()),
            Some("workflow_engine_unreachable_or_missing_domains".to_string()),
        )
    } else {
        (
            json!({
                "spaceId": space_id,
                "domains": [
                    {
                        "id": "default",
                        "attributionMode": "attributed",
                        "governanceVisibility": "full",
                        "auditabilityLevel": "standard"
                    }
                ]
            }),
            Some("fallback".to_string()),
            Some("no_canonical_or_cached_attribution_domains".to_string()),
        )
    };
    let envelope = build_decision_envelope(
        format!("system_attribution_domain:{space_id}:default"),
        format!("workflow:attribution_domains:{space_id}"),
        format!("attribution_domains_{space_id}"),
        "ok".to_string(),
        vec![format!("decision_ack:attribution_domains_{space_id}")],
        Vec::new(),
        source_of_truth,
        None,
        None,
        None,
        degraded_reason,
        None,
        None,
        Some(payload),
    );
    let projection_path = decision_projection_cache_dir().join(format!(
        "projection_attribution_domains_{}.json",
        sanitize_fs_component(space_id)
    ));
    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&projection_path, &value);
    }
    Json(envelope).into_response()
}

async fn get_system_governance_scope(Path(space_id): Path<String>) -> impl IntoResponse {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPACE_ID",
            "space_id is required",
            None,
        );
    }
    let cache_path = decision_projection_cache_dir().join(format!(
        "governance_scope_{}.json",
        sanitize_fs_component(space_id)
    ));
    let canonical_actor = std::env::var("NOSTRA_DECISION_ACTOR_PRINCIPAL")
        .ok()
        .and_then(|value| Principal::from_text(value.trim()).ok())
        .unwrap_or_else(Principal::anonymous);
    let canonical = match GovernanceClient::from_env() {
        Ok(client) => client
            .evaluate_action_scope_with_actor(
                space_id,
                "governance:scope",
                "attributed",
                "informational",
                &canonical_actor,
            )
            .await
            .ok(),
        Err(_) => None,
    };
    if decision_canonical_only_enabled() && canonical.is_none() {
        return decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "CANONICAL_SOURCE_REQUIRED",
            "Canonical-only mode is enabled and governance scope could not be loaded from canister",
            Some(json!({ "spaceId": space_id })),
        );
    }
    let (payload, source_of_truth, degraded_reason, policy_ref, policy_version) =
        if let Some(evaluation) = canonical {
            (
                governance_scope_payload(space_id, &evaluation),
                Some("canister".to_string()),
                None,
                evaluation.policy_ref.clone(),
                Some(evaluation.policy_version),
            )
        } else if let Some(cached) = load_cached_value(&cache_path) {
            (
                cached,
                Some("cache".to_string()),
                Some("governance_unreachable_or_missing_scope".to_string()),
                None,
                None,
            )
        } else {
            (
                json!({
                    "spaceId": space_id,
                    "scope": {
                        "appliesTo": ["ActionTargets"],
                        "revocable": true,
                        "forkable": true
                    }
                }),
                Some("fallback".to_string()),
                Some("no_canonical_or_cached_governance_scope".to_string()),
                None,
                None,
            )
        };
    let envelope = build_decision_envelope(
        format!("system_governance_scope:{space_id}"),
        format!("workflow:governance_scope:{space_id}"),
        format!("governance_scope_{space_id}"),
        "ok".to_string(),
        vec![format!("decision_ack:governance_scope_{space_id}")],
        Vec::new(),
        source_of_truth,
        None,
        policy_ref,
        policy_version,
        degraded_reason,
        None,
        None,
        Some(payload),
    );
    let projection_path = decision_projection_cache_dir().join(format!(
        "projection_governance_scope_{}.json",
        sanitize_fs_component(space_id)
    ));
    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&projection_path, &value);
    }
    Json(envelope).into_response()
}

async fn get_system_replay_contract(Path(mutation_id): Path<String>) -> impl IntoResponse {
    let mutation_id = mutation_id.trim();
    if mutation_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MUTATION_ID",
            "mutation_id is required",
            None,
        );
    }
    let cache_path = decision_projection_cache_dir().join(format!(
        "replay_contract_{}.json",
        sanitize_fs_component(mutation_id)
    ));
    let canonical = match WorkflowEngineClient::from_env() {
        Ok(client) => {
            let replay = client.get_replay_contract(mutation_id).await.ok().flatten();
            if let Some(mut replay) = replay {
                if replay.lineage_id.is_none() {
                    if let Ok(Some(lineage)) =
                        client.get_decision_lineage_by_mutation(mutation_id).await
                    {
                        replay.lineage_id = Some(lineage.lineage_id);
                        replay.policy_ref = lineage.policy_ref;
                        replay.policy_snapshot_ref = lineage.policy_snapshot_ref;
                        replay.decision_digest = Some(lineage.decision_digest);
                        replay.evidence_refs = lineage.evidence_refs;
                    }
                }
                Some(replay)
            } else {
                None
            }
        }
        Err(_) => None,
    };
    if decision_canonical_only_enabled() && canonical.is_none() {
        return decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "CANONICAL_SOURCE_REQUIRED",
            "Canonical-only mode is enabled and replay contract could not be loaded from canister",
            Some(json!({ "mutationId": mutation_id })),
        );
    }
    let (payload, status, source_of_truth, lineage_id, policy_ref, degraded_reason) =
        if let Some(contract) = canonical {
            (
                replay_contract_payload(&contract),
                "available".to_string(),
                Some("canister".to_string()),
                contract.lineage_id.clone(),
                contract
                    .policy_ref
                    .clone()
                    .or_else(|| contract.policy_snapshot_ref.clone()),
                None,
            )
        } else if let Some(cached) = load_cached_value(&cache_path) {
            (
                cached,
                "available".to_string(),
                Some("cache".to_string()),
                None,
                None,
                Some("workflow_engine_unreachable_or_missing_replay_contract".to_string()),
            )
        } else {
            (
                json!({
                    "mutationId": mutation_id,
                    "workflowId": "workflow:unknown",
                    "executionProfileRef": "system_execution_profile:unknown",
                    "attributionDomainRef": "system_attribution_domain:unknown",
                    "deterministicInputHash": null
                }),
                "missing".to_string(),
                Some("fallback".to_string()),
                None,
                None,
                Some("no_canonical_or_cached_replay_contract".to_string()),
            )
        };
    let workflow_id = payload
        .get("workflowId")
        .and_then(|value| value.as_str())
        .unwrap_or("workflow:unknown")
        .to_string();
    let envelope = build_decision_envelope(
        format!("system_replay_contract:{mutation_id}"),
        workflow_id,
        mutation_id.to_string(),
        status.clone(),
        if status == "available" {
            Vec::new()
        } else {
            vec![format!("decision_escalate:{mutation_id}")]
        },
        Vec::new(),
        source_of_truth,
        lineage_id,
        policy_ref,
        None,
        degraded_reason,
        None,
        None,
        Some(payload),
    );
    let projection_path = decision_projection_cache_dir().join(format!(
        "projection_replay_contract_{}.json",
        sanitize_fs_component(mutation_id)
    ));
    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&projection_path, &value);
    }
    Json(envelope).into_response()
}

async fn get_system_decision_gates_latest() -> impl IntoResponse {
    let gate_path = testing_gate_summary_path();
    let summary = load_cached_value(&gate_path);
    let run_id = summary
        .as_ref()
        .and_then(|value| value.get("latest_run_id"))
        .and_then(|value| value.as_str())
        .unwrap_or("latest");
    let blockers_pass = summary
        .as_ref()
        .and_then(|value| value.get("required_blockers_pass"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let status = if blockers_pass { "ready" } else { "blocked" };
    let mutation_id = format!("decision_gate_{run_id}");
    let envelope = build_decision_envelope(
        format!("system_test_gate:{run_id}"),
        "workflow:system_decision_gates".to_string(),
        mutation_id.clone(),
        status.to_string(),
        if blockers_pass {
            Vec::new()
        } else {
            vec![
                format!("decision_escalate:{mutation_id}"),
                format!("decision_ack:{mutation_id}"),
            ]
        },
        summary
            .as_ref()
            .and_then(|value| value.get("failures"))
            .and_then(|value| value.as_array())
            .map(|rows| {
                rows.iter()
                    .filter_map(|row| row.get("code").and_then(|code| code.as_str()))
                    .map(|code| format!("test-gate:{code}"))
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),
        Some(if summary.is_some() {
            "cache".to_string()
        } else {
            "fallback".to_string()
        }),
        None,
        None,
        None,
        if summary.is_some() {
            None
        } else {
            Some("missing_test_gate_summary".to_string())
        },
        None,
        None,
        summary,
    );
    let cache_path = decision_projection_cache_dir().join("decision_gates_latest.json");
    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&cache_path, &value);
    }
    Json(envelope).into_response()
}

async fn get_system_mutation_gates(
    Path((space_id, mutation_id)): Path<(String, String)>,
) -> impl IntoResponse {
    let started_at = Instant::now();
    let space_id = space_id.trim();
    let mutation_id = mutation_id.trim();
    if space_id.is_empty() || mutation_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_MUTATION_GATE_INPUT",
            "space_id and mutation_id are required",
            Some(json!({ "spaceId": space_id, "mutationId": mutation_id })),
        );
    }

    let cache_path = decision_projection_cache_dir().join(format!(
        "decision_gate_{}.json",
        sanitize_fs_component(mutation_id)
    ));

    let workflow_client = WorkflowEngineClient::from_env().ok();
    let governance_client = GovernanceClient::from_env().ok();

    let mut degraded = Vec::<String>::new();
    let assessment = if let Some(client) = workflow_client.as_ref() {
        match client
            .get_epistemic_assessment_by_mutation(mutation_id)
            .await
        {
            Ok(value) => value,
            Err(err) => {
                degraded.push(format!("assessment_query_failed:{err}"));
                None
            }
        }
    } else {
        degraded.push("workflow_engine_client_unavailable".to_string());
        None
    };

    let replay_contract = if let Some(client) = workflow_client.as_ref() {
        match client.get_replay_contract(mutation_id).await {
            Ok(Some(mut value)) => {
                if value.lineage_id.is_none() {
                    if let Ok(Some(lineage)) =
                        client.get_decision_lineage_by_mutation(mutation_id).await
                    {
                        value.lineage_id = Some(lineage.lineage_id);
                        value.policy_ref = lineage.policy_ref;
                        value.policy_snapshot_ref = lineage.policy_snapshot_ref;
                        value.decision_digest = Some(lineage.decision_digest);
                        value.evidence_refs = lineage.evidence_refs;
                    }
                }
                Some(value)
            }
            Ok(None) => None,
            Err(err) => {
                degraded.push(format!("replay_query_failed:{err}"));
                None
            }
        }
    } else {
        None
    };

    let domain_mode = replay_contract
        .as_ref()
        .map(infer_domain_mode_from_replay)
        .unwrap_or_else(|| "attributed".to_string());
    let gate_level = if assessment.is_some() {
        "release_blocker"
    } else {
        "informational"
    };

    let governance_eval = if let Some(client) = governance_client.as_ref() {
        let principal = std::env::var("NOSTRA_DECISION_ACTOR_PRINCIPAL")
            .ok()
            .and_then(|value| Principal::from_text(value.trim()).ok())
            .unwrap_or_else(Principal::anonymous);
        match client
            .evaluate_action_scope_with_actor(
                space_id,
                "governance:decision",
                &domain_mode,
                gate_level,
                &principal,
            )
            .await
        {
            Ok(value) => Some(value),
            Err(err) => {
                degraded.push(format!("governance_query_failed:{err}"));
                None
            }
        }
    } else {
        degraded.push("governance_client_unavailable".to_string());
        None
    };
    if decision_canonical_only_enabled() && (assessment.is_none() || replay_contract.is_none()) {
        return decision_surface_error(
            StatusCode::SERVICE_UNAVAILABLE,
            "CANONICAL_SOURCE_REQUIRED",
            "Canonical-only mode is enabled and mutation gate dependencies are unavailable",
            Some(json!({ "spaceId": space_id, "mutationId": mutation_id })),
        );
    }

    let latest_gate_summary = load_cached_value(&testing_gate_summary_path());
    let blockers_pass = latest_gate_summary
        .as_ref()
        .and_then(|value| value.get("required_blockers_pass"))
        .and_then(|value| value.as_bool())
        .unwrap_or(false);

    let mut status = if !blockers_pass { "blocked" } else { "ready" }.to_string();
    if let Some(assessment) = assessment.as_ref() {
        status = assessment_gate_status(assessment);
    }
    if replay_contract.is_none() {
        status = "missing".to_string();
    }
    if let Some(eval) = governance_eval.as_ref() {
        if eval.gate_decision.eq_ignore_ascii_case("block") {
            status = "blocked".to_string();
        } else if eval.gate_decision.eq_ignore_ascii_case("review") && status == "ready" {
            status = "review".to_string();
        }
    }
    if !blockers_pass {
        status = "blocked".to_string();
    }

    let mut required_actions = Vec::<String>::new();
    if matches!(status.as_str(), "blocked" | "missing") {
        required_actions.push(format!("decision_escalate:{mutation_id}"));
        required_actions.push(format!("decision_ack:{mutation_id}"));
    } else if matches!(status.as_str(), "review" | "require_simulation") {
        required_actions.push(format!("decision_escalate:{mutation_id}"));
    }
    if let Some(eval) = governance_eval.as_ref() {
        for action in eval.required_actions.iter() {
            let action = format!("{action}:{mutation_id}");
            if !required_actions.contains(&action) {
                required_actions.push(action);
            }
        }
    }

    let mut evidence_refs = Vec::<String>::new();
    if let Some(contract) = replay_contract.as_ref() {
        evidence_refs.extend(contract.evidence_refs.clone());
        evidence_refs.push(format!("replay_hash:{}", contract.deterministic_input_hash));
    }
    if let Some(assessment) = assessment.as_ref() {
        for reason in assessment.reasons.iter() {
            evidence_refs.push(format!("assessment_reason:{reason}"));
        }
    }
    if let Some(summary) = latest_gate_summary.as_ref() {
        if let Some(rows) = summary.get("failures").and_then(|value| value.as_array()) {
            for code in rows
                .iter()
                .filter_map(|row| row.get("code").and_then(|code| code.as_str()))
            {
                evidence_refs.push(format!("test-gate:{code}"));
            }
        }
    }
    evidence_refs.sort();
    evidence_refs.dedup();

    let payload = json!({
        "spaceId": space_id,
        "mutationId": mutation_id,
        "assessment": assessment,
        "replayContract": replay_contract,
        "governanceEvaluation": governance_eval,
        "testGateSummary": latest_gate_summary
    });

    let source_of_truth = if degraded.is_empty() {
        Some("canister".to_string())
    } else if load_cached_envelope(&cache_path).is_some() {
        Some("cache".to_string())
    } else {
        Some("fallback".to_string())
    };
    let degraded_reason = if degraded.is_empty() {
        None
    } else {
        Some(degraded.join(";"))
    };
    let lineage_id = replay_contract
        .as_ref()
        .and_then(|entry| entry.lineage_id.clone());
    let policy_ref = governance_eval
        .as_ref()
        .and_then(|entry| entry.policy_ref.clone())
        .or_else(|| {
            replay_contract
                .as_ref()
                .and_then(|entry| entry.policy_ref.clone())
        })
        .or_else(|| {
            replay_contract
                .as_ref()
                .and_then(|entry| entry.policy_snapshot_ref.clone())
        });
    let policy_version = governance_eval.as_ref().map(|entry| entry.policy_version);

    let envelope = build_decision_envelope(
        format!("blackwell_gate:{mutation_id}"),
        assessment
            .as_ref()
            .map(|entry| entry.workflow_id.clone())
            .unwrap_or_else(|| "workflow:unknown".to_string()),
        mutation_id.to_string(),
        status,
        required_actions,
        evidence_refs,
        source_of_truth,
        lineage_id,
        policy_ref,
        policy_version,
        degraded_reason,
        None,
        None,
        Some(payload),
    );

    if let Ok(value) = serde_json::to_value(&envelope) {
        let _ = persist_json(&cache_path, &value);
    }
    record_decision_gate_telemetry(
        space_id,
        &envelope.status,
        envelope.source_of_truth.as_deref(),
        envelope.degraded_reason.as_deref(),
        started_at.elapsed().as_millis().min(u128::from(u64::MAX)) as u64,
    );
    Json(envelope).into_response()
}

async fn get_system_decision_plane(Path(space_id): Path<String>) -> impl IntoResponse {
    async fn extract_surface(
        label: &str,
        response: axum::response::Response,
        degraded: &mut Vec<String>,
    ) -> Option<DecisionSurfaceEnvelope> {
        let status = response.status();
        let bytes = match to_bytes(response.into_body(), 2 * 1024 * 1024).await {
            Ok(bytes) => bytes,
            Err(err) => {
                degraded.push(format!("{label}:body_read_failed:{err}"));
                return None;
            }
        };
        if !status.is_success() {
            let details = serde_json::from_slice::<ErrorResponse>(&bytes)
                .ok()
                .map(|entry| entry.error_code)
                .unwrap_or_else(|| format!("http_{}", status.as_u16()));
            degraded.push(format!("{label}:{details}"));
            return None;
        }
        match serde_json::from_slice::<DecisionSurfaceEnvelope>(&bytes) {
            Ok(surface) => Some(surface),
            Err(err) => {
                degraded.push(format!("{label}:decode_failed:{err}"));
                None
            }
        }
    }

    let started_at = Instant::now();
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPACE_ID",
            "space_id is required",
            None,
        );
    }

    let mut surfaces = Vec::<DecisionSurfaceEnvelope>::new();
    let mut degraded = Vec::<String>::new();
    let mut loaded_labels = Vec::<&str>::new();

    if let Some(surface) = extract_surface(
        "execution_profile",
        get_system_execution_profile(Path(space_id.to_string()))
            .await
            .into_response(),
        &mut degraded,
    )
    .await
    {
        loaded_labels.push("execution_profile");
        surfaces.push(surface);
    }
    if let Some(surface) = extract_surface(
        "attribution_domains",
        get_system_attribution_domains(Path(space_id.to_string()))
            .await
            .into_response(),
        &mut degraded,
    )
    .await
    {
        loaded_labels.push("attribution_domains");
        surfaces.push(surface);
    }
    if let Some(surface) = extract_surface(
        "governance_scope",
        get_system_governance_scope(Path(space_id.to_string()))
            .await
            .into_response(),
        &mut degraded,
    )
    .await
    {
        loaded_labels.push("governance_scope");
        surfaces.push(surface);
    }
    if let Some(surface) = extract_surface(
        "decision_gates_latest",
        get_system_decision_gates_latest().await.into_response(),
        &mut degraded,
    )
    .await
    {
        loaded_labels.push("decision_gates_latest");
        surfaces.push(surface);
    }

    let canonical_gate_mutation = match WorkflowEngineClient::from_env() {
        Ok(client) => match client.list_space_decision_lineage(space_id, 1).await {
            Ok(rows) => rows.first().map(|entry| entry.mutation_id.clone()),
            Err(err) => {
                degraded.push(format!("decision_lineage_lookup_failed:{err}"));
                None
            }
        },
        Err(err) => {
            degraded.push(format!("workflow_client_unavailable:{err}"));
            None
        }
    };
    let gate_mutation = canonical_gate_mutation
        .or_else(|| {
            surfaces
                .iter()
                .find(|surface| surface.surface_id.starts_with("system_test_gate:"))
                .map(|surface| surface.mutation_id.clone())
        })
        .unwrap_or_else(|| "decision_gate_latest".to_string());
    if let Some(surface) = extract_surface(
        "mutation_gate",
        get_system_mutation_gates(Path((space_id.to_string(), gate_mutation)))
            .await
            .into_response(),
        &mut degraded,
    )
    .await
    {
        loaded_labels.push("mutation_gate");
        surfaces.push(surface);
    }

    surfaces.sort_by(|left, right| left.surface_id.cmp(&right.surface_id));

    let required_labels = [
        "execution_profile",
        "attribution_domains",
        "governance_scope",
        "decision_gates_latest",
        "mutation_gate",
    ];
    for label in required_labels
        .iter()
        .filter(|label| !loaded_labels.contains(label))
    {
        degraded.push(format!("{label}:missing_required_surface"));
    }

    let surface_status_blocked = surfaces.iter().any(|surface| {
        matches!(
            surface.status.as_str(),
            "blocked" | "missing" | "require_simulation"
        )
    });
    let all_surfaces_failed = surfaces.is_empty();
    let degraded_present = !degraded.is_empty();
    let has_actionable_surface = surfaces
        .iter()
        .any(|surface| !surface.required_actions.is_empty());
    let blocked = surface_status_blocked
        || all_surfaces_failed
        || (degraded_present && !has_actionable_surface);
    let require_escalation = blocked || degraded_present;

    let aggregate_source = if degraded_present || all_surfaces_failed {
        Some("fallback".to_string())
    } else if surfaces
        .iter()
        .any(|surface| matches!(surface.source_of_truth.as_deref(), Some("fallback")))
    {
        Some("fallback".to_string())
    } else if surfaces
        .iter()
        .any(|surface| matches!(surface.source_of_truth.as_deref(), Some("cache")))
    {
        Some("cache".to_string())
    } else {
        Some("canister".to_string())
    };
    let digest = build_decision_envelope(
        format!("system_decision_plane:{space_id}"),
        "workflow:system_decision_plane".to_string(),
        format!("decision_plane_{space_id}"),
        if blocked {
            "blocked".to_string()
        } else {
            "ready".to_string()
        },
        if require_escalation {
            vec![format!("decision_escalate:decision_plane_{space_id}")]
        } else {
            Vec::new()
        },
        degraded
            .iter()
            .map(|entry| format!("decision-plane:{entry}"))
            .collect::<Vec<_>>(),
        aggregate_source,
        None,
        None,
        None,
        if degraded.is_empty() {
            None
        } else {
            Some(degraded.join(";"))
        },
        None,
        None,
        Some(json!({
            "spaceId": space_id,
            "surfaceCount": surfaces.len(),
            "latencyMs": started_at.elapsed().as_millis(),
            "requiredSurfaceLabels": required_labels,
            "loadedSurfaceLabels": loaded_labels
        })),
    );

    Json(DecisionPlaneResponse {
        space_id: space_id.to_string(),
        surfaces,
        digest: Some(digest),
    })
    .into_response()
}

async fn post_system_decision_ack(
    headers: HeaderMap,
    Json(payload): Json<DecisionActionRequest>,
) -> impl IntoResponse {
    match record_decision_action("ack", &headers, payload).await {
        Ok(envelope) => Json(envelope).into_response(),
        Err(err) => err,
    }
}

async fn post_system_decision_escalate(
    headers: HeaderMap,
    Json(payload): Json<DecisionActionRequest>,
) -> impl IntoResponse {
    match record_decision_action("escalate", &headers, payload).await {
        Ok(envelope) => Json(envelope).into_response(),
        Err(err) => err,
    }
}

async fn get_system_decision_telemetry() -> impl IntoResponse {
    Json(decision_telemetry_snapshot())
}

async fn get_system_decision_telemetry_by_space(Path(space_id): Path<String>) -> impl IntoResponse {
    let space_id = space_id.trim();
    if space_id.is_empty() {
        return decision_surface_error(
            StatusCode::BAD_REQUEST,
            "INVALID_SPACE_ID",
            "space_id is required",
            None,
        );
    }
    Json(decision_telemetry_snapshot_by_space(space_id)).into_response()
}

async fn list_canisters() -> Json<Vec<CanisterInfo>> {
    let output = dfx_command()
        .arg("canister")
        .arg("id")
        .arg("--all")
        .output();

    let mut canisters = Vec::new();

    if let Ok(o) = output {
        let stdout = String::from_utf8_lossy(&o.stdout);
        for line in stdout.lines() {
            if let Some((name, id)) = line.split_once(":") {
                // Check status for each (could be slow, maybe optimize later)
                let status_output = dfx_command()
                    .arg("canister")
                    .arg("status")
                    .arg(id.trim())
                    .output();

                let status = if let Ok(s) = status_output {
                    if String::from_utf8_lossy(&s.stderr).contains("Running")
                        || String::from_utf8_lossy(&s.stdout).contains("Running")
                    {
                        "Running".to_string()
                    } else {
                        "Stopped".to_string()
                    }
                } else {
                    "Unknown".to_string()
                };

                canisters.push(CanisterInfo {
                    name: name.trim().to_string(),
                    id: id.trim().to_string(),
                    status,
                });
            }
        }
    }

    // Mock data if dfx is offline for UI testing
    if canisters.is_empty() {
        canisters.push(CanisterInfo {
            name: "internet_identity (mock)".into(),
            id: "rdmx6-jaaaa-aaaaa-aaadq-cai".into(),
            status: "Running".into(),
        });
        canisters.push(CanisterInfo {
            name: "cortex_backend (mock)".into(),
            id: "bkyz2-fmaaa-aaaaa-qaaaq-cai".into(),
            status: "Stopped".into(),
        });
    }

    Json(canisters)
}

async fn ws_handler(ws: WebSocketUpgrade, State(state): State<GatewayState>) -> impl IntoResponse {
    ws.on_upgrade(|socket| handle_socket(socket, state))
}

async fn handle_socket(socket: WebSocket, state: GatewayState) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = state.broadcast_tx.subscribe();

    // Spawn a task to forward broadcast messages to this client
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    // Handle incoming messages
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            if let Message::Text(text) = msg {
                tracing::info!("Received: {}", text);
                // Echo for now, Logic Layer will process this
            }
        }
    });

    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentContributionRequest {
    #[serde(alias = "contribution_id")]
    contribution_id: String,
    #[serde(default)]
    #[serde(alias = "agent_id")]
    agent_id: Option<String>,
    #[serde(default)]
    #[serde(alias = "authority_level")]
    authority_level: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentContributionResponse {
    accepted: bool,
    run_id: String,
    workflow_id: String,
    status: String,
    started_at: String,
    stream_channel: String,
    runtime_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    temporal_workflow_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    temporal_run_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    projection_mode: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentContributionApprovalRequest {
    decision: String,
    #[serde(default)]
    rationale: Option<String>,
    actor: String,
    #[serde(default)]
    #[serde(alias = "decision_ref")]
    decision_ref: Option<String>,
    #[serde(default)]
    #[serde(alias = "actor_principal")]
    actor_principal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentContributionApprovalResponse {
    accepted: bool,
    run_id: String,
    status: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentSimulationEvaluation {
    success: bool,
    violation_count: usize,
    risk_score: usize,
    siqs_score: f32,
    session_id: String,
    structural_diff_summary: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct AgentRunRecord {
    #[serde(flatten)]
    run: AgentRun,
    #[serde(default)]
    events: Vec<AgentRunEvent>,
    #[serde(default)]
    pending_action_target: Option<ActionTarget>,
    #[serde(default)]
    approval: Option<AgentContributionApprovalRequest>,
}

fn agent_runs_dir() -> PathBuf {
    decision_surface_log_dir().join("agent_runs")
}

fn agent_run_path(space_id: &str, run_id: &str) -> PathBuf {
    agent_runs_dir().join(format!(
        "{}__{}.json",
        sanitize_fs_component(space_id),
        sanitize_fs_component(run_id)
    ))
}

fn persist_agent_run_record(record: &AgentRunRecord) -> Result<(), String> {
    let path = agent_run_path(&record.run.space_id, &record.run.run_id);
    let value = serde_json::to_value(record).map_err(|err| err.to_string())?;
    persist_json(&path, &value)
}

fn load_agent_run_record(space_id: &str, run_id: &str) -> Result<AgentRunRecord, String> {
    let path = agent_run_path(space_id, run_id);
    let raw = fs::read_to_string(path).map_err(|err| err.to_string())?;
    serde_json::from_str::<AgentRunRecord>(&raw).map_err(|err| err.to_string())
}

fn next_agent_run_id(space_id: &str, contribution_id: &str) -> String {
    format!(
        "agent_run_{}_{}_{}",
        sanitize_fs_component(space_id),
        sanitize_fs_component(contribution_id),
        Utc::now().timestamp_millis()
    )
}

fn parse_authority_level(raw: Option<&str>) -> Result<AuthorityLevel, String> {
    let normalized = raw
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("l1");
    AuthorityLevel::from_str(normalized)
}

fn next_execution_id(run_id: &str) -> String {
    format!("exec_{}", sanitize_fs_component(run_id))
}

fn next_attempt_id(execution_id: &str) -> String {
    format!("{execution_id}_{}", Utc::now().timestamp_millis())
}

fn normalize_agent_identity(raw: Option<&str>) -> Option<String> {
    raw.map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
}

fn default_agent_identity() -> String {
    normalize_agent_identity(std::env::var("NOSTRA_AGENT_ID").ok().as_deref())
        .or_else(|| {
            normalize_agent_identity(std::env::var("NOSTRA_DEFAULT_AGENT_ID").ok().as_deref())
        })
        .unwrap_or_else(|| "agent:cortex-default".to_string())
}

fn resolve_agent_identity(request_agent_id: Option<&str>, headers: &HeaderMap) -> String {
    normalize_agent_identity(
        headers
            .get("x-cortex-agent-id")
            .and_then(|value| value.to_str().ok()),
    )
    .or_else(|| normalize_agent_identity(request_agent_id))
    .unwrap_or_else(default_agent_identity)
}

fn agent_approval_timeout_seconds(space_id: &str) -> u64 {
    let space_env_key = format!(
        "CORTEX_AGENT_APPROVAL_TIMEOUT_SECONDS_{}",
        sanitize_fs_component(space_id).to_ascii_uppercase()
    );
    std::env::var(space_env_key)
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .or_else(|| {
            std::env::var("CORTEX_AGENT_APPROVAL_TIMEOUT_SECONDS")
                .ok()
                .and_then(|value| value.trim().parse::<u64>().ok())
        })
        .unwrap_or(3600)
}

fn temporal_runtime_root() -> PathBuf {
    decision_surface_log_dir().join("temporal_bridge_runtime")
}

fn legacy_temporal_runtime_root() -> PathBuf {
    decision_surface_log_dir().join("temporal_runtime")
}

fn temporal_runtime_start_commands_dir() -> PathBuf {
    temporal_runtime_root().join("commands").join("start")
}

fn temporal_runtime_signal_commands_dir() -> PathBuf {
    temporal_runtime_root().join("commands").join("signal")
}

fn temporal_runtime_snapshots_dir() -> PathBuf {
    temporal_runtime_root().join("snapshots")
}

fn temporal_runtime_snapshot_path(run_id: &str) -> PathBuf {
    temporal_runtime_snapshots_dir().join(format!("{}.json", sanitize_fs_component(run_id)))
}

fn temporal_runtime_start_command_path(run_id: &str) -> PathBuf {
    temporal_runtime_start_commands_dir().join(format!(
        "{}_{}.json",
        sanitize_fs_component(run_id),
        Utc::now().timestamp_millis()
    ))
}

fn temporal_runtime_signal_command_path(run_id: &str) -> PathBuf {
    temporal_runtime_signal_commands_dir().join(format!(
        "{}_{}.json",
        sanitize_fs_component(run_id),
        Utc::now().timestamp_millis()
    ))
}

fn ensure_temporal_runtime_dirs() -> Result<(), String> {
    fs::create_dir_all(temporal_runtime_start_commands_dir()).map_err(|err| err.to_string())?;
    fs::create_dir_all(temporal_runtime_signal_commands_dir()).map_err(|err| err.to_string())?;
    fs::create_dir_all(temporal_runtime_snapshots_dir()).map_err(|err| err.to_string())
}

fn persist_temporal_start_command(
    run: &AgentRun,
    binding: &TemporalRunBinding,
    approval_timeout_seconds: u64,
) -> Result<(), String> {
    ensure_temporal_runtime_dirs()?;
    let command = TemporalBridgeStartCommand {
        run_id: run.run_id.clone(),
        workflow_id: binding.workflow_id.clone(),
        space_id: run.space_id.clone(),
        contribution_id: run.contribution_id.clone(),
        approval_timeout_seconds,
        task_queue: binding
            .task_queue
            .clone()
            .unwrap_or_else(|| "SIMULATION_TASK_QUEUE".to_string()),
        namespace: binding
            .namespace
            .clone()
            .unwrap_or_else(|| "default".to_string()),
    };
    persist_json(
        &temporal_runtime_start_command_path(&run.run_id),
        &serde_json::to_value(command).map_err(|err| err.to_string())?,
    )
}

fn persist_temporal_signal_command(
    run_id: &str,
    payload: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    ensure_temporal_runtime_dirs()?;
    let command = TemporalBridgeSignalCommand {
        run_id: run_id.to_string(),
        decision: payload.decision.clone(),
        rationale: payload.rationale.clone(),
        actor: payload.actor.clone(),
        decision_ref: payload.decision_ref.clone(),
    };
    persist_json(
        &temporal_runtime_signal_command_path(run_id),
        &serde_json::to_value(command).map_err(|err| err.to_string())?,
    )
}

fn temporal_cli_binary() -> String {
    std::env::var("CORTEX_TEMPORAL_CLI_BIN")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "temporal".to_string())
}

fn temporal_cli_address() -> String {
    std::env::var("CORTEX_TEMPORAL_ADDRESS")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "127.0.0.1:7233".to_string())
}

fn temporal_cli_namespace_default() -> String {
    std::env::var("CORTEX_TEMPORAL_NAMESPACE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "default".to_string())
}

fn temporal_cli_workflow_type() -> String {
    std::env::var("CORTEX_TEMPORAL_WORKFLOW_TYPE")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "ArchitectAndEvaluateWorkflow".to_string())
}

#[cfg(feature = "temporal-sdk-native")]
fn temporal_sdk_target_url() -> String {
    std::env::var("CORTEX_TEMPORAL_SDK_TARGET_URL")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| format!("http://{}", temporal_cli_address()))
}

#[cfg(feature = "temporal-sdk-native")]
fn temporal_sdk_client_name() -> String {
    std::env::var("CORTEX_TEMPORAL_SDK_CLIENT_NAME")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| "cortex-desktop-gateway".to_string())
}

#[cfg(feature = "temporal-sdk-native")]
fn temporal_sdk_client_version() -> String {
    std::env::var("CORTEX_TEMPORAL_SDK_CLIENT_VERSION")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemporalSdkTransport {
    Cli,
    Native,
}

fn temporal_sdk_transport() -> TemporalSdkTransport {
    match std::env::var("CORTEX_TEMPORAL_SDK_TRANSPORT")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("cli") => TemporalSdkTransport::Cli,
        Some("native") => TemporalSdkTransport::Native,
        Some(_) => TemporalSdkTransport::Cli,
        None => {
            #[cfg(feature = "temporal-sdk-native")]
            {
                TemporalSdkTransport::Native
            }
            #[cfg(not(feature = "temporal-sdk-native"))]
            {
                TemporalSdkTransport::Cli
            }
        }
    }
}

fn temporal_cli_command_timeout() -> Option<String> {
    std::env::var("CORTEX_TEMPORAL_COMMAND_TIMEOUT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .or_else(|| Some("15s".to_string()))
}

fn temporal_cli_command(namespace: Option<&str>) -> Command {
    let mut command = Command::new(temporal_cli_binary());
    command.arg("--output").arg("json");
    command.arg("--address").arg(temporal_cli_address());
    command
        .arg("--namespace")
        .arg(namespace.unwrap_or("default"));
    if let Some(timeout) = temporal_cli_command_timeout() {
        command.arg("--command-timeout").arg(timeout);
    }
    command
}

fn run_temporal_cli_command(mut command: Command, context: &str) -> Result<String, String> {
    let output = command.output().map_err(|err| {
        format!(
            "{}: failed to execute temporal CLI binary `{}`: {}",
            context,
            temporal_cli_binary(),
            err
        )
    })?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!(
            "{}: temporal CLI exited with status {}: {}",
            context,
            output.status,
            if stderr.is_empty() {
                stdout
            } else if stdout.is_empty() {
                stderr
            } else {
                format!("stderr={} stdout={}", stderr, stdout)
            }
        ))
    }
}

fn parse_temporal_start_run_id(stdout: &str) -> Option<String> {
    let parsed = serde_json::from_str::<Value>(stdout).ok()?;
    let pointer_candidates = [
        "/runId",
        "/workflowExecution/runId",
        "/execution/runId",
        "/result/runId",
    ];
    for pointer in pointer_candidates {
        if let Some(value) = parsed.pointer(pointer).and_then(Value::as_str) {
            let run_id = value.trim();
            if !run_id.is_empty() {
                return Some(run_id.to_string());
            }
        }
    }
    None
}

fn parse_temporal_snapshot_candidate(value: &Value) -> Option<TemporalBridgeRunSnapshot> {
    match value {
        Value::Object(_) => serde_json::from_value::<TemporalBridgeRunSnapshot>(value.clone()).ok(),
        Value::String(raw) => serde_json::from_str::<TemporalBridgeRunSnapshot>(raw)
            .ok()
            .or_else(|| {
                serde_json::from_str::<Value>(raw)
                    .ok()
                    .and_then(|nested| parse_temporal_snapshot_candidate(&nested))
            }),
        _ => None,
    }
}

fn parse_temporal_query_snapshot_stdout(stdout: &str) -> Result<TemporalBridgeRunSnapshot, String> {
    if let Ok(snapshot) = serde_json::from_str::<TemporalBridgeRunSnapshot>(stdout) {
        return Ok(snapshot);
    }
    let parsed = serde_json::from_str::<Value>(stdout).map_err(|err| {
        format!(
            "temporal query output is not valid JSON snapshot payload: {}",
            err
        )
    })?;
    let pointer_candidates = [
        "",
        "/result",
        "/result/data",
        "/result/payloads/0",
        "/result/payloads/0/data",
        "/queryResult",
        "/queryResult/data",
        "/response",
        "/response/data",
        "/payload",
        "/payload/data",
        "/data",
    ];
    for pointer in pointer_candidates {
        let candidate = if pointer.is_empty() {
            Some(&parsed)
        } else {
            parsed.pointer(pointer)
        };
        if let Some(candidate) = candidate {
            if let Some(snapshot) = parse_temporal_snapshot_candidate(candidate) {
                return Ok(snapshot);
            }
        }
    }
    Err(format!(
        "temporal query did not contain parseable `{}` payload",
        TEMPORAL_WORKFLOW_QUERY_RUN_SNAPSHOT
    ))
}

async fn start_temporal_sdk_workflow(
    run: &AgentRun,
    binding: &TemporalRunBinding,
    approval_timeout_seconds: u64,
) -> Result<Option<String>, String> {
    match temporal_sdk_transport() {
        TemporalSdkTransport::Cli => {
            start_temporal_sdk_workflow_cli(run, binding, approval_timeout_seconds)
        }
        TemporalSdkTransport::Native => {
            start_temporal_sdk_workflow_native(run, binding, approval_timeout_seconds).await
        }
    }
}

fn start_temporal_sdk_workflow_cli(
    run: &AgentRun,
    binding: &TemporalRunBinding,
    approval_timeout_seconds: u64,
) -> Result<Option<String>, String> {
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);
    let task_queue = binding
        .task_queue
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("SIMULATION_TASK_QUEUE")
        .to_string();
    let mut command = temporal_cli_command(Some(&namespace));
    let input = json!({
        "contribution_id": run.contribution_id,
        "space_id": run.space_id
    })
    .to_string();
    command
        .arg("workflow")
        .arg("start")
        .arg("--workflow-id")
        .arg(&binding.workflow_id)
        .arg("--type")
        .arg(temporal_cli_workflow_type())
        .arg("--task-queue")
        .arg(task_queue)
        .arg("--input")
        .arg(input)
        .arg("--run-timeout")
        .arg(format!("{}s", approval_timeout_seconds.max(1)))
        .arg("--id-conflict-policy")
        .arg("UseExisting");
    let stdout = run_temporal_cli_command(
        command,
        &format!("temporal_sdk_start_failed run_id={}", run.run_id),
    )?;
    Ok(parse_temporal_start_run_id(&stdout))
}

#[cfg(feature = "temporal-sdk-native")]
async fn start_temporal_sdk_workflow_native(
    run: &AgentRun,
    binding: &TemporalRunBinding,
    approval_timeout_seconds: u64,
) -> Result<Option<String>, String> {
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);
    let task_queue = binding
        .task_queue
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or("SIMULATION_TASK_QUEUE")
        .to_string();
    let target_url =
        reqwest::Url::parse(&temporal_sdk_target_url()).map_err(|err| err.to_string())?;
    let options = TemporalClientOptionsBuilder::default()
        .target_url(target_url)
        .client_name(temporal_sdk_client_name())
        .client_version(temporal_sdk_client_version())
        .identity(format!("cortex-desktop-gateway-{}", std::process::id()))
        .build()
        .map_err(|err| err.to_string())?;
    let client = options
        .connect(namespace, None)
        .await
        .map_err(|err| format!("temporal_sdk_connect_failed: {}", err))?;

    let input_payload = json!({
        "contribution_id": run.contribution_id,
        "space_id": run.space_id
    })
    .as_json_payload()
    .map_err(|err| format!("temporal_sdk_start_payload_encode_failed: {}", err))?;

    let mut workflow_options = TemporalWorkflowOptions::default();
    workflow_options.id_conflict_policy = WorkflowIdConflictPolicy::UseExisting;
    workflow_options.run_timeout = Some(Duration::from_secs(approval_timeout_seconds.max(1)));

    let start_response = client
        .start_workflow(
            vec![input_payload],
            task_queue,
            binding.workflow_id.clone(),
            temporal_cli_workflow_type(),
            Some(run.run_id.clone()),
            workflow_options,
        )
        .await
        .map_err(|err| format!("temporal_sdk_start_failed run_id={}: {}", run.run_id, err))?;

    let run_id = start_response.run_id.trim().to_string();
    if run_id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(run_id))
    }
}

#[cfg(not(feature = "temporal-sdk-native"))]
async fn start_temporal_sdk_workflow_native(
    run: &AgentRun,
    binding: &TemporalRunBinding,
    approval_timeout_seconds: u64,
) -> Result<Option<String>, String> {
    let _ = (run, binding, approval_timeout_seconds);
    Err("temporal_sdk_native_unavailable_build_with_feature_temporal-sdk-native".to_string())
}

async fn signal_temporal_sdk_workflow(
    run: &AgentRunRecord,
    payload: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    match temporal_sdk_transport() {
        TemporalSdkTransport::Cli => signal_temporal_sdk_workflow_cli(run, payload),
        TemporalSdkTransport::Native => signal_temporal_sdk_workflow_native(run, payload).await,
    }
}

fn signal_temporal_sdk_workflow_cli(
    run: &AgentRunRecord,
    payload: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    let binding = run
        .run
        .temporal_binding
        .as_ref()
        .ok_or_else(|| "temporal_sdk_signal_failed_missing_binding".to_string())?;
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);
    let mut command = temporal_cli_command(Some(&namespace));
    command
        .arg("workflow")
        .arg("signal")
        .arg("--workflow-id")
        .arg(&binding.workflow_id)
        .arg("--name")
        .arg(TEMPORAL_WORKFLOW_SIGNAL_HUMAN_APPROVAL);
    let signal_payload = json!({
        "decision": payload.decision,
        "rationale": payload.rationale,
        "actor": payload.actor,
        "decisionRef": payload.decision_ref,
        "spaceId": run.run.space_id,
        "scenarioId": format!("sim-{}", run.run.contribution_id),
        "runId": run.run.run_id
    })
    .to_string();
    command.arg("--input").arg(signal_payload);
    let _ = run_temporal_cli_command(
        command,
        &format!("temporal_sdk_signal_failed run_id={}", run.run.run_id),
    )?;
    Ok(())
}

#[cfg(feature = "temporal-sdk-native")]
async fn signal_temporal_sdk_workflow_native(
    run: &AgentRunRecord,
    payload: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    let binding = run
        .run
        .temporal_binding
        .as_ref()
        .ok_or_else(|| "temporal_sdk_signal_failed_missing_binding".to_string())?;
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);

    let target_url =
        reqwest::Url::parse(&temporal_sdk_target_url()).map_err(|err| err.to_string())?;
    let options = TemporalClientOptionsBuilder::default()
        .target_url(target_url)
        .client_name(temporal_sdk_client_name())
        .client_version(temporal_sdk_client_version())
        .identity(format!("cortex-desktop-gateway-{}", std::process::id()))
        .build()
        .map_err(|err| err.to_string())?;
    let client = options
        .connect(namespace, None)
        .await
        .map_err(|err| format!("temporal_sdk_connect_failed: {}", err))?;

    let signal_payload = json!({
        "decision": payload.decision,
        "rationale": payload.rationale,
        "actor": payload.actor,
        "decisionRef": payload.decision_ref,
        "spaceId": run.run.space_id,
        "scenarioId": format!("sim-{}", run.run.contribution_id),
        "runId": run.run.run_id
    })
    .as_json_payload()
    .map_err(|err| format!("temporal_sdk_signal_payload_encode_failed: {}", err))?;
    let signal_input = vec![signal_payload].into_payloads();
    let temporal_run_id = binding.temporal_run_id.clone().unwrap_or_default();

    client
        .signal_workflow_execution(
            binding.workflow_id.clone(),
            temporal_run_id,
            TEMPORAL_WORKFLOW_SIGNAL_HUMAN_APPROVAL.to_string(),
            signal_input,
            payload.decision_ref.clone(),
        )
        .await
        .map_err(|err| {
            format!(
                "temporal_sdk_signal_failed run_id={}: {}",
                run.run.run_id, err
            )
        })?;
    Ok(())
}

#[cfg(not(feature = "temporal-sdk-native"))]
async fn signal_temporal_sdk_workflow_native(
    run: &AgentRunRecord,
    payload: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    let _ = (run, payload);
    Err("temporal_sdk_native_unavailable_build_with_feature_temporal-sdk-native".to_string())
}

fn load_temporal_run_snapshot(run_id: &str) -> Result<TemporalBridgeRunSnapshot, String> {
    let path = temporal_runtime_snapshot_path(run_id);
    let raw = match fs::read_to_string(&path) {
        Ok(raw) => raw,
        Err(_) => {
            let legacy_path = legacy_temporal_runtime_root()
                .join("snapshots")
                .join(format!("{}.json", sanitize_fs_component(run_id)));
            fs::read_to_string(legacy_path).map_err(|err| err.to_string())?
        }
    };
    serde_json::from_str::<TemporalBridgeRunSnapshot>(&raw).map_err(|err| err.to_string())
}

async fn load_temporal_run_snapshot_for_backend(
    space_id: &str,
    run_id: &str,
    backend: TemporalExecutionBackend,
) -> Result<TemporalBridgeRunSnapshot, String> {
    match backend {
        TemporalExecutionBackend::Bridge => load_temporal_run_snapshot(run_id),
        TemporalExecutionBackend::Sdk => {
            query_temporal_sdk_workflow_snapshot(space_id, run_id).await
        }
    }
}

async fn query_temporal_sdk_workflow_snapshot(
    space_id: &str,
    run_id: &str,
) -> Result<TemporalBridgeRunSnapshot, String> {
    let run = load_agent_run_record(space_id, run_id)?;
    let binding = run.run.temporal_binding.as_ref().ok_or_else(|| {
        format!(
            "temporal_sdk_query_failed_missing_binding run_id={}",
            run_id
        )
    })?;
    match temporal_sdk_transport() {
        TemporalSdkTransport::Cli => query_temporal_sdk_workflow_snapshot_cli(&run, binding),
        TemporalSdkTransport::Native => {
            query_temporal_sdk_workflow_snapshot_native(&run, binding).await
        }
    }
}

fn query_temporal_sdk_workflow_snapshot_cli(
    run: &AgentRunRecord,
    binding: &TemporalRunBinding,
) -> Result<TemporalBridgeRunSnapshot, String> {
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);
    let mut command = temporal_cli_command(Some(&namespace));
    command
        .arg("workflow")
        .arg("query")
        .arg("--workflow-id")
        .arg(&binding.workflow_id)
        .arg("--name")
        .arg(TEMPORAL_WORKFLOW_QUERY_RUN_SNAPSHOT);
    let stdout = run_temporal_cli_command(
        command,
        &format!("temporal_sdk_query_failed run_id={}", run.run.run_id),
    )?;
    parse_temporal_query_snapshot_stdout(&stdout)
}

#[cfg(feature = "temporal-sdk-native")]
async fn query_temporal_sdk_workflow_snapshot_native(
    run: &AgentRunRecord,
    binding: &TemporalRunBinding,
) -> Result<TemporalBridgeRunSnapshot, String> {
    let namespace = binding
        .namespace
        .as_ref()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(temporal_cli_namespace_default);
    let target_url =
        reqwest::Url::parse(&temporal_sdk_target_url()).map_err(|err| err.to_string())?;
    let options = TemporalClientOptionsBuilder::default()
        .target_url(target_url)
        .client_name(temporal_sdk_client_name())
        .client_version(temporal_sdk_client_version())
        .identity(format!("cortex-desktop-gateway-{}", std::process::id()))
        .build()
        .map_err(|err| err.to_string())?;
    let client = options
        .connect(namespace, None)
        .await
        .map_err(|err| format!("temporal_sdk_connect_failed: {}", err))?;
    let temporal_run_id = binding.temporal_run_id.clone().unwrap_or_default();
    let query = WorkflowQuery {
        query_type: TEMPORAL_WORKFLOW_QUERY_RUN_SNAPSHOT.to_string(),
        query_args: None,
        header: None,
    };
    let response = client
        .query_workflow_execution(binding.workflow_id.clone(), temporal_run_id, query)
        .await
        .map_err(|err| {
            format!(
                "temporal_sdk_query_failed run_id={}: {}",
                run.run.run_id, err
            )
        })?;
    let payloads = response.query_result.ok_or_else(|| {
        format!(
            "temporal_sdk_query_failed_missing_payload run_id={}",
            run.run.run_id
        )
    })?;
    let payload = payloads.payloads.into_iter().next().ok_or_else(|| {
        format!(
            "temporal_sdk_query_failed_empty_payload run_id={}",
            run.run.run_id
        )
    })?;
    let query_value = Value::from_json_payload(&payload)
        .or_else(|_| serde_json::from_slice::<Value>(&payload.data).map_err(|err| err.into()))
        .map_err(|err| {
            format!(
                "temporal_sdk_query_payload_decode_failed run_id={}: {}",
                run.run.run_id, err
            )
        })?;
    parse_temporal_snapshot_candidate(&query_value).ok_or_else(|| {
        format!(
            "temporal_sdk_query_failed_unparseable_snapshot run_id={}",
            run.run.run_id
        )
    })
}

#[cfg(not(feature = "temporal-sdk-native"))]
async fn query_temporal_sdk_workflow_snapshot_native(
    run: &AgentRunRecord,
    binding: &TemporalRunBinding,
) -> Result<TemporalBridgeRunSnapshot, String> {
    let _ = (run, binding);
    Err("temporal_sdk_native_unavailable_build_with_feature_temporal-sdk-native".to_string())
}

fn agent_run_status_from_str(raw: &str) -> AgentRunStatus {
    match raw.trim().to_ascii_lowercase().as_str() {
        "queued" => AgentRunStatus::Queued,
        "simulating" => AgentRunStatus::Simulating,
        "waiting_approval" => AgentRunStatus::WaitingApproval,
        "applying" => AgentRunStatus::Applying,
        "completed" => AgentRunStatus::Completed,
        "rejected" => AgentRunStatus::Rejected,
        "failed" => AgentRunStatus::Failed,
        _ => AgentRunStatus::Failed,
    }
}

fn is_terminal_status(status: &AgentRunStatus) -> bool {
    matches!(
        status,
        AgentRunStatus::Completed | AgentRunStatus::Rejected | AgentRunStatus::Failed
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum AgentRuntimeMode {
    GatewayPrimary,
    TemporalShadow,
    TemporalPrimary,
}

impl AgentRuntimeMode {
    fn as_str(self) -> &'static str {
        match self {
            Self::GatewayPrimary => "gateway_primary",
            Self::TemporalShadow => "temporal_shadow",
            Self::TemporalPrimary => "temporal_primary",
        }
    }
}

fn agent_runtime_mode() -> AgentRuntimeMode {
    match std::env::var("CORTEX_AGENT_RUNTIME_MODE")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("gateway_primary") => AgentRuntimeMode::GatewayPrimary,
        Some("temporal_primary") => AgentRuntimeMode::TemporalPrimary,
        Some("temporal_shadow") | None => AgentRuntimeMode::TemporalShadow,
        Some(_) => AgentRuntimeMode::TemporalShadow,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TemporalExecutionBackend {
    Bridge,
    Sdk,
}

fn temporal_execution_backend() -> TemporalExecutionBackend {
    match std::env::var("CORTEX_TEMPORAL_EXECUTION_BACKEND")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("sdk") => TemporalExecutionBackend::Sdk,
        Some("bridge") | None => TemporalExecutionBackend::Bridge,
        Some(_) => TemporalExecutionBackend::Bridge,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct TemporalPromotionGateDecision {
    generated_at: String,
    configured_mode: String,
    effective_mode: String,
    fallback_mode: String,
    min_runs: usize,
    max_age_hours: u64,
    observed_runs: usize,
    critical_count: usize,
    eligible: bool,
    reason: String,
}

fn temporal_promotion_burnin_min_runs() -> usize {
    std::env::var("CORTEX_TEMPORAL_BURNIN_MIN_RUNS")
        .ok()
        .and_then(|value| value.trim().parse::<usize>().ok())
        .unwrap_or(5)
        .max(1)
}

fn temporal_promotion_burnin_max_age_hours() -> u64 {
    std::env::var("CORTEX_TEMPORAL_BURNIN_MAX_AGE_HOURS")
        .ok()
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(24)
        .max(1)
}

fn temporal_primary_fallback_mode() -> AgentRuntimeMode {
    match std::env::var("CORTEX_TEMPORAL_PROMOTION_FALLBACK")
        .ok()
        .map(|value| value.trim().to_ascii_lowercase())
        .as_deref()
    {
        Some("gateway_primary") => AgentRuntimeMode::GatewayPrimary,
        Some("temporal_shadow") | None => AgentRuntimeMode::TemporalShadow,
        Some(_) => AgentRuntimeMode::TemporalShadow,
    }
}

fn temporal_shadow_metrics_dir() -> PathBuf {
    decision_surface_log_dir().join("metrics")
}

fn parse_shadow_critical_count(entry: &Value) -> usize {
    entry
        .get("shadowDivergence")
        .and_then(|value| value.get("criticalCount"))
        .and_then(|value| value.as_u64())
        .map(|value| value as usize)
        .or_else(|| {
            entry
                .get("shadowDivergence")
                .and_then(|value| value.get("divergences"))
                .and_then(|value| value.as_array())
                .map(|divergences| {
                    divergences
                        .iter()
                        .filter(|divergence| {
                            divergence
                                .get("severity")
                                .and_then(|value| value.as_str())
                                .map(|severity| severity.eq_ignore_ascii_case("critical"))
                                .unwrap_or(false)
                        })
                        .count()
                })
        })
        .unwrap_or(0)
}

fn parse_shadow_generated_at(entry: &Value) -> Option<DateTime<Utc>> {
    entry
        .get("generatedAt")
        .and_then(|value| value.as_str())
        .and_then(|raw| DateTime::parse_from_rfc3339(raw).ok())
        .map(|dt| dt.with_timezone(&Utc))
}

fn evaluate_temporal_promotion_gate(
    configured_mode: AgentRuntimeMode,
) -> TemporalPromotionGateDecision {
    let min_runs = temporal_promotion_burnin_min_runs();
    let max_age_hours = temporal_promotion_burnin_max_age_hours();
    let cutoff = Utc::now() - chrono::Duration::hours(max_age_hours as i64);
    let fallback_mode = temporal_primary_fallback_mode();

    let mut observed_runs = 0usize;
    let mut critical_count = 0usize;

    if let Ok(entries) = fs::read_dir(temporal_shadow_metrics_dir()) {
        for entry in entries.filter_map(|item| item.ok()) {
            let path = entry.path();
            let Some(file_name) = path.file_name().and_then(|value| value.to_str()) else {
                continue;
            };
            if !file_name.starts_with("agent_run_shadow_diff_") || !file_name.ends_with(".json") {
                continue;
            }
            let Ok(raw) = fs::read_to_string(path) else {
                continue;
            };
            let Ok(parsed) = serde_json::from_str::<Value>(&raw) else {
                continue;
            };
            let Some(generated_at) = parse_shadow_generated_at(&parsed) else {
                continue;
            };
            if generated_at < cutoff {
                continue;
            }
            observed_runs = observed_runs.saturating_add(1);
            critical_count = critical_count.saturating_add(parse_shadow_critical_count(&parsed));
        }
    }

    let (eligible, reason) = if observed_runs < min_runs {
        (
            false,
            format!(
                "burn_in_window_insufficient_runs: observed={} required={}",
                observed_runs, min_runs
            ),
        )
    } else if critical_count > 0 {
        (
            false,
            format!(
                "critical_divergence_detected: critical_count={} window_hours={}",
                critical_count, max_age_hours
            ),
        )
    } else {
        (true, "promotion_gate_passed_zero_critical".to_string())
    };

    let effective_mode = if configured_mode == AgentRuntimeMode::TemporalPrimary && !eligible {
        fallback_mode
    } else {
        configured_mode
    };

    TemporalPromotionGateDecision {
        generated_at: now_iso(),
        configured_mode: configured_mode.as_str().to_string(),
        effective_mode: effective_mode.as_str().to_string(),
        fallback_mode: fallback_mode.as_str().to_string(),
        min_runs,
        max_age_hours,
        observed_runs,
        critical_count,
        eligible,
        reason,
    }
}

fn persist_temporal_promotion_gate_decision(decision: &TemporalPromotionGateDecision) {
    let path = decision_surface_log_dir().join("metrics").join(format!(
        "temporal_promotion_gate_{}.json",
        Utc::now().timestamp_millis()
    ));
    let _ = persist_json(
        &path,
        &serde_json::to_value(decision).unwrap_or_else(|_| json!(null)),
    );
}

fn effective_runtime_mode_for_run() -> AgentRuntimeMode {
    let configured = agent_runtime_mode();
    if configured != AgentRuntimeMode::TemporalPrimary {
        return configured;
    }
    let decision = evaluate_temporal_promotion_gate(configured);
    persist_temporal_promotion_gate_decision(&decision);
    if decision.eligible {
        AgentRuntimeMode::TemporalPrimary
    } else {
        temporal_primary_fallback_mode()
    }
}

fn projection_mode_label(
    mode: AgentRuntimeMode,
    backend: TemporalExecutionBackend,
) -> &'static str {
    match (mode, backend) {
        (AgentRuntimeMode::GatewayPrimary, _) => "gateway_primary",
        (AgentRuntimeMode::TemporalShadow, TemporalExecutionBackend::Bridge) => {
            "temporal_bridge_shadow"
        }
        (AgentRuntimeMode::TemporalPrimary, TemporalExecutionBackend::Bridge) => {
            "temporal_bridge_primary"
        }
        (AgentRuntimeMode::TemporalShadow, TemporalExecutionBackend::Sdk) => "temporal_sdk_shadow",
        (AgentRuntimeMode::TemporalPrimary, TemporalExecutionBackend::Sdk) => {
            "temporal_sdk_primary"
        }
    }
}

fn temporal_backend_from_projection_mode(
    projection_mode: Option<&str>,
) -> TemporalExecutionBackend {
    match projection_mode {
        Some(mode) if mode.starts_with("temporal_sdk_") => TemporalExecutionBackend::Sdk,
        Some(mode) if mode.starts_with("temporal_bridge_") => TemporalExecutionBackend::Bridge,
        _ => temporal_execution_backend(),
    }
}

fn run_status_name(status: &AgentRunStatus) -> &'static str {
    match status {
        AgentRunStatus::Queued => "queued",
        AgentRunStatus::Simulating => "simulating",
        AgentRunStatus::WaitingApproval => "waiting_approval",
        AgentRunStatus::Applying => "applying",
        AgentRunStatus::Completed => "completed",
        AgentRunStatus::Rejected => "rejected",
        AgentRunStatus::Failed => "failed",
    }
}

fn emit_agent_event(
    state: &GatewayState,
    record: &mut AgentRunRecord,
    event_type: &str,
    payload: Value,
) {
    let timestamp = now_iso();
    let sequence = record
        .events
        .iter()
        .map(|event| event.sequence)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    let event = AgentRunEvent {
        event_type: event_type.to_string(),
        run_id: record.run.run_id.clone(),
        space_id: record.run.space_id.clone(),
        timestamp: timestamp.clone(),
        sequence,
        payload,
    };
    record.run.updated_at = timestamp;
    record.events.push(event.clone());
    if let Ok(serialized) = serde_json::to_string(&event) {
        let _ = state.broadcast_tx.send(Message::Text(serialized));
    }
}

fn agent_intent_to_action_target(intent: AgentIntent) -> ActionTarget {
    match intent {
        AgentIntent::CreateContextNode { node_id, content } => ActionTarget {
            protocol: "ic".to_string(),
            address: "kg-canister".to_string(),
            method: "create_context_node".to_string(),
            payload: serde_json::to_vec(&json!({
                "nodeId": node_id,
                "content": content
            }))
            .unwrap_or_default(),
        },
        AgentIntent::ProposeSchemaMutation { schema_json } => ActionTarget {
            protocol: "ic".to_string(),
            address: "kg-canister".to_string(),
            method: "propose_schema_mutation".to_string(),
            payload: serde_json::to_vec(&json!({
                "schemaJson": schema_json
            }))
            .unwrap_or_default(),
        },
        AgentIntent::ExecuteSimulation { scenario_id } => ActionTarget {
            protocol: "ic".to_string(),
            address: "simulation-canister".to_string(),
            method: "execute_simulation".to_string(),
            payload: serde_json::to_vec(&json!({ "scenarioId": scenario_id })).unwrap_or_default(),
        },
        AgentIntent::ApplyActionTarget { action_target } => action_target,
    }
}

fn simulation_action_from_action_target(target: &ActionTarget) -> DomainSimulationAction {
    let payload_json =
        serde_json::from_slice::<Value>(&target.payload).unwrap_or_else(|_| json!({}));
    match target.method.as_str() {
        "create_context_node" => {
            let node_id = payload_json
                .get("nodeId")
                .and_then(|value| value.as_str())
                .unwrap_or("context_node")
                .to_string();
            let content = payload_json
                .get("content")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let mut attributes = BTreeMap::new();
            attributes.insert("content".to_string(), content);
            DomainSimulationAction::AddNode {
                node_id,
                node_type: "context_node".to_string(),
                attributes,
            }
        }
        "propose_schema_mutation" => {
            let schema_json = payload_json
                .get("schemaJson")
                .and_then(|value| value.as_str())
                .unwrap_or("")
                .to_string();
            let mut attributes = BTreeMap::new();
            attributes.insert("schema_json".to_string(), schema_json);
            DomainSimulationAction::AddNode {
                node_id: format!("schema_mutation_{}", sanitize_fs_component(&target.address)),
                node_type: "schema_mutation_proposal".to_string(),
                attributes,
            }
        }
        "execute_simulation" => DomainSimulationAction::SubmitProposal {
            proposal_type: "execute_simulation".to_string(),
            payload: String::from_utf8_lossy(&target.payload).to_string(),
        },
        _ => DomainSimulationAction::AgentAction {
            target: serde_json::to_string(&json!({
                "protocol": target.protocol,
                "address": target.address,
                "method": target.method,
                "payload": payload_json
            }))
            .unwrap_or_else(|_| "{}".to_string()),
        },
    }
}

fn build_agent_simulation_scenario(
    scenario_id: &str,
    action: &DomainSimulationAction,
) -> DomainScenarioDefinition {
    let round_action = match action {
        DomainSimulationAction::AddNode {
            node_id,
            node_type,
            attributes,
        } => DomainScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "add_node".to_string(),
            node_id: Some(node_id.clone()),
            node_type: Some(node_type.clone()),
            attributes: Some(attributes.clone()),
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: None,
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: None,
        },
        DomainSimulationAction::SubmitProposal {
            proposal_type,
            payload,
        } => DomainScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "submit_proposal".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: Some(proposal_type.clone()),
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: serde_json::from_str(payload).ok(),
        },
        _ => DomainScenarioRoundAction {
            actor: "systems_architect".to_string(),
            action: "submit_proposal".to_string(),
            node_id: None,
            node_type: None,
            attributes: None,
            source: None,
            target: None,
            edge_kind: None,
            proposal_type: Some("agent_action".to_string()),
            proposal: None,
            choice: None,
            key: None,
            value: None,
            payload: serde_json::from_str(
                &serde_json::to_string(action).unwrap_or_else(|_| "{}".to_string()),
            )
            .ok(),
        },
    };

    DomainScenarioDefinition {
        scenario: DomainScenarioMetadata {
            id: scenario_id.to_string(),
            name: format!("Agent run {scenario_id}"),
            seed: now_secs(),
            commons_version: "nostra-core-v0".to_string(),
            siqs_version: "1.0.0".to_string(),
        },
        constraints: DomainScenarioConstraints {
            max_mutations: 4,
            max_rounds: 2,
            max_runtime_ms: 1_000,
        },
        rounds: vec![DomainScenarioRound {
            round: 1,
            actions: vec![round_action],
        }],
    }
}

fn evaluate_agent_plan(
    run_id: &str,
    contribution_id: &str,
) -> Result<(AgentSimulationEvaluation, ActionTarget), String> {
    let intent = AgentIntent::CreateContextNode {
        node_id: format!("ctx_{}", sanitize_fs_component(contribution_id)),
        content: format!("Proposed contribution: {contribution_id}"),
    };
    let action_target = agent_intent_to_action_target(intent);
    let action = simulation_action_from_action_target(&action_target);
    let scenario = build_agent_simulation_scenario(&format!("sim-{run_id}"), &action);

    let mut base_graph = DomainGraph::default();
    base_graph.add_node(DomainNode {
        id: "space:agent-runtime".to_string(),
        node_type: "space".to_string(),
        attributes: BTreeMap::new(),
    });

    let rules = vec![
        DomainIntegrityRule {
            id: "rule-space-exists".to_string(),
            name: "Space root exists".to_string(),
            description: "Simulation requires a space root.".to_string(),
            scope: DomainIntegrityScope::Global,
            predicate: DomainIntegrityPredicate {
                target: DomainNodeSelector {
                    entity_type: Some("space".to_string()),
                    tags: None,
                },
                relation: None,
                constraint: DomainConstraint::MustExist,
            },
            severity: DomainSeverity::Critical,
            remediation_hint: None,
        },
        DomainIntegrityRule {
            id: "rule-context-node-dependency".to_string(),
            name: "Context node has dependency".to_string(),
            description: "Context nodes should maintain dependency lineage.".to_string(),
            scope: DomainIntegrityScope::Global,
            predicate: DomainIntegrityPredicate {
                target: DomainNodeSelector {
                    entity_type: Some("context_node".to_string()),
                    tags: None,
                },
                relation: Some(DomainEdgeSelector {
                    edge_kind: DomainEdgeKind::DependsOn,
                    direction: DomainDirection::Outgoing,
                }),
                constraint: DomainConstraint::MinCount(1),
            },
            severity: DomainSeverity::Warning,
            remediation_hint: None,
        },
    ];

    let session = run_domain_session(&base_graph, &rules, &scenario);
    let result = session
        .result
        .ok_or_else(|| "simulation produced no result".to_string())?;
    let risk_score = result.violation_summary.risk_score;
    let siqs_score = (100.0_f32 - (risk_score as f32 * 4.0)).clamp(0.0, 100.0);
    let evaluation = AgentSimulationEvaluation {
        success: !result.aborted
            && result.violation_summary.critical == 0
            && result.violation_summary.violation == 0,
        violation_count: result.violations.len(),
        risk_score,
        siqs_score,
        session_id: session.session_id,
        structural_diff_summary: format!(
            "nodes_added={} nodes_removed={} edges_added={} edges_removed={} attrs_changed={}",
            result.structural_diff.nodes_added.len(),
            result.structural_diff.nodes_removed.len(),
            result.structural_diff.edges_added.len(),
            result.structural_diff.edges_removed.len(),
            result.structural_diff.attributes_changed.len(),
        ),
    };
    Ok((evaluation, action_target))
}

fn build_a2ui_surface_payload(
    space_id: &str,
    run_id: &str,
    simulation: &AgentSimulationEvaluation,
) -> Value {
    json!({
        "type": "Container",
        "children": {
            "explicitList": [
                {
                    "id": "sim-header",
                    "componentProperties": {
                        "Heading": {
                            "text": format!("GSMS Evaluation For Space: {}", space_id)
                        }
                    }
                },
                {
                    "id": "sim-spatial-plane",
                    "componentProperties": {
                        "SpatialPlane": {
                            "plane_id": format!("sp-{}", run_id),
                            "surface_class": "execution",
                            "focus_bounds": { "x": 0, "y": 0, "w": 1100, "h": 620 },
                            "commands": [
                                {
                                    "op": "create_shape",
                                    "shape": {
                                        "id": "shape-1",
                                        "kind": "note",
                                        "x": 120,
                                        "y": 110,
                                        "w": 200,
                                        "h": 90,
                                        "text": "contribution intent"
                                    }
                                },
                                {
                                    "op": "create_shape",
                                    "shape": {
                                        "id": "shape-2",
                                        "kind": "note",
                                        "x": 430,
                                        "y": 250,
                                        "w": 220,
                                        "h": 95,
                                        "text": "deterministic replay"
                                    }
                                },
                                {
                                    "op": "create_shape",
                                    "shape": {
                                        "id": "shape-3",
                                        "kind": "note",
                                        "x": 760,
                                        "y": 120,
                                        "w": 220,
                                        "h": 95,
                                        "text": "approval gate"
                                    }
                                },
                                {
                                    "op": "create_shape",
                                    "shape": {
                                        "id": "edge-1",
                                        "kind": "arrow",
                                        "x": 320,
                                        "y": 150,
                                        "to_x": 430,
                                        "to_y": 250
                                    }
                                },
                                {
                                    "op": "create_shape",
                                    "shape": {
                                        "id": "edge-2",
                                        "kind": "arrow",
                                        "x": 650,
                                        "y": 300,
                                        "to_x": 760,
                                        "to_y": 165
                                    }
                                }
                            ]
                        }
                    }
                },
                {
                    "id": "sim-diff",
                    "componentProperties": {
                        "DiffViewer": {
                            "diffText": format!(
                                "+ session={}\\n+ {}\\n+ violations={}\\n+ risk={}\\n+ SIQS={:.2}",
                                simulation.session_id,
                                simulation.structural_diff_summary,
                                simulation.violation_count,
                                simulation.risk_score,
                                simulation.siqs_score
                            )
                        }
                    }
                },
                {
                    "id": "sim-approval",
                    "componentProperties": {
                        "ApprovalControls": {
                            "spaceId": space_id,
                            "runId": run_id,
                            "scenarioId": simulation.session_id,
                            "decisionRef": format!("DEC-{}", sanitize_fs_component(run_id))
                        }
                    }
                }
            ]
        }
    })
}

fn persist_approval_bridge_record(
    run: &AgentRunRecord,
    approval: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    let action_id = format!("decision_ack_{}", sanitize_fs_component(&run.run.run_id));
    let record = json!({
        "schemaVersion": "1.0.0",
        "actionId": action_id,
        "action": "ack",
        "decisionGateId": format!("agent_run_gate:{}", run.run.run_id),
        "workflowId": run.run.workflow_id,
        "mutationId": run.run.run_id,
        "actionTarget": "agent_contribution_apply",
        "riskStatement": format!("riskScore={}", run.run.simulation.as_ref().and_then(|value| value.get("riskScore")).cloned().unwrap_or(json!(0))),
        "rollbackPath": "operator_rejects_and_run_stops_before_apply",
        "evidenceRefs": [
            format!("agent_runs/{}.json", run.run.run_id)
        ],
        "lineageId": format!("lineage:{}", run.run.run_id),
        "policyRef": "policy:agent_contribution_approval_bridge",
        "actorRef": approval.actor,
        "note": approval.rationale,
        "createdAt": now_iso()
    });
    let path = decision_actions_dir().join(format!("{}.json", action_id));
    persist_json(&path, &record)
}

fn agent_authority_dir() -> PathBuf {
    decision_surface_log_dir().join("authority")
}

fn agent_proposals_dir() -> PathBuf {
    decision_surface_log_dir().join("agent_proposals")
}

fn agent_replay_dir() -> PathBuf {
    decision_surface_log_dir().join("agent_replay")
}

fn persist_agent_recommendation_record(
    run: &AgentRunRecord,
    action_target: &ActionTarget,
    reason: &str,
    detail: Option<Value>,
) -> Result<String, String> {
    let recommendation_id = format!(
        "agent_recommendation_{}_{}",
        sanitize_fs_component(&run.run.run_id),
        Utc::now().timestamp_millis()
    );
    let path = decision_actions_dir().join(format!("{}.json", recommendation_id));
    let payload = json!({
        "schemaVersion": "1.0.0",
        "recommendationId": recommendation_id,
        "createdAt": now_iso(),
        "spaceId": run.run.space_id,
        "runId": run.run.run_id,
        "workflowId": run.run.workflow_id,
        "agentId": run.run.agent_id.clone(),
        "executionId": run.run.execution_id,
        "attemptId": run.run.attempt_id,
        "authorityLevel": run.run.authority_level.unwrap_or_default().as_str(),
        "reason": reason,
        "detail": detail,
        "actionTarget": action_target
    });
    persist_json(&path, &payload)?;
    Ok(recommendation_id)
}

fn persist_agent_proposal_record(
    run: &AgentRunRecord,
    action_target: &ActionTarget,
    approval_signal: &AgentApprovalSignal,
) -> Result<String, String> {
    let proposal_id = format!(
        "agent_proposal_{}_{}",
        sanitize_fs_component(&run.run.run_id),
        Utc::now().timestamp_millis()
    );
    let path = agent_proposals_dir().join(format!("{}.json", proposal_id));
    let payload = json!({
        "schemaVersion": "1.0.0",
        "proposalId": proposal_id,
        "createdAt": now_iso(),
        "spaceId": run.run.space_id,
        "runId": run.run.run_id,
        "workflowId": run.run.workflow_id,
        "agentId": run.run.agent_id.clone(),
        "executionId": run.run.execution_id,
        "attemptId": run.run.attempt_id,
        "authorityLevel": "l1",
        "decisionRef": approval_signal.decision_ref,
        "actor": approval_signal.actor,
        "rationale": approval_signal.rationale,
        "actionTarget": action_target
    });
    persist_json(&path, &payload)?;
    Ok(proposal_id)
}

async fn lookup_replay_contract_refs(run: &AgentRunRecord) -> (Option<String>, Option<String>) {
    let Ok(client) = WorkflowEngineClient::from_env() else {
        return (None, None);
    };
    match client.get_replay_contract(run.run.run_id.as_str()).await {
        Ok(Some(contract)) => (
            Some(format!("replay_contract:{}", contract.mutation_id)),
            contract.lineage_id,
        ),
        _ => (None, None),
    }
}

async fn persist_agent_replay_artifact(
    run: &AgentRunRecord,
    action_target: &Option<ActionTarget>,
) -> Result<(String, String, Option<String>, Option<String>), String> {
    let input_payload = match action_target {
        Some(target) => Some(serde_json::to_value(target).map_err(|err| err.to_string())?),
        None => None,
    };
    let input_snapshot_hash = hash_json_value(&input_payload);
    let output_payload = json!({
        "simulation": run.run.simulation.clone(),
        "surfaceUpdate": run.run.surface_update.clone(),
        "authorityOutcome": run.run.authority_outcome.clone(),
    });
    let output_snapshot_hash = hash_json_value(&Some(output_payload));
    let (replay_contract_ref, lineage_id) = lookup_replay_contract_refs(run).await;

    let execution_id = run
        .run
        .execution_id
        .clone()
        .unwrap_or_else(|| next_execution_id(&run.run.run_id));
    let attempt_id = run
        .run
        .attempt_id
        .clone()
        .unwrap_or_else(|| next_attempt_id(execution_id.as_str()));
    let path = agent_replay_dir().join(format!(
        "{}__{}.json",
        sanitize_fs_component(&execution_id),
        sanitize_fs_component(&attempt_id)
    ));
    let payload = json!({
        "schemaVersion": "1.0.0",
        "capturedAt": now_iso(),
        "spaceId": run.run.space_id,
        "runId": run.run.run_id,
        "workflowId": run.run.workflow_id,
        "agentId": run.run.agent_id.clone(),
        "executionId": execution_id,
        "attemptId": attempt_id,
        "authorityLevel": run.run.authority_level.unwrap_or_default().as_str(),
        "inputSnapshotHash": input_snapshot_hash,
        "outputSnapshotHash": output_snapshot_hash,
        "modelFingerprint": std::env::var("NOSTRA_AGENT_MODEL").ok(),
        "toolStateHash": hash_json_value(&Some(json!({
            "eventCount": run.events.len(),
            "latestEventType": run.events.last().map(|event| event.event_type.clone()),
        }))),
        "replayContractRef": replay_contract_ref,
        "lineageId": lineage_id,
    });
    persist_json(&path, &payload)?;

    Ok((
        input_snapshot_hash,
        output_snapshot_hash,
        replay_contract_ref,
        lineage_id,
    ))
}

async fn emit_execution_lifecycle(
    run: &AgentRunRecord,
    phase: AgentExecutionPhase,
    status: &str,
    action_target: &Option<ActionTarget>,
    persist_replay: bool,
) -> Result<(), String> {
    let (input_snapshot_hash, output_snapshot_hash, replay_contract_ref, lineage_id) =
        if persist_replay {
            persist_agent_replay_artifact(run, action_target).await?
        } else {
            let input_payload = match action_target {
                Some(target) => Some(serde_json::to_value(target).map_err(|err| err.to_string())?),
                None => None,
            };
            (
                hash_json_value(&input_payload),
                hash_json_value(&run.run.surface_update),
                None,
                None,
            )
        };

    let authority_scope = run.run.authority_level.unwrap_or_default();
    let record = AgentExecutionRecord {
        schema_version: "1.0.0".to_string(),
        execution_id: run
            .run
            .execution_id
            .clone()
            .unwrap_or_else(|| next_execution_id(&run.run.run_id)),
        attempt_id: run
            .run
            .attempt_id
            .clone()
            .unwrap_or_else(|| next_attempt_id(&run.run.run_id)),
        agent_id: run
            .run
            .agent_id
            .clone()
            .unwrap_or_else(default_agent_identity),
        workflow_id: run.run.workflow_id.clone(),
        phase,
        status: status.to_string(),
        authority_scope,
        input_snapshot_hash,
        output_snapshot_hash,
        timestamp: now_iso(),
        space_id: Some(run.run.space_id.clone()),
        model_fingerprint: std::env::var("NOSTRA_AGENT_MODEL").ok(),
        tool_state_hash: Some(hash_json_value(&Some(json!({
            "eventCount": run.events.len(),
            "latestEventType": run.events.last().map(|event| event.event_type.clone()),
        })))),
        confidence: run
            .run
            .simulation
            .as_ref()
            .and_then(|value| value.get("siqsScore"))
            .and_then(|value| value.as_f64()),
        promotion_level: Some(authority_scope.as_str().to_string()),
        started_at: Some(run.run.started_at.clone()),
        ended_at: if matches!(
            run.run.status,
            AgentRunStatus::Completed | AgentRunStatus::Rejected | AgentRunStatus::Failed
        ) {
            Some(now_iso())
        } else {
            None
        },
        replay_contract_ref,
        lineage_id,
        evidence_refs: vec![format!("agent_runs/{}.json", run.run.run_id)],
    };

    emit_agent_execution_record(decision_surface_log_dir().as_path(), &record).await
}

async fn apply_authority_guard(
    run: &AgentRunRecord,
    action_target: &ActionTarget,
    approval_signal: &AgentApprovalSignal,
) -> Result<AuthorityExecutionOutcome, String> {
    let authority_level = run.run.authority_level.unwrap_or_default();
    if !authority_level.is_v1_supported() {
        let recommendation_id = persist_agent_recommendation_record(
            run,
            action_target,
            "authority_level_not_supported_v1",
            Some(json!({ "authorityLevel": authority_level.as_str() })),
        )?;
        return Ok(AuthorityExecutionOutcome {
            accepted: false,
            action_target: action_target.clone(),
            applied_at: now_iso(),
            host_receipt: Some(recommendation_id),
            error: Some("v1_fail_closed_l3_l4".to_string()),
        });
    }

    if authority_level == AuthorityLevel::L0 {
        let recommendation_id = persist_agent_recommendation_record(
            run,
            action_target,
            "authority_l0_recommendation_only",
            None,
        )?;
        return Ok(AuthorityExecutionOutcome {
            accepted: false,
            action_target: action_target.clone(),
            applied_at: now_iso(),
            host_receipt: Some(recommendation_id),
            error: Some("l0_recommendation_only".to_string()),
        });
    }

    if authority_level == AuthorityLevel::L1 {
        let proposal_id = persist_agent_proposal_record(run, action_target, approval_signal)?;
        return Ok(AuthorityExecutionOutcome {
            accepted: false,
            action_target: action_target.clone(),
            applied_at: now_iso(),
            host_receipt: Some(proposal_id),
            error: Some("l1_proposal_created".to_string()),
        });
    }

    let actor_principal = match approval_signal
        .actor_principal
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        Some(value) => match Principal::from_text(value) {
            Ok(principal) => principal,
            Err(err) => {
                let recommendation_id = persist_agent_recommendation_record(
                    run,
                    action_target,
                    "l2_invalid_actor_principal",
                    Some(json!({ "reason": err.to_string() })),
                )?;
                return Ok(AuthorityExecutionOutcome {
                    accepted: false,
                    action_target: action_target.clone(),
                    applied_at: now_iso(),
                    host_receipt: Some(recommendation_id),
                    error: Some("l2_invalid_actor_principal".to_string()),
                });
            }
        },
        None => {
            let recommendation_id = persist_agent_recommendation_record(
                run,
                action_target,
                "l2_missing_actor_principal",
                None,
            )?;
            return Ok(AuthorityExecutionOutcome {
                accepted: false,
                action_target: action_target.clone(),
                applied_at: now_iso(),
                host_receipt: Some(recommendation_id),
                error: Some("l2_missing_actor_principal".to_string()),
            });
        }
    };

    #[cfg(test)]
    let l2_scope_override = test_override_agent_l2_scope_evaluation();
    #[cfg(not(test))]
    let l2_scope_override: Option<Result<ActionScopeEvaluation, String>> = None;

    let scope = if let Some(scope_result) = l2_scope_override {
        match scope_result {
            Ok(scope) => scope,
            Err(err) => {
                let recommendation_id = persist_agent_recommendation_record(
                    run,
                    action_target,
                    "l2_governance_scope_evaluation_failed",
                    Some(json!({ "reason": err })),
                )?;
                return Ok(AuthorityExecutionOutcome {
                    accepted: false,
                    action_target: action_target.clone(),
                    applied_at: now_iso(),
                    host_receipt: Some(recommendation_id),
                    error: Some("l2_governance_scope_evaluation_failed".to_string()),
                });
            }
        }
    } else {
        let governance = match GovernanceClient::from_env() {
            Ok(client) => client,
            Err(err) => {
                let recommendation_id = persist_agent_recommendation_record(
                    run,
                    action_target,
                    "l2_governance_client_unavailable",
                    Some(json!({ "reason": err })),
                )?;
                return Ok(AuthorityExecutionOutcome {
                    accepted: false,
                    action_target: action_target.clone(),
                    applied_at: now_iso(),
                    host_receipt: Some(recommendation_id),
                    error: Some("l2_governance_client_unavailable".to_string()),
                });
            }
        };
        match governance
            .evaluate_action_scope_with_actor(
                run.run.space_id.as_str(),
                "agent_contribution_apply",
                "attributed",
                "release_blocker",
                &actor_principal,
            )
            .await
        {
            Ok(scope) => scope,
            Err(err) => {
                let recommendation_id = persist_agent_recommendation_record(
                    run,
                    action_target,
                    "l2_governance_scope_evaluation_failed",
                    Some(json!({ "reason": err })),
                )?;
                return Ok(AuthorityExecutionOutcome {
                    accepted: false,
                    action_target: action_target.clone(),
                    applied_at: now_iso(),
                    host_receipt: Some(recommendation_id),
                    error: Some("l2_governance_scope_evaluation_failed".to_string()),
                });
            }
        }
    };
    if !scope.allowed || scope.gate_decision.eq_ignore_ascii_case("block") {
        let recommendation_id = persist_agent_recommendation_record(
            run,
            action_target,
            "l2_governance_scope_blocked",
            Some(json!({
                "reason": scope.reason,
                "gateDecision": scope.gate_decision,
                "requiredActions": scope.required_actions,
                "policyRef": scope.policy_ref,
                "policyVersion": scope.policy_version,
            })),
        )?;
        return Ok(AuthorityExecutionOutcome {
            accepted: false,
            action_target: action_target.clone(),
            applied_at: now_iso(),
            host_receipt: Some(recommendation_id),
            error: Some("l2_governance_blocked".to_string()),
        });
    }

    let evaluation = evaluate_agent_gate(&cortex_runtime::ports::AgentEvaluationLoopRequest {
        run_id: run.run.run_id.clone(),
        workflow_id: run.run.workflow_id.clone(),
        space_id: run.run.space_id.clone(),
        authority_level: authority_level.as_str().to_string(),
        simulation: run.run.simulation.clone(),
        action_target: Some(format!(
            "{}://{}/{}",
            action_target.protocol, action_target.address, action_target.method
        )),
    })
    .await;
    if !evaluation.allowed {
        let recommendation_id = persist_agent_recommendation_record(
            run,
            action_target,
            "l2_evaluation_loop_blocked",
            Some(json!({
                "gateOutcome": evaluation.gate_outcome,
                "reasons": evaluation.reasons,
                "confidenceScore": evaluation.confidence_score,
                "sourceReliability": evaluation.source_reliability
            })),
        )?;
        return Ok(AuthorityExecutionOutcome {
            accepted: false,
            action_target: action_target.clone(),
            applied_at: now_iso(),
            host_receipt: Some(recommendation_id),
            error: Some("l2_evaluation_loop_blocked".to_string()),
        });
    }

    let applied_at = now_iso();
    let receipt = format!(
        "authority:{}:{}:{}",
        sanitize_fs_component(&run.run.space_id),
        sanitize_fs_component(&run.run.run_id),
        Utc::now().timestamp_millis()
    );
    let outcome = AuthorityExecutionOutcome {
        accepted: true,
        action_target: action_target.clone(),
        applied_at: applied_at.clone(),
        host_receipt: Some(receipt.clone()),
        error: None,
    };
    let record = json!({
        "schemaVersion": "1.0.0",
        "runId": run.run.run_id,
        "spaceId": run.run.space_id,
        "appliedAt": applied_at,
        "receipt": receipt,
        "authorityLevel": authority_level.as_str(),
        "actionTarget": action_target
    });
    let path =
        agent_authority_dir().join(format!("{}.json", sanitize_fs_component(&run.run.run_id)));
    persist_json(&path, &record)?;
    Ok(outcome)
}

fn persist_agent_run_shadow_comparison(run: &AgentRunRecord) -> Result<(), String> {
    let projection_mode = run
        .run
        .temporal_binding
        .as_ref()
        .and_then(|binding| binding.projection_mode.clone())
        .unwrap_or_else(|| "gateway_primary".to_string());
    if !projection_mode.ends_with("_shadow") {
        return Ok(());
    }
    let shadow_summary = run.run.shadow_summary.as_ref();
    let (shadow_status, divergence_payload, critical_count) = if let Some(summary) = shadow_summary
    {
        (
            summary.status.clone(),
            serde_json::to_value(summary).unwrap_or_else(|_| json!(null)),
            summary.critical_count,
        )
    } else {
        ("pending_temporal_bridge".to_string(), json!(null), 0)
    };
    let payload = json!({
        "schemaVersion": "1.0.0",
        "generatedAt": now_iso(),
        "runtimeMode": projection_mode,
        "runId": run.run.run_id,
        "spaceId": run.run.space_id,
        "status": run_status_name(&run.run.status),
        "eventCount": run.events.len(),
        "latestEventType": run.events.last().map(|event| event.event_type.clone()),
        "latestEventSequence": run.events.last().map(|event| event.sequence),
        "shadowStatus": shadow_status,
        "shadowDivergence": divergence_payload,
        "promotionPolicy": "strict_zero_critical",
        "promotionEligible": critical_count == 0
    });
    let path = decision_surface_log_dir().join("metrics").join(format!(
        "agent_run_shadow_diff_{}.json",
        sanitize_fs_component(&run.run.run_id)
    ));
    persist_json(&path, &payload)
}

fn persist_temporal_approval_signal(
    run: &AgentRunRecord,
    approval: &AgentContributionApprovalRequest,
) -> Result<(), String> {
    let Some(binding) = run.run.temporal_binding.as_ref() else {
        return Ok(());
    };
    if binding.status.as_deref() == Some("start_failed") {
        return Ok(());
    }
    let projection_mode = binding
        .projection_mode
        .as_deref()
        .unwrap_or("gateway_primary");
    if matches!(projection_mode, "gateway_primary" | "gateway_fallback") {
        return Ok(());
    }
    let payload = json!({
        "schemaVersion": "1.0.0",
        "runtimeMode": projection_mode,
        "recordedAt": now_iso(),
        "runId": run.run.run_id,
        "workflowId": run.run.workflow_id,
        "scenarioId": format!("sim-{}", run.run.contribution_id),
        "spaceId": run.run.space_id,
        "decision": approval.decision,
        "rationale": approval.rationale,
        "actor": approval.actor,
        "decisionRef": approval.decision_ref,
        "actorPrincipal": approval.actor_principal
    });
    let signal_dir = decision_surface_log_dir().join("temporal_signals");
    let run_path = signal_dir.join(format!("{}.json", sanitize_fs_component(&run.run.run_id)));
    persist_json(&run_path, &payload)?;
    let scenario_path = signal_dir.join(format!(
        "scenario__{}.json",
        sanitize_fs_component(&format!("sim-{}", run.run.contribution_id))
    ));
    persist_json(&scenario_path, &payload)
}

fn register_temporal_projector(
    state: &GatewayState,
    run_id: &str,
    handle: tokio::task::JoinHandle<()>,
) {
    if let Ok(mut projectors) = state.temporal_projectors.lock() {
        if let Some(existing) = projectors.remove(run_id) {
            existing.abort();
        }
        projectors.insert(run_id.to_string(), handle);
    }
}

fn unregister_temporal_projector(state: &GatewayState, run_id: &str) {
    if let Ok(mut projectors) = state.temporal_projectors.lock() {
        projectors.remove(run_id);
    }
}

fn value_hash(value: &Option<Value>) -> Option<String> {
    let serialized = serde_json::to_vec(value.as_ref()?).ok()?;
    let mut hasher = Sha256::new();
    hasher.update(serialized);
    Some(hex::encode(hasher.finalize()))
}

fn compare_gateway_vs_temporal(
    run: &AgentRunRecord,
    temporal: &TemporalBridgeRunSnapshot,
) -> ShadowComparisonSummary {
    let mut divergences = Vec::<ShadowDivergenceRecord>::new();
    let expected_status = run_status_name(&run.run.status).to_string();
    if expected_status != temporal.status {
        divergences.push(ShadowDivergenceRecord {
            severity: ShadowDivergenceSeverity::Critical,
            code: "status_mismatch".to_string(),
            message: "Terminal lifecycle status differs between gateway and temporal projection."
                .to_string(),
            expected: Some(json!(expected_status)),
            actual: Some(json!(temporal.status)),
        });
    }
    if is_terminal_status(&run.run.status) != temporal.terminal {
        divergences.push(ShadowDivergenceRecord {
            severity: ShadowDivergenceSeverity::Critical,
            code: "terminal_mismatch".to_string(),
            message: "Terminal state flag differs between gateway and temporal projection."
                .to_string(),
            expected: Some(json!(is_terminal_status(&run.run.status))),
            actual: Some(json!(temporal.terminal)),
        });
    }

    let gateway_authority_present = run.run.authority_outcome.is_some();
    let temporal_authority_present = temporal.authority_outcome.is_some();
    if gateway_authority_present != temporal_authority_present {
        divergences.push(ShadowDivergenceRecord {
            severity: ShadowDivergenceSeverity::Critical,
            code: "authority_presence_mismatch".to_string(),
            message:
                "Mutation authority outcome presence differs between gateway and temporal paths."
                    .to_string(),
            expected: Some(json!(gateway_authority_present)),
            actual: Some(json!(temporal_authority_present)),
        });
    }
    if let (Some(left), Some(right)) = (&run.run.authority_outcome, &temporal.authority_outcome) {
        if left.accepted != right.accepted {
            divergences.push(ShadowDivergenceRecord {
                severity: ShadowDivergenceSeverity::Critical,
                code: "authority_acceptance_mismatch".to_string(),
                message: "Authority acceptance differs across paths.".to_string(),
                expected: Some(json!(left.accepted)),
                actual: Some(json!(right.accepted)),
            });
        }
    }

    let gateway_siqs = run
        .run
        .simulation
        .as_ref()
        .and_then(|value| value.get("siqsScore"))
        .and_then(|value| value.as_f64());
    let temporal_siqs = temporal
        .simulation
        .as_ref()
        .and_then(|value| value.get("siqsScore"))
        .and_then(|value| value.as_f64());
    if let (Some(left), Some(right)) = (gateway_siqs, temporal_siqs) {
        let delta = (left - right).abs();
        if delta > 0.5 {
            divergences.push(ShadowDivergenceRecord {
                severity: ShadowDivergenceSeverity::Warning,
                code: "siqs_delta".to_string(),
                message: "SIQS drift exceeded tolerance (0.5).".to_string(),
                expected: Some(json!(left)),
                actual: Some(json!(right)),
            });
        }
    }

    let gateway_surface_hash = value_hash(&run.run.surface_update);
    let temporal_surface_hash = value_hash(&temporal.surface_update);
    if gateway_surface_hash.is_some()
        && temporal_surface_hash.is_some()
        && gateway_surface_hash != temporal_surface_hash
    {
        divergences.push(ShadowDivergenceRecord {
            severity: ShadowDivergenceSeverity::Warning,
            code: "surface_hash_mismatch".to_string(),
            message: "Surface update payload hash differs.".to_string(),
            expected: gateway_surface_hash.map(Value::String),
            actual: temporal_surface_hash.map(Value::String),
        });
    }

    if run.events.len() != temporal.events.len() {
        divergences.push(ShadowDivergenceRecord {
            severity: ShadowDivergenceSeverity::Info,
            code: "event_count_drift".to_string(),
            message: "Event count differs between paths.".to_string(),
            expected: Some(json!(run.events.len())),
            actual: Some(json!(temporal.events.len())),
        });
    }

    let critical_count = divergences
        .iter()
        .filter(|entry| entry.severity == ShadowDivergenceSeverity::Critical)
        .count();
    let warning_count = divergences
        .iter()
        .filter(|entry| entry.severity == ShadowDivergenceSeverity::Warning)
        .count();
    let info_count = divergences
        .iter()
        .filter(|entry| entry.severity == ShadowDivergenceSeverity::Info)
        .count();
    let status = if critical_count > 0 {
        "diverged"
    } else if warning_count > 0 {
        "matched_with_warnings"
    } else {
        "matched"
    };

    ShadowComparisonSummary {
        compared_at: now_iso(),
        status: status.to_string(),
        critical_count,
        warning_count,
        info_count,
        divergences,
    }
}

async fn project_temporal_primary_run(
    state: GatewayState,
    space_id: String,
    run_id: String,
    backend: TemporalExecutionBackend,
) {
    let deadline = Instant::now() + Duration::from_secs(60 * 70);
    loop {
        if Instant::now() > deadline {
            tracing::warn!("Temporal projection timed out for run {}", run_id);
            break;
        }
        let snapshot =
            match load_temporal_run_snapshot_for_backend(&space_id, &run_id, backend).await {
                Ok(snapshot) => snapshot,
                Err(_) => {
                    tokio::time::sleep(Duration::from_millis(500)).await;
                    continue;
                }
            };
        let mut record = match load_agent_run_record(&space_id, &run_id) {
            Ok(record) => record,
            Err(_) => break,
        };
        let mut last_sequence = record
            .events
            .iter()
            .map(|event| event.sequence)
            .max()
            .unwrap_or(0);
        let mut sorted_events = snapshot.events.clone();
        sorted_events.sort_by_key(|event| event.sequence);
        for event in &sorted_events {
            if event.sequence <= last_sequence {
                continue;
            }
            record.events.push(event.clone());
            record.run.updated_at = event.timestamp.clone();
            if let Ok(serialized) = serde_json::to_string(event) {
                let _ = state.broadcast_tx.send(Message::Text(serialized));
            }
            last_sequence = event.sequence;
        }
        record.run.status = agent_run_status_from_str(&snapshot.status);
        record.run.updated_at = snapshot.updated_at.clone();
        record.run.simulation = snapshot.simulation.clone();
        record.run.surface_update = snapshot.surface_update.clone();
        record.run.authority_outcome = snapshot.authority_outcome.clone();
        if let Some(binding) = record.run.temporal_binding.as_mut() {
            binding.status = Some(snapshot.status.clone());
            binding.last_projected_sequence = Some(last_sequence);
        }
        if persist_agent_run_record(&record).is_err() {
            break;
        }
        if snapshot.terminal {
            break;
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }
    unregister_temporal_projector(&state, &run_id);
}

async fn monitor_temporal_shadow_run(
    state: GatewayState,
    space_id: String,
    run_id: String,
    backend: TemporalExecutionBackend,
) {
    let deadline = Instant::now() + Duration::from_secs(60 * 70);
    loop {
        if Instant::now() > deadline {
            let mut run = match load_agent_run_record(&space_id, &run_id) {
                Ok(run) => run,
                Err(_) => break,
            };
            run.run.shadow_summary = Some(ShadowComparisonSummary {
                compared_at: now_iso(),
                status: "temporal_unavailable".to_string(),
                critical_count: 1,
                warning_count: 0,
                info_count: 0,
                divergences: vec![ShadowDivergenceRecord {
                    severity: ShadowDivergenceSeverity::Critical,
                    code: "temporal_snapshot_timeout".to_string(),
                    message: "Temporal snapshot was unavailable before comparison timeout."
                        .to_string(),
                    expected: None,
                    actual: None,
                }],
            });
            let _ = persist_agent_run_record(&run);
            let _ = persist_agent_run_shadow_comparison(&run);
            break;
        }

        let run = match load_agent_run_record(&space_id, &run_id) {
            Ok(run) => run,
            Err(_) => break,
        };
        let temporal =
            match load_temporal_run_snapshot_for_backend(&space_id, &run_id, backend).await {
                Ok(snapshot) => snapshot,
                Err(_) => {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    continue;
                }
            };

        if !is_terminal_status(&run.run.status) || !temporal.terminal {
            tokio::time::sleep(Duration::from_secs(1)).await;
            continue;
        }

        let mut updated = run.clone();
        updated.run.shadow_summary = Some(compare_gateway_vs_temporal(&updated, &temporal));
        let _ = persist_agent_run_record(&updated);
        let _ = persist_agent_run_shadow_comparison(&updated);
        break;
    }
    unregister_temporal_projector(&state, &run_id);
}

async fn drive_agent_run_lifecycle(state: GatewayState, space_id: String, run_id: String) {
    let mut record = match load_agent_run_record(&space_id, &run_id) {
        Ok(run) => run,
        Err(err) => {
            tracing::error!("Failed to load run {run_id}: {err}");
            return;
        }
    };

    record.run.status = AgentRunStatus::Simulating;
    let simulating_status = run_status_name(&record.run.status).to_string();
    let contribution_id = record.run.contribution_id.clone();
    emit_agent_event(
        &state,
        &mut record,
        "run_started",
        json!({
            "status": simulating_status,
            "contributionId": contribution_id
        }),
    );
    let _ = persist_agent_run_record(&record);
    if let Err(err) = emit_execution_lifecycle(
        &record,
        AgentExecutionPhase::Simulation,
        simulating_status.as_str(),
        &record.pending_action_target,
        false,
    )
    .await
    {
        tracing::warn!(
            "Failed to emit execution lifecycle start record for run {}: {}",
            run_id,
            err
        );
    }

    let (simulation, action_target) = match evaluate_agent_plan(&run_id, &record.run.contribution_id)
    {
        Ok(value) => value,
        Err(err) => {
            record.run.status = AgentRunStatus::Failed;
            emit_agent_event(&state, &mut record, "run_failed", json!({ "error": err }));
            let _ = persist_agent_run_record(&record);
            let _ = emit_execution_lifecycle(
                &record,
                AgentExecutionPhase::Terminal,
                "failed",
                &record.pending_action_target,
                true,
            )
            .await;
            let _ = persist_agent_run_shadow_comparison(&record);
            return;
        }
    };

    let simulation_value = json!({
        "success": simulation.success,
        "violationCount": simulation.violation_count,
        "riskScore": simulation.risk_score,
        "siqsScore": simulation.siqs_score,
        "sessionId": simulation.session_id,
        "structuralDiffSummary": simulation.structural_diff_summary
    });
    let surface_update = build_a2ui_surface_payload(&space_id, &run_id, &simulation);
    record.run.simulation = Some(simulation_value.clone());
    record.run.surface_update = Some(surface_update.clone());
    record.pending_action_target = Some(action_target.clone());
    record.run.status = AgentRunStatus::WaitingApproval;
    emit_agent_event(&state, &mut record, "simulation_ready", simulation_value);
    emit_agent_event(
        &state,
        &mut record,
        "surface_update",
        json!({ "surfaceUpdate": surface_update }),
    );
    let waiting_status = run_status_name(&record.run.status).to_string();
    emit_agent_event(
        &state,
        &mut record,
        "approval_required",
        json!({ "status": waiting_status }),
    );
    let _ = persist_agent_run_record(&record);

    let (approval_tx, approval_rx) = tokio::sync::oneshot::channel::<AgentApprovalSignal>();
    if let Ok(mut waiters) = state.approval_waiters.lock() {
        waiters.insert(run_id.clone(), approval_tx);
    }

    let approval_timeout_seconds = record.run.approval_timeout_seconds.unwrap_or(3600).max(1);
    let signal =
        tokio::time::timeout(Duration::from_secs(approval_timeout_seconds), approval_rx).await;
    if let Ok(mut waiters) = state.approval_waiters.lock() {
        waiters.remove(&run_id);
    }

    let mut record = match load_agent_run_record(&space_id, &run_id) {
        Ok(run) => run,
        Err(err) => {
            tracing::error!("Failed to reload run {run_id}: {err}");
            return;
        }
    };

    let signal = match signal {
        Ok(Ok(value)) => value,
        Ok(Err(_)) => {
            record.run.status = AgentRunStatus::Failed;
            emit_agent_event(
                &state,
                &mut record,
                "run_failed",
                json!({ "error": "approval_signal_channel_closed" }),
            );
            let _ = persist_agent_run_record(&record);
            let _ = emit_execution_lifecycle(
                &record,
                AgentExecutionPhase::Terminal,
                "failed",
                &record.pending_action_target,
                true,
            )
            .await;
            let _ = persist_agent_run_shadow_comparison(&record);
            return;
        }
        Err(_) => {
            record.run.status = AgentRunStatus::Failed;
            emit_agent_event(
                &state,
                &mut record,
                "run_failed",
                json!({ "error": "approval_timeout" }),
            );
            let _ = persist_agent_run_record(&record);
            let _ = emit_execution_lifecycle(
                &record,
                AgentExecutionPhase::Terminal,
                "failed",
                &record.pending_action_target,
                true,
            )
            .await;
            let _ = persist_agent_run_shadow_comparison(&record);
            return;
        }
    };

    if signal.decision.eq_ignore_ascii_case("approved") {
        record.run.status = AgentRunStatus::Applying;
        let applying_status = run_status_name(&record.run.status).to_string();
        emit_agent_event(
            &state,
            &mut record,
            "run_started",
            json!({ "status": applying_status }),
        );
        let _ = persist_agent_run_record(&record);

        if let Some(target) = record.pending_action_target.as_ref() {
            match apply_authority_guard(&record, target, &signal).await {
                Ok(outcome) => {
                    record.run.authority_outcome = Some(outcome.clone());
                    record.run.status = AgentRunStatus::Completed;
                    let completed_status = run_status_name(&record.run.status).to_string();
                    emit_agent_event(
                        &state,
                        &mut record,
                        "run_completed",
                        json!({
                            "status": completed_status,
                            "authorityOutcome": outcome
                        }),
                    );
                    let _ = persist_agent_run_record(&record);
                    let _ = emit_execution_lifecycle(
                        &record,
                        AgentExecutionPhase::Terminal,
                        completed_status.as_str(),
                        &record.pending_action_target,
                        true,
                    )
                    .await;
                    let _ = persist_agent_run_shadow_comparison(&record);
                }
                Err(err) => {
                    record.run.status = AgentRunStatus::Failed;
                    emit_agent_event(&state, &mut record, "run_failed", json!({ "error": err }));
                    let _ = persist_agent_run_record(&record);
                    let _ = emit_execution_lifecycle(
                        &record,
                        AgentExecutionPhase::Terminal,
                        "failed",
                        &record.pending_action_target,
                        true,
                    )
                    .await;
                    let _ = persist_agent_run_shadow_comparison(&record);
                }
            }
        } else {
            record.run.status = AgentRunStatus::Failed;
            emit_agent_event(
                &state,
                &mut record,
                "run_failed",
                json!({ "error": "missing_pending_action_target" }),
            );
            let _ = persist_agent_run_record(&record);
            let _ = emit_execution_lifecycle(
                &record,
                AgentExecutionPhase::Terminal,
                "failed",
                &record.pending_action_target,
                true,
            )
            .await;
            let _ = persist_agent_run_shadow_comparison(&record);
        }
    } else {
        record.run.status = AgentRunStatus::Rejected;
        let rejected_status = run_status_name(&record.run.status).to_string();
        emit_agent_event(
            &state,
            &mut record,
            "run_completed",
            json!({
                "status": rejected_status,
                "decision": signal.decision,
                "actor": signal.actor,
                "rationale": signal.rationale,
                "decisionRef": signal.decision_ref
            }),
        );
        let _ = persist_agent_run_record(&record);
        let _ = emit_execution_lifecycle(
            &record,
            AgentExecutionPhase::Terminal,
            rejected_status.as_str(),
            &record.pending_action_target,
            true,
        )
        .await;
        let _ = persist_agent_run_shadow_comparison(&record);
    }
}

async fn post_agent_contribution(
    State(state): State<GatewayState>,
    Path(space_id): Path<String>,
    headers: HeaderMap,
    Json(payload): Json<AgentContributionRequest>,
) -> impl IntoResponse {
    let normalized_space = space_id.trim().to_string();
    if normalized_space.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "space_id is required"
            })),
        )
            .into_response();
    }
    if payload.contribution_id.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "contributionId is required"
            })),
        )
            .into_response();
    }

    tracing::info!(
        "Dispatching Agent Contribution {} for Space {}",
        payload.contribution_id,
        normalized_space
    );

    let authority_level = match parse_authority_level(payload.authority_level.as_deref()) {
        Ok(value) => value,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(json!({
                    "error": "invalid_authority_level",
                    "reason": err,
                    "expected": ["l0", "l1", "l2", "l3", "l4"]
                })),
            )
                .into_response();
        }
    };
    let run_id = next_agent_run_id(&normalized_space, &payload.contribution_id);
    let workflow_id = format!("wf-{}-{}", normalized_space, payload.contribution_id);
    let execution_id = next_execution_id(run_id.as_str());
    let attempt_id = next_attempt_id(execution_id.as_str());
    let agent_id = resolve_agent_identity(payload.agent_id.as_deref(), &headers);
    let mode = effective_runtime_mode_for_run();
    let temporal_backend = temporal_execution_backend();
    let approval_timeout_seconds = agent_approval_timeout_seconds(&normalized_space);
    let temporal_binding = match mode {
        AgentRuntimeMode::GatewayPrimary => None,
        AgentRuntimeMode::TemporalShadow | AgentRuntimeMode::TemporalPrimary => {
            Some(TemporalRunBinding {
                workflow_id: workflow_id.clone(),
                temporal_run_id: match temporal_backend {
                    TemporalExecutionBackend::Bridge => Some(run_id.clone()),
                    TemporalExecutionBackend::Sdk => None,
                },
                task_queue: Some(
                    std::env::var("CORTEX_TEMPORAL_TASK_QUEUE")
                        .unwrap_or_else(|_| "SIMULATION_TASK_QUEUE".to_string()),
                ),
                namespace: Some(
                    std::env::var("CORTEX_TEMPORAL_NAMESPACE")
                        .unwrap_or_else(|_| "default".to_string()),
                ),
                projection_mode: Some(projection_mode_label(mode, temporal_backend).to_string()),
                status: Some("queued".to_string()),
                last_projected_sequence: Some(0),
            })
        }
    };
    let started_at = now_iso();
    let mut record = AgentRunRecord {
        run: AgentRun {
            run_id: run_id.clone(),
            workflow_id: workflow_id.clone(),
            space_id: normalized_space.clone(),
            contribution_id: payload.contribution_id.clone(),
            agent_id: Some(agent_id.clone()),
            status: AgentRunStatus::Queued,
            started_at: started_at.clone(),
            updated_at: started_at.clone(),
            stream_channel: Some("/ws".to_string()),
            simulation: None,
            surface_update: None,
            authority_outcome: None,
            authority_level: Some(authority_level),
            execution_id: Some(execution_id),
            attempt_id: Some(attempt_id),
            temporal_binding,
            shadow_summary: None,
            approval_timeout_seconds: Some(approval_timeout_seconds),
        },
        events: Vec::new(),
        pending_action_target: None,
        approval: None,
    };
    let queued_status = run_status_name(&record.run.status).to_string();
    emit_agent_event(
        &state,
        &mut record,
        "run_started",
        json!({
            "status": queued_status,
            "contributionId": payload.contribution_id,
            "agentId": agent_id
        }),
    );
    if let Err(err) = persist_agent_run_record(&record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "failed_to_persist_agent_run",
                "reason": err
            })),
        )
            .into_response();
    }

    let mut temporal_started = false;
    let mut projection_mode = projection_mode_label(mode, temporal_backend).to_string();
    let mut temporal_run_id_from_start: Option<String> = None;

    if mode != AgentRuntimeMode::GatewayPrimary {
        if let Some(binding) = record.run.temporal_binding.as_ref() {
            let start_result = match temporal_backend {
                TemporalExecutionBackend::Bridge => {
                    persist_temporal_start_command(&record.run, binding, approval_timeout_seconds)
                        .map(|_| binding.temporal_run_id.clone())
                }
                TemporalExecutionBackend::Sdk => {
                    start_temporal_sdk_workflow(&record.run, binding, approval_timeout_seconds)
                        .await
                }
            };
            match start_result {
                Ok(started_run_id) => {
                    temporal_started = true;
                    temporal_run_id_from_start = started_run_id;
                }
                Err(err) => {
                    tracing::warn!(
                        "Failed to enqueue temporal start command for run {}: {}",
                        run_id,
                        err
                    );
                    if let Some(binding) = record.run.temporal_binding.as_mut() {
                        binding.projection_mode = Some("gateway_fallback".to_string());
                        binding.status = Some("start_failed".to_string());
                    }
                    let _ = persist_agent_run_record(&record);
                    projection_mode = "gateway_fallback".to_string();
                }
            }
        }
    }
    if let Some(run_id_value) = temporal_run_id_from_start {
        if let Some(binding) = record.run.temporal_binding.as_mut() {
            binding.temporal_run_id = Some(run_id_value);
        }
        let _ = persist_agent_run_record(&record);
    }

    if mode == AgentRuntimeMode::GatewayPrimary
        || mode == AgentRuntimeMode::TemporalShadow
        || (mode == AgentRuntimeMode::TemporalPrimary && !temporal_started)
    {
        let state_for_task = state.clone();
        let space_for_task = normalized_space.clone();
        let run_for_task = run_id.clone();
        tokio::spawn(async move {
            drive_agent_run_lifecycle(state_for_task, space_for_task, run_for_task).await;
        });
    }

    if mode == AgentRuntimeMode::TemporalPrimary && temporal_started {
        let state_for_projector = state.clone();
        let projector_run_id = run_id.clone();
        let projector_space = normalized_space.clone();
        let projector_backend = temporal_backend;
        let handle = tokio::spawn(async move {
            project_temporal_primary_run(
                state_for_projector.clone(),
                projector_space,
                projector_run_id.clone(),
                projector_backend,
            )
            .await;
        });
        register_temporal_projector(&state, &run_id, handle);
    }

    if mode == AgentRuntimeMode::TemporalShadow && temporal_started {
        let state_for_monitor = state.clone();
        let monitor_run_id = run_id.clone();
        let monitor_space = normalized_space.clone();
        let monitor_backend = temporal_backend;
        let handle = tokio::spawn(async move {
            monitor_temporal_shadow_run(
                state_for_monitor.clone(),
                monitor_space,
                monitor_run_id.clone(),
                monitor_backend,
            )
            .await;
        });
        register_temporal_projector(&state, &run_id, handle);
    }

    let runtime_mode_response = if projection_mode == "gateway_fallback" {
        "gateway_primary".to_string()
    } else {
        mode.as_str().to_string()
    };

    let response = AgentContributionResponse {
        accepted: true,
        run_id: run_id.clone(),
        workflow_id: workflow_id.clone(),
        status: "queued".to_string(),
        started_at,
        stream_channel: "/ws".to_string(),
        runtime_mode: runtime_mode_response,
        temporal_workflow_id: record
            .run
            .temporal_binding
            .as_ref()
            .map(|binding| binding.workflow_id.clone()),
        temporal_run_id: record
            .run
            .temporal_binding
            .as_ref()
            .and_then(|binding| binding.temporal_run_id.clone()),
        projection_mode: Some(projection_mode),
    };
    (StatusCode::ACCEPTED, Json(response)).into_response()
}

async fn get_agent_contribution_run(
    Path((space_id, run_id)): Path<(String, String)>,
) -> impl IntoResponse {
    match load_agent_run_record(&space_id, &run_id) {
        Ok(record) => Json(record).into_response(),
        Err(err) => (
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "agent_run_not_found",
                "runId": run_id,
                "spaceId": space_id,
                "reason": err
            })),
        )
            .into_response(),
    }
}

async fn post_agent_contribution_approval(
    State(state): State<GatewayState>,
    Path((space_id, run_id)): Path<(String, String)>,
    Json(payload): Json<AgentContributionApprovalRequest>,
) -> impl IntoResponse {
    let decision = payload.decision.trim().to_ascii_lowercase();
    let normalized_decision_ref = payload
        .decision_ref
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
        .unwrap_or_else(|| format!("DEC-{}", sanitize_fs_component(&run_id)));
    let payload = AgentContributionApprovalRequest {
        decision: payload.decision,
        rationale: payload.rationale,
        actor: payload.actor,
        decision_ref: Some(normalized_decision_ref.clone()),
        actor_principal: payload
            .actor_principal
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty()),
    };

    if !matches!(
        decision.as_str(),
        "approved" | "rejected" | "needs_modification"
    ) {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "invalid_decision",
                "expected": ["approved", "rejected", "needs_modification"]
            })),
        )
            .into_response();
    }
    if payload.actor.trim().is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(json!({
                "error": "actor is required"
            })),
        )
            .into_response();
    }

    let mut record = match load_agent_run_record(&space_id, &run_id) {
        Ok(value) => value,
        Err(err) => {
            return (
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "agent_run_not_found",
                    "reason": err
                })),
            )
                .into_response();
        }
    };

    if let Some(existing) = record.approval.as_ref() {
        let existing_ref = existing
            .decision_ref
            .as_ref()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty());
        if existing_ref == Some(normalized_decision_ref.as_str()) {
            if existing.decision.eq_ignore_ascii_case(&decision) && existing.actor == payload.actor
            {
                let response = AgentContributionApprovalResponse {
                    accepted: true,
                    run_id,
                    status: run_status_name(&record.run.status).to_string(),
                };
                return Json(response).into_response();
            }
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "duplicate_decision_ref_mismatch",
                    "runId": run_id
                })),
            )
                .into_response();
        }
    }

    if record.run.status != AgentRunStatus::WaitingApproval {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "run_not_waiting_approval",
                "status": run_status_name(&record.run.status)
            })),
        )
            .into_response();
    }

    let temporal_binding_active = record
        .run
        .temporal_binding
        .as_ref()
        .map(|binding| binding.status.as_deref() != Some("start_failed"))
        .unwrap_or(false);
    let temporal_projection_mode = record
        .run
        .temporal_binding
        .as_ref()
        .and_then(|binding| binding.projection_mode.as_deref());
    let temporal_primary_projection = record
        .run
        .temporal_binding
        .as_ref()
        .and_then(|binding| binding.projection_mode.as_deref())
        .map(|mode| mode.ends_with("_primary"))
        .unwrap_or(false);
    if temporal_binding_active {
        let signal_result = match temporal_backend_from_projection_mode(temporal_projection_mode) {
            TemporalExecutionBackend::Bridge => persist_temporal_signal_command(&run_id, &payload),
            TemporalExecutionBackend::Sdk => signal_temporal_sdk_workflow(&record, &payload).await,
        };
        if let Err(err) = signal_result {
            if temporal_primary_projection {
                return (
                    StatusCode::SERVICE_UNAVAILABLE,
                    Json(json!({
                        "error": "temporal_primary_signal_failed",
                        "runId": run_id,
                        "reason": err
                    })),
                )
                    .into_response();
            }
            tracing::warn!(
                "Failed to enqueue temporal approval signal command for {}: {}",
                run_id,
                err
            );
        }
    }
    record.approval = Some(payload.clone());
    if let Err(err) = persist_approval_bridge_record(&record, &payload) {
        tracing::warn!(
            "Failed to persist decision bridge record for {}: {}",
            run_id,
            err
        );
    }
    if let Err(err) = persist_temporal_approval_signal(&record, &payload) {
        tracing::warn!(
            "Failed to persist temporal approval bridge signal for {}: {}",
            run_id,
            err
        );
    }
    if let Err(err) = persist_agent_run_record(&record) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "error": "failed_to_persist_agent_approval",
                "reason": err
            })),
        )
            .into_response();
    }

    let sender = state
        .approval_waiters
        .lock()
        .ok()
        .and_then(|mut waiters| waiters.remove(&run_id));
    if let Some(sender) = sender {
        if sender
            .send(AgentApprovalSignal {
                decision: decision.clone(),
                rationale: payload.rationale.clone(),
                actor: payload.actor.clone(),
                decision_ref: payload.decision_ref.clone(),
                actor_principal: payload.actor_principal.clone(),
            })
            .is_err()
        {
            return (
                StatusCode::CONFLICT,
                Json(json!({
                    "error": "approval_waiter_closed",
                    "runId": run_id
                })),
            )
                .into_response();
        }
    } else if !(temporal_primary_projection && temporal_binding_active) {
        return (
            StatusCode::CONFLICT,
            Json(json!({
                "error": "approval_waiter_missing",
                "runId": run_id
            })),
        )
            .into_response();
    }

    let response = AgentContributionApprovalResponse {
        accepted: true,
        run_id,
        status: "approval_recorded".to_string(),
    };
    Json(response).into_response()
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use std::path::Path;
    use std::sync::{LazyLock, Mutex};

    fn testing_env_lock() -> &'static Mutex<()> {
        static LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));
        &LOCK
    }

    fn acquire_testing_env_lock() -> std::sync::MutexGuard<'static, ()> {
        testing_env_lock()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    struct TestingLogDirGuard {
        previous: Option<String>,
    }

    impl TestingLogDirGuard {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("NOSTRA_TESTING_LOG_DIR").ok();
            std::env::set_var("NOSTRA_TESTING_LOG_DIR", path.display().to_string());
            Self { previous }
        }
    }

    impl Drop for TestingLogDirGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var("NOSTRA_TESTING_LOG_DIR", previous);
            } else {
                std::env::remove_var("NOSTRA_TESTING_LOG_DIR");
            }
        }
    }

    struct SiqLogDirGuard {
        previous: Option<String>,
    }

    impl SiqLogDirGuard {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("NOSTRA_SIQ_LOG_DIR").ok();
            std::env::set_var("NOSTRA_SIQ_LOG_DIR", path.display().to_string());
            Self { previous }
        }
    }

    impl Drop for SiqLogDirGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var("NOSTRA_SIQ_LOG_DIR", previous);
            } else {
                std::env::remove_var("NOSTRA_SIQ_LOG_DIR");
            }
        }
    }

    struct MotokoGraphLogDirGuard {
        previous: Option<String>,
    }

    impl MotokoGraphLogDirGuard {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("NOSTRA_MOTOKO_GRAPH_LOG_DIR").ok();
            std::env::set_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR", path.display().to_string());
            Self { previous }
        }
    }

    impl Drop for MotokoGraphLogDirGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR", previous);
            } else {
                std::env::remove_var("NOSTRA_MOTOKO_GRAPH_LOG_DIR");
            }
        }
    }

    struct DecisionSurfaceLogDirGuard {
        previous: Option<String>,
    }

    impl DecisionSurfaceLogDirGuard {
        fn set(path: &std::path::Path) -> Self {
            let previous = std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR").ok();
            std::env::set_var(
                "NOSTRA_DECISION_SURFACE_LOG_DIR",
                path.display().to_string(),
            );
            Self { previous }
        }
    }

    impl Drop for DecisionSurfaceLogDirGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var("NOSTRA_DECISION_SURFACE_LOG_DIR", previous);
            } else {
                std::env::remove_var("NOSTRA_DECISION_SURFACE_LOG_DIR");
            }
        }
    }

    struct EnvVarGuard {
        key: &'static str,
        previous: Option<String>,
    }

    impl EnvVarGuard {
        fn set(key: &'static str, value: &str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::set_var(key, value);
            Self { key, previous }
        }

        fn unset(key: &'static str) -> Self {
            let previous = std::env::var(key).ok();
            std::env::remove_var(key);
            Self { key, previous }
        }
    }

    impl Drop for EnvVarGuard {
        fn drop(&mut self) {
            if let Some(previous) = self.previous.as_ref() {
                std::env::set_var(self.key, previous);
            } else {
                std::env::remove_var(self.key);
            }
        }
    }

    struct TestTempDir {
        path: std::path::PathBuf,
    }

    impl TestTempDir {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "cortex-testing-fixture-{}-{}",
                std::process::id(),
                nonce
            ));
            std::fs::create_dir_all(&path).expect("create temp fixture dir");
            Self { path }
        }

        fn path(&self) -> &Path {
            &self.path
        }
    }

    impl Drop for TestTempDir {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.path);
        }
    }

    fn write_testing_fixture(root: &std::path::Path) {
        let runs_dir = root.join("runs");
        std::fs::create_dir_all(&runs_dir).expect("fixture runs dir");
        let testing_artifact_path = workspace_logs_dir().join("testing").display().to_string();

        let catalog = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-08T00:00:00Z",
          "tests": [
            {
              "test_id": "rust:fixture:blocker",
              "name": "fixture blocker",
              "layer": "L1_UNIT",
              "stack": "rust",
              "owner": "Systems Steward",
              "path": "fixture.rs",
              "command": "cargo test",
              "artifacts": [testing_artifact_path.clone()],
              "gate_level": "release_blocker",
              "destructive": false,
              "tags": ["fixture"],
              "last_seen_commit": "abc123",
              "updated_at": "2026-02-08T00:00:00Z"
            },
            {
              "test_id": "rust:fixture:info",
              "name": "fixture info",
              "layer": "L1_UNIT",
              "stack": "rust",
              "owner": "Systems Steward",
              "path": "fixture_info.rs",
              "command": "cargo test",
              "artifacts": [testing_artifact_path.clone()],
              "gate_level": "informational",
              "destructive": false,
              "tags": ["fixture"],
              "last_seen_commit": "abc123",
              "updated_at": "2026-02-08T00:00:00Z"
            }
          ]
        });

        let run = json!({
          "schema_version": "1.0.0",
          "run_id": "run_fixture",
          "started_at": "2026-02-08T00:00:00Z",
          "finished_at": "2026-02-08T00:00:05Z",
          "agent_id": "fixture-agent",
          "environment": "local_ide",
          "git_commit": "abc123",
          "results": [
            { "test_id": "rust:fixture:blocker", "status": "pass", "duration_ms": 123, "error_summary": "" },
            { "test_id": "rust:fixture:info", "status": "warn", "duration_ms": 87, "error_summary": "" }
          ],
          "artifacts": [testing_artifact_path],
          "warnings": []
        });

        let gate = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-08T00:00:05Z",
          "mode": "advisory",
          "catalog_valid": true,
          "run_artifacts_valid": true,
          "required_blockers_pass": true,
          "overall_verdict": "ready",
          "latest_run_id": "run_fixture",
          "failures": [],
          "counts": { "pass": 1, "fail": 0, "warn": 1, "pending": 0 }
        });

        std::fs::write(
            root.join("test_catalog_latest.json"),
            serde_json::to_vec_pretty(&catalog).expect("catalog json"),
        )
        .expect("write catalog");
        std::fs::write(
            runs_dir.join("run_fixture.json"),
            serde_json::to_vec_pretty(&run).expect("run json"),
        )
        .expect("write run");
        std::fs::write(
            root.join("test_gate_summary_latest.json"),
            serde_json::to_vec_pretty(&gate).expect("gate json"),
        )
        .expect("write gate");
    }

    fn write_siq_fixture(root: &std::path::Path) {
        let runs_dir = root.join("runs");
        std::fs::create_dir_all(&runs_dir).expect("fixture siq runs dir");

        let coverage = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-23T00:00:00Z",
          "integrity_set": ["097", "118", "121", "123"],
          "contributions": [
            {"contribution_id": "121", "status": "draft"},
            {"contribution_id": "123", "status": "active"}
          ]
        });

        let dependency = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-23T00:00:00Z",
          "integrity_set": ["097", "118", "121", "123"],
          "overall_closure_state": "ready",
          "rows": [
            {"contribution_id": "121", "closure_state": "ready", "missing_dependencies": []}
          ]
        });

        let summary = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-23T00:00:00Z",
          "mode": "observe",
          "latest_run_id": "siq_fixture_run",
          "overall_verdict": "ready",
          "required_gates_pass": true,
          "counts": { "pass": 3, "fail": 0 },
          "failures": []
        });

        let projection = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-23T00:00:00Z",
          "run_id": "siq_fixture_run",
          "graph_fingerprint": "abcdef1234567890",
          "integrity_set": ["097", "118", "121", "123"],
          "edge_types": [
            "contribution_has_rule",
            "rule_has_run",
            "run_emits_violation",
            "violation_backed_by_evidence",
            "contribution_has_waiver"
          ],
          "entities": {
            "contributions": [{"id": "contribution:121", "kind": "Contribution"}],
            "rules": [{"id": "rule:siq_governance_execution_contract", "kind": "Rule"}],
            "gate_runs": [{"id": "run:siq_fixture_run", "kind": "GateRun"}],
            "violations": [],
            "evidence": [{"id": "evidence:research/121-cortex-memory-fs/PLAN.md", "kind": "Evidence"}],
            "waivers": []
          },
          "edges": [
            {
              "edge_id": "contribution_has_rule:contribution:121->rule:siq_governance_execution_contract",
              "type": "contribution_has_rule",
              "from": "contribution:121",
              "to": "rule:siq_governance_execution_contract"
            }
          ]
        });

        let run = json!({
          "schema_version": "1.0.0",
          "run_id": "siq_fixture_run",
          "generated_at": "2026-02-23T00:00:00Z",
          "mode": "observe",
          "policy_path": "shared/standards/alignment_contracts.toml",
          "policy_version": 1,
          "overall_verdict": "ready",
          "required_gates_pass": true,
          "counts": { "pass": 3, "fail": 0 },
          "failures": [],
          "results": [],
          "git_commit": "abc123"
        });

        std::fs::write(
            root.join("siq_coverage_latest.json"),
            serde_json::to_vec_pretty(&coverage).expect("siq coverage json"),
        )
        .expect("write siq coverage");
        std::fs::write(
            root.join("siq_dependency_closure_latest.json"),
            serde_json::to_vec_pretty(&dependency).expect("siq dependency json"),
        )
        .expect("write siq dependency");
        std::fs::write(
            root.join("siq_gate_summary_latest.json"),
            serde_json::to_vec_pretty(&summary).expect("siq summary json"),
        )
        .expect("write siq summary");
        std::fs::write(
            root.join("graph_projection_latest.json"),
            serde_json::to_vec_pretty(&projection).expect("siq projection json"),
        )
        .expect("write siq projection");
        std::fs::write(
            runs_dir.join("siq_fixture_run.json"),
            serde_json::to_vec_pretty(&run).expect("siq run json"),
        )
        .expect("write siq run");
    }

    fn write_motoko_graph_fixture(root: &std::path::Path) {
        let history_dir = root.join("history");
        let pending_dir = root.join("decisions").join("pending");
        let monitoring_runs_dir = root.join("monitoring_runs");
        std::fs::create_dir_all(&history_dir).expect("fixture history dir");
        std::fs::create_dir_all(&pending_dir).expect("fixture pending dir");
        std::fs::create_dir_all(&monitoring_runs_dir).expect("fixture monitoring dir");
        let analysis_file = workspace_research_dir()
            .join("reference")
            .join("analysis")
            .join("motoko-graph.md")
            .display()
            .to_string();

        let snapshot = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-08T10:00:00Z",
          "contribution_id": "078",
          "status": {
            "gate_result": "G2_DUAL_PATH_PASS",
            "posture": "watch-first",
            "authority_mode": "recommendation_only",
            "runtime_dependency_promotion": "deferred"
          },
          "workloads": [
            {
              "path": "vessel",
              "workload": 120,
              "edge_workload": 120,
              "seconds_per_edge": 0.001234,
              "cycles_per_edge": 200.1,
              "memory_per_edge_bytes": 64.5,
              "walk_count": 1
            }
          ],
          "stability": [
            {
              "path": "vessel",
              "steps_total": 10,
              "steps_pass": 10,
              "steps_blocked": 0,
              "first_attempt_pass": 9,
              "retries_consumed": 1,
              "port_conflicts": 0,
              "reliability_percent": 100.0
            }
          ],
          "workflow_stages": [
            { "stage": "M15", "status": "pass", "evidence": "report" },
            { "stage": "M16", "status": "pass", "evidence": "report" },
            { "stage": "M17", "status": "pass", "evidence": "report" }
          ],
          "evidence": {
            "gate_file": "/tmp/m16_dual_path/gate.txt",
            "m4_metrics_file": "/tmp/m16_dual_path/m4_metrics.tsv",
            "m8_metrics_file": "/tmp/m16_dual_path/m8_metrics.tsv",
            "stability_file": "/tmp/m16_dual_path/path_stability.tsv",
            "analysis_file": analysis_file,
            "m8_pass_count": 2
          },
          "history_event_id": "kg_snapshot_20260208T100000Z_abcdef123456"
        });

        let decision_event = json!({
          "schema_version": "1.0.0",
          "decision_event_id": "kg_decision_20260208_abcdef123456",
          "captured_at": "2026-02-08T10:10:00Z",
          "contribution": "078",
          "decision_date": "2026-02-08",
          "selected_option": "Hold Deferred",
          "rationale": "Maintain watch-first posture.",
          "posture_before": "watch-first",
          "posture_after": "watch-first",
          "authority_mode": "recommendation_only",
          "evidence_refs": ["/tmp/m16_dual_path/gate.txt"],
          "steward": "Research Steward",
          "owner": "Nostra Architecture Team",
          "follow_up_actions": ["Continue monitoring"],
          "source": "cortex-desktop:/kg/motoko-graph",
          "status": "pending"
        });

        let monitoring_run = json!({
          "schema_version": "1.0.0",
          "run_id": "monitor_fixture_001",
          "started_at": "2026-02-08T10:30:00Z",
          "finished_at": "2026-02-08T10:30:10Z",
          "gateway_base": "http://127.0.0.1:3000",
          "overall_status": "warn",
          "required_failures": 0,
          "warnings": 1,
          "checks": [
            {"name": "Generate motoko-graph snapshot", "required": true, "status": "pass", "details": ""},
            {"name": "Check gateway health endpoint", "required": false, "status": "warn", "details": "gateway offline"}
          ]
        });

        let monitoring_trend = json!({
          "schema_version": "1.0.0",
          "generated_at": "2026-02-08T10:31:00Z",
          "windows": {
            "7d": {
              "total_runs": 1,
              "pass_runs": 0,
              "warn_runs": 1,
              "fail_runs": 0,
              "reliability_percent": 100.0,
              "warning_rate_percent": 100.0,
              "required_failure_rate_percent": 0.0,
              "gateway_warning_rate_percent": 100.0,
              "mean_duration_seconds": 10.0,
              "p95_duration_seconds": 10.0,
              "last_success_at": "2026-02-08T10:30:10Z"
            },
            "30d": {
              "total_runs": 1,
              "pass_runs": 0,
              "warn_runs": 1,
              "fail_runs": 0,
              "reliability_percent": 100.0,
              "warning_rate_percent": 100.0,
              "required_failure_rate_percent": 0.0,
              "gateway_warning_rate_percent": 100.0,
              "mean_duration_seconds": 10.0,
              "p95_duration_seconds": 10.0,
              "last_success_at": "2026-02-08T10:30:10Z"
            }
          },
          "latest": {
            "run_id": "monitor_fixture_001",
            "overall_status": "warn",
            "required_failures": 0,
            "warnings": 1,
            "duration_seconds": 10.0,
            "started_at": "2026-02-08T10:30:00Z",
            "finished_at": "2026-02-08T10:30:10Z"
          },
          "last_applied_decision_event_id": "kg_decision_20260208_fixture",
          "next_action": "START_GATEWAY",
          "advisory_recommendation": "Hold Deferred"
        });

        std::fs::write(
            root.join("snapshot_latest.json"),
            serde_json::to_vec_pretty(&snapshot).expect("snapshot json"),
        )
        .expect("write snapshot");
        std::fs::write(
            history_dir.join("kg_snapshot_fixture.json"),
            serde_json::to_vec_pretty(&snapshot).expect("history snapshot json"),
        )
        .expect("write history");
        std::fs::write(
            pending_dir.join("kg_decision_fixture.json"),
            serde_json::to_vec_pretty(&decision_event).expect("decision json"),
        )
        .expect("write pending decision");
        std::fs::write(
            monitoring_runs_dir.join("monitor_fixture_001.json"),
            serde_json::to_vec_pretty(&monitoring_run).expect("monitoring run json"),
        )
        .expect("write monitoring run");
        std::fs::write(
            root.join("monitoring_trend_latest.json"),
            serde_json::to_vec_pretty(&monitoring_trend).expect("monitoring trend json"),
        )
        .expect("write monitoring trend");
    }

    fn read_jsonl(path: &std::path::Path) -> Vec<serde_json::Value> {
        read_jsonl_vec(path)
    }

    async fn response_json(response: axum::response::Response) -> serde_json::Value {
        let bytes = to_bytes(response.into_body(), usize::MAX)
            .await
            .expect("response body bytes");
        serde_json::from_slice(&bytes).expect("response json")
    }

    fn heap_emit_fixture(
        request_id: &str,
        title: &str,
        emitted_at: &str,
        mirror_mentions: bool,
        files_key_format: &str,
    ) -> serde_json::Value {
        json!({
            "schema_version": "1.0.0",
            "mode": "heap",
            "workspace_id": "01ARZ3NDEKTSV4RRFFQ69G5FAV",
            "source": {
                "agent_id": "agent-test",
                "request_id": request_id,
                "emitted_at": emitted_at
            },
            "block": {
                "type": "widget",
                "title": title
            },
            "content": {
                "payload_type": "a2ui",
                "a2ui": {
                    "surface_id": format!("surface:{}", request_id),
                    "protocol_version": "1.0.0",
                    "renderer": "react",
                    "tree": {
                        "surfaceId": format!("surface:{}", request_id),
                        "title": title,
                        "root": "root",
                        "components": [{
                            "id": "root",
                            "type": "Card",
                            "props": { "title": title },
                            "children": []
                        }]
                    }
                }
            },
            "relations": {
                "tags": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAX" }],
                "mentions": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAY", "label": "Project Alpha" }],
                "page_links": [{ "to_block_id": "01ARZ3NDEKTSV4RRFFQ69G5FAZ" }]
            },
            "files": [{
                "hash": "abc12345",
                "file_size": 42,
                "name": "report.png",
                "mime_type": "image/png"
            }],
            "projection_hints": {
                "mirror_mentions_to_relations": mirror_mentions,
                "files_key_format": files_key_format,
                "relation_map_version": "relations_v1"
            },
            "crdt_projection": {
                "artifact_id": format!("artifact-{}", request_id)
            }
        })
    }

    #[test]
    fn agent_contribution_request_accepts_camel_and_snake_case_keys() {
        let camel: AgentContributionRequest =
            serde_json::from_value(json!({ "contributionId": "contribution-alpha" }))
                .expect("camelCase payload should deserialize");
        assert_eq!(camel.contribution_id, "contribution-alpha");

        let snake: AgentContributionRequest =
            serde_json::from_value(json!({ "contribution_id": "contribution-beta" }))
                .expect("snake_case payload should deserialize");
        assert_eq!(snake.contribution_id, "contribution-beta");
    }

    #[test]
    fn authority_level_parser_defaults_to_l1_and_validates_range() {
        assert_eq!(parse_authority_level(None).unwrap(), AuthorityLevel::L1);
        assert_eq!(
            parse_authority_level(Some("l2")).unwrap(),
            AuthorityLevel::L2
        );
        assert!(parse_authority_level(Some("invalid")).is_err());
    }

    #[test]
    fn agent_identity_resolution_prioritizes_header_then_request_then_env() {
        let _env_lock = acquire_testing_env_lock();
        let _agent_env = EnvVarGuard::set("NOSTRA_AGENT_ID", "agent:env");
        let _default_agent_env =
            EnvVarGuard::set("NOSTRA_DEFAULT_AGENT_ID", "agent:legacy-default");

        let no_header = HeaderMap::new();
        assert_eq!(
            resolve_agent_identity(Some("agent:request"), &no_header),
            "agent:request"
        );
        assert_eq!(resolve_agent_identity(None, &no_header), "agent:env");

        let mut with_header = HeaderMap::new();
        with_header.insert(
            "x-cortex-agent-id",
            "agent:header".parse().expect("header value"),
        );
        assert_eq!(
            resolve_agent_identity(Some("agent:request"), &with_header),
            "agent:header"
        );
    }

    #[test]
    fn agent_contribution_approval_request_accepts_decision_ref_alias() {
        let camel: AgentContributionApprovalRequest = serde_json::from_value(json!({
            "decision": "approved",
            "actor": "systems-steward",
            "decisionRef": "DEC-123"
        }))
        .expect("camelCase decisionRef should deserialize");
        assert_eq!(camel.decision_ref.as_deref(), Some("DEC-123"));

        let snake: AgentContributionApprovalRequest = serde_json::from_value(json!({
            "decision": "approved",
            "actor": "systems-steward",
            "decision_ref": "DEC-456"
        }))
        .expect("snake_case decision_ref should deserialize");
        assert_eq!(snake.decision_ref.as_deref(), Some("DEC-456"));

        let with_actor_principal: AgentContributionApprovalRequest = serde_json::from_value(json!({
            "decision": "approved",
            "actor": "systems-steward",
            "decision_ref": "DEC-789",
            "actor_principal": "2vxsx-fae"
        }))
        .expect("snake_case actor_principal should deserialize");
        assert_eq!(
            with_actor_principal.actor_principal.as_deref(),
            Some("2vxsx-fae")
        );
    }

    #[test]
    fn temporal_query_snapshot_parser_accepts_nested_result_data_json() {
        let snapshot = json!({
            "schemaVersion": "1.0.0",
            "runId": "run-123",
            "workflowId": "wf-123",
            "spaceId": "space-alpha",
            "contributionId": "contribution-alpha",
            "status": "waiting_approval",
            "startedAt": "2026-02-22T00:00:00Z",
            "updatedAt": "2026-02-22T00:00:01Z",
            "sequence": 3,
            "events": [],
            "simulation": null,
            "surfaceUpdate": null,
            "authorityOutcome": null,
            "providerTrace": null,
            "approvalTimeoutSeconds": 3600,
            "terminal": false,
            "error": null
        });
        let payload = json!({
            "result": {
                "data": snapshot.to_string()
            }
        })
        .to_string();
        let parsed =
            parse_temporal_query_snapshot_stdout(&payload).expect("nested data should parse");
        assert_eq!(parsed.run_id, "run-123");
        assert_eq!(parsed.workflow_id, "wf-123");
    }

    #[test]
    fn temporal_query_snapshot_parser_rejects_invalid_payload_shape() {
        let payload = json!({
            "result": {
                "data": "not-json"
            }
        })
        .to_string();
        let err = parse_temporal_query_snapshot_stdout(&payload)
            .expect_err("invalid payload should fail parsing");
        assert!(err.contains("run_snapshot"));
    }

    #[test]
    fn temporal_promotion_gate_blocks_primary_when_critical_divergence_present() {
        let _env_lock = acquire_testing_env_lock();
        let fixture = TestTempDir::new();
        let _decision_guard = DecisionSurfaceLogDirGuard::set(fixture.path());
        let _burnin_runs = EnvVarGuard::set("CORTEX_TEMPORAL_BURNIN_MIN_RUNS", "1");
        let _burnin_hours = EnvVarGuard::set("CORTEX_TEMPORAL_BURNIN_MAX_AGE_HOURS", "24");
        let _fallback = EnvVarGuard::set("CORTEX_TEMPORAL_PROMOTION_FALLBACK", "temporal_shadow");

        let metrics_dir = fixture.path().join("metrics");
        std::fs::create_dir_all(&metrics_dir).expect("create metrics dir");
        std::fs::write(
            metrics_dir.join("agent_run_shadow_diff_fixture.json"),
            serde_json::to_vec_pretty(&json!({
                "generatedAt": Utc::now().to_rfc3339(),
                "shadowDivergence": {
                    "criticalCount": 1,
                    "warningCount": 0,
                    "infoCount": 0,
                    "divergences": []
                }
            }))
            .expect("shadow diff json"),
        )
        .expect("write shadow diff");

        let decision = evaluate_temporal_promotion_gate(AgentRuntimeMode::TemporalPrimary);
        assert!(!decision.eligible);
        assert_eq!(decision.effective_mode, "temporal_shadow");
    }

    #[test]
    fn temporal_promotion_gate_allows_primary_when_burn_in_has_zero_critical() {
        let _env_lock = acquire_testing_env_lock();
        let fixture = TestTempDir::new();
        let _decision_guard = DecisionSurfaceLogDirGuard::set(fixture.path());
        let _burnin_runs = EnvVarGuard::set("CORTEX_TEMPORAL_BURNIN_MIN_RUNS", "1");
        let _burnin_hours = EnvVarGuard::set("CORTEX_TEMPORAL_BURNIN_MAX_AGE_HOURS", "24");
        let _fallback = EnvVarGuard::set("CORTEX_TEMPORAL_PROMOTION_FALLBACK", "gateway_primary");

        let metrics_dir = fixture.path().join("metrics");
        std::fs::create_dir_all(&metrics_dir).expect("create metrics dir");
        std::fs::write(
            metrics_dir.join("agent_run_shadow_diff_fixture_clean.json"),
            serde_json::to_vec_pretty(&json!({
                "generatedAt": Utc::now().to_rfc3339(),
                "shadowDivergence": {
                    "criticalCount": 0,
                    "warningCount": 0,
                    "infoCount": 0,
                    "divergences": []
                }
            }))
            .expect("shadow diff json"),
        )
        .expect("write shadow diff");

        let decision = evaluate_temporal_promotion_gate(AgentRuntimeMode::TemporalPrimary);
        assert!(decision.eligible);
        assert_eq!(decision.effective_mode, "temporal_primary");
    }

    #[tokio::test]
    async fn agent_contribution_approval_rejects_missing_actor() {
        let response = post_agent_contribution_approval(
            State(GatewayState::new()),
            Path(("space-alpha".to_string(), "run-alpha".to_string())),
            Json(AgentContributionApprovalRequest {
                decision: "approved".to_string(),
                rationale: Some("ship it".to_string()),
                actor: "   ".to_string(),
                decision_ref: Some("DEC-missing-actor".to_string()),
                actor_principal: None,
            }),
        )
        .await
        .into_response();

        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["error"], "actor is required");
    }

    #[tokio::test]
    async fn agent_contribution_approval_primary_temporal_signal_failure_returns_service_unavailable()
    {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());

        let space_id = "space-agent".to_string();
        let run_id = "run-agent-primary-signal-fail".to_string();
        let now = "2026-02-22T00:00:00Z".to_string();

        let record = AgentRunRecord {
            run: AgentRun {
                run_id: run_id.clone(),
                workflow_id: "wf-space-agent-primary".to_string(),
                space_id: space_id.clone(),
                contribution_id: "contribution-primary".to_string(),
                agent_id: Some("agent:tests".to_string()),
                status: AgentRunStatus::WaitingApproval,
                started_at: now.clone(),
                updated_at: now,
                stream_channel: Some("/ws".to_string()),
                simulation: None,
                surface_update: None,
                authority_outcome: None,
                authority_level: None,
                execution_id: None,
                attempt_id: None,
                temporal_binding: Some(TemporalRunBinding {
                    workflow_id: "wf-space-agent-primary".to_string(),
                    temporal_run_id: Some(run_id.clone()),
                    task_queue: Some("SIMULATION_TASK_QUEUE".to_string()),
                    namespace: Some("default".to_string()),
                    projection_mode: Some("temporal_sdk_primary".to_string()),
                    status: Some("queued".to_string()),
                    last_projected_sequence: Some(0),
                }),
                shadow_summary: None,
                approval_timeout_seconds: Some(3600),
            },
            events: Vec::new(),
            pending_action_target: None,
            approval: None,
        };
        persist_agent_run_record(&record).expect("persist test run record");

        let response = post_agent_contribution_approval(
            State(GatewayState::new()),
            Path((space_id.clone(), run_id.clone())),
            Json(AgentContributionApprovalRequest {
                decision: "approved".to_string(),
                rationale: Some("ship".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-signal-fail-1".to_string()),
                actor_principal: None,
            }),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_json(response).await;
        assert_eq!(body["error"], "temporal_primary_signal_failed");

        let persisted = load_agent_run_record(&space_id, &run_id).expect("reload run");
        assert!(persisted.approval.is_none());
    }

    #[tokio::test]
    async fn agent_contribution_approval_duplicate_decision_ref_is_idempotent() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());

        let space_id = "space-agent".to_string();
        let run_id = "run-agent-001".to_string();
        let now = "2026-02-22T00:00:00Z".to_string();

        let record = AgentRunRecord {
            run: AgentRun {
                run_id: run_id.clone(),
                workflow_id: "wf-space-agent-contribution".to_string(),
                space_id: space_id.clone(),
                contribution_id: "contribution-001".to_string(),
                agent_id: Some("agent:tests".to_string()),
                status: AgentRunStatus::WaitingApproval,
                started_at: now.clone(),
                updated_at: now,
                stream_channel: Some("/ws".to_string()),
                simulation: None,
                surface_update: None,
                authority_outcome: None,
                authority_level: None,
                execution_id: None,
                attempt_id: None,
                temporal_binding: None,
                shadow_summary: None,
                approval_timeout_seconds: Some(3600),
            },
            events: Vec::new(),
            pending_action_target: None,
            approval: None,
        };
        persist_agent_run_record(&record).expect("persist test run record");

        let state = GatewayState::new();
        let (approval_tx, approval_rx) = tokio::sync::oneshot::channel::<AgentApprovalSignal>();
        if let Ok(mut waiters) = state.approval_waiters.lock() {
            waiters.insert(run_id.clone(), approval_tx);
        }
        tokio::spawn(async move {
            let _ = approval_rx.await;
        });

        let payload = AgentContributionApprovalRequest {
            decision: "approved".to_string(),
            rationale: Some("approved in test".to_string()),
            actor: "systems-steward".to_string(),
            decision_ref: Some("DEC-idempotent-001".to_string()),
            actor_principal: None,
        };

        let first = post_agent_contribution_approval(
            State(state.clone()),
            Path((space_id.clone(), run_id.clone())),
            Json(payload.clone()),
        )
        .await
        .into_response();
        assert_eq!(first.status(), StatusCode::OK);
        let first_body = response_json(first).await;
        assert_eq!(first_body["accepted"], true);

        let second =
            post_agent_contribution_approval(State(state), Path((space_id, run_id)), Json(payload))
                .await
                .into_response();
        assert_eq!(second.status(), StatusCode::OK);
        let second_body = response_json(second).await;
        assert_eq!(second_body["accepted"], true);
    }

    fn test_authority_record(authority_level: AuthorityLevel) -> AgentRunRecord {
        AgentRunRecord {
            run: AgentRun {
                run_id: "run-authority-test".to_string(),
                workflow_id: "wf-authority-test".to_string(),
                space_id: "space-authority-test".to_string(),
                contribution_id: "contribution-authority-test".to_string(),
                agent_id: Some("agent:tests-authority".to_string()),
                status: AgentRunStatus::WaitingApproval,
                started_at: "2026-02-24T00:00:00Z".to_string(),
                updated_at: "2026-02-24T00:00:00Z".to_string(),
                stream_channel: Some("/ws".to_string()),
                simulation: Some(json!({
                    "riskScore": 10,
                    "siqsScore": 98.0
                })),
                surface_update: Some(json!({"surface": "ok"})),
                authority_outcome: None,
                authority_level: Some(authority_level),
                execution_id: Some("exec-authority-test".to_string()),
                attempt_id: Some("attempt-authority-test".to_string()),
                temporal_binding: None,
                shadow_summary: None,
                approval_timeout_seconds: Some(3600),
            },
            events: Vec::new(),
            pending_action_target: Some(ActionTarget {
                protocol: "ic".to_string(),
                address: "kg-canister".to_string(),
                method: "create_context_node".to_string(),
                payload: Vec::new(),
            }),
            approval: None,
        }
    }

    #[tokio::test]
    async fn apply_authority_guard_l0_creates_recommendation() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let run = test_authority_record(AuthorityLevel::L0);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l0".to_string()),
                actor_principal: None,
            },
        )
        .await
        .expect("authority outcome");
        assert!(!outcome.accepted);
        assert_eq!(outcome.error.as_deref(), Some("l0_recommendation_only"));
    }

    #[tokio::test]
    async fn apply_authority_guard_l1_creates_proposal() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let run = test_authority_record(AuthorityLevel::L1);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l1".to_string()),
                actor_principal: None,
            },
        )
        .await
        .expect("authority outcome");
        assert!(!outcome.accepted);
        assert_eq!(outcome.error.as_deref(), Some("l1_proposal_created"));
        let proposal_dir = temp.path().join("agent_proposals");
        assert!(proposal_dir.exists());
    }

    #[tokio::test]
    async fn apply_authority_guard_l2_missing_actor_principal_is_deferred() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let run = test_authority_record(AuthorityLevel::L2);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l2".to_string()),
                actor_principal: None,
            },
        )
        .await
        .expect("authority outcome");
        assert!(!outcome.accepted);
        assert_eq!(outcome.error.as_deref(), Some("l2_missing_actor_principal"));
    }

    #[tokio::test]
    async fn apply_authority_guard_l2_allows_apply_when_governance_and_evaluation_pass() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let _mock_scope = EnvVarGuard::set("NOSTRA_TEST_AGENT_L2_SCOPE_EVAL", "allow");
        let run = test_authority_record(AuthorityLevel::L2);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l2-pass".to_string()),
                actor_principal: Some("2vxsx-fae".to_string()),
            },
        )
        .await
        .expect("authority outcome");
        assert!(outcome.accepted);
        assert!(outcome.error.is_none());
        let authority_path = temp
            .path()
            .join("authority")
            .join("run-authority-test.json");
        assert!(authority_path.exists());
    }

    #[tokio::test]
    async fn apply_authority_guard_l3_fails_closed() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let run = test_authority_record(AuthorityLevel::L3);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l3".to_string()),
                actor_principal: Some("2vxsx-fae".to_string()),
            },
        )
        .await
        .expect("authority outcome");
        assert!(!outcome.accepted);
        assert_eq!(outcome.error.as_deref(), Some("v1_fail_closed_l3_l4"));
    }

    #[tokio::test]
    async fn apply_authority_guard_l4_fails_closed() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let run = test_authority_record(AuthorityLevel::L4);
        let target = run.pending_action_target.clone().expect("target");
        let outcome = apply_authority_guard(
            &run,
            &target,
            &AgentApprovalSignal {
                decision: "approved".to_string(),
                rationale: Some("test".to_string()),
                actor: "systems-steward".to_string(),
                decision_ref: Some("DEC-test-l4".to_string()),
                actor_principal: Some("2vxsx-fae".to_string()),
            },
        )
        .await
        .expect("authority outcome");
        assert!(!outcome.accepted);
        assert_eq!(outcome.error.as_deref(), Some("v1_fail_closed_l3_l4"));
    }

    #[tokio::test]
    async fn execution_lifecycle_emits_agent_execution_lifecycle_event() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let mut run = test_authority_record(AuthorityLevel::L1);
        run.run.status = AgentRunStatus::Completed;
        emit_execution_lifecycle(
            &run,
            AgentExecutionPhase::Terminal,
            "completed",
            &run.pending_action_target,
            true,
        )
        .await
        .expect("execution lifecycle");
        let lifecycle_path = temp
            .path()
            .join("events")
            .join("agent_execution_lifecycle.jsonl");
        let rows = read_jsonl(&lifecycle_path);
        assert!(!rows.is_empty());
        assert_eq!(rows[0]["eventType"], "AgentExecutionLifecycle");
        assert_eq!(
            rows[0]["cloudEvent"]["type"],
            json!("AgentExecutionLifecycle")
        );
    }

    fn risky_decision_payload(mutation_id: &str) -> DecisionActionRequest {
        DecisionActionRequest {
            space_id: Some("space-default".to_string()),
            decision_gate_id: Some(format!("blackwell_gate:{mutation_id}")),
            workflow_id: Some("wf-test".to_string()),
            mutation_id: Some(mutation_id.to_string()),
            action_target: Some("governance:release".to_string()),
            domain_mode: Some("attributed".to_string()),
            gate_level: Some("release_blocker".to_string()),
            actor_ref: None,
            risk_statement: Some("bounded risk acknowledged".to_string()),
            rollback_path: Some("rollback: revert decision action".to_string()),
            evidence_refs: vec!["test:evidence".to_string()],
            note: Some("test payload".to_string()),
        }
    }

    fn decision_headers() -> HeaderMap {
        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-ic-principal",
            "2vxsx-fae".parse().expect("principal header"),
        );
        headers
    }

    fn test_governance_envelope(
        actor_id: &str,
        approved_by: &str,
        rationale: &str,
        approved_at: &str,
        decision_id: &str,
    ) -> ArtifactGovernanceEnvelope {
        let nonce = format!("nonce-{}", Utc::now().timestamp_millis());
        let expires_at = (Utc::now() + chrono::Duration::minutes(30)).to_rfc3339();
        let material = format!(
            "{}|{}|{}|{}|{}|{}",
            actor_id, approved_by, approved_at, decision_id, nonce, expires_at
        );
        let signature = artifact_governance_signature_secret()
            .map(|secret| signature_hash(&secret, &material))
            .unwrap_or_else(|| "test-signature".to_string());
        ArtifactGovernanceEnvelope {
            approved_by: approved_by.to_string(),
            rationale: rationale.to_string(),
            approved_at: approved_at.to_string(),
            actor_id: actor_id.to_string(),
            decision_proof: ArtifactPrivilegeDecisionProof {
                decision_id: decision_id.to_string(),
                signature,
                signer: "test-signer".to_string(),
                algorithm: Some("ed25519".to_string()),
                nonce: Some(nonce.clone()),
                expires_at: Some(expires_at.clone()),
            },
            nonce: Some(nonce),
            expires_at: Some(expires_at),
        }
    }

    #[test]
    fn acp_native_entry_reports_unreachable_when_worker_absent() {
        let entry = build_acp_native_entry(None);
        assert_eq!(entry.name, "acp_pilot_ops");
        assert_eq!(entry.status, "worker-unreachable");
        assert_eq!(entry.source, "cortex-worker");
        assert!(entry.read_only);
        let automation = entry
            .automation
            .expect("automation descriptor should always be present");
        assert_eq!(
            automation.last_status.as_deref(),
            Some("worker-unreachable")
        );
        assert!(!automation.can_run_now);
    }

    #[test]
    fn acp_native_entry_reports_running_when_active_workflow_exists() {
        let entry = build_acp_native_entry(Some(WorkerAcpAutomationStatus {
            automation_key: Some("acp_pilot_ops".to_string()),
            enabled: true,
            paused: false,
            interval_secs: Some(3600),
            active_workflow_id: Some("acp_pilot_ops-123".to_string()),
            last_workflow_id: Some("acp_pilot_ops-122".to_string()),
            last_run_at: Some("2026-02-08T03:50:00Z".to_string()),
            last_status: Some("Running".to_string()),
        }));
        assert_eq!(entry.status, "running");
        let automation = entry
            .automation
            .expect("automation descriptor should be present");
        assert_eq!(automation.interval_secs, 3600);
        assert!(automation.can_pause);
        assert!(!automation.can_resume);
    }

    #[test]
    fn acp_native_entry_reports_paused_with_resume_capability() {
        let entry = build_acp_native_entry(Some(WorkerAcpAutomationStatus {
            automation_key: Some("acp_pilot_ops".to_string()),
            enabled: true,
            paused: true,
            interval_secs: Some(3600),
            active_workflow_id: None,
            last_workflow_id: Some("acp_pilot_ops-123".to_string()),
            last_run_at: Some("2026-02-08T04:00:00Z".to_string()),
            last_status: Some("Paused".to_string()),
        }));
        assert_eq!(entry.status, "paused");
        let automation = entry
            .automation
            .expect("automation descriptor should be present");
        assert!(!automation.can_pause);
        assert!(automation.can_resume);
        assert_eq!(
            automation.pause_reason.as_deref(),
            Some("Paused by operator or policy.")
        );
    }

    #[tokio::test]
    async fn workflow_catalog_always_contains_acp_native_entry() {
        let Json(catalog) = list_workflow_catalog().await;
        assert!(catalog.iter().any(|entry| {
            entry.name == "acp_pilot_ops" && entry.source == "cortex-worker" && entry.read_only
        }));
    }

    #[tokio::test]
    async fn cortex_layout_spec_exposes_expanded_navigation_planes() {
        let Json(spec) = get_cortex_layout_spec().await;
        let routes: std::collections::HashSet<String> = spec
            .navigation_graph
            .entries
            .iter()
            .map(|entry| entry.route_id.clone())
            .collect();
        let required_routes = [
            "/spaces",
            "/heap",
            "/synthesis",
            "/playground",
            "/studio",
            "/workflows",
            "/contributions",
            "/labs",
            "/system",
            "/artifacts",
            "/vfs",
            "/logs",
            "/settings",
            "/inbox",
            "/agents",
            "/discovery",
            "/metrics",
            "/memory",
            "/simulation",
        ];
        for route in required_routes {
            assert!(routes.contains(route), "missing required route: {route}");
        }
    }

    #[tokio::test]
    async fn capability_graph_includes_expanded_route_nodes_with_expected_contracts() {
        let response = get_system_capability_graph().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        let nodes = body["nodes"].as_array().expect("nodes array");

        let assert_route_contract = |route: &str, pattern_id: &str, role: &str| {
            let node = nodes
                .iter()
                .find(|candidate| candidate["route_id"].as_str() == Some(route))
                .unwrap_or_else(|| panic!("missing capability graph node for route {}", route));
            assert_eq!(node["pattern_id"].as_str(), Some(pattern_id));
            assert_eq!(node["required_role"].as_str(), Some(role));
        };

        assert_route_contract("/vfs", "pattern.system", "operator");
        assert_route_contract("/logs", "pattern.system", "operator");
        assert_route_contract("/settings", "pattern.system", "operator");
        assert_route_contract("/inbox", "pattern.workflow", "operator");
        assert_route_contract("/agents", "pattern.system", "operator");
        assert_route_contract("/discovery", "pattern.spaces", "viewer");
        assert_route_contract("/metrics", "pattern.testing", "steward");
        assert_route_contract("/memory", "pattern.studio", "operator");
        assert_route_contract("/simulation", "pattern.studio", "operator");
    }

    #[test]
    fn shared_contract_fixture_has_required_phase3_routes() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../shared/fixtures/cortex_ux_contract_fixture.json"
        ))
        .expect("fixture should parse");
        let routes: Vec<String> = fixture["layoutSpec"]["navigationGraph"]["entries"]
            .as_array()
            .expect("entries array")
            .iter()
            .filter_map(|entry| entry["routeId"].as_str().map(str::to_string))
            .collect();
        assert!(routes.iter().any(|route| route == "/studio"));
        assert!(routes.iter().any(|route| route == "/artifacts"));
        assert!(routes.iter().any(|route| route == "/workflows"));
    }

    #[test]
    fn shared_contract_fixture_declares_phase7_realtime_endpoints() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../shared/fixtures/cortex_ux_contract_fixture.json"
        ))
        .expect("fixture should parse");
        let endpoints = fixture["collaboration"]["realtime"]["endpoints"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        let required = [
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/status",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/integrity",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/connect",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/disconnect",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/backlog",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/resync",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/ack",
            "/api/cortex/studio/artifacts/:artifact_id/collab/realtime/ack/reset",
            "/api/cortex/runtime/slo/status",
            "/api/cortex/runtime/slo/breaches",
            "/ws/cortex/collab",
        ];
        for endpoint in required {
            assert!(
                endpoints
                    .iter()
                    .any(|item| item.as_str().unwrap_or_default() == endpoint),
                "missing required phase6 realtime endpoint in shared fixture: {endpoint}"
            );
        }
    }

    #[test]
    fn shared_contract_fixture_declares_phase7_governance_metadata_fields() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../shared/fixtures/cortex_ux_contract_fixture.json"
        ))
        .expect("fixture should parse");
        let fields = fixture["collaboration"]["realtime"]["requiredGovernanceMetadata"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        for required in ["nonce", "expiresAt", "decisionProof.signature"] {
            assert!(
                fields
                    .iter()
                    .any(|item| item.as_str().unwrap_or_default() == required),
                "missing required phase7 governance metadata field in fixture: {required}"
            );
        }
    }

    #[test]
    fn shared_contract_fixture_drift_requires_approval_metadata() {
        let fixture: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../shared/fixtures/cortex_ux_contract_fixture.json"
        ))
        .expect("fixture should parse");
        let defaults = default_persisted_shell_contract();

        let fixture_routes: std::collections::HashSet<String> =
            fixture["layoutSpec"]["navigationGraph"]["entries"]
                .as_array()
                .expect("entries array")
                .iter()
                .filter_map(|entry| entry["routeId"].as_str().map(str::to_string))
                .collect();
        let default_routes: std::collections::HashSet<String> = defaults
            .layout_spec
            .navigation_graph
            .entries
            .iter()
            .map(|entry| entry.route_id.clone())
            .collect();

        let fixture_patterns: std::collections::HashSet<String> = fixture["matrix"]
            .as_array()
            .expect("matrix array")
            .iter()
            .filter_map(|entry| entry["patternId"].as_str().map(str::to_string))
            .collect();
        let default_patterns: std::collections::HashSet<String> = defaults
            .patterns
            .iter()
            .map(|pattern| pattern.pattern_id.clone())
            .collect();

        let drift_detected =
            fixture_routes != default_routes || fixture_patterns != default_patterns;
        if drift_detected {
            let approved_by = fixture["approvalMetadata"]["approvedBy"]
                .as_str()
                .map(str::trim)
                .unwrap_or_default();
            let rationale = fixture["approvalMetadata"]["rationale"]
                .as_str()
                .map(str::trim)
                .unwrap_or_default();
            let approved_at = fixture["approvalMetadata"]["approvedAt"]
                .as_str()
                .map(str::trim)
                .unwrap_or_default();
            assert!(
                !approved_by.is_empty() && !rationale.is_empty() && !approved_at.is_empty(),
                "Route/pattern drift requires approvalMetadata with approvedBy/rationale/approvedAt"
            );
        }
    }

    #[tokio::test]
    async fn cortex_layout_spec_can_be_persisted_via_api() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let contract_path = temp.path().join("contract_override.json");
        let _dir_guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        let _contract_guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_CONTRACT_PATH",
            contract_path.display().to_string().as_str(),
        );

        let mut contract = default_persisted_shell_contract();
        contract.layout_spec.layout_id = "cortex.desktop.shell.persisted.test".to_string();
        contract.updated_at = "2026-02-09T03:00:00Z".to_string();

        let response = post_cortex_layout_spec(Json(contract)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let Json(spec) = get_cortex_layout_spec().await;
        assert_eq!(spec.layout_id, "cortex.desktop.shell.persisted.test");
    }

    #[tokio::test]
    async fn cortex_layout_evaluate_blocks_structural_change_without_hitl() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let payload = UxLayoutEvaluationRequest {
            candidate_id: "candidate-structural".to_string(),
            route_id: "/workflows".to_string(),
            view_capability_id: "view.workflows".to_string(),
            structural_change: true,
            metrics: crate::services::cortex_ux::UxCandidateMetrics {
                task_success: 0.9,
                time_to_decision_seconds: 35.0,
                nav_depth: 2,
                accessibility_score: 0.95,
                consistency_score: 0.9,
            },
            gates: crate::services::cortex_ux::UxAutoGates {
                accessibility: true,
                decision_safety_semantics: true,
                offline_integrity: true,
                policy_compliance: true,
            },
            approval: None,
        };

        let response = post_cortex_layout_evaluate(Json(payload)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(
            body["evaluation"]["promotionStatus"],
            "blocked_hitl_required"
        );
    }

    #[tokio::test]
    async fn cortex_layout_evaluate_generates_promotion_decision_when_hitl_present() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let payload = UxLayoutEvaluationRequest {
            candidate_id: "candidate-approve".to_string(),
            route_id: "/workflows".to_string(),
            view_capability_id: "view.workflows".to_string(),
            structural_change: true,
            metrics: crate::services::cortex_ux::UxCandidateMetrics {
                task_success: 0.88,
                time_to_decision_seconds: 52.0,
                nav_depth: 2,
                accessibility_score: 0.92,
                consistency_score: 0.89,
            },
            gates: crate::services::cortex_ux::UxAutoGates {
                accessibility: true,
                decision_safety_semantics: true,
                offline_integrity: true,
                policy_compliance: true,
            },
            approval: Some(crate::services::cortex_ux::UxApprovalPayload {
                approved_by: "Systems Steward".to_string(),
                rationale: "Promote after matrix review.".to_string(),
                timestamp: "2026-02-09T02:00:00Z".to_string(),
            }),
        };

        let response = post_cortex_layout_evaluate(Json(payload)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(
            body["evaluation"]["promotionStatus"],
            "eligible_hitl_approved"
        );
        assert!(body["promotionDecision"].is_object());

        let eval_log = read_jsonl(&temp.path().join("candidate_evaluations.jsonl"));
        assert!(!eval_log.is_empty());
        assert!(eval_log.iter().any(|entry| {
            entry["candidateId"] == "candidate-approve"
                && entry["promotionStatus"] == "eligible_hitl_approved"
        }));
        let decision_log = read_jsonl(&temp.path().join("promotion_decisions.jsonl"));
        assert!(!decision_log.is_empty());
    }

    #[tokio::test]
    async fn cortex_feedback_endpoint_persists_event() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let payload = UxFeedbackEvent {
            event_id: "feedback-1".to_string(),
            route_id: "/studio".to_string(),
            view_id: "view.studio".to_string(),
            action_id: Some("bridge:feedback".to_string()),
            friction_tag: "bridge-validation".to_string(),
            severity: "info".to_string(),
            free_text: Some("Bridge looks stable.".to_string()),
            session_id: "test-session".to_string(),
            run_id: None,
            timestamp: now_iso(),
        };

        let response = post_cortex_feedback_ux(Json(payload)).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["accepted"], true);
        assert_eq!(body["eventId"], "feedback-1");

        let feedback_log = read_jsonl(&temp.path().join("feedback_events.jsonl"));
        assert!(
            feedback_log
                .iter()
                .any(|entry| { entry["eventId"] == "feedback-1" && entry["routeId"] == "/studio" })
        );
    }

    #[tokio::test]
    async fn cortex_feedback_triage_updates_queue_item() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let payload = UxFeedbackEvent {
            event_id: "feedback-triage-1".to_string(),
            route_id: "/studio".to_string(),
            view_id: "view.studio".to_string(),
            action_id: Some("lane:feedback".to_string()),
            friction_tag: "layout".to_string(),
            severity: "warn".to_string(),
            free_text: None,
            session_id: "test-session".to_string(),
            run_id: None,
            timestamp: now_iso(),
        };
        let response = post_cortex_feedback_ux(Json(payload)).await;
        assert_eq!(response.status(), StatusCode::OK);

        let Json(queue_before) =
            get_cortex_feedback_ux(Query(CortexFeedbackQuery::default())).await;
        let queue_id = queue_before
            .items
            .iter()
            .find(|item| {
                item.route_id == "/studio"
                    && item.view_id == "view.studio"
                    && item.friction_tag == "layout"
            })
            .expect("feedback queue item for triage test")
            .queue_id
            .clone();

        let triage_req = CortexFeedbackTriageRequest {
            queue_id,
            status: "triaged".to_string(),
            priority: Some("high".to_string()),
            assigned_to: Some("Systems Steward".to_string()),
            notes: Some("validated for candidate scoring".to_string()),
            baseline_metric_date: Some("2026-02-09T03:00:00Z".to_string()),
            post_release_metric_date: Some("2026-02-16T03:00:00Z".to_string()),
        };
        let triage_resp = post_cortex_feedback_triage(Json(triage_req)).await;
        assert_eq!(triage_resp.status(), StatusCode::OK);
        let body = response_json(triage_resp).await;
        assert_eq!(body["accepted"], true);
        assert_eq!(body["item"]["status"], "triaged");
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn artifacts_publish_requires_steward_role() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut create_headers = HeaderMap::new();
        create_headers.insert(
            "x-cortex-role",
            "operator".parse().expect("operator header"),
        );
        let create_response = post_cortex_artifact_create(
            create_headers,
            Json(ArtifactCreateRequest {
                artifact_id: Some("artifact-test-1".to_string()),
                title: "Artifact".to_string(),
                content: Some("content".to_string()),
                markdown_source: None,
            }),
        )
        .await;
        assert_eq!(create_response.status(), StatusCode::OK);

        let mut publish_headers = HeaderMap::new();
        publish_headers.insert(
            "x-cortex-role",
            "operator".parse().expect("operator header"),
        );
        let publish_response = post_cortex_artifact_publish(
            publish_headers,
            Path("artifact-test-1".to_string()),
            Json(ArtifactPublishRequest {
                lease_id: None,
                expected_revision_id: None,
                notes: Some("ship it".to_string()),
                governance: None,
            }),
        )
        .await;
        assert_eq!(publish_response.status(), StatusCode::FORBIDDEN);
    }

    #[tokio::test]
    async fn artifacts_save_requires_active_lease_and_matching_revision() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert("x-cortex-actor", "actor-1".parse().expect("actor header"));
        let create_response = post_cortex_artifact_create(
            headers.clone(),
            Json(ArtifactCreateRequest {
                artifact_id: Some("artifact-save-1".to_string()),
                title: "Artifact Save".to_string(),
                content: Some("# title".to_string()),
                markdown_source: None,
            }),
        )
        .await;
        assert_eq!(create_response.status(), StatusCode::OK);
        let create_body = response_json(create_response).await;
        let head_revision = create_body["headRevisionId"]
            .as_str()
            .expect("head revision")
            .to_string();

        let checkout_response = post_cortex_artifact_checkout(
            headers.clone(),
            Path("artifact-save-1".to_string()),
            Json(ArtifactCheckoutRequest::default()),
        )
        .await;
        assert_eq!(checkout_response.status(), StatusCode::OK);
        let checkout_body = response_json(checkout_response).await;
        let lease_id = checkout_body["leaseId"].as_str().expect("lease id");

        let save_response = post_cortex_artifact_save(
            headers.clone(),
            Path("artifact-save-1".to_string()),
            Json(ArtifactSaveRequest {
                lease_id: lease_id.to_string(),
                expected_revision_id: head_revision,
                markdown_source: "# updated".to_string(),
                title: None,
                notes: Some("save".to_string()),
            }),
        )
        .await;
        assert_eq!(save_response.status(), StatusCode::OK);

        let revisions_response =
            get_cortex_artifact_revisions(Path("artifact-save-1".to_string())).await;
        assert_eq!(revisions_response.status(), StatusCode::OK);
        let revisions_body = response_json(revisions_response).await;
        assert_eq!(
            revisions_body["revisions"]
                .as_array()
                .map(|items| items.len()),
            Some(2)
        );
    }

    #[tokio::test]
    async fn heap_emit_rejects_invalid_workspace_id_and_logs_rejection() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-1".parse().expect("actor header"),
        );

        let mut payload = heap_emit_fixture(
            "req-invalid-workspace",
            "Invalid Workspace",
            "2026-02-23T10:00:00Z",
            true,
            "hash:file_size",
        );
        payload["workspace_id"] = json!("not-a-ulid");

        let response = post_cortex_heap_emit(headers, Json(payload)).await;
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "HEAP_SCHEMA_INVALID");

        let rejections: Vec<HeapEmitRejectionEvent> =
            read_jsonl_vec(&cortex_ux_heap_emit_rejections_log_path());
        assert_eq!(rejections.len(), 1);
        assert_eq!(rejections[0].code, "HEAP_SCHEMA_INVALID");
    }

    #[tokio::test]
    async fn heap_emit_is_idempotent_and_canonicalizes_file_keys() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-2".parse().expect("actor header"),
        );

        let payload = heap_emit_fixture(
            "req-idempotent",
            "Idempotent Block",
            "2026-02-23T10:05:00Z",
            true,
            "hash",
        );

        let first = post_cortex_heap_emit(headers.clone(), Json(payload.clone())).await;
        assert_eq!(first.status(), StatusCode::OK);
        let first_body = response_json(first).await;
        assert_eq!(first_body["accepted"], true);
        assert_eq!(first_body["idempotent"], false);

        let second = post_cortex_heap_emit(headers.clone(), Json(payload)).await;
        assert_eq!(second.status(), StatusCode::OK);
        let second_body = response_json(second).await;
        assert_eq!(second_body["accepted"], true);
        assert_eq!(second_body["idempotent"], true);

        let projections = read_heap_projection_store();
        assert_eq!(projections.len(), 1);
        assert_eq!(projections[0].projection.file_keys, vec!["abc12345:42"]);
        assert_eq!(projections[0].projection.tags.len(), 1);

        let query_response = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            tag: Some("01ARZ3NDEKTSV4RRFFQ69G5FAX".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(query_response.status(), StatusCode::OK);
        let query_body = response_json(query_response).await;
        assert_eq!(query_body["count"], 1);
        assert_eq!(
            query_body["items"][0]["projection"]["pageLinks"][0],
            "01ARZ3NDEKTSV4RRFFQ69G5FAZ"
        );
    }

    #[tokio::test]
    async fn heap_emit_mention_mirror_policy_controls_query_visibility() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-3".parse().expect("actor header"),
        );

        let payload = heap_emit_fixture(
            "req-mention-policy",
            "Mention Policy Block",
            "2026-02-23T10:10:00Z",
            false,
            "hash:file_size",
        );
        let emit = post_cortex_heap_emit(headers, Json(payload)).await;
        assert_eq!(emit.status(), StatusCode::OK);

        let mention_query = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            mention: Some("Project Alpha".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(mention_query.status(), StatusCode::OK);
        let mention_body = response_json(mention_query).await;
        assert_eq!(mention_body["count"], 0);
    }

    #[tokio::test]
    async fn heap_query_attribute_filter_matches_projected_attributes() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-attr".parse().expect("actor header"),
        );

        let mut payload = heap_emit_fixture(
            "req-attribute-filter",
            "Attribute Filter Block",
            "2026-02-23T10:12:00Z",
            true,
            "hash:file_size",
        );
        payload["block"]["attributes"] = json!({
            "priority": "P0",
            "component": "gateway"
        });

        let emit = post_cortex_heap_emit(headers, Json(payload)).await;
        assert_eq!(emit.status(), StatusCode::OK);

        let by_key_value = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            attribute: Some("priority:P0".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(by_key_value.status(), StatusCode::OK);
        let by_key_value_body = response_json(by_key_value).await;
        assert_eq!(by_key_value_body["count"], 1);

        let by_key_only = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            attribute: Some("component".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(by_key_only.status(), StatusCode::OK);
        let by_key_only_body = response_json(by_key_only).await;
        assert_eq!(by_key_only_body["count"], 1);
    }

    #[tokio::test]
    async fn heap_query_page_link_filter_matches_projection() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-pagelink".parse().expect("actor header"),
        );

        let payload = heap_emit_fixture(
            "req-page-link-filter",
            "Page Link Filter Block",
            "2026-02-23T10:14:00Z",
            true,
            "hash:file_size",
        );
        let emit = post_cortex_heap_emit(headers, Json(payload)).await;
        assert_eq!(emit.status(), StatusCode::OK);

        let filtered = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            page_link: Some("01ARZ3NDEKTSV4RRFFQ69G5FAZ".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(filtered.status(), StatusCode::OK);
        let filtered_body = response_json(filtered).await;
        assert_eq!(filtered_body["count"], 1);

        let none = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            page_link: Some("01ARZ3NDEKTSV4RRFFQ69G5FAA".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(none.status(), StatusCode::OK);
        let none_body = response_json(none).await;
        assert_eq!(none_body["count"], 0);
    }

    #[tokio::test]
    async fn heap_query_changed_since_alias_matches_from_ts() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-changed-since".parse().expect("actor header"),
        );

        let payload = heap_emit_fixture(
            "req-changed-since",
            "Changed Since Alias Block",
            "2026-02-23T10:18:00Z",
            true,
            "hash:file_size",
        );
        let emit = post_cortex_heap_emit(headers, Json(payload)).await;
        assert_eq!(emit.status(), StatusCode::OK);

        let by_from_ts = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            from_ts: Some("2026-02-23T10:00:00Z".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(by_from_ts.status(), StatusCode::OK);
        let by_from_ts_body = response_json(by_from_ts).await;

        let by_changed_since = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            changed_since: Some("2026-02-23T10:00:00Z".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(by_changed_since.status(), StatusCode::OK);
        let by_changed_since_body = response_json(by_changed_since).await;

        assert_eq!(by_from_ts_body["count"], by_changed_since_body["count"]);

        let from_ts_wins = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            from_ts: Some("2026-02-23T10:00:00Z".to_string()),
            changed_since: Some("not-a-date".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(from_ts_wins.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn heap_query_usage_metrics_track_changed_since_alias_and_page_link_filter() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        reset_heap_gateway_usage_metrics();

        let response = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            changed_since: Some("2026-02-23T10:00:00Z".to_string()),
            page_link: Some("01ARZ3NDEKTSV4RRFFQ69G5FAZ".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let after_alias = heap_gateway_usage_snapshot();
        assert_eq!(after_alias.blocks_changed_since_alias_hits, 1);
        assert_eq!(after_alias.blocks_page_link_filter_hits, 1);

        let from_ts_wins = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            from_ts: Some("2026-02-23T10:00:00Z".to_string()),
            changed_since: Some("not-a-date".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(from_ts_wins.status(), StatusCode::OK);
        let after_precedence = heap_gateway_usage_snapshot();
        assert_eq!(after_precedence.blocks_changed_since_alias_hits, 1);
        assert_eq!(after_precedence.blocks_page_link_filter_hits, 1);
    }

    #[tokio::test]
    async fn heap_changed_blocks_usage_metrics_track_endpoint_alias_and_page_link_filter() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        reset_heap_gateway_usage_metrics();

        let response = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            changed_since: Some("2026-02-23T10:00:00Z".to_string()),
            page_link: Some("01ARZ3NDEKTSV4RRFFQ69G5FAZ".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let first = heap_gateway_usage_snapshot();
        assert_eq!(first.changed_blocks_endpoint_hits, 1);
        assert_eq!(first.changed_blocks_changed_since_alias_hits, 1);
        assert_eq!(first.changed_blocks_page_link_filter_hits, 1);

        let second_response = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(second_response.status(), StatusCode::OK);
        let second = heap_gateway_usage_snapshot();
        assert_eq!(second.changed_blocks_endpoint_hits, 2);
        assert_eq!(second.changed_blocks_changed_since_alias_hits, 1);
        assert_eq!(second.changed_blocks_page_link_filter_hits, 1);
    }

    #[tokio::test]
    async fn heap_query_cursor_paginates_in_stable_reverse_chron_order() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-4".parse().expect("actor header"),
        );

        let first_payload = heap_emit_fixture(
            "req-page-1",
            "First Block",
            "2026-02-23T10:15:00Z",
            true,
            "hash:file_size",
        );
        let second_payload = heap_emit_fixture(
            "req-page-2",
            "Second Block",
            "2026-02-23T10:16:00Z",
            true,
            "hash:file_size",
        );

        let first_emit = post_cortex_heap_emit(headers.clone(), Json(first_payload)).await;
        assert_eq!(first_emit.status(), StatusCode::OK);
        let second_emit = post_cortex_heap_emit(headers, Json(second_payload)).await;
        assert_eq!(second_emit.status(), StatusCode::OK);

        let first_page = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(1),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(first_page.status(), StatusCode::OK);
        let first_page_body = response_json(first_page).await;
        assert_eq!(first_page_body["count"], 1);
        assert_eq!(first_page_body["hasMore"], true);
        let first_artifact_id = first_page_body["items"][0]["projection"]["artifactId"]
            .as_str()
            .expect("first artifact id")
            .to_string();
        let cursor = first_page_body["nextCursor"]
            .as_str()
            .expect("cursor")
            .to_string();

        let second_page = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(1),
            cursor: Some(cursor),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(second_page.status(), StatusCode::OK);
        let second_page_body = response_json(second_page).await;
        assert_eq!(second_page_body["count"], 1);
        let second_artifact_id = second_page_body["items"][0]["projection"]["artifactId"]
            .as_str()
            .expect("second artifact id")
            .to_string();
        assert_ne!(first_artifact_id, second_artifact_id);
    }

    #[tokio::test]
    async fn heap_changed_blocks_returns_changed_and_deleted_with_cursor() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-delta".parse().expect("actor header"),
        );

        let first_payload = heap_emit_fixture(
            "req-delta-a",
            "Delta A",
            "2026-02-23T10:30:00Z",
            true,
            "hash:file_size",
        );
        let second_payload = heap_emit_fixture(
            "req-delta-b",
            "Delta B",
            "2026-02-23T10:31:00Z",
            true,
            "hash:file_size",
        );
        let first_emit = post_cortex_heap_emit(headers.clone(), Json(first_payload)).await;
        assert_eq!(first_emit.status(), StatusCode::OK);
        let second_emit = post_cortex_heap_emit(headers.clone(), Json(second_payload)).await;
        assert_eq!(second_emit.status(), StatusCode::OK);

        let delete =
            post_cortex_heap_block_delete(headers, Path("artifact-req-delta-a".to_string())).await;
        assert_eq!(delete.status(), StatusCode::OK);

        let first_page = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(1),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(first_page.status(), StatusCode::OK);
        let first_page_body = response_json(first_page).await;
        assert_eq!(first_page_body["count"], 1);
        assert_eq!(first_page_body["hasMore"], true);
        let cursor = first_page_body["nextCursor"]
            .as_str()
            .expect("cursor")
            .to_string();

        let second_page = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(1),
            cursor: Some(cursor),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(second_page.status(), StatusCode::OK);
        let second_page_body = response_json(second_page).await;
        assert_eq!(second_page_body["count"], 1);
        assert_eq!(second_page_body["hasMore"], false);

        let full = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(full.status(), StatusCode::OK);
        let full_body = response_json(full).await;
        assert_eq!(full_body["count"], 2);
        assert_eq!(
            full_body["changed"].as_array().map(|items| items.len()),
            Some(1)
        );
        assert_eq!(
            full_body["deleted"].as_array().map(|items| items.len()),
            Some(1)
        );
    }

    #[tokio::test]
    async fn heap_changed_blocks_honors_space_scope() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-delta-scope".parse().expect("actor header"),
        );

        let payload_a = heap_emit_fixture(
            "req-delta-scope-a",
            "Scope A",
            "2026-02-23T10:34:00Z",
            true,
            "hash:file_size",
        );
        let emit_a = post_cortex_heap_emit(headers.clone(), Json(payload_a)).await;
        assert_eq!(emit_a.status(), StatusCode::OK);

        let mut payload_b = heap_emit_fixture(
            "req-delta-scope-b",
            "Scope B",
            "2026-02-23T10:35:00Z",
            true,
            "hash:file_size",
        );
        payload_b["workspace_id"] = json!("01ARZ3NDEKTSV4RRFFQ69G5FB0");
        let emit_b = post_cortex_heap_emit(headers, Json(payload_b)).await;
        assert_eq!(emit_b.status(), StatusCode::OK);

        let scoped = get_cortex_heap_changed_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(scoped.status(), StatusCode::OK);
        let scoped_body = response_json(scoped).await;
        assert_eq!(scoped_body["count"], 1);
    }

    #[tokio::test]
    async fn heap_block_actions_pin_and_delete_update_projection_and_visibility() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-cortex-actor",
            "actor-heap-5".parse().expect("actor header"),
        );

        let payload = heap_emit_fixture(
            "req-actions",
            "Action Block",
            "2026-02-23T10:20:00Z",
            true,
            "hash:file_size",
        );
        let emit = post_cortex_heap_emit(headers.clone(), Json(payload)).await;
        assert_eq!(emit.status(), StatusCode::OK);

        let pin =
            post_cortex_heap_block_pin(headers.clone(), Path("artifact-req-actions".to_string()))
                .await;
        assert_eq!(pin.status(), StatusCode::OK);
        let pin_body = response_json(pin).await;
        assert_eq!(pin_body["accepted"], true);
        assert_eq!(pin_body["action"], "pin");

        let visible_query = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(visible_query.status(), StatusCode::OK);
        let visible_body = response_json(visible_query).await;
        assert_eq!(visible_body["count"], 1);
        assert!(visible_body["items"][0]["pinnedAt"].is_string());
        assert!(visible_body["items"][0]["deletedAt"].is_null());

        let delete =
            post_cortex_heap_block_delete(headers, Path("artifact-req-actions".to_string())).await;
        assert_eq!(delete.status(), StatusCode::OK);
        let delete_body = response_json(delete).await;
        assert_eq!(delete_body["accepted"], true);
        assert_eq!(delete_body["action"], "delete");

        let default_query = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(10),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(default_query.status(), StatusCode::OK);
        let default_body = response_json(default_query).await;
        assert_eq!(default_body["count"], 0);

        let include_deleted_query = get_cortex_heap_blocks(Query(HeapBlocksQuery {
            space_id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string()),
            limit: Some(10),
            include_deleted: Some(true),
            ..HeapBlocksQuery::default()
        }))
        .await;
        assert_eq!(include_deleted_query.status(), StatusCode::OK);
        let include_deleted_body = response_json(include_deleted_query).await;
        assert_eq!(include_deleted_body["count"], 1);
        assert!(include_deleted_body["items"][0]["deletedAt"].is_string());
    }

    #[tokio::test]
    async fn capability_graph_response_remains_backward_compatible_with_additive_metadata() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_capability_graph().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;

        assert!(body["schema_version"].is_string());
        assert!(body["generated_at"].is_string());
        assert!(body["source_of_truth"].is_string());
        assert!(body["nodes"].is_array());
        assert!(body["edges"].is_array());

        assert!(body["graph_hash"].is_string());
        assert!(body["capabilities_version"].is_string());
        assert!(body["layout_hints"].is_object());
        assert!(body["legend"].is_object());
    }

    #[tokio::test]
    async fn capability_graph_hash_is_deterministic_for_same_contract_state() {
        let _lock = acquire_testing_env_lock();
        let first = get_system_capability_graph().await.into_response();
        let second = get_system_capability_graph().await.into_response();
        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(second.status(), StatusCode::OK);

        let first_body = response_json(first).await;
        let second_body = response_json(second).await;

        assert_eq!(first_body["graph_hash"], second_body["graph_hash"]);
        assert_eq!(first_body["nodes"], second_body["nodes"]);
        assert_eq!(first_body["edges"], second_body["edges"]);
        assert_eq!(
            first_body["capabilities_version"],
            second_body["capabilities_version"]
        );
    }

    #[tokio::test]
    async fn system_capability_catalog_returns_versioned_catalog() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_capability_catalog().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["schemaVersion"], "1.0.0");
        assert!(body["catalogVersion"].is_string());
        assert!(body["catalogHash"].is_string());
        assert!(body["nodes"].is_array());
        assert!(body["edges"].is_array());
    }

    #[tokio::test]
    async fn space_capability_graph_put_requires_steward_role() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _workspace_guard = EnvVarGuard::set(
            "NOSTRA_WORKSPACE_ROOT",
            temp.path().display().to_string().as_str(),
        );

        let create = post_create_space(Json(json!({
            "space_id": "space-cap-gate",
            "creation_mode": "blank",
            "owner": "systems-steward"
        })))
        .await
        .into_response();
        assert_eq!(create.status(), StatusCode::CREATED);

        let baseline = get_space_capability_graph(Path("space-cap-gate".to_string()))
            .await
            .into_response();
        assert_eq!(baseline.status(), StatusCode::OK);
        let baseline_body = response_json(baseline).await;
        let mut graph: SpaceCapabilityGraph =
            serde_json::from_value(baseline_body).expect("parse graph");
        graph.lineage_ref = Some("decision:space-cap-gate-1".to_string());

        let mut operator_headers = HeaderMap::new();
        operator_headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        operator_headers.insert(
            "x-cortex-actor",
            "operator-1".parse().expect("actor header"),
        );
        let rejected = put_space_capability_graph(
            Path("space-cap-gate".to_string()),
            operator_headers,
            Json(graph.clone()),
        )
        .await
        .into_response();
        assert_eq!(rejected.status(), StatusCode::FORBIDDEN);

        let mut steward_headers = HeaderMap::new();
        steward_headers.insert("x-cortex-role", "steward".parse().expect("role header"));
        steward_headers.insert("x-cortex-actor", "steward-1".parse().expect("actor header"));
        let accepted = put_space_capability_graph(
            Path("space-cap-gate".to_string()),
            steward_headers,
            Json(graph),
        )
        .await
        .into_response();
        assert_eq!(accepted.status(), StatusCode::OK);
        let accepted_body = response_json(accepted).await;
        assert_eq!(accepted_body["accepted"], true);
        assert!(accepted_body["capabilityGraphHash"].is_string());
    }

    #[tokio::test]
    async fn space_navigation_plan_is_deterministic_and_role_filtered() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _workspace_guard = EnvVarGuard::set(
            "NOSTRA_WORKSPACE_ROOT",
            temp.path().display().to_string().as_str(),
        );

        let create = post_create_space(Json(json!({
            "space_id": "space-nav-plan",
            "creation_mode": "blank",
            "owner": "systems-steward"
        })))
        .await
        .into_response();
        assert_eq!(create.status(), StatusCode::CREATED);

        let first = get_space_navigation_plan(
            Path("space-nav-plan".to_string()),
            Query(SpaceNavigationPlanQuery {
                actor_role: Some("viewer".to_string()),
                intent: Some("navigate".to_string()),
                density: Some("comfortable".to_string()),
            }),
        )
        .await
        .into_response();
        let second = get_space_navigation_plan(
            Path("space-nav-plan".to_string()),
            Query(SpaceNavigationPlanQuery {
                actor_role: Some("viewer".to_string()),
                intent: Some("navigate".to_string()),
                density: Some("comfortable".to_string()),
            }),
        )
        .await
        .into_response();

        assert_eq!(first.status(), StatusCode::OK);
        assert_eq!(second.status(), StatusCode::OK);
        let first_body = response_json(first).await;
        let second_body = response_json(second).await;
        assert_eq!(first_body["planHash"], second_body["planHash"]);

        let visible_routes: Vec<&serde_json::Value> = first_body["entries"]
            .as_array()
            .expect("entries array")
            .iter()
            .filter(|entry| entry["requiredRole"].as_str().unwrap_or_default() == "viewer")
            .collect();
        assert!(!visible_routes.is_empty());
        assert!(
            !first_body["entries"]
                .as_array()
                .expect("entries array")
                .iter()
                .any(|entry| entry["routeId"].as_str() == Some("/logs")),
            "viewer plan should not include operator-only /logs"
        );
    }

    #[tokio::test]
    async fn feedback_overdue_returns_shipped_items_past_threshold() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );

        write_feedback_queue_items(&[UxFeedbackQueueItem {
            queue_id: "queue-overdue-1".to_string(),
            dedupe_key: "route|view|tag|action".to_string(),
            route_id: "/workflows".to_string(),
            view_id: "view.workflows".to_string(),
            friction_tag: "latency".to_string(),
            severity: "warn".to_string(),
            status: UX_STATUS_SHIPPED.to_string(),
            priority: "high".to_string(),
            assigned_to: Some("Systems Steward".to_string()),
            notes: None,
            baseline_metric_date: Some("2026-01-01T00:00:00Z".to_string()),
            post_release_metric_date: Some("2026-01-05T00:00:00Z".to_string()),
            first_seen_at: "2025-12-31T00:00:00Z".to_string(),
            updated_at: "2026-01-01T00:00:00Z".to_string(),
            event_count: 4,
        }])
        .expect("write feedback queue");

        let Json(response) =
            get_cortex_feedback_overdue(Query(CortexFeedbackOverdueQuery { days: Some(7) })).await;
        assert_eq!(response.threshold_days, 7);
        assert_eq!(response.items.len(), 1);
        assert_eq!(response.items[0].queue_id, "queue-overdue-1");
    }

    #[tokio::test]
    async fn cortex_layout_source_state_reports_fallback_without_workflow_engine_id() {
        let _lock = acquire_testing_env_lock();
        let _guard = EnvVarGuard::unset("CANISTER_ID_WORKFLOW_ENGINE");
        let Json(state) = get_cortex_layout_source_state().await;
        assert_eq!(state.source_of_truth, "local_json");
        assert!(state.fallback_active);
    }

    #[test]
    fn cortex_layout_drift_report_schema_is_stable() {
        let _lock = acquire_testing_env_lock();
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        let Json(report) = runtime.block_on(get_cortex_layout_drift_report());
        assert_eq!(report.schema_version, "1.0.0");
        assert!(!report.generated_at.trim().is_empty());
        assert!(!report.source_of_truth.trim().is_empty());
        assert!(
            report
                .route_diff
                .iter()
                .all(|entry| !entry.trim().is_empty())
        );
        assert!(
            report
                .capability_diff
                .iter()
                .all(|entry| !entry.trim().is_empty())
        );
        assert!(
            report
                .pattern_diff
                .iter()
                .all(|entry| !entry.trim().is_empty())
        );
    }

    #[tokio::test]
    async fn cortex_runtime_sync_endpoints_report_schema() {
        let _lock = acquire_testing_env_lock();
        let _guard = EnvVarGuard::unset("CANISTER_ID_WORKFLOW_ENGINE");

        let Json(status) = get_cortex_runtime_sync_status().await;
        assert_eq!(status.schema_version, "1.0.0");
        assert!(!status.generated_at.trim().is_empty());
        assert!(
            status.mode == "local_mirror_fallback" || status.mode == "workflow_engine_vfs_primary"
        );

        let replay_response = post_cortex_runtime_sync_replay().await;
        assert_eq!(replay_response.status(), StatusCode::OK);
        let replay_body = response_json(replay_response).await;
        assert_eq!(replay_body["schemaVersion"], "1.0.0");
        assert!(replay_body["attempted"].is_number());
        assert!(replay_body["pendingAfter"].is_number());
    }

    #[test]
    fn artifact_collab_session_open_op_close_roundtrip() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        let _wf_guard = EnvVarGuard::unset("CANISTER_ID_WORKFLOW_ENGINE");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        runtime.block_on(async {
            let mut headers = HeaderMap::new();
            headers.insert("x-cortex-role", "operator".parse().expect("role header"));
            headers.insert(
                "x-cortex-actor",
                "actor-collab-1".parse().expect("actor header"),
            );

            let create_response = post_cortex_artifact_create(
                headers.clone(),
                Json(ArtifactCreateRequest {
                    artifact_id: Some("artifact-collab-1".to_string()),
                    title: "Artifact Collab".to_string(),
                    content: Some("# collab".to_string()),
                    markdown_source: None,
                }),
            )
            .await;
            assert_eq!(create_response.status(), StatusCode::OK);
            let create_body = response_json(create_response).await;
            let head_revision = create_body["headRevisionId"]
                .as_str()
                .expect("head revision")
                .to_string();

            let session_open = post_cortex_artifact_collab_session_open(
                headers.clone(),
                Path("artifact-collab-1".to_string()),
                Json(ArtifactCollabSessionOpenRequest {
                    lease_ttl_secs: Some(300),
                }),
            )
            .await;
            assert_eq!(session_open.status(), StatusCode::OK);
            let session_body = response_json(session_open).await;
            let session_id = session_body["sessionId"]
                .as_str()
                .expect("session id")
                .to_string();

            let op_response = post_cortex_artifact_collab_op(
                headers.clone(),
                Path("artifact-collab-1".to_string()),
                Json(ArtifactCollabOpRequest {
                    session_id: session_id.clone(),
                    expected_head_revision_id: head_revision,
                    proposed_base_revision_id: None,
                    op_type: "replace_markdown".to_string(),
                    payload_markdown: "# collab updated".to_string(),
                }),
            )
            .await;
            assert_eq!(op_response.status(), StatusCode::OK);
            let op_body = response_json(op_response).await;
            assert_eq!(op_body["op"]["sequence"], 1);
            assert_eq!(op_body["mergeResult"]["mergeStatus"], "applied_head");

            let list_response =
                get_cortex_artifact_collab_session(Path("artifact-collab-1".to_string())).await;
            assert_eq!(list_response.status(), StatusCode::OK);
            let list_body = response_json(list_response).await;
            assert_eq!(
                list_body["sessions"].as_array().map(|items| items.len()),
                Some(1)
            );

            let close_response = post_cortex_artifact_collab_session_close(
                headers,
                Path("artifact-collab-1".to_string()),
                Json(ArtifactCollabSessionCloseRequest { session_id }),
            )
            .await;
            assert_eq!(close_response.status(), StatusCode::OK);
            let close_body = response_json(close_response).await;
            assert_eq!(close_body["active"], false);
        });
    }

    #[test]
    fn artifact_collab_batch_ops_state_presence_and_ordering_roundtrip() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        let _wf_guard = EnvVarGuard::unset("CANISTER_ID_WORKFLOW_ENGINE");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        runtime.block_on(async {
            let mut actor_one_headers = HeaderMap::new();
            actor_one_headers.insert("x-cortex-role", "operator".parse().expect("role header"));
            actor_one_headers.insert(
                "x-cortex-actor",
                "actor-crdt-1".parse().expect("actor header"),
            );

            let create_response = post_cortex_artifact_create(
                actor_one_headers.clone(),
                Json(ArtifactCreateRequest {
                    artifact_id: Some("artifact-crdt-1".to_string()),
                    title: "Artifact CRDT".to_string(),
                    content: Some("# seed".to_string()),
                    markdown_source: None,
                }),
            )
            .await;
            assert_eq!(create_response.status(), StatusCode::OK);

            let session_open = post_cortex_artifact_collab_session_open(
                actor_one_headers.clone(),
                Path("artifact-crdt-1".to_string()),
                Json(ArtifactCollabSessionOpenRequest {
                    lease_ttl_secs: Some(300),
                }),
            )
            .await;
            assert_eq!(session_open.status(), StatusCode::OK);
            let session_body = response_json(session_open).await;
            let session_id = session_body["sessionId"]
                .as_str()
                .expect("session id")
                .to_string();

            let batch_one = post_cortex_artifact_collab_op_batch(
                actor_one_headers.clone(),
                Path("artifact-crdt-1".to_string()),
                Json(ArtifactCollabOpBatchRequest {
                    session_id: session_id.clone(),
                    batch_sequence: 1,
                    expected_head_revision_id: None,
                    operations: vec![
                        ArtifactCollabBatchOperation {
                            op_id: "op-b".to_string(),
                            lamport: 20,
                            markdown_source: "# from op-b".to_string(),
                            stream_channel: Some("stream:cortex:artifact-crdt-1".to_string()),
                        },
                        ArtifactCollabBatchOperation {
                            op_id: "op-a".to_string(),
                            lamport: 10,
                            markdown_source: "# from op-a".to_string(),
                            stream_channel: Some("stream:cortex:artifact-crdt-1".to_string()),
                        },
                    ],
                    cursor: Some(ArtifactCollabCursor {
                        line: 1,
                        column: 1,
                        selection_start: None,
                        selection_end: None,
                    }),
                }),
            )
            .await;
            assert_eq!(batch_one.status(), StatusCode::OK);
            let batch_one_body = response_json(batch_one).await;
            assert_eq!(batch_one_body["applied"].as_u64(), Some(2));
            assert_eq!(batch_one_body["idempotent"].as_u64(), Some(0));
            assert_eq!(batch_one_body["materializedMarkdown"], "# from op-b");

            let stale_sequence = post_cortex_artifact_collab_op_batch(
                actor_one_headers.clone(),
                Path("artifact-crdt-1".to_string()),
                Json(ArtifactCollabOpBatchRequest {
                    session_id: session_id.clone(),
                    batch_sequence: 1,
                    expected_head_revision_id: None,
                    operations: vec![ArtifactCollabBatchOperation {
                        op_id: "op-stale".to_string(),
                        lamport: 30,
                        markdown_source: "# stale".to_string(),
                        stream_channel: None,
                    }],
                    cursor: None,
                }),
            )
            .await;
            assert_eq!(stale_sequence.status(), StatusCode::CONFLICT);

            let mut actor_two_headers = HeaderMap::new();
            actor_two_headers.insert("x-cortex-role", "operator".parse().expect("role header"));
            actor_two_headers.insert(
                "x-cortex-actor",
                "actor-crdt-2".parse().expect("actor header"),
            );

            let batch_two = post_cortex_artifact_collab_op_batch(
                actor_two_headers.clone(),
                Path("artifact-crdt-1".to_string()),
                Json(ArtifactCollabOpBatchRequest {
                    session_id: session_id.clone(),
                    batch_sequence: 2,
                    expected_head_revision_id: None,
                    operations: vec![
                        ArtifactCollabBatchOperation {
                            op_id: "op-a".to_string(),
                            lamport: 31,
                            markdown_source: "# duplicate".to_string(),
                            stream_channel: None,
                        },
                        ArtifactCollabBatchOperation {
                            op_id: "op-c".to_string(),
                            lamport: 32,
                            markdown_source: "# from actor2".to_string(),
                            stream_channel: None,
                        },
                    ],
                    cursor: Some(ArtifactCollabCursor {
                        line: 1,
                        column: 8,
                        selection_start: None,
                        selection_end: None,
                    }),
                }),
            )
            .await;
            assert_eq!(batch_two.status(), StatusCode::OK);
            let batch_two_body = response_json(batch_two).await;
            assert_eq!(batch_two_body["applied"].as_u64(), Some(1));
            assert_eq!(batch_two_body["idempotent"].as_u64(), Some(1));
            assert_eq!(batch_two_body["materializedMarkdown"], "# from actor2");

            let state_response =
                get_cortex_artifact_collab_state(Path("artifact-crdt-1".to_string())).await;
            assert_eq!(state_response.status(), StatusCode::OK);
            let state_body = response_json(state_response).await;
            assert_eq!(state_body["materializedMarkdown"], "# from actor2");
            assert!(
                state_body["opCount"].as_u64().unwrap_or_default() >= 3,
                "expected at least three applied operations"
            );
            assert!(
                state_body["presence"]
                    .as_array()
                    .map(|items| items.len())
                    .unwrap_or_default()
                    >= 2
            );

            let ops_response = get_cortex_artifact_collab_ops(
                Path("artifact-crdt-1".to_string()),
                Query(ArtifactCollabOpsQuery {
                    since_sequence: None,
                    limit: Some(20),
                }),
            )
            .await;
            assert_eq!(ops_response.status(), StatusCode::OK);
            let ops_body = response_json(ops_response).await;
            assert_eq!(ops_body["count"].as_u64(), Some(3));
        });
    }

    #[test]
    fn artifact_collab_force_resolve_requires_steward_and_records_governance() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        let _wf_guard = EnvVarGuard::unset("CANISTER_ID_WORKFLOW_ENGINE");
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("build tokio runtime");
        runtime.block_on(async {
            let mut operator_headers = HeaderMap::new();
            operator_headers.insert("x-cortex-role", "operator".parse().expect("role header"));
            operator_headers.insert(
                "x-cortex-actor",
                "actor-force-1".parse().expect("actor header"),
            );

            let create_response = post_cortex_artifact_create(
                operator_headers.clone(),
                Json(ArtifactCreateRequest {
                    artifact_id: Some("artifact-force-1".to_string()),
                    title: "Artifact Force Resolve".to_string(),
                    content: Some("# seed".to_string()),
                    markdown_source: None,
                }),
            )
            .await;
            assert_eq!(create_response.status(), StatusCode::OK);

            let session_open = post_cortex_artifact_collab_session_open(
                operator_headers.clone(),
                Path("artifact-force-1".to_string()),
                Json(ArtifactCollabSessionOpenRequest {
                    lease_ttl_secs: Some(300),
                }),
            )
            .await;
            assert_eq!(session_open.status(), StatusCode::OK);
            let session_body = response_json(session_open).await;
            let session_id = session_body["sessionId"]
                .as_str()
                .expect("session id")
                .to_string();

            let denied = post_cortex_artifact_collab_force_resolve(
                operator_headers.clone(),
                Path("artifact-force-1".to_string()),
                Json(ArtifactCollabForceResolveRequest {
                    session_id: session_id.clone(),
                    markdown_source: "# operator denied".to_string(),
                    approved_by: "Ops".to_string(),
                    rationale: "attempt without steward".to_string(),
                    approved_at: "2026-02-09T12:00:00Z".to_string(),
                    governance: None,
                    cursor: None,
                }),
            )
            .await;
            assert_eq!(denied.status(), StatusCode::FORBIDDEN);

            let mut steward_headers = HeaderMap::new();
            steward_headers.insert("x-cortex-role", "steward".parse().expect("role header"));
            steward_headers.insert(
                "x-cortex-actor",
                "steward-force-1".parse().expect("actor header"),
            );

            let approved = post_cortex_artifact_collab_force_resolve(
                steward_headers,
                Path("artifact-force-1".to_string()),
                Json(ArtifactCollabForceResolveRequest {
                    session_id: session_id.clone(),
                    markdown_source: "# steward force resolved".to_string(),
                    approved_by: "Systems Steward".to_string(),
                    rationale: "resolve deterministic divergence".to_string(),
                    approved_at: "2026-02-09T12:05:00Z".to_string(),
                    governance: Some(test_governance_envelope(
                        "steward-force-1",
                        "Systems Steward",
                        "resolve deterministic divergence",
                        "2026-02-09T12:05:00Z",
                        "decision-force-resolve-1",
                    )),
                    cursor: Some(ArtifactCollabCursor {
                        line: 1,
                        column: 1,
                        selection_start: None,
                        selection_end: None,
                    }),
                }),
            )
            .await;
            assert_eq!(approved.status(), StatusCode::OK);
            let body = response_json(approved).await;
            assert_eq!(body["accepted"], true);
            assert_eq!(
                body["promotionDecision"]["promotionAction"],
                "force_resolve_conflict"
            );

            let state =
                get_cortex_artifact_collab_state(Path("artifact-force-1".to_string())).await;
            assert_eq!(state.status(), StatusCode::OK);
            let state_body = response_json(state).await;
            assert_eq!(
                state_body["materializedMarkdown"],
                "# steward force resolved"
            );
        });
    }

    #[test]
    fn artifact_governance_nonce_replay_is_rejected() {
        let _lock = acquire_testing_env_lock();
        let envelope = test_governance_envelope(
            "steward-replay-1",
            "Systems Steward",
            "nonce replay test",
            "2026-02-09T12:00:00Z",
            "decision-replay-1",
        );
        assert!(require_governance_envelope("steward-replay-1", Some(&envelope)).is_ok());
        let replay = require_governance_envelope("steward-replay-1", Some(&envelope))
            .expect_err("replayed nonce must be rejected");
        assert_eq!(replay.status(), StatusCode::CONFLICT);
    }

    #[test]
    fn realtime_feature_flag_defaults_enabled_and_supports_kill_switch() {
        let _unset = EnvVarGuard::unset("CORTEX_COLLAB_REALTIME");
        assert!(realtime_feature_enabled());

        let _off = EnvVarGuard::set("CORTEX_COLLAB_REALTIME", "off");
        assert!(!realtime_feature_enabled());
    }

    #[tokio::test]
    async fn cortex_runtime_slo_endpoints_return_payloads() {
        let Json(status) = get_cortex_runtime_slo_status().await;
        assert_eq!(status.schema_version, "1.0.0");
        let Json(breaches) = get_cortex_runtime_slo_breaches().await;
        assert!(breaches.iter().all(|entry| !entry.metric.trim().is_empty()));
    }

    #[tokio::test]
    async fn cortex_runtime_closeout_tasks_endpoint_returns_summary() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let ledger_path = temp.path().join("TASKS.json");
        let ledger = json!({
            "schema_version": "1.0.0",
            "contribution_id": "116-cortex-realtime-ga-trust-hardening",
            "generated_at": "2026-02-10T00:00:00Z",
            "tasks": [
                {
                    "task_id": "P7-100-fixture-overdue",
                    "title": "fixture overdue",
                    "owner": "systems-steward",
                    "status": "pending",
                    "due_at_utc": "2026-02-09T00:00:00Z",
                    "kind": "canary",
                    "acceptance": [],
                    "evidence_paths": [],
                    "validation_commands": [],
                    "depends_on": [],
                    "last_updated_at": "2026-02-10T00:00:00Z"
                },
                {
                    "task_id": "P7-101-fixture-complete",
                    "title": "fixture complete",
                    "owner": "systems-steward",
                    "status": "complete",
                    "due_at_utc": "2026-02-11T00:00:00Z",
                    "kind": "verification",
                    "acceptance": [],
                    "evidence_paths": [],
                    "validation_commands": [],
                    "depends_on": [],
                    "last_updated_at": "2026-02-10T00:00:00Z"
                }
            ]
        });
        std::fs::write(
            &ledger_path,
            serde_json::to_string_pretty(&ledger).expect("serialize fixture ledger"),
        )
        .expect("write fixture ledger");
        let _ledger_guard = EnvVarGuard::set(
            "CORTEX_CLOSEOUT_TASKS_PATH",
            ledger_path.to_string_lossy().as_ref(),
        );

        let response = get_cortex_runtime_closeout_tasks(Query(CortexCloseoutTasksQuery {
            contribution_id: Some("116-cortex-realtime-ga-trust-hardening".to_string()),
            as_of: Some("2026-02-10T12:00:00Z".to_string()),
        }))
        .await;
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["summary"]["total"], 2);
        assert_eq!(body["summary"]["overdue"], 1);
        assert_eq!(body["summary"]["complete"], 1);
        assert_eq!(
            body["summary"]["status_counts"]["pending"],
            serde_json::Value::from(1)
        );
        assert_eq!(
            body["summary"]["status_counts"]["complete"],
            serde_json::Value::from(1)
        );
        assert_eq!(body["tasks"][0]["task_id"], "P7-100-fixture-overdue");
        assert_eq!(body["tasks"][0]["overdue"], true);
    }

    #[test]
    fn testing_gate_surface_uses_required_surface_id_convention() {
        let summary = TestGateSummaryArtifact {
            schema_version: "1.0.0".to_string(),
            generated_at: "2026-02-08T00:00:00Z".to_string(),
            mode: "advisory".to_string(),
            catalog_valid: true,
            run_artifacts_valid: true,
            required_blockers_pass: false,
            overall_verdict: "not-ready".to_string(),
            latest_run_id: Some("run_123".to_string()),
            failures: vec![TestGateFailure {
                code: "BLOCKER_FAILURE".to_string(),
                message: "release blocker failed".to_string(),
            }],
            counts: TestGateCounts {
                pass: 1,
                fail: 1,
                warn: 0,
                pending: 0,
            },
        };
        let surface = synthesize_testing_gate_surface(&summary);
        assert_eq!(surface["surfaceId"], "system_test_gate:run_123");
    }

    #[test]
    fn testing_gate_surface_is_deterministic_for_same_input() {
        let summary = TestGateSummaryArtifact {
            schema_version: "1.0.0".to_string(),
            generated_at: "2026-02-08T00:00:00Z".to_string(),
            mode: "blocking".to_string(),
            catalog_valid: true,
            run_artifacts_valid: true,
            required_blockers_pass: true,
            overall_verdict: "ready".to_string(),
            latest_run_id: Some("run_abc".to_string()),
            failures: vec![],
            counts: TestGateCounts {
                pass: 4,
                fail: 0,
                warn: 0,
                pending: 0,
            },
        };
        assert_eq!(
            synthesize_testing_gate_surface(&summary),
            synthesize_testing_gate_surface(&summary)
        );
    }

    #[tokio::test]
    async fn testing_endpoints_return_payloads_with_fixture_artifacts() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        write_testing_fixture(temp.path());
        let _guard = TestingLogDirGuard::set(temp.path());

        let catalog_response = get_testing_catalog().await.into_response();
        assert_eq!(catalog_response.status(), StatusCode::OK);
        let catalog_json = response_json(catalog_response).await;
        assert_eq!(catalog_json["schema_version"], "1.0.0");
        assert_eq!(catalog_json["tests"].as_array().map(|v| v.len()), Some(2));

        let runs_response = get_testing_runs(Query(TestingRunsQuery {
            limit: Some(10),
            status: None,
        }))
        .await
        .into_response();
        assert_eq!(runs_response.status(), StatusCode::OK);
        let runs_json = response_json(runs_response).await;
        assert_eq!(runs_json.as_array().map(|v| v.len()), Some(1));

        let run_response = get_testing_run(Path("run_fixture".to_string()))
            .await
            .into_response();
        assert_eq!(run_response.status(), StatusCode::OK);
        let run_json = response_json(run_response).await;
        assert_eq!(run_json["run_id"], "run_fixture");

        let gate_response = get_testing_gates_latest().await.into_response();
        assert_eq!(gate_response.status(), StatusCode::OK);
        let gate_json = response_json(gate_response).await;
        assert_eq!(gate_json["summary"]["overall_verdict"], "ready");

        let health_response = get_testing_health().await.into_response();
        assert_eq!(health_response.status(), StatusCode::OK);
        let health_json = response_json(health_response).await;
        assert_eq!(health_json["status"], "ok");
        assert_eq!(health_json["runsCount"], 1);
    }

    #[tokio::test]
    async fn testing_catalog_missing_returns_structured_not_found_error() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = TestingLogDirGuard::set(temp.path());

        let response = get_testing_catalog().await.into_response();
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "NOT_FOUND");
        assert_eq!(body["error"], "Testing artifact not found");
        assert!(body["details"].is_object());
    }

    #[tokio::test]
    async fn testing_run_rejects_invalid_pathlike_run_id() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        write_testing_fixture(temp.path());
        let _guard = TestingLogDirGuard::set(temp.path());

        let response = get_testing_run(Path("../bad".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "INVALID_RUN_ID");
        assert_eq!(body["details"]["run_id"], "../bad");
    }

    #[tokio::test]
    async fn siq_endpoints_return_payloads_with_fixture_artifacts() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        write_siq_fixture(temp.path());
        let _guard = SiqLogDirGuard::set(temp.path());

        let coverage_response = get_siq_coverage().await.into_response();
        assert_eq!(coverage_response.status(), StatusCode::OK);
        let coverage_json = response_json(coverage_response).await;
        assert_eq!(coverage_json["schema_version"], "1.0.0");

        let dependency_response = get_siq_dependency_closure().await.into_response();
        assert_eq!(dependency_response.status(), StatusCode::OK);
        let dependency_json = response_json(dependency_response).await;
        assert_eq!(dependency_json["overall_closure_state"], "ready");

        let gates_response = get_siq_gates_latest().await.into_response();
        assert_eq!(gates_response.status(), StatusCode::OK);
        let gates_json = response_json(gates_response).await;
        assert_eq!(gates_json["overall_verdict"], "ready");

        let graph_response = get_siq_graph_projection().await.into_response();
        assert_eq!(graph_response.status(), StatusCode::OK);
        let graph_json = response_json(graph_response).await;
        assert_eq!(graph_json["run_id"], "siq_fixture_run");

        let runs_response = get_siq_runs(Query(SiqRunsQuery { limit: Some(10) }))
            .await
            .into_response();
        assert_eq!(runs_response.status(), StatusCode::OK);
        let runs_json = response_json(runs_response).await;
        assert_eq!(runs_json.as_array().map(|v| v.len()), Some(1));

        let run_response = get_siq_run(Path("siq_fixture_run".to_string()))
            .await
            .into_response();
        assert_eq!(run_response.status(), StatusCode::OK);
        let run_json = response_json(run_response).await;
        assert_eq!(run_json["run_id"], "siq_fixture_run");

        let health_response = get_siq_health().await.into_response();
        assert_eq!(health_response.status(), StatusCode::OK);
        let health_json = response_json(health_response).await;
        assert_eq!(health_json["status"], "ok");
        assert_eq!(health_json["runsCount"], 1);
    }

    #[tokio::test]
    async fn siq_run_rejects_invalid_pathlike_run_id() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        write_siq_fixture(temp.path());
        let _guard = SiqLogDirGuard::set(temp.path());

        let response = get_siq_run(Path("../bad".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "INVALID_RUN_ID");
        assert_eq!(body["details"]["run_id"], "../bad");
    }

    #[tokio::test]
    async fn motoko_graph_endpoints_return_payloads_with_fixture_artifacts() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        write_motoko_graph_fixture(temp.path());
        let _guard = MotokoGraphLogDirGuard::set(temp.path());

        let snapshot_response = get_motoko_graph_snapshot().await.into_response();
        assert_eq!(snapshot_response.status(), StatusCode::OK);
        let snapshot_json = response_json(snapshot_response).await;
        assert_eq!(snapshot_json["contribution_id"], "078");
        assert_eq!(
            snapshot_json["status"]["authority_mode"],
            "recommendation_only"
        );

        let history_response = get_motoko_graph_decision_history().await.into_response();
        assert_eq!(history_response.status(), StatusCode::OK);
        let history_json = response_json(history_response).await;
        assert_eq!(history_json.as_array().map(|v| v.len()), Some(1));
        assert_eq!(history_json[0]["selected_option"], "Hold Deferred");

        let health_response = get_motoko_graph_health().await.into_response();
        assert_eq!(health_response.status(), StatusCode::OK);
        let health_json = response_json(health_response).await;
        assert_eq!(health_json["status"], "ok");
        assert_eq!(health_json["historyCount"], 1);
        assert_eq!(health_json["pendingCount"], 1);

        let trends_response = get_motoko_graph_monitoring_trends().await.into_response();
        assert_eq!(trends_response.status(), StatusCode::OK);
        let trends_json = response_json(trends_response).await;
        assert_eq!(trends_json["next_action"], "START_GATEWAY");
        assert_eq!(trends_json["latest"]["run_id"], "monitor_fixture_001");

        let runs_response =
            get_motoko_graph_monitoring_runs(Query(MotokoGraphMonitoringRunsQuery {
                limit: Some(10),
            }))
            .await
            .into_response();
        assert_eq!(runs_response.status(), StatusCode::OK);
        let runs_json = response_json(runs_response).await;
        assert_eq!(runs_json.as_array().map(|v| v.len()), Some(1));
        assert_eq!(runs_json[0]["run_id"], "monitor_fixture_001");
    }

    #[tokio::test]
    async fn motoko_graph_capture_writes_pending_event_with_deterministic_id() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        std::fs::create_dir_all(temp.path().join("decisions").join("pending"))
            .expect("create pending dir");
        let _guard = MotokoGraphLogDirGuard::set(temp.path());

        let payload = DecisionCaptureRequest {
            schema_version: "1.0.0".to_string(),
            contribution: "078".to_string(),
            decision_date: "2026-02-08".to_string(),
            selected_option: "Request Additional Evidence".to_string(),
            rationale: "Need additional confidence for dependency progression.".to_string(),
            posture_before: "watch-first".to_string(),
            posture_after: "watch-first".to_string(),
            authority_mode: "recommendation_only".to_string(),
            evidence_refs: vec!["/tmp/m16_dual_path/gate.txt".to_string()],
            steward: "Research Steward".to_string(),
            owner: "Nostra Architecture Team".to_string(),
            follow_up_actions: vec!["Run scoped evidence cycle".to_string()],
            source: "cortex-desktop:/kg/motoko-graph".to_string(),
        };

        let response1 = capture_motoko_graph_decision(Json(payload.clone()))
            .await
            .into_response();
        assert_eq!(response1.status(), StatusCode::OK);
        let body1 = response_json(response1).await;

        let response2 = capture_motoko_graph_decision(Json(payload))
            .await
            .into_response();
        assert_eq!(response2.status(), StatusCode::OK);
        let body2 = response_json(response2).await;

        assert_eq!(body1["decisionEventId"], body2["decisionEventId"]);
        let decision_id = body1["decisionEventId"]
            .as_str()
            .expect("decision id string");
        let path = temp
            .path()
            .join("decisions")
            .join("pending")
            .join(format!("{}.json", decision_id));
        assert!(path.exists());
    }

    #[tokio::test]
    async fn motoko_graph_capture_rejects_invalid_option() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = MotokoGraphLogDirGuard::set(temp.path());

        let payload = DecisionCaptureRequest {
            schema_version: "1.0.0".to_string(),
            contribution: "078".to_string(),
            decision_date: "2026-02-08".to_string(),
            selected_option: "Promote Now".to_string(),
            rationale: "invalid option".to_string(),
            posture_before: "watch-first".to_string(),
            posture_after: "conditional".to_string(),
            authority_mode: "recommendation_only".to_string(),
            evidence_refs: vec!["/tmp/m16_dual_path/gate.txt".to_string()],
            steward: "Research Steward".to_string(),
            owner: "Nostra Architecture Team".to_string(),
            follow_up_actions: vec!["none".to_string()],
            source: "cortex-desktop:/kg/motoko-graph".to_string(),
        };

        let response = capture_motoko_graph_decision(Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "INVALID_DECISION_CAPTURE");
    }

    #[tokio::test]
    async fn motoko_graph_capture_accepts_all_supported_options() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = MotokoGraphLogDirGuard::set(temp.path());
        let options = [
            "Hold Deferred",
            "Conditional Progression",
            "Request Additional Evidence",
        ];

        for option in options {
            let payload = DecisionCaptureRequest {
                schema_version: "1.0.0".to_string(),
                contribution: "078".to_string(),
                decision_date: "2026-02-08".to_string(),
                selected_option: option.to_string(),
                rationale: format!("fixture rationale for {}", option),
                posture_before: "watch-first".to_string(),
                posture_after: "watch-first".to_string(),
                authority_mode: "recommendation_only".to_string(),
                evidence_refs: vec!["/tmp/m16_dual_path/gate.txt".to_string()],
                steward: "Research Steward".to_string(),
                owner: "Nostra Architecture Team".to_string(),
                follow_up_actions: vec!["Follow up".to_string()],
                source: "cortex-desktop:/kg/motoko-graph".to_string(),
            };
            let response = capture_motoko_graph_decision(Json(payload))
                .await
                .into_response();
            assert_eq!(response.status(), StatusCode::OK);
            let body = response_json(response).await;
            let path = body["path"].as_str().expect("path string");
            assert!(std::path::Path::new(path).exists());
        }
    }

    #[tokio::test]
    async fn system_execution_profile_response_has_required_fields() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_execution_profile(Path("space-alpha".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert!(body.get("surfaceId").is_some());
        assert!(body.get("workflowId").is_some());
        assert!(body.get("mutationId").is_some());
        assert!(body.get("status").is_some());
        assert!(body.get("requiredActions").is_some());
        assert!(body.get("evidenceRefs").is_some());
        assert!(body.get("lastUpdatedAt").is_some());
    }

    #[tokio::test]
    async fn decision_ack_rejects_missing_quality_payload_for_risky_gate() {
        let _lock = acquire_testing_env_lock();
        let payload = DecisionActionRequest {
            space_id: Some("space-default".to_string()),
            decision_gate_id: Some("blackwell_gate:mut-123".to_string()),
            workflow_id: Some("wf-123".to_string()),
            mutation_id: Some("mut-123".to_string()),
            action_target: Some("governance:merge".to_string()),
            domain_mode: None,
            gate_level: None,
            actor_ref: None,
            risk_statement: None,
            rollback_path: None,
            evidence_refs: vec![],
            note: Some("missing required fields".to_string()),
        };
        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-ic-principal",
            "2vxsx-fae".parse().expect("principal header"),
        );
        let response = post_system_decision_ack(headers, Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "INVALID_OVERRIDE_PAYLOAD");
    }

    #[tokio::test]
    async fn decision_escalate_writes_deterministic_action_record() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let payload = DecisionActionRequest {
            space_id: Some("space-default".to_string()),
            decision_gate_id: Some("blackwell_gate:mut-456".to_string()),
            workflow_id: Some("wf-456".to_string()),
            mutation_id: Some("mut-456".to_string()),
            action_target: Some("governance:release".to_string()),
            domain_mode: Some("anonymous".to_string()),
            gate_level: Some("release_blocker".to_string()),
            actor_ref: None,
            risk_statement: Some(
                "Risk includes governance drift if released prematurely.".to_string(),
            ),
            rollback_path: Some(
                "Rollback path: revert decision and restore previous scope.".to_string(),
            ),
            evidence_refs: vec!["/tmp/evidence/run-1.json".to_string()],
            note: Some("escalating for steward review".to_string()),
        };

        let mut headers = HeaderMap::new();
        headers.insert("x-cortex-role", "operator".parse().expect("role header"));
        headers.insert(
            "x-ic-principal",
            "2vxsx-fae".parse().expect("principal header"),
        );

        let response_1 = post_system_decision_escalate(headers.clone(), Json(payload.clone()))
            .await
            .into_response();
        assert_eq!(response_1.status(), StatusCode::OK);
        let body_1 = response_json(response_1).await;

        let response_2 = post_system_decision_escalate(headers, Json(payload))
            .await
            .into_response();
        assert_eq!(response_2.status(), StatusCode::OK);
        let body_2 = response_json(response_2).await;

        let action_id_1 = body_1["payload"]["actionId"].as_str().expect("action id 1");
        let action_id_2 = body_2["payload"]["actionId"].as_str().expect("action id 2");
        assert_eq!(action_id_1, action_id_2);

        let action_path = temp
            .path()
            .join("actions")
            .join(format!("{}.json", action_id_1));
        assert!(action_path.exists());
    }

    #[tokio::test]
    async fn decision_escalate_allows_required_action_when_policy_blocks() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "block");

        let payload = DecisionActionRequest {
            space_id: Some("space-default".to_string()),
            decision_gate_id: Some("blackwell_gate:mut-required".to_string()),
            workflow_id: Some("wf-required".to_string()),
            mutation_id: Some("mut-required".to_string()),
            action_target: Some("governance:viewspec:ratify".to_string()),
            domain_mode: Some("attributed".to_string()),
            gate_level: Some("release_blocker".to_string()),
            actor_ref: None,
            risk_statement: Some("Escalating required governance gate action.".to_string()),
            rollback_path: Some(
                "Rollback path: reject proposal if gate remains blocked.".to_string(),
            ),
            evidence_refs: vec!["/tmp/evidence/required-escalate.json".to_string()],
            note: Some("required action bridge test".to_string()),
        };
        let response = post_system_decision_escalate(decision_headers(), Json(payload))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["status"], "escalate");
    }

    #[test]
    fn viewspec_required_actions_missing_until_ack_and_escalate_are_recorded() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());
        let actions_dir = temp.path().join("actions");
        std::fs::create_dir_all(&actions_dir).expect("create actions dir");

        let proposal_id = "proposal-required-actions";
        let action_target = "governance:viewspec:ratify";
        let actor_ref = "2vxsx-fae#operator";
        let required_actions = vec!["decision_ack".to_string(), "decision_escalate".to_string()];

        let ack_record = DecisionActionRecord {
            schema_version: "1.0.0".to_string(),
            action_id: "ack-record".to_string(),
            action: "ack".to_string(),
            decision_gate_id: "viewspec_gate:governance_viewspec_ratify:proposal-required-actions"
                .to_string(),
            workflow_id: "viewspec_governance:proposal-required-actions".to_string(),
            mutation_id: proposal_id.to_string(),
            action_target: action_target.to_string(),
            risk_statement: "risk".to_string(),
            rollback_path: "rollback".to_string(),
            evidence_refs: vec!["/tmp/evidence/ack.json".to_string()],
            lineage_id: "lineage:ack".to_string(),
            policy_ref: Some("policy:1".to_string()),
            actor_ref: Some(actor_ref.to_string()),
            note: Some("ack".to_string()),
            created_at: "2026-02-10T00:00:00Z".to_string(),
        };
        let ack_path = actions_dir.join("ack-record.json");
        std::fs::write(
            &ack_path,
            serde_json::to_vec_pretty(&ack_record).expect("serialize ack"),
        )
        .expect("write ack");

        let missing_after_ack = missing_viewspec_required_actions(
            proposal_id,
            action_target,
            "2vxsx-fae",
            "operator",
            &required_actions,
        )
        .expect("compute missing after ack");
        assert_eq!(missing_after_ack, vec!["decision_escalate".to_string()]);

        let escalate_record = DecisionActionRecord {
            schema_version: "1.0.0".to_string(),
            action_id: "escalate-record".to_string(),
            action: "escalate".to_string(),
            decision_gate_id: "viewspec_gate:governance_viewspec_ratify:proposal-required-actions"
                .to_string(),
            workflow_id: "viewspec_governance:proposal-required-actions".to_string(),
            mutation_id: proposal_id.to_string(),
            action_target: action_target.to_string(),
            risk_statement: "risk".to_string(),
            rollback_path: "rollback".to_string(),
            evidence_refs: vec!["/tmp/evidence/escalate.json".to_string()],
            lineage_id: "lineage:escalate".to_string(),
            policy_ref: Some("policy:1".to_string()),
            actor_ref: Some(actor_ref.to_string()),
            note: Some("escalate".to_string()),
            created_at: "2026-02-10T00:01:00Z".to_string(),
        };
        let escalate_path = actions_dir.join("escalate-record.json");
        std::fs::write(
            &escalate_path,
            serde_json::to_vec_pretty(&escalate_record).expect("serialize escalate"),
        )
        .expect("write escalate");

        let missing_after_all = missing_viewspec_required_actions(
            proposal_id,
            action_target,
            "2vxsx-fae",
            "operator",
            &required_actions,
        )
        .expect("compute missing after all");
        assert!(missing_after_all.is_empty());
    }

    #[tokio::test]
    async fn system_mutation_gates_returns_structured_envelope() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());

        let response =
            get_system_mutation_gates(Path(("space-alpha".to_string(), "mut-alpha".to_string())))
                .await
                .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["surfaceId"], "blackwell_gate:mut-alpha");
        assert!(body.get("workflowId").is_some());
        assert!(body.get("mutationId").is_some());
        assert!(body.get("requiredActions").is_some());
        assert!(body.get("sourceOfTruth").is_some());
    }

    #[tokio::test]
    async fn system_decision_plane_returns_digest_surface() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_decision_plane(Path("space-alpha".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["spaceId"], "space-alpha");
        assert!(body["surfaces"].is_array());
        assert!(body["digest"].is_object());
    }

    #[tokio::test]
    async fn system_decision_telemetry_returns_snapshot_shape() {
        let _lock = acquire_testing_env_lock();
        let temp = TestTempDir::new();
        let _guard = DecisionSurfaceLogDirGuard::set(temp.path());

        let _ = get_system_mutation_gates(Path((
            "space-alpha".to_string(),
            "mut-telemetry".to_string(),
        )))
        .await
        .into_response();

        let response = get_system_decision_telemetry().await.into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert!(body.get("schemaVersion").is_some());
        assert!(body.get("decisionGateSamples").is_some());
        assert!(body.get("sourceOfTruthCounts").is_some());
    }

    #[tokio::test]
    async fn system_decision_telemetry_by_space_includes_space_id() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_decision_telemetry_by_space(Path("space-zeta".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["spaceId"], "space-zeta");
        assert_eq!(body["scopeSpaceId"], "space-zeta");
    }

    #[tokio::test]
    async fn decision_ack_rejects_unsigned_when_signed_mode_required_all() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "required_all");
        let _secret = EnvVarGuard::set("NOSTRA_DECISION_SIGNING_SECRET", "test-signing-secret");
        let _mock_binding = EnvVarGuard::unset("NOSTRA_TEST_DECISION_ROLE_BINDING");
        let _mock_policy = EnvVarGuard::unset("NOSTRA_TEST_DECISION_POLICY_EVAL");

        let response = post_system_decision_ack(
            decision_headers(),
            Json(risky_decision_payload("mut-required-signature")),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "MISSING_DECISION_SIGNATURE");
    }

    #[tokio::test]
    async fn decision_ack_warn_mode_accepts_unsigned_with_auth_metadata() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "warn");
        let _secret = EnvVarGuard::set("NOSTRA_DECISION_SIGNING_SECRET", "test-signing-secret");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let response = post_system_decision_ack(
            decision_headers(),
            Json(risky_decision_payload("mut-warn-mode")),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["authStatus"], "warn");
        assert_eq!(body["authReason"], "signature_missing_warn_only");
    }

    #[tokio::test]
    async fn decision_ack_rejects_missing_canister_binding_for_risky_gate_without_env_fallback() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _allow_env = EnvVarGuard::set("NOSTRA_DECISION_ALLOW_ENV_ROLE_FALLBACK", "false");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "__missing__");
        let _mock_policy = EnvVarGuard::unset("NOSTRA_TEST_DECISION_POLICY_EVAL");

        let response = post_system_decision_ack(
            decision_headers(),
            Json(risky_decision_payload("mut-missing-binding")),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "MISSING_CANISTER_ROLE_BINDING");
    }

    #[tokio::test]
    async fn decision_ack_rejects_when_mock_policy_blocks_action() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "block");

        let response = post_system_decision_ack(
            decision_headers(),
            Json(risky_decision_payload("mut-policy-block")),
        )
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::FORBIDDEN);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "POLICY_GATE_BLOCKED");
    }

    #[tokio::test]
    async fn canonical_only_execution_profile_requires_canister_source() {
        let _lock = acquire_testing_env_lock();
        let _canonical = EnvVarGuard::set("NOSTRA_DECISION_CANONICAL_ONLY", "true");
        let _workflow_canister =
            EnvVarGuard::set("CANISTER_ID_WORKFLOW_ENGINE", "invalid-principal");
        let response = get_system_execution_profile(Path("space-canonical".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::SERVICE_UNAVAILABLE);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "CANONICAL_SOURCE_REQUIRED");
    }

    #[tokio::test]
    async fn decision_plane_digest_blocks_and_uses_canonical_source_contract_when_degraded() {
        let _lock = acquire_testing_env_lock();
        let _canonical = EnvVarGuard::set("NOSTRA_DECISION_CANONICAL_ONLY", "true");
        let _workflow_canister =
            EnvVarGuard::set("CANISTER_ID_WORKFLOW_ENGINE", "invalid-principal");
        let _governance_canister = EnvVarGuard::set("CANISTER_ID_GOVERNANCE", "invalid-principal");
        let response = get_system_decision_plane(Path("space-degraded".to_string()))
            .await
            .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        assert_eq!(body["digest"]["status"], "blocked");
        assert_eq!(body["digest"]["sourceOfTruth"], "fallback");
        let actions = body["digest"]["requiredActions"]
            .as_array()
            .expect("required actions");
        assert!(
            actions
                .iter()
                .filter_map(|value| value.as_str())
                .any(|value| value == "decision_escalate:decision_plane_space-degraded")
        );
    }

    #[tokio::test]
    async fn mutation_gate_source_of_truth_uses_contracted_enum() {
        let _lock = acquire_testing_env_lock();
        let response = get_system_mutation_gates(Path((
            "space-alpha".to_string(),
            "mut-source-contract".to_string(),
        )))
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::OK);
        let body = response_json(response).await;
        let source = body["sourceOfTruth"]
            .as_str()
            .expect("source of truth present");
        assert!(matches!(source, "canister" | "cache" | "fallback"));
    }

    #[tokio::test]
    async fn viewspec_candidate_stage_rejects_hash_mismatch() {
        let _lock = acquire_testing_env_lock();
        let set_id = format!("test_set_{}", Utc::now().timestamp_millis());
        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some("space-test".to_string()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id.clone()),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some("space-test".to_string()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "staging mismatch test".to_string(),
                expected_input_hash: "deadbeef".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::CONFLICT);
        let body = response_json(stage_response).await;
        assert_eq!(body["errorCode"], "VIEWSPEC_STAGE_HASH_MISMATCH");
    }

    #[tokio::test]
    async fn viewspec_candidate_stage_persists_without_lock() {
        let _lock = acquire_testing_env_lock();
        let set_id = format!("test_set_stage_{}", Utc::now().timestamp_millis());
        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some("space-test".to_string()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id.clone()),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some("space-test".to_string()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let reload_response = get_cortex_viewspec_candidate_set(Path(candidate_set_id.clone()))
            .await
            .into_response();
        assert_eq!(reload_response.status(), StatusCode::OK);

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "stage valid candidate".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let get_response = get_cortex_viewspec(
            Path(view_spec_id),
            Query(ViewSpecLookupQuery {
                space_id: Some("space-test".to_string()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
        )
        .await
        .into_response();
        assert_eq!(get_response.status(), StatusCode::OK);
        let spec_body = response_json(get_response).await;
        assert!(spec_body.get("lock").is_none() || spec_body["lock"].is_null());
    }

    #[tokio::test]
    async fn viewspec_learning_signal_ingestion_rejects_missing_space_id() {
        let _lock = acquire_testing_env_lock();
        let response = post_cortex_viewspec_learning_signals(Json(ViewSpecLearningSignalRequest {
            signal_id: None,
            event_type: "candidate_staged".to_string(),
            view_spec_id: "missing_view_spec".to_string(),
            space_id: None,
            actor: "tester".to_string(),
            timestamp: None,
            payload: Value::Null,
        }))
        .await
        .into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let body = response_json(response).await;
        assert_eq!(body["errorCode"], "VIEWSPEC_LEARNING_SPACE_REQUIRED");
    }

    #[tokio::test]
    async fn viewspec_learning_stage_lock_propose_emit_signals_and_recompute() {
        let _lock = acquire_testing_env_lock();
        let unique = Utc::now().timestamp_millis();
        let space_id = format!("space-learning-{}", unique);
        let set_id = format!("test_set_learning_{}", unique);

        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some(space_id.clone()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "phase3 stage".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let lock_response = post_cortex_viewspec_lock(Json(ViewSpecLockRequest {
            view_spec_id: view_spec_id.clone(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            locked_by: "tester".to_string(),
            rationale: "phase3 lock".to_string(),
            structural_change: Some(false),
            approved_by: None,
            approved_at: None,
        }))
        .await
        .into_response();
        assert_eq!(lock_response.status(), StatusCode::OK);

        let propose_response = post_cortex_viewspec_propose(
            Path(view_spec_id.clone()),
            Json(ViewSpecProposeRequest {
                proposed_by: "tester".to_string(),
                rationale: "phase3 propose".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(propose_response.status(), StatusCode::OK);

        let signals = load_viewspec_learning_signals(space_id.as_str())
            .await
            .expect("load learning signals");
        let events = signals
            .iter()
            .filter(|signal| signal.view_spec_id == view_spec_id)
            .map(|signal| signal.event_type.clone())
            .collect::<Vec<_>>();
        assert!(events.iter().any(|event| event == "candidate_staged"));
        assert!(events.iter().any(|event| event == "viewspec_locked"));
        assert!(events.iter().any(|event| event == "viewspec_proposed"));

        let recompute_response = post_cortex_viewspec_learning_profile_recompute(
            Path(space_id.clone()),
            Json(ViewSpecLearningRecomputeRequest {
                actor: "tester".to_string(),
                reason: Some("phase3 test".to_string()),
            }),
        )
        .await
        .into_response();
        assert_eq!(recompute_response.status(), StatusCode::OK);
        let recompute_body = response_json(recompute_response).await;
        assert_eq!(recompute_body["accepted"], true);
        assert_eq!(
            recompute_body["profile"]["policy"]["autoApplyEnabled"],
            false
        );
        assert_eq!(
            recompute_body["profile"]["policy"]["globalMergeEnabled"],
            false
        );
        assert!(
            recompute_body["profile"]["signalCount"]
                .as_u64()
                .unwrap_or_default()
                >= 3
        );

        let profile_response = get_cortex_viewspec_learning_profile(Path(space_id.clone()))
            .await
            .into_response();
        assert_eq!(profile_response.status(), StatusCode::OK);

        let confidence_response = post_cortex_viewspec_confidence_recompute(
            Path(view_spec_id),
            Json(ViewSpecConfidenceRecomputeRequest {
                scope: Some(ViewSpecScope {
                    space_id: Some(space_id.clone()),
                    route_id: Some("/studio".to_string()),
                    role: Some("operator".to_string()),
                }),
            }),
        )
        .await
        .into_response();
        assert_eq!(confidence_response.status(), StatusCode::OK);
        let confidence_body = response_json(confidence_response).await;
        assert_eq!(confidence_body["spaceId"], space_id);
        assert_eq!(confidence_body["persisted"], false);
    }

    #[tokio::test]
    async fn viewspec_proposal_cannot_ratify_without_approved_review() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let unique = Utc::now().timestamp_millis();
        let space_id = format!("space-proposal-{}", unique);
        let set_id = format!("test_set_proposal_{}", unique);

        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some(space_id.clone()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "stage for proposal".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let propose_response = post_cortex_viewspec_propose(
            Path(view_spec_id),
            Json(ViewSpecProposeRequest {
                proposed_by: "proposer".to_string(),
                rationale: "submit for governance".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(propose_response.status(), StatusCode::OK);
        let propose_body = response_json(propose_response).await;
        let proposal_id = propose_body["proposal"]["proposalId"]
            .as_str()
            .expect("proposal id")
            .to_string();

        let ratify_response = post_cortex_viewspec_proposal_ratify(
            decision_headers(),
            Path(proposal_id),
            Json(ViewSpecProposalDecisionRequest {
                decided_by: "ratifier".to_string(),
                rationale: "attempt ratify without review".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(ratify_response.status(), StatusCode::CONFLICT);
        let body = response_json(ratify_response).await;
        assert_eq!(body["errorCode"], "VIEWSPEC_PROPOSAL_RATIFY_INVALID_STATE");
    }

    #[tokio::test]
    async fn viewspec_proposal_ratify_writes_active_scope_and_events() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let unique = Utc::now().timestamp_millis();
        let space_id = format!("space-ratify-{}", unique);
        let set_id = format!("test_set_ratify_{}", unique);

        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some(space_id.clone()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "stage for ratify".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let propose_response = post_cortex_viewspec_propose(
            Path(view_spec_id.clone()),
            Json(ViewSpecProposeRequest {
                proposed_by: "proposer".to_string(),
                rationale: "submit for ratify".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(propose_response.status(), StatusCode::OK);
        let propose_body = response_json(propose_response).await;
        let proposal_id = propose_body["proposal"]["proposalId"]
            .as_str()
            .expect("proposal id")
            .to_string();

        let review_response = post_cortex_viewspec_proposal_review(
            decision_headers(),
            Path(proposal_id.clone()),
            Json(ViewSpecProposalReviewRequest {
                reviewed_by: "reviewer".to_string(),
                summary: "approved review".to_string(),
                checks: vec!["validator_passed".to_string()],
                approved: true,
            }),
        )
        .await
        .into_response();
        assert_eq!(review_response.status(), StatusCode::OK);

        let ratify_response = post_cortex_viewspec_proposal_ratify(
            decision_headers(),
            Path(proposal_id.clone()),
            Json(ViewSpecProposalDecisionRequest {
                decided_by: "ratifier".to_string(),
                rationale: "ratify approved proposal".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(ratify_response.status(), StatusCode::OK);
        let ratify_body = response_json(ratify_response).await;
        assert_eq!(ratify_body["gateStatus"], "allow");
        assert_eq!(ratify_body["sourceOfTruth"], "canister");
        let scope_key = ratify_body["proposal"]["scopeKey"]
            .as_str()
            .expect("scope key")
            .to_string();

        let active_response = get_cortex_viewspec_active(Query(ViewSpecActiveQuery {
            scope_key: Some(scope_key),
        }))
        .await
        .into_response();
        assert_eq!(active_response.status(), StatusCode::OK);
        let active_body = response_json(active_response).await;
        let active = active_body["active"].as_array().expect("active records");
        assert_eq!(active.len(), 1);
        assert_eq!(active[0]["activeViewSpecId"], view_spec_id);
        assert_eq!(active[0]["adoptedFromProposalId"], proposal_id);

        let proposal_events = store_read_jsonl::<ViewSpecEventRecord>(
            viewspec_proposal_events_key(&Utc::now().format("%Y-%m-%d").to_string()).as_str(),
        )
        .await
        .expect("load proposal events");
        assert!(
            proposal_events
                .iter()
                .any(|event| event.event_type == "viewspec_proposal_ratified")
        );

        let learning_signals = load_viewspec_learning_signals(space_id.as_str())
            .await
            .expect("load learning signals");
        assert!(
            learning_signals
                .iter()
                .any(|signal| signal.event_type == "proposal_ratified")
        );
    }

    #[tokio::test]
    async fn viewspec_governance_blocks_ratify_without_canonical_binding() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "__missing__");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let unique = Utc::now().timestamp_millis();
        let space_id = format!("space-ratify-block-{}", unique);
        let set_id = format!("test_set_ratify_block_{}", unique);

        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some(space_id.clone()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "stage for governance block".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let propose_response = post_cortex_viewspec_propose(
            Path(view_spec_id),
            Json(ViewSpecProposeRequest {
                proposed_by: "proposer".to_string(),
                rationale: "submit".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(propose_response.status(), StatusCode::OK);
        let propose_body = response_json(propose_response).await;
        let proposal_id = propose_body["proposal"]["proposalId"]
            .as_str()
            .expect("proposal id")
            .to_string();

        let review_response = post_cortex_viewspec_proposal_review(
            decision_headers(),
            Path(proposal_id.clone()),
            Json(ViewSpecProposalReviewRequest {
                reviewed_by: "reviewer".to_string(),
                summary: "approved".to_string(),
                checks: vec![],
                approved: true,
            }),
        )
        .await
        .into_response();
        assert_eq!(review_response.status(), StatusCode::OK);

        let ratify_response = post_cortex_viewspec_proposal_ratify(
            decision_headers(),
            Path(proposal_id),
            Json(ViewSpecProposalDecisionRequest {
                decided_by: "ratifier".to_string(),
                rationale: "attempt canonical ratify".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(ratify_response.status(), StatusCode::FORBIDDEN);
        let body = response_json(ratify_response).await;
        assert_eq!(
            body["errorCode"],
            "VIEWSPEC_GOVERNANCE_ROLE_BINDING_MISSING"
        );
    }

    #[tokio::test]
    async fn viewspec_replay_digest_is_deterministic_for_same_state() {
        let _lock = acquire_testing_env_lock();
        let _mode = EnvVarGuard::set("NOSTRA_DECISION_SIGNED_MODE", "off");
        let _mock_binding = EnvVarGuard::set("NOSTRA_TEST_DECISION_ROLE_BINDING", "operator");
        let _mock_policy = EnvVarGuard::set("NOSTRA_TEST_DECISION_POLICY_EVAL", "allow");

        let unique = Utc::now().timestamp_millis();
        let space_id = format!("space-digest-{}", unique);
        let set_id = format!("test_set_digest_{}", unique);

        let generate_response = post_cortex_viewspec_candidates(Json(ViewSpecCandidateRequest {
            intent: "Show project progress clearly".to_string(),
            scope: Some(ViewSpecScope {
                space_id: Some(space_id.clone()),
                route_id: Some("/studio".to_string()),
                role: Some("operator".to_string()),
            }),
            generation_mode: Some("deterministic_scaffold".to_string()),
            candidate_set_id: Some(set_id),
            actor_id: Some("tester".to_string()),
            actor_role: Some("operator".to_string()),
            space_id: Some(space_id.clone()),
            constraints: vec![],
            count: Some(1),
            created_by: Some("tester".to_string()),
            source_mode: Some("human".to_string()),
        }))
        .await
        .into_response();
        assert_eq!(generate_response.status(), StatusCode::OK);
        let generate_body = response_json(generate_response).await;
        let candidate_set_id = generate_body["candidateSetId"]
            .as_str()
            .expect("candidate set id")
            .to_string();
        let candidate_id = generate_body["candidates"][0]["candidateId"]
            .as_str()
            .expect("candidate id")
            .to_string();
        let expected_hash = generate_body["candidates"][0]["inputHash"]
            .as_str()
            .expect("input hash")
            .to_string();

        let stage_response = post_cortex_viewspec_candidate_stage(
            Path(candidate_set_id),
            Json(ViewSpecCandidateStageRequest {
                candidate_id,
                staged_by: "tester".to_string(),
                rationale: "stage for digest".to_string(),
                expected_input_hash: expected_hash,
            }),
        )
        .await
        .into_response();
        assert_eq!(stage_response.status(), StatusCode::OK);
        let stage_body = response_json(stage_response).await;
        let view_spec_id = stage_body["viewSpecId"]
            .as_str()
            .expect("view spec id")
            .to_string();

        let propose_response = post_cortex_viewspec_propose(
            Path(view_spec_id),
            Json(ViewSpecProposeRequest {
                proposed_by: "proposer".to_string(),
                rationale: "submit for digest".to_string(),
            }),
        )
        .await
        .into_response();
        assert_eq!(propose_response.status(), StatusCode::OK);
        let propose_body = response_json(propose_response).await;
        let proposal_id = propose_body["proposal"]["proposalId"]
            .as_str()
            .expect("proposal id")
            .to_string();

        let digest_a = get_cortex_viewspec_proposal_digest(Path(proposal_id.clone()))
            .await
            .into_response();
        assert_eq!(digest_a.status(), StatusCode::OK);
        let body_a = response_json(digest_a).await;
        let replay_a = get_cortex_viewspec_proposal_replay(Path(proposal_id.clone()))
            .await
            .into_response();
        assert_eq!(replay_a.status(), StatusCode::OK);
        let replay_body_a = response_json(replay_a).await;

        let digest_b = get_cortex_viewspec_proposal_digest(Path(proposal_id.clone()))
            .await
            .into_response();
        assert_eq!(digest_b.status(), StatusCode::OK);
        let body_b = response_json(digest_b).await;
        let replay_b = get_cortex_viewspec_proposal_replay(Path(proposal_id))
            .await
            .into_response();
        assert_eq!(replay_b.status(), StatusCode::OK);
        let replay_body_b = response_json(replay_b).await;

        assert_eq!(body_a["digest"]["digest"], body_b["digest"]["digest"]);
        assert_eq!(
            replay_body_a["replay"]["runId"],
            replay_body_b["replay"]["runId"]
        );
        assert_eq!(
            replay_body_a["replay"]["signalCount"],
            replay_body_b["replay"]["signalCount"]
        );
    }

    #[tokio::test]
    async fn spatial_experiment_event_pipeline_persists_summary_and_readback() {
        let _lock = acquire_testing_env_lock();
        let run_id = format!("spatial-run-{}", Utc::now().timestamp_millis());
        let timestamp = "2026-02-22T12:00:00Z".to_string();

        let start_response =
            post_cortex_viewspec_spatial_experiment_event(Json(SpatialExperimentEventRequest {
                run_id: run_id.clone(),
                space_id: "nostra-governance-v0".to_string(),
                mode: "evaluation_phase5".to_string(),
                surface_variant: "compare".to_string(),
                event_type: "run_start".to_string(),
                timestamp: timestamp.clone(),
                payload: json!({
                    "selectedNodeId": "space-node-1",
                    "surfaceClass": "execution"
                }),
                build_id: Some("build-test-1".to_string()),
                host: "cortex-web".to_string(),
            }))
            .await
            .into_response();
        assert_eq!(start_response.status(), StatusCode::OK);
        let start_body = response_json(start_response).await;
        assert_eq!(start_body["accepted"], true);
        let stored_key = start_body["storedKey"]
            .as_str()
            .expect("stored key")
            .to_string();

        let click_response =
            post_cortex_viewspec_spatial_experiment_event(Json(SpatialExperimentEventRequest {
                run_id: run_id.clone(),
                space_id: "nostra-governance-v0".to_string(),
                mode: "evaluation_phase5".to_string(),
                surface_variant: "spatial".to_string(),
                event_type: "button_click".to_string(),
                timestamp: timestamp.clone(),
                payload: json!({
                    "action": "approve",
                    "label": "Approve Changes"
                }),
                build_id: Some("build-test-1".to_string()),
                host: "cortex-web".to_string(),
            }))
            .await
            .into_response();
        assert_eq!(click_response.status(), StatusCode::OK);

        let run_end_response =
            post_cortex_viewspec_spatial_experiment_event(Json(SpatialExperimentEventRequest {
                run_id: run_id.clone(),
                space_id: "nostra-governance-v0".to_string(),
                mode: "evaluation_phase5".to_string(),
                surface_variant: "compare".to_string(),
                event_type: "run_end".to_string(),
                timestamp: timestamp.clone(),
                payload: json!({
                    "metrics": {
                        "timeToFirstInteractionMs": 1200,
                        "taskCompletionMs": 5400,
                        "approvalDecisionCount": 1,
                        "spatialInteractionCount": 2,
                        "adapterFallbackRate": 0.0,
                        "errorEventCount": 0
                    },
                    "improvementScore": 4.2,
                    "recommendation": "go",
                    "complexityDelta": {
                        "bundleDeltaKb": 12.5,
                        "runtimeOverheadMs": 21.0,
                        "adapterFallbackRate": 0.0
                    },
                    "verdictRationale": "Stable replay and positive operator signal."
                }),
                build_id: Some("build-test-1".to_string()),
                host: "cortex-web".to_string(),
            }))
            .await
            .into_response();
        assert_eq!(run_end_response.status(), StatusCode::OK);

        let summary_response = get_cortex_viewspec_spatial_experiment_run(Path(run_id.clone()))
            .await
            .into_response();
        assert_eq!(summary_response.status(), StatusCode::OK);
        let summary_body = response_json(summary_response).await;
        assert_eq!(summary_body["runId"], run_id);
        assert_eq!(summary_body["spaceId"], "nostra-governance-v0");
        assert_eq!(summary_body["mode"], "evaluation_phase5");
        assert_eq!(summary_body["surfaceVariant"], "compare");
        assert_eq!(summary_body["recommendation"], "go");
        assert_eq!(summary_body["improvementScore"], 4.2);
        assert_eq!(
            summary_body["metrics"]["approvalDecisionCount"],
            serde_json::Value::from(1)
        );
        assert_eq!(summary_body["eventKey"], stored_key);
        assert!(
            summary_body["eventCount"].as_u64().unwrap_or_default() >= 3,
            "event count should include run_start, button_click, and run_end"
        );

        let date = spatial_experiment_event_date(timestamp.as_str());
        let event_log_key = spatial_experiment_events_key(date.as_str());
        let stored_events =
            store_read_jsonl::<SpatialExperimentEventRecord>(event_log_key.as_str())
                .await
                .expect("read stored spatial experiment events");
        assert!(
            stored_events
                .iter()
                .any(|event| event.run_id == run_id && event.event_type == "run_end")
        );
    }

    #[tokio::test]
    async fn spatial_experiment_event_endpoint_rejects_invalid_inputs() {
        let _lock = acquire_testing_env_lock();

        let unsupported_event_response =
            post_cortex_viewspec_spatial_experiment_event(Json(SpatialExperimentEventRequest {
                run_id: "spatial-run-invalid".to_string(),
                space_id: "nostra-governance-v0".to_string(),
                mode: "evaluation_phase5".to_string(),
                surface_variant: "compare".to_string(),
                event_type: "unknown_event".to_string(),
                timestamp: "2026-02-22T12:00:00Z".to_string(),
                payload: json!({}),
                build_id: Some("build-test-1".to_string()),
                host: "cortex-web".to_string(),
            }))
            .await
            .into_response();
        assert_eq!(unsupported_event_response.status(), StatusCode::BAD_REQUEST);
        let unsupported_body = response_json(unsupported_event_response).await;
        assert_eq!(
            unsupported_body["errorCode"],
            "UNSUPPORTED_SPATIAL_EXPERIMENT_EVENT"
        );

        let invalid_run_response =
            post_cortex_viewspec_spatial_experiment_event(Json(SpatialExperimentEventRequest {
                run_id: "../bad-run".to_string(),
                space_id: "nostra-governance-v0".to_string(),
                mode: "evaluation_phase5".to_string(),
                surface_variant: "compare".to_string(),
                event_type: "run_start".to_string(),
                timestamp: "2026-02-22T12:00:00Z".to_string(),
                payload: json!({}),
                build_id: Some("build-test-1".to_string()),
                host: "cortex-web".to_string(),
            }))
            .await
            .into_response();
        assert_eq!(invalid_run_response.status(), StatusCode::BAD_REQUEST);
        let invalid_body = response_json(invalid_run_response).await;
        assert_eq!(
            invalid_body["errorCode"],
            "INVALID_SPATIAL_EXPERIMENT_RUN_ID"
        );

        let invalid_get_response =
            get_cortex_viewspec_spatial_experiment_run(Path("../bad-run".to_string()))
                .await
                .into_response();
        assert_eq!(invalid_get_response.status(), StatusCode::BAD_REQUEST);
        let invalid_get_body = response_json(invalid_get_response).await;
        assert_eq!(
            invalid_get_body["errorCode"],
            "INVALID_SPATIAL_EXPERIMENT_RUN_ID"
        );
    }

    #[test]
    fn signature_requirement_matrix_respects_mode() {
        assert!(!signature_required_for_gate(
            DecisionSignedMode::Off,
            "release_blocker",
            true
        ));
        assert!(!signature_required_for_gate(
            DecisionSignedMode::Warn,
            "release_blocker",
            true
        ));
        assert!(signature_required_for_gate(
            DecisionSignedMode::RequiredP0P1,
            "release_blocker",
            false
        ));
        assert!(!signature_required_for_gate(
            DecisionSignedMode::RequiredP0P1,
            "informational",
            false
        ));
        assert!(signature_required_for_gate(
            DecisionSignedMode::RequiredAll,
            "informational",
            false
        ));
    }

    #[test]
    fn queue_conflict_state_detects_error_signals() {
        assert!(mutation_conflict_state(Some("Conflict detected on replay")));
        assert!(!mutation_conflict_state(Some(
            "timed out waiting for replica"
        )));
    }

    #[test]
    fn timestamp_iso_is_rfc3339_for_unix_seconds() {
        let formatted = timestamp_iso(1_700_000_000).expect("timestamp to format");
        assert!(formatted.contains('T'));
        assert!(formatted.ends_with('Z') || formatted.contains('+'));
    }
}
