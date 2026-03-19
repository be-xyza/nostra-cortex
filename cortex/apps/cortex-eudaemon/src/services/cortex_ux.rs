use chrono::Utc;
use cortex_domain::ux as domain_ux;
use std::env;
use std::fs;
use std::path::PathBuf;

pub use cortex_domain::ux::types::{
    ActionActiveFilters, ActionFeatureFlags, ActionSelectionContext, ActionZoneLayoutHint,
    ActionZonePlan, ArtifactAuditEvent, ArtifactCapabilityManifest, CompiledActionPlan,
    CompiledActionPlanRequest, ConfirmationStyle, PageType, PatternContract,
    PersistedShellLayoutSpec, ShellLayoutSpec, SurfaceZone, ToolbarActionConfirmation,
    ToolbarActionDescriptor, ToolbarActionEmphasis, ToolbarActionGroup, ToolbarActionKind,
    ToolbarActionSelectionConstraints, ToolbarActionStewardGate, UX_STATUS_APPROVED,
    UX_STATUS_BLOCKED_MISSING_BASELINE, UX_STATUS_CANDIDATE, UX_STATUS_NEW,
    UX_STATUS_OVERDUE_REMEASUREMENT, UX_STATUS_REJECTED, UX_STATUS_REMEASURED, UX_STATUS_SHIPPED,
    UxApprovalPayload, UxAutoGates, UxCandidateEvaluation, UxCandidateMetrics, UxFeedbackEvent,
    UxFeedbackQueueItem, UxLayoutEvaluationRequest, UxPromotionApproval, UxPromotionDecision,
    UxPromotionRejection, ViewCapabilityManifest, ViewCapabilityMatrixRow,
};

pub fn now_iso() -> String {
    Utc::now().to_rfc3339()
}

pub fn role_rank(role: &str) -> u8 {
    domain_ux::role_rank(role)
}

pub fn valid_feedback_status(status: &str) -> bool {
    domain_ux::valid_feedback_status(status)
}

pub fn default_shell_layout_spec() -> ShellLayoutSpec {
    domain_ux::default_shell_layout_spec()
}

pub fn default_artifact_capability_manifest() -> ArtifactCapabilityManifest {
    domain_ux::default_artifact_capability_manifest()
}

pub fn evaluate_cuqs(request: UxLayoutEvaluationRequest) -> UxCandidateEvaluation {
    domain_ux::evaluate_cuqs(request)
}

pub fn default_persisted_shell_contract() -> PersistedShellLayoutSpec {
    domain_ux::default_persisted_shell_contract(&now_iso())
}

pub fn load_persisted_shell_contract() -> Result<PersistedShellLayoutSpec, String> {
    let path = contract_path();
    let raw = fs::read_to_string(&path).map_err(|err| err.to_string())?;
    serde_json::from_str::<PersistedShellLayoutSpec>(&raw).map_err(|err| err.to_string())
}

pub fn save_persisted_shell_contract(contract: &PersistedShellLayoutSpec) -> Result<(), String> {
    if contract.layout_spec.layout_id.trim().is_empty() {
        return Err("layout_spec.layout_id must not be empty".to_string());
    }
    if contract.layout_spec.navigation_graph.entries.is_empty() {
        return Err("layout_spec.navigation_graph.entries must not be empty".to_string());
    }
    if contract.navigation_contract.approved_by.trim().is_empty() {
        return Err("navigation_contract.approved_by must not be empty".to_string());
    }
    if contract.navigation_contract.rationale.trim().is_empty() {
        return Err("navigation_contract.rationale must not be empty".to_string());
    }
    for entry in &contract.layout_spec.navigation_graph.entries {
        if let Some(slot) = entry.nav_slot.as_deref() {
            let trimmed = slot.trim();
            if !trimmed.is_empty() && !domain_ux::valid_nav_slot(trimmed) {
                return Err(format!(
                    "layout_spec.navigation_graph.entries has invalid nav_slot: route_id={} nav_slot={}",
                    entry.route_id, trimmed
                ));
            }
        }
    }
    let mut candidate = contract.clone();
    candidate.updated_at = now_iso();
    let encoded = serde_json::to_string_pretty(&candidate).map_err(|err| err.to_string())?;
    let path = contract_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    fs::write(path, encoded).map_err(|err| err.to_string())
}

pub fn resolve_shell_layout_spec() -> ShellLayoutSpec {
    load_persisted_shell_contract()
        .map(|persisted| persisted.layout_spec)
        .unwrap_or_else(|_| default_shell_layout_spec())
}

pub fn resolve_view_capability_manifests() -> Vec<ViewCapabilityManifest> {
    load_persisted_shell_contract()
        .map(|persisted| persisted.view_capabilities)
        .unwrap_or_else(|_| domain_ux::default_view_capability_manifests())
}

pub fn resolve_pattern_contracts() -> Vec<PatternContract> {
    load_persisted_shell_contract()
        .map(|persisted| persisted.patterns)
        .unwrap_or_else(|_| domain_ux::default_pattern_contracts())
}

pub fn resolve_capability_matrix() -> Vec<ViewCapabilityMatrixRow> {
    load_persisted_shell_contract()
        .map(|persisted| persisted.capability_matrix)
        .unwrap_or_else(|_| {
            let caps = domain_ux::default_view_capability_manifests();
            domain_ux::build_capability_matrix(&caps)
        })
}

fn cortex_ux_log_dir() -> PathBuf {
    if let Ok(path) = env::var("NOSTRA_CORTEX_UX_LOG_DIR") {
        return PathBuf::from(path);
    }
    env::temp_dir().join("nostra").join("cortex_ux")
}

fn contract_path() -> PathBuf {
    if let Ok(path) = env::var("NOSTRA_CORTEX_UX_CONTRACT_PATH") {
        return PathBuf::from(path);
    }
    cortex_ux_log_dir().join("cortex_ux_contract_v1.json")
}
