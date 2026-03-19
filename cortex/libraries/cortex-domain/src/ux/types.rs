use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub const UX_STATUS_NEW: &str = "new";
pub const UX_STATUS_CANDIDATE: &str = "candidate";
pub const UX_STATUS_APPROVED: &str = "approved";
pub const UX_STATUS_SHIPPED: &str = "shipped";
pub const UX_STATUS_REMEASURED: &str = "remeasured";
pub const UX_STATUS_REJECTED: &str = "rejected";
pub const UX_STATUS_OVERDUE_REMEASUREMENT: &str = "overdue_remeasurement";
pub const UX_STATUS_BLOCKED_MISSING_BASELINE: &str = "blocked_missing_baseline";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEntryNavMeta {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_count: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub badge_tone: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub attention_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub collapsible_hint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NavigationEntrySpec {
    pub route_id: String,
    pub label: String,
    pub icon: String,
    pub category: String,
    pub required_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_slot: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nav_meta: Option<NavigationEntryNavMeta>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct NavigationGraphSpec {
    pub entries: Vec<NavigationEntrySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedNavigationGraphSpec {
    pub schema_version: String,
    pub updated_at: String,
    pub approved_by: String,
    pub rationale: String,
    pub navigation_graph: NavigationGraphSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ShellLayoutSpec {
    pub layout_id: String,
    pub navigation_graph: NavigationGraphSpec,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewCapabilityManifest {
    pub route_id: String,
    pub route_label: String,
    pub view_capability_id: String,
    pub pattern_id: String,
    pub promotion_status: String,
    pub operator_critical: bool,
    pub required_role: String,
    pub approval_required: bool,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct PatternContract {
    pub pattern_id: String,
    pub label: String,
    pub required_role: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ViewCapabilityMatrixRow {
    pub route_id: String,
    pub view_capability_id: String,
    pub pattern_id: String,
    pub required_role: String,
    pub approval_required: bool,
    pub operator_critical: bool,
    pub promotion_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompilationContext {
    pub space_id: String,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompiledNavigationEntry {
    pub capability_id: String,
    pub route_id: String,
    pub label: String,
    pub icon: String,
    pub category: String,
    pub required_role: String,
    pub nav_slot: String,
    pub nav_band: String,
    pub surfacing_heuristic: String,
    pub operational_frequency: String,
    pub rank: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompiledSurfacingPlan {
    pub primary_core: Vec<String>,
    pub secondary: BTreeMap<String, Vec<String>>,
    pub contextual_deep: Vec<String>,
    pub hidden: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompiledNavigationPlan {
    pub schema_version: String,
    pub generated_at: String,
    pub space_id: String,
    pub actor_role: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<String>,
    pub plan_hash: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authz_engine: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authz_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub authz_decision_version: Option<String>,
    pub entries: Vec<CompiledNavigationEntry>,
    pub surfacing: CompiledSurfacingPlan,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum SurfaceZone {
    HeapPageBar,
    HeapSelectionBar,
    HeapDetailFooter,
    HeapDetailHeader,
    HeapCardMenu,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "snake_case")]
pub enum PageType {
    HeapBoard,
    HeapDetail,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionSelectionContext {
    pub selected_artifact_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_artifact_id: Option<String>,
    pub selected_count: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_block_types: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionActiveFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub filter_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_tags: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_page_links: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionFeatureFlags {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heap_create_flow_enabled: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub heap_parity_enabled: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompiledActionPlanRequest {
    pub schema_version: String,
    pub space_id: String,
    pub actor_role: String,
    pub route_id: String,
    pub page_type: PageType,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub density: Option<String>,
    pub zones: Vec<SurfaceZone>,
    pub selection: ActionSelectionContext,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub active_filters: Option<ActionActiveFilters>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub feature_flags: Option<ActionFeatureFlags>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarActionKind {
    Command,
    Mutation,
    Navigation,
    PanelToggle,
    Download,
    Destructive,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarActionGroup {
    Primary,
    Secondary,
    Danger,
    Selection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolbarActionEmphasis {
    Default,
    Primary,
    Accent,
    Danger,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarActionSelectionConstraints {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub min_selected: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_selected: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub require_single_selection: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ConfirmationStyle {
    Danger,
    Default,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarActionConfirmation {
    pub required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub style: Option<ConfirmationStyle>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarActionStewardGate {
    pub required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ToolbarActionDescriptor {
    pub id: String,
    pub capability_id: String,
    pub zone: SurfaceZone,
    pub label: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub icon: Option<String>,
    pub kind: ToolbarActionKind,
    pub action: String,
    pub priority: u32,
    pub group: ToolbarActionGroup,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub emphasis: Option<ToolbarActionEmphasis>,
    pub visible: bool,
    pub enabled: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub disabled_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selection_constraints: Option<ToolbarActionSelectionConstraints>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confirmation: Option<ToolbarActionConfirmation>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub steward_gate: Option<ToolbarActionStewardGate>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ActionZoneLayoutHint {
    Row,
    Pillbar,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActionZonePlan {
    pub zone: SurfaceZone,
    pub layout_hint: ActionZoneLayoutHint,
    pub actions: Vec<ToolbarActionDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct CompiledActionPlan {
    pub schema_version: String,
    pub generated_at: String,
    pub plan_hash: String,
    pub space_id: String,
    pub route_id: String,
    pub page_type: PageType,
    pub actor_role: String,
    pub zones: Vec<ActionZonePlan>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UxCandidateMetrics {
    pub task_success: f32,
    pub time_to_decision_seconds: f32,
    pub nav_depth: u32,
    pub accessibility_score: f32,
    pub consistency_score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxAutoGates {
    pub accessibility: bool,
    pub decision_safety_semantics: bool,
    pub offline_integrity: bool,
    pub policy_compliance: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxApprovalPayload {
    pub approved_by: String,
    pub rationale: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UxLayoutEvaluationRequest {
    pub candidate_id: String,
    pub route_id: String,
    pub view_capability_id: String,
    pub structural_change: bool,
    pub metrics: UxCandidateMetrics,
    pub gates: UxAutoGates,
    #[serde(default)]
    pub approval: Option<UxApprovalPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct UxCandidateEvaluation {
    pub candidate_id: String,
    pub route_id: String,
    pub view_capability_id: String,
    pub cuqs_score: f32,
    pub promotion_status: String,
    pub blocked_reasons: Vec<String>,
    #[serde(default)]
    pub approved_by: Option<String>,
    #[serde(default)]
    pub approval_rationale: Option<String>,
    #[serde(default)]
    pub approved_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxFeedbackEvent {
    pub event_id: String,
    pub route_id: String,
    pub view_id: String,
    #[serde(default)]
    pub action_id: Option<String>,
    pub friction_tag: String,
    pub severity: String,
    #[serde(default)]
    pub free_text: Option<String>,
    pub session_id: String,
    #[serde(default)]
    pub run_id: Option<String>,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxFeedbackQueueItem {
    pub queue_id: String,
    pub dedupe_key: String,
    pub route_id: String,
    pub view_id: String,
    pub friction_tag: String,
    pub severity: String,
    pub status: String,
    pub priority: String,
    #[serde(default)]
    pub assigned_to: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub baseline_metric_date: Option<String>,
    #[serde(default)]
    pub post_release_metric_date: Option<String>,
    pub first_seen_at: String,
    pub updated_at: String,
    pub event_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxPromotionDecision {
    pub decision_id: String,
    pub candidate_id: String,
    pub route_id: String,
    pub view_capability_id: String,
    pub promotion_action: String,
    pub approved_by: String,
    pub rationale: String,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxPromotionApproval {
    pub candidate_id: String,
    pub route_id: String,
    pub view_capability_id: String,
    pub approved_by: String,
    pub rationale: String,
    pub approved_at: String,
    pub baseline_metric_date: String,
    pub post_release_metric_date: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct UxPromotionRejection {
    pub candidate_id: String,
    pub route_id: String,
    pub view_capability_id: String,
    pub rejected_by: String,
    pub rationale: String,
    pub rejected_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactAuditEvent {
    pub audit_id: String,
    pub artifact_id: String,
    pub action: String,
    pub actor_role: String,
    pub actor_id: String,
    pub route_id: String,
    pub timestamp: String,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ArtifactCapabilityManifest {
    pub storage_backend: String,
    pub single_writer: bool,
    pub required_role_create: String,
    pub required_role_publish: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct PersistedShellLayoutSpec {
    pub schema_version: String,
    pub updated_at: String,
    pub layout_spec: ShellLayoutSpec,
    pub navigation_contract: PersistedNavigationGraphSpec,
    pub view_capabilities: Vec<ViewCapabilityManifest>,
    pub patterns: Vec<PatternContract>,
    pub capability_matrix: Vec<ViewCapabilityMatrixRow>,
}
