use crate::capabilities::navigation_graph::{
    OperationalFrequency, PlatformCapabilityCatalog, SpaceCapabilityGraph,
    SpaceCapabilityNodeOverride, SurfacingHeuristic,
};
use crate::ux::types::{
    ActionZoneLayoutHint, ActionZonePlan, ArtifactCapabilityManifest, CompilationContext,
    CompiledActionPlan, CompiledActionPlanRequest, CompiledNavigationEntry, CompiledNavigationPlan,
    CompiledSurfacingPlan, ConfirmationStyle, NavigationEntryNavMeta, NavigationEntrySpec,
    NavigationGraphSpec, PageType, PatternContract, PersistedNavigationGraphSpec,
    PersistedShellLayoutSpec, ShellLayoutSpec, SurfaceZone, ToolbarActionConfirmation,
    ToolbarActionDescriptor, ToolbarActionEmphasis, ToolbarActionGroup, ToolbarActionKind,
    ToolbarActionSelectionConstraints, ToolbarActionStewardGate, UX_STATUS_CANDIDATE,
    UxCandidateEvaluation, UxLayoutEvaluationRequest, ViewCapabilityManifest,
    ViewCapabilityMatrixRow,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub fn role_rank(role: &str) -> u8 {
    match role.to_ascii_lowercase().as_str() {
        "viewer" => 1,
        "editor" => 2,
        "operator" => 3,
        "steward" => 4,
        "admin" => 5,
        _ => 0,
    }
}

pub fn has_route_access(required_role: &str, actor_role: &str) -> bool {
    role_rank(actor_role) >= role_rank(required_role)
}

pub fn clamp_required_role_floor(base_required_role: &str, candidate_role: Option<&str>) -> String {
    let base = base_required_role.to_ascii_lowercase();
    let candidate = candidate_role
        .map(|value| value.to_ascii_lowercase())
        .unwrap_or_else(|| base.clone());
    if role_rank(&candidate) < role_rank(&base) {
        return base;
    }
    candidate
}

pub fn valid_feedback_status(status: &str) -> bool {
    matches!(
        status,
        "new"
            | "deduped"
            | "triaged"
            | "candidate"
            | "approved"
            | "shipped"
            | "remeasured"
            | "rejected"
            | "overdue_remeasurement"
            | "blocked_missing_baseline"
    )
}

pub fn default_shell_layout_spec() -> ShellLayoutSpec {
    ShellLayoutSpec {
        layout_id: "cortex.desktop.shell.v1".to_string(),
        navigation_graph: NavigationGraphSpec {
            entries: vec![
                nav(
                    "/spaces",
                    "Spaces",
                    "SP",
                    "core",
                    "viewer",
                    Some("primary_workspace"),
                    Some(nav_meta(Some(2), "default", false, None, "expanded")),
                ),
                nav(
                    "/heap",
                    "Heap Canvas",
                    "HP",
                    "core",
                    "operator",
                    Some("primary_workspace"),
                    Some(nav_meta(
                        Some(1),
                        "info",
                        true,
                        Some("Live heap"),
                        "expanded",
                    )),
                ),
                nav(
                    "/studio",
                    "Studio",
                    "ST",
                    "bridge",
                    "operator",
                    Some("secondary_build"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/workflows",
                    "Workflows",
                    "FL",
                    "bridge",
                    "operator",
                    Some("primary_execute"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/contributions",
                    "Contributions",
                    "CG",
                    "bridge",
                    "operator",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/artifacts",
                    "Artifacts",
                    "AR",
                    "bridge",
                    "operator",
                    Some("secondary_build"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/vfs",
                    "Virtual File System",
                    "FS",
                    "bridge",
                    "operator",
                    Some("secondary_build"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/logs",
                    "System Logs",
                    "LG",
                    "bridge",
                    "operator",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/settings",
                    "Settings & Preferences",
                    "SE",
                    "secondary",
                    "operator",
                    Some("secondary_admin"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/inbox",
                    "Inbox & Approvals",
                    "IN",
                    "core",
                    "operator",
                    Some("primary_attention"),
                    Some(nav_meta(
                        Some(1),
                        "critical",
                        true,
                        Some("Approvals"),
                        "expanded",
                    )),
                ),
                nav(
                    "/agents",
                    "Agent Fleet",
                    "AG",
                    "core",
                    "operator",
                    Some("secondary_agents"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/discovery",
                    "Global Discovery",
                    "DS",
                    "bridge",
                    "viewer",
                    Some("secondary_agents"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/metrics",
                    "Telemetry & SLOs",
                    "MT",
                    "secondary",
                    "steward",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "warn", false, Some("SLO"), "expanded")),
                ),
                nav(
                    "/memory",
                    "Agent Memory FS",
                    "MF",
                    "secondary",
                    "operator",
                    Some("secondary_agents"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/simulation",
                    "Simulation Adapter",
                    "SM",
                    "bridge",
                    "operator",
                    Some("secondary_agents"),
                    Some(nav_meta(Some(0), "default", false, None, "expanded")),
                ),
                nav(
                    "/labs",
                    "Labs",
                    "LB",
                    "secondary",
                    "viewer",
                    Some("labs"),
                    Some(nav_meta(Some(0), "default", false, None, "rail")),
                ),
                nav(
                    "/system",
                    "System",
                    "SY",
                    "secondary",
                    "operator",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "critical", false, Some("Ops"), "rail")),
                ),
                nav(
                    "/system/siq",
                    "SIQ Gates",
                    "SQ",
                    "secondary",
                    "operator",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "warn", false, Some("SIQ"), "rail")),
                ),
                nav(
                    "/testing",
                    "Testing & Gates",
                    "TG",
                    "secondary",
                    "operator",
                    Some("secondary_ops"),
                    Some(nav_meta(Some(0), "default", false, None, "rail")),
                ),
            ],
        },
    }
}

pub fn default_artifact_capability_manifest() -> ArtifactCapabilityManifest {
    ArtifactCapabilityManifest {
        storage_backend: "vfs+local-json".to_string(),
        single_writer: true,
        required_role_create: "operator".to_string(),
        required_role_publish: "steward".to_string(),
    }
}

pub fn default_view_capability_manifests() -> Vec<ViewCapabilityManifest> {
    vec![
        cap(
            "/spaces",
            "Spaces",
            "view.spaces",
            "pattern.spaces",
            UX_STATUS_CANDIDATE,
            "viewer",
            false,
            "Nostra Space Workspace Contexts",
        ),
        cap(
            "/workflows",
            "Workflows",
            "view.workflows",
            "pattern.workflow",
            UX_STATUS_CANDIDATE,
            "operator",
            true,
            "Global Workflow operations fallback lane",
        ),
        cap(
            "/inbox",
            "Inbox & Approvals",
            "view.inbox",
            "pattern.workflow",
            UX_STATUS_CANDIDATE,
            "operator",
            true,
            "Operator Human-in-the-Loop gate review and approval inbox",
        ),
        cap(
            "/studio",
            "Studio",
            "view.studio",
            "pattern.studio",
            UX_STATUS_CANDIDATE,
            "operator",
            true,
            "Global Studio artifact fallback lane",
        ),
        cap(
            "/artifacts",
            "Artifacts",
            "view.artifacts",
            "pattern.artifacts",
            UX_STATUS_CANDIDATE,
            "operator",
            true,
            "Global Artifact collaboration fallback lane",
        ),
        cap(
            "/testing",
            "Testing",
            "view.testing",
            "pattern.testing",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Testing telemetry lane",
        ),
        cap(
            "/contributions",
            "DPub Workbench",
            "view.contributions",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Global Contribution graph and path analysis fallback lane",
        ),
        cap(
            "/vfs",
            "Virtual File System",
            "view.vfs",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Virtual filesystem browser and workspace storage controls",
        ),
        cap(
            "/logs",
            "System Logs",
            "view.logs",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Operational runtime logs and event stream inspection lane",
        ),
        cap(
            "/settings",
            "Settings & Preferences",
            "view.settings",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            true,
            "Runtime preference policy and execution profile settings lane",
        ),
        cap(
            "/agents",
            "Agent Fleet",
            "view.agents",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Active agent runtime visibility and control plane",
        ),
        cap(
            "/discovery",
            "Global Discovery",
            "view.discovery",
            "pattern.spaces",
            UX_STATUS_CANDIDATE,
            "viewer",
            false,
            "Cross-space graph discovery and global search projection lane",
        ),
        cap(
            "/metrics",
            "Telemetry & SLOs",
            "view.metrics",
            "pattern.testing",
            UX_STATUS_CANDIDATE,
            "steward",
            false,
            "Runtime SLO status, breach tracking, and transport observability lane",
        ),
        cap(
            "/memory",
            "Agent Memory FS",
            "view.memory",
            "pattern.studio",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Agent episodic memory branching and context filesystem inspection",
        ),
        cap(
            "/simulation",
            "Simulation Adapter",
            "view.simulation",
            "pattern.studio",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Simulation adapter status, controls, and injection lane",
        ),
        cap(
            "/heap",
            "Heap Canvas",
            "view.heap",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "Infinite spatial CRDT Heap board",
        ),
        cap(
            "/labs",
            "Labs",
            "view.labs",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "viewer",
            false,
            "Laboratory surfaces for governed experimentation",
        ),
        cap(
            "/system",
            "System",
            "view.system",
            "pattern.system",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "System capability graph and operational governance surfaces",
        ),
        cap(
            "/system/siq",
            "SIQ Gates",
            "view.siq",
            "pattern.testing",
            UX_STATUS_CANDIDATE,
            "operator",
            false,
            "System Integrity + Quality (SIQ) gate intake surface and artifacts summary",
        ),
    ]
}

pub fn default_pattern_contracts() -> Vec<PatternContract> {
    vec![
        PatternContract {
            pattern_id: "pattern.spaces".to_string(),
            label: "Spaces".to_string(),
            required_role: "viewer".to_string(),
            description: "Space sovereignty and Workspace encapsulation".to_string(),
        },
        PatternContract {
            pattern_id: "pattern.workflow".to_string(),
            label: "Workflow".to_string(),
            required_role: "operator".to_string(),
            description: "Workflow decision and execution surfaces".to_string(),
        },
        PatternContract {
            pattern_id: "pattern.studio".to_string(),
            label: "Studio".to_string(),
            required_role: "operator".to_string(),
            description: "Studio synthesis and qualification surfaces".to_string(),
        },
        PatternContract {
            pattern_id: "pattern.artifacts".to_string(),
            label: "Artifacts".to_string(),
            required_role: "operator".to_string(),
            description: "Artifact collaboration and publication surfaces".to_string(),
        },
        PatternContract {
            pattern_id: "pattern.testing".to_string(),
            label: "Testing".to_string(),
            required_role: "operator".to_string(),
            description: "Testing and reliability surfaces".to_string(),
        },
        PatternContract {
            pattern_id: "pattern.system".to_string(),
            label: "System".to_string(),
            required_role: "operator".to_string(),
            description: "System and contribution graph observability surfaces".to_string(),
        },
    ]
}

pub fn build_capability_matrix(caps: &[ViewCapabilityManifest]) -> Vec<ViewCapabilityMatrixRow> {
    caps.iter()
        .map(|cap| ViewCapabilityMatrixRow {
            route_id: cap.route_id.clone(),
            view_capability_id: cap.view_capability_id.clone(),
            pattern_id: cap.pattern_id.clone(),
            required_role: cap.required_role.clone(),
            approval_required: cap.approval_required,
            operator_critical: cap.operator_critical,
            promotion_status: cap.promotion_status.clone(),
        })
        .collect()
}

pub fn default_persisted_shell_contract(updated_at: &str) -> PersistedShellLayoutSpec {
    let layout_spec = default_shell_layout_spec();
    let view_capabilities = default_view_capability_manifests();
    let patterns = default_pattern_contracts();
    let capability_matrix = build_capability_matrix(&view_capabilities);

    PersistedShellLayoutSpec {
        schema_version: "1.0.0".to_string(),
        updated_at: updated_at.to_string(),
        layout_spec: layout_spec.clone(),
        navigation_contract: PersistedNavigationGraphSpec {
            schema_version: "1.0.0".to_string(),
            updated_at: updated_at.to_string(),
            approved_by: "system".to_string(),
            rationale: "default shell contract".to_string(),
            navigation_graph: layout_spec.navigation_graph,
        },
        view_capabilities,
        patterns,
        capability_matrix,
    }
}

pub fn valid_nav_slot(value: &str) -> bool {
    matches!(
        value.trim(),
        "primary_attention"
            | "primary_workspace"
            | "primary_execute"
            | "secondary_build"
            | "secondary_ops"
            | "secondary_agents"
            | "secondary_admin"
            | "labs"
            | "hidden"
    )
}

fn slot_weight(slot: &str) -> u32 {
    match slot {
        "primary_attention" => 900,
        "primary_workspace" => 850,
        "primary_execute" => 800,
        "secondary_ops" => 600,
        "secondary_build" => 550,
        "secondary_agents" => 500,
        "secondary_admin" => 450,
        "labs" => 200,
        "hidden" => 0,
        _ => 600,
    }
}

fn infer_nav_slot_for_route(route_id: &str) -> &'static str {
    match route_id {
        "/inbox" | "/notifications" => "primary_attention",
        "/spaces" | "/heap" => "primary_workspace",
        "/workflows" | "/flows" => "primary_execute",
        "/studio" | "/artifacts" | "/vfs" => "secondary_build",
        "/agents" | "/discovery" | "/memory" | "/simulation" => "secondary_agents",
        "/settings" => "secondary_admin",
        "/labs" => "labs",
        _ => "secondary_ops",
    }
}

fn resolve_nav_slot(
    route_id: &str,
    nav_entry: Option<&NavigationEntrySpec>,
    heuristic: &SurfacingHeuristic,
    category: &str,
) -> String {
    if let Some(raw_slot) = nav_entry.and_then(|entry| entry.nav_slot.as_deref()) {
        let trimmed = raw_slot.trim();
        if valid_nav_slot(trimmed) {
            return trimmed.to_string();
        }
        return "secondary_ops".to_string();
    }

    if nav_entry
        .and_then(|entry| entry.nav_meta.as_ref())
        .and_then(|meta| meta.attention)
        .unwrap_or(false)
    {
        return "primary_attention".to_string();
    }

    if matches!(heuristic, SurfacingHeuristic::PrimaryCore) {
        match category {
            "workflow" | "execution" => return "primary_execute".to_string(),
            "core" | "workspace" => return "primary_workspace".to_string(),
            _ => return "primary_workspace".to_string(),
        }
    }

    infer_nav_slot_for_route(route_id).to_string()
}

pub fn evaluate_cuqs(request: UxLayoutEvaluationRequest) -> UxCandidateEvaluation {
    let mut blocked_reasons = Vec::new();
    if !request.gates.accessibility {
        blocked_reasons.push("accessibility_gate_failed".to_string());
    }
    if !request.gates.decision_safety_semantics {
        blocked_reasons.push("decision_safety_gate_failed".to_string());
    }
    if !request.gates.offline_integrity {
        blocked_reasons.push("offline_integrity_gate_failed".to_string());
    }
    if !request.gates.policy_compliance {
        blocked_reasons.push("policy_compliance_gate_failed".to_string());
    }

    let nav_factor = 1.0_f32 - ((request.metrics.nav_depth as f32) / 10.0_f32).min(0.5_f32);
    let time_factor =
        (1.0_f32 - (request.metrics.time_to_decision_seconds / 180.0_f32)).clamp(0.0_f32, 1.0_f32);
    let cuqs_score = ((request.metrics.task_success * 0.35_f32)
        + (request.metrics.accessibility_score * 0.2_f32)
        + (request.metrics.consistency_score * 0.2_f32)
        + (time_factor * 0.15_f32)
        + (nav_factor * 0.1_f32))
        .clamp(0.0_f32, 1.0_f32);

    let (promotion_status, approved_by, approval_rationale, approved_at) =
        if !blocked_reasons.is_empty() {
            ("blocked_gates".to_string(), None, None, None)
        } else if request.structural_change {
            match request.approval {
                Some(approval) => (
                    "eligible_hitl_approved".to_string(),
                    Some(approval.approved_by),
                    Some(approval.rationale),
                    Some(approval.timestamp),
                ),
                None => ("blocked_hitl_required".to_string(), None, None, None),
            }
        } else if cuqs_score >= 0.75_f32 {
            ("eligible_auto".to_string(), None, None, None)
        } else {
            (UX_STATUS_CANDIDATE.to_string(), None, None, None)
        };

    UxCandidateEvaluation {
        candidate_id: request.candidate_id,
        route_id: request.route_id,
        view_capability_id: request.view_capability_id,
        cuqs_score,
        promotion_status,
        blocked_reasons,
        approved_by,
        approval_rationale,
        approved_at,
    }
}

pub fn compile_navigation_plan(
    catalog: &PlatformCapabilityCatalog,
    space_graph: &SpaceCapabilityGraph,
    layout_spec: &ShellLayoutSpec,
    context: &CompilationContext,
) -> CompiledNavigationPlan {
    compile_navigation_plan_with_generated_at(
        catalog,
        space_graph,
        layout_spec,
        context,
        &now_epoch_seconds(),
    )
}

fn compile_navigation_plan_with_generated_at(
    catalog: &PlatformCapabilityCatalog,
    space_graph: &SpaceCapabilityGraph,
    layout_spec: &ShellLayoutSpec,
    context: &CompilationContext,
    generated_at: &str,
) -> CompiledNavigationPlan {
    let override_by_id: BTreeMap<String, &SpaceCapabilityNodeOverride> = space_graph
        .nodes
        .iter()
        .map(|item| (item.capability_id.0.clone(), item))
        .collect();
    let nav_entry_by_route: BTreeMap<String, &NavigationEntrySpec> = layout_spec
        .navigation_graph
        .entries
        .iter()
        .map(|item| (item.route_id.clone(), item))
        .collect();
    let actor_role = context.actor_role.to_ascii_lowercase();

    let mut candidates: Vec<(u32, u32, CompiledNavigationEntry)> = Vec::new();

    for node in catalog.nodes.iter() {
        let override_node = override_by_id.get(&node.id.0).copied();
        let is_active = override_node.map(|item| item.is_active).unwrap_or(true);
        if !is_active {
            continue;
        }

        let route_id = match node.route_id.as_ref() {
            Some(value) if !value.trim().is_empty() => value.to_string(),
            _ => continue,
        };

        let base_required_role = node
            .required_role
            .as_deref()
            .unwrap_or("viewer")
            .to_string();
        let required_role = clamp_required_role_floor(
            &base_required_role,
            override_node
                .and_then(|item| item.local_required_role.as_deref())
                .map(|value| value.trim())
                .filter(|value| !value.is_empty()),
        );
        if !has_route_access(&required_role, &actor_role) {
            continue;
        }

        let heuristic = override_node
            .and_then(|item| item.surfacing_heuristic.clone())
            .unwrap_or_else(|| node.surfacing_heuristic.clone());
        let frequency = override_node
            .and_then(|item| item.operational_frequency.clone())
            .unwrap_or_else(|| node.operational_frequency.clone());
        let nav_entry = nav_entry_by_route.get(&route_id).copied();

        let category = nav_entry
            .map(|entry| entry.category.clone())
            .or_else(|| node.category.clone())
            .unwrap_or_else(|| "uncategorized".to_string());
        let icon = node
            .icon
            .clone()
            .or_else(|| nav_entry.map(|entry| entry.icon.clone()))
            .unwrap_or_else(|| "--".to_string());
        let label = override_node
            .and_then(|item| item.local_alias.clone())
            .unwrap_or_else(|| node.name.clone());
        let nav_slot = resolve_nav_slot(&route_id, nav_entry, &heuristic, &category);
        if nav_slot == "hidden" {
            continue;
        }
        let nav_band = nav_band_for_heuristic(&heuristic).to_string();
        let score = heuristic_weight(&heuristic) + frequency_weight(&frequency);

        candidates.push((
            slot_weight(&nav_slot),
            score,
            CompiledNavigationEntry {
                capability_id: node.id.0.clone(),
                route_id,
                label,
                icon,
                category,
                required_role,
                nav_slot,
                nav_band,
                surfacing_heuristic: surfacing_name(&heuristic).to_string(),
                operational_frequency: frequency_name(&frequency).to_string(),
                rank: 0,
            },
        ));
    }

    candidates.sort_by(|left, right| {
        right
            .0
            .cmp(&left.0)
            .then_with(|| right.1.cmp(&left.1))
            .then_with(|| left.2.category.cmp(&right.2.category))
            .then_with(|| left.2.label.cmp(&right.2.label))
            .then_with(|| left.2.route_id.cmp(&right.2.route_id))
            .then_with(|| left.2.capability_id.cmp(&right.2.capability_id))
    });

    let mut entries: Vec<CompiledNavigationEntry> =
        candidates.into_iter().map(|item| item.2).collect();
    for (index, entry) in entries.iter_mut().enumerate() {
        entry.rank = (index + 1) as u32;
    }

    let mut secondary: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut primary_core = Vec::new();
    let mut contextual_deep = Vec::new();
    let mut hidden = Vec::new();
    for entry in entries.iter() {
        match entry.surfacing_heuristic.as_str() {
            "primary_core" => primary_core.push(entry.route_id.clone()),
            "secondary" => secondary
                .entry(entry.category.clone())
                .or_default()
                .push(entry.route_id.clone()),
            "contextual_deep" => contextual_deep.push(entry.route_id.clone()),
            _ => hidden.push(entry.route_id.clone()),
        }
    }
    let surfacing = CompiledSurfacingPlan {
        primary_core,
        secondary,
        contextual_deep,
        hidden,
    };

    let plan_hash = hash_json_hex(&json!({
        "schemaVersion": "1.0.0",
        "spaceId": context.space_id,
        "actorRole": actor_role,
        "intent": context.intent,
        "density": context.density,
        "entries": entries,
        "surfacing": surfacing,
    }));

    CompiledNavigationPlan {
        schema_version: "1.0.0".to_string(),
        generated_at: generated_at.to_string(),
        space_id: context.space_id.clone(),
        actor_role: context.actor_role.clone(),
        intent: context.intent.clone(),
        density: context.density.clone(),
        plan_hash,
        authz_engine: None,
        authz_mode: None,
        authz_decision_version: None,
        entries,
        surfacing,
    }
}

fn heuristic_weight(value: &SurfacingHeuristic) -> u32 {
    match value {
        SurfacingHeuristic::PrimaryCore => 400,
        SurfacingHeuristic::Secondary => 300,
        SurfacingHeuristic::ContextualDeep => 200,
        SurfacingHeuristic::Hidden => 100,
    }
}

fn frequency_weight(value: &OperationalFrequency) -> u32 {
    match value {
        OperationalFrequency::Continuous => 40,
        OperationalFrequency::Daily => 30,
        OperationalFrequency::AdHoc => 20,
        OperationalFrequency::Rare => 10,
    }
}

fn nav_band_for_heuristic(value: &SurfacingHeuristic) -> &'static str {
    match value {
        SurfacingHeuristic::PrimaryCore => "primary",
        SurfacingHeuristic::Secondary => "secondary",
        SurfacingHeuristic::ContextualDeep => "contextual",
        SurfacingHeuristic::Hidden => "contextual",
    }
}

fn surfacing_name(value: &SurfacingHeuristic) -> &'static str {
    match value {
        SurfacingHeuristic::PrimaryCore => "primary_core",
        SurfacingHeuristic::Secondary => "secondary",
        SurfacingHeuristic::ContextualDeep => "contextual_deep",
        SurfacingHeuristic::Hidden => "hidden",
    }
}

fn frequency_name(value: &OperationalFrequency) -> &'static str {
    match value {
        OperationalFrequency::Continuous => "continuous",
        OperationalFrequency::Daily => "daily",
        OperationalFrequency::AdHoc => "ad_hoc",
        OperationalFrequency::Rare => "rare",
    }
}

fn now_epoch_seconds() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs().to_string())
        .unwrap_or_else(|_| "0".to_string())
}

fn hash_json_hex<T: serde::Serialize>(value: &T) -> String {
    let encoded = serde_json::to_vec(value).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(encoded);
    hex::encode(hasher.finalize())
}

fn nav(
    route_id: &str,
    label: &str,
    icon: &str,
    category: &str,
    required_role: &str,
    nav_slot: Option<&str>,
    nav_meta: Option<NavigationEntryNavMeta>,
) -> NavigationEntrySpec {
    NavigationEntrySpec {
        route_id: route_id.to_string(),
        label: label.to_string(),
        icon: icon.to_string(),
        category: category.to_string(),
        required_role: required_role.to_string(),
        nav_slot: nav_slot.map(|value| value.to_string()),
        nav_meta,
    }
}

fn nav_meta(
    badge_count: Option<u32>,
    badge_tone: &str,
    attention: bool,
    attention_label: Option<&str>,
    collapsible_hint: &str,
) -> NavigationEntryNavMeta {
    NavigationEntryNavMeta {
        badge_count,
        badge_tone: Some(badge_tone.to_string()),
        attention: Some(attention),
        attention_label: attention_label.map(str::to_string),
        collapsible_hint: Some(collapsible_hint.to_string()),
    }
}

fn cap(
    route_id: &str,
    route_label: &str,
    view_capability_id: &str,
    pattern_id: &str,
    promotion_status: &str,
    required_role: &str,
    approval_required: bool,
    description: &str,
) -> ViewCapabilityManifest {
    ViewCapabilityManifest {
        route_id: route_id.to_string(),
        route_label: route_label.to_string(),
        view_capability_id: view_capability_id.to_string(),
        pattern_id: pattern_id.to_string(),
        promotion_status: promotion_status.to_string(),
        operator_critical: true,
        required_role: required_role.to_string(),
        approval_required,
        description: description.to_string(),
    }
}

pub fn compile_action_plan(
    _catalog: &PlatformCapabilityCatalog,
    _space_graph: &SpaceCapabilityGraph,
    _layout_spec: &ShellLayoutSpec,
    request: &CompiledActionPlanRequest,
) -> CompiledActionPlan {
    let generated_at = now_epoch_seconds();

    let mut page_bar_actions = vec![];
    let mut selection_bar_actions = vec![];
    let mut detail_footer_actions = vec![];
    let mut detail_header_actions = vec![];
    let mut card_menu_actions = vec![];
    let selection_count = request.selection.selected_count;
    let heap_create_enabled = request
        .feature_flags
        .as_ref()
        .and_then(|f| f.heap_create_flow_enabled)
        .unwrap_or(true);
    let heap_parity_enabled = request
        .feature_flags
        .as_ref()
        .and_then(|f| f.heap_parity_enabled)
        .unwrap_or(true);

    let disabled_by_parity = || {
        if heap_parity_enabled {
            None
        } else {
            Some("Heap parity features are disabled.".to_string())
        }
    };

    if heap_create_enabled && request.page_type == PageType::HeapBoard {
        page_bar_actions.push(ToolbarActionDescriptor {
            id: "action.heap.create".to_string(),
            capability_id: "cap.heap.create".to_string(),
            zone: SurfaceZone::HeapPageBar,
            label: "Create".to_string(),
            short_label: Some("Create".to_string()),
            icon: Some("plus".to_string()),
            kind: ToolbarActionKind::PanelToggle,
            action: "create_block".to_string(),
            priority: 100,
            group: ToolbarActionGroup::Primary,
            emphasis: Some(ToolbarActionEmphasis::Primary),
            visible: true,
            enabled: true,
            disabled_reason: None,
            selection_constraints: None,
            confirmation: None,
            steward_gate: None,
        });
    }

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.regenerate".to_string(),
        capability_id: "cap.heap.regenerate".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Regen".to_string(),
        short_label: Some("Regen".to_string()),
        icon: Some("refresh-cw".to_string()),
        kind: ToolbarActionKind::Mutation,
        action: "regenerate".to_string(),
        priority: 95,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.refine_selection".to_string(),
        capability_id: "cap.heap.refine".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Refine Selection".to_string(),
        short_label: Some("Refine".to_string()),
        icon: Some("wand-2".to_string()),
        kind: ToolbarActionKind::Command,
        action: "refine_selection".to_string(),
        priority: 90,
        group: ToolbarActionGroup::Selection,
        emphasis: None,
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count > 0,
        disabled_reason: disabled_by_parity(),
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: None,
            require_single_selection: None,
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.synthesize".to_string(),
        capability_id: "cap.heap.synthesize".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Synthesize".to_string(),
        short_label: None,
        icon: Some("sparkles".to_string()),
        kind: ToolbarActionKind::Command,
        action: "synthesize".to_string(),
        priority: 85,
        group: ToolbarActionGroup::Primary,
        emphasis: Some(ToolbarActionEmphasis::Accent),
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count >= 3,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count < 3 {
            Some("Select at least 3 blocks".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(3),
            max_selected: None,
            require_single_selection: None,
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.export".to_string(),
        capability_id: "cap.heap.export".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Export".to_string(),
        short_label: None,
        icon: Some("download".to_string()),
        kind: ToolbarActionKind::Download,
        action: "export".to_string(),
        priority: 50,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count > 0,
        disabled_reason: disabled_by_parity(),
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: None,
            require_single_selection: None,
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.history".to_string(),
        capability_id: "cap.heap.history".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "History".to_string(),
        short_label: None,
        icon: Some("history".to_string()),
        kind: ToolbarActionKind::Command,
        action: "history".to_string(),
        priority: 45,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.publish".to_string(),
        capability_id: "cap.heap.publish".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Publish".to_string(),
        short_label: None,
        icon: Some("upload".to_string()),
        kind: ToolbarActionKind::Mutation,
        action: "publish".to_string(),
        priority: 40,
        group: ToolbarActionGroup::Primary,
        emphasis: Some(ToolbarActionEmphasis::Primary),
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: Some(ToolbarActionStewardGate { required: true }),
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.pin".to_string(),
        capability_id: "cap.heap.pin".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Pin".to_string(),
        short_label: None,
        icon: Some("pin".to_string()),
        kind: ToolbarActionKind::Mutation,
        action: "pin".to_string(),
        priority: 30,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count > 0,
        disabled_reason: disabled_by_parity(),
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: None,
            require_single_selection: None,
        }),
        confirmation: None,
        steward_gate: None,
    });

    selection_bar_actions.push(ToolbarActionDescriptor {
        id: "action.heap.delete".to_string(),
        capability_id: "cap.heap.delete".to_string(),
        zone: SurfaceZone::HeapSelectionBar,
        label: "Delete".to_string(),
        short_label: None,
        icon: Some("trash-2".to_string()),
        kind: ToolbarActionKind::Destructive,
        action: "delete".to_string(),
        priority: 10,
        group: ToolbarActionGroup::Danger,
        emphasis: Some(ToolbarActionEmphasis::Danger),
        visible: selection_count > 0,
        enabled: heap_parity_enabled && selection_count > 0,
        disabled_reason: disabled_by_parity(),
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: None,
            require_single_selection: None,
        }),
        confirmation: Some(ToolbarActionConfirmation {
            required: true,
            style: Some(ConfirmationStyle::Danger),
            title: Some("Delete Blocks".to_string()),
            message: Some("Are you sure you want to delete the selected blocks?".to_string()),
        }),
        steward_gate: None,
    });

    let discussion_action = ToolbarActionDescriptor {
        id: "action.heap.detail.discussion".to_string(),
        capability_id: "cap.heap.discussion".to_string(),
        zone: SurfaceZone::HeapDetailFooter,
        label: "Discussion".to_string(),
        short_label: None,
        icon: Some("message-square".to_string()),
        kind: ToolbarActionKind::Navigation,
        action: "view_discussion".to_string(),
        priority: 90,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: true,
        enabled: true,
        disabled_reason: None,
        selection_constraints: None,
        confirmation: None,
        steward_gate: None,
    };

    let edit_action = ToolbarActionDescriptor {
        id: "action.heap.detail.edit".to_string(),
        capability_id: "cap.heap.edit".to_string(),
        zone: SurfaceZone::HeapDetailFooter,
        label: "Edit".to_string(),
        short_label: None,
        icon: Some("file-text".to_string()),
        kind: ToolbarActionKind::Command,
        action: "edit".to_string(),
        priority: 80,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: true,
        enabled: false,
        disabled_reason: Some("Edit flow is not implemented for heap detail yet.".to_string()),
        selection_constraints: None,
        confirmation: None,
        steward_gate: None,
    };

    let regenerate_action = ToolbarActionDescriptor {
        id: "action.heap.detail.regenerate".to_string(),
        capability_id: "cap.heap.regenerate".to_string(),
        zone: SurfaceZone::HeapDetailFooter,
        label: "Regenerate".to_string(),
        short_label: None,
        icon: Some("refresh-cw".to_string()),
        kind: ToolbarActionKind::Mutation,
        action: "regenerate".to_string(),
        priority: 70,
        group: ToolbarActionGroup::Primary,
        emphasis: Some(ToolbarActionEmphasis::Primary),
        visible: true,
        enabled: heap_parity_enabled,
        disabled_reason: disabled_by_parity(),
        selection_constraints: None,
        confirmation: None,
        steward_gate: None,
    };

    detail_footer_actions.push(discussion_action.clone());
    detail_footer_actions.push(edit_action.clone());
    detail_footer_actions.push(regenerate_action.clone());

    detail_header_actions.push(ToolbarActionDescriptor {
        zone: SurfaceZone::HeapDetailHeader,
        ..discussion_action.clone()
    });
    detail_header_actions.push(ToolbarActionDescriptor {
        zone: SurfaceZone::HeapDetailHeader,
        ..edit_action.clone()
    });
    detail_header_actions.push(ToolbarActionDescriptor {
        zone: SurfaceZone::HeapDetailHeader,
        ..regenerate_action.clone()
    });

    card_menu_actions.push(ToolbarActionDescriptor {
        id: "action.heap.card.discussion".to_string(),
        capability_id: "cap.heap.discussion".to_string(),
        zone: SurfaceZone::HeapCardMenu,
        label: "Discussion".to_string(),
        short_label: None,
        icon: Some("message-square".to_string()),
        kind: ToolbarActionKind::Navigation,
        action: "view_discussion".to_string(),
        priority: 80,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: true,
        enabled: true,
        disabled_reason: None,
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: None,
    });
    card_menu_actions.push(ToolbarActionDescriptor {
        id: "action.heap.card.history".to_string(),
        capability_id: "cap.heap.history".to_string(),
        zone: SurfaceZone::HeapCardMenu,
        label: "History".to_string(),
        short_label: None,
        icon: Some("history".to_string()),
        kind: ToolbarActionKind::Command,
        action: "history".to_string(),
        priority: 70,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: true,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: None,
    });
    card_menu_actions.push(ToolbarActionDescriptor {
        id: "action.heap.card.pin".to_string(),
        capability_id: "cap.heap.pin".to_string(),
        zone: SurfaceZone::HeapCardMenu,
        label: "Pin".to_string(),
        short_label: None,
        icon: Some("pin".to_string()),
        kind: ToolbarActionKind::Mutation,
        action: "pin".to_string(),
        priority: 50,
        group: ToolbarActionGroup::Secondary,
        emphasis: None,
        visible: true,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: None,
        steward_gate: None,
    });
    card_menu_actions.push(ToolbarActionDescriptor {
        id: "action.heap.card.delete".to_string(),
        capability_id: "cap.heap.delete".to_string(),
        zone: SurfaceZone::HeapCardMenu,
        label: "Delete".to_string(),
        short_label: None,
        icon: Some("trash-2".to_string()),
        kind: ToolbarActionKind::Destructive,
        action: "delete".to_string(),
        priority: 10,
        group: ToolbarActionGroup::Danger,
        emphasis: Some(ToolbarActionEmphasis::Danger),
        visible: true,
        enabled: heap_parity_enabled && selection_count == 1,
        disabled_reason: if !heap_parity_enabled {
            disabled_by_parity()
        } else if selection_count != 1 {
            Some("Requires exactly one selected block.".to_string())
        } else {
            None
        },
        selection_constraints: Some(ToolbarActionSelectionConstraints {
            min_selected: Some(1),
            max_selected: Some(1),
            require_single_selection: Some(true),
        }),
        confirmation: Some(ToolbarActionConfirmation {
            required: true,
            style: Some(ConfirmationStyle::Danger),
            title: Some("Delete Block".to_string()),
            message: Some("Are you sure you want to delete this block?".to_string()),
        }),
        steward_gate: None,
    });

    let requested_zones: std::collections::HashSet<SurfaceZone> =
        request.zones.iter().cloned().collect();
    let zones = vec![
        ActionZonePlan {
            zone: SurfaceZone::HeapPageBar,
            layout_hint: ActionZoneLayoutHint::Row,
            actions: page_bar_actions,
        },
        ActionZonePlan {
            zone: SurfaceZone::HeapSelectionBar,
            layout_hint: ActionZoneLayoutHint::Pillbar,
            actions: selection_bar_actions,
        },
        ActionZonePlan {
            zone: SurfaceZone::HeapDetailFooter,
            layout_hint: ActionZoneLayoutHint::Row,
            actions: detail_footer_actions,
        },
        ActionZonePlan {
            zone: SurfaceZone::HeapDetailHeader,
            layout_hint: ActionZoneLayoutHint::Pillbar,
            actions: detail_header_actions,
        },
        ActionZonePlan {
            zone: SurfaceZone::HeapCardMenu,
            layout_hint: ActionZoneLayoutHint::Row,
            actions: card_menu_actions,
        },
    ]
    .into_iter()
    .filter(|zone| requested_zones.contains(&zone.zone))
    .collect();

    CompiledActionPlan {
        schema_version: "1.0.0".to_string(),
        generated_at: generated_at.to_string(),
        plan_hash: format!("mock-hash-{}", request.space_id),
        space_id: request.space_id.clone(),
        route_id: request.route_id.clone(),
        page_type: request.page_type.clone(),
        actor_role: request.actor_role.clone(),
        zones,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compile_action_plan, compile_navigation_plan_with_generated_at, default_shell_layout_spec,
        default_view_capability_manifests, nav_meta, valid_nav_slot,
    };
    use crate::capabilities::navigation_graph::{
        CapabilityId, CapabilityNode, OperationalFrequency, PlatformCapabilityCatalog,
        SpaceCapabilityGraph, SpaceCapabilityNodeOverride, SurfacingHeuristic,
    };
    use crate::ux::types::{
        ActionFeatureFlags, ActionSelectionContext, CompilationContext, CompiledActionPlanRequest,
        NavigationEntrySpec, NavigationGraphSpec, PageType, ShellLayoutSpec, SurfaceZone,
    };
    use std::collections::HashSet;

    #[test]
    fn default_shell_layout_spec_contains_expanded_navigation_planes() {
        let routes: HashSet<String> = default_shell_layout_spec()
            .navigation_graph
            .entries
            .iter()
            .map(|entry| entry.route_id.clone())
            .collect();
        let required_routes = [
            "/spaces",
            "/heap",
            "/studio",
            "/workflows",
            "/contributions",
            "/labs",
            "/system",
            "/system/siq",
            "/testing",
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
            assert!(routes.contains(route), "missing default route: {route}");
        }
    }

    #[test]
    fn default_view_capability_manifest_covers_expanded_planes() {
        let route_set: HashSet<String> = default_view_capability_manifests()
            .iter()
            .map(|entry| entry.route_id.clone())
            .collect();
        let required_capability_routes = [
            "/spaces",
            "/heap",
            "/studio",
            "/workflows",
            "/contributions",
            "/labs",
            "/system",
            "/system/siq",
            "/testing",
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
        for route in required_capability_routes {
            assert!(
                route_set.contains(route),
                "missing view capability for route: {route}"
            );
        }
    }

    #[test]
    fn compile_navigation_plan_hash_is_deterministic_for_reordered_inputs() {
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            intent: Some("navigate".to_string()),
            density: Some("comfortable".to_string()),
        };

        let mut catalog_one = test_catalog();
        let mut catalog_two = test_catalog();
        catalog_two.nodes.reverse();
        catalog_one.edges.clear();
        catalog_two.edges.clear();

        let graph = test_space_graph();
        let first = compile_navigation_plan_with_generated_at(
            &catalog_one,
            &graph,
            &layout,
            &context,
            "1000",
        );
        let second = compile_navigation_plan_with_generated_at(
            &catalog_two,
            &graph,
            &layout,
            &context,
            "2000",
        );

        assert_eq!(first.plan_hash, second.plan_hash);
        assert_eq!(first.entries, second.entries);
    }

    #[test]
    fn compile_navigation_plan_applies_inactive_override() {
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            intent: None,
            density: None,
        };
        let mut graph = test_space_graph();
        graph.nodes.push(SpaceCapabilityNodeOverride {
            capability_id: CapabilityId("cap.logs".to_string()),
            local_alias: None,
            is_active: false,
            local_required_role: None,
            local_additional_required_claims: vec![],
            surfacing_heuristic: None,
            operational_frequency: None,
            placement_constraint: None,
        });

        let plan = compile_navigation_plan_with_generated_at(
            &test_catalog(),
            &graph,
            &layout,
            &context,
            "1000",
        );

        assert!(
            !plan.entries.iter().any(|entry| entry.route_id == "/logs"),
            "inactive override should remove /logs"
        );
    }

    #[test]
    fn compile_navigation_plan_clamps_override_to_platform_role_floor() {
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "viewer".to_string(),
            intent: None,
            density: None,
        };
        let mut graph = test_space_graph();
        graph.nodes.push(SpaceCapabilityNodeOverride {
            capability_id: CapabilityId("cap.logs".to_string()),
            local_alias: Some("Logs (local)".to_string()),
            is_active: true,
            local_required_role: Some("viewer".to_string()),
            local_additional_required_claims: vec![],
            surfacing_heuristic: None,
            operational_frequency: None,
            placement_constraint: None,
        });

        let plan = compile_navigation_plan_with_generated_at(
            &test_catalog(),
            &graph,
            &layout,
            &context,
            "1000",
        );

        assert!(
            !plan.entries.iter().any(|entry| entry.route_id == "/logs"),
            "viewer should not see /logs when local override weakens an operator floor"
        );
    }

    #[test]
    fn compile_navigation_plan_filters_by_role_before_placement() {
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "viewer".to_string(),
            intent: None,
            density: None,
        };
        let plan = compile_navigation_plan_with_generated_at(
            &test_catalog(),
            &test_space_graph(),
            &layout,
            &context,
            "1000",
        );
        assert!(
            !plan.entries.iter().any(|entry| entry.route_id == "/logs"),
            "viewer should not see operator lane /logs"
        );
    }

    #[test]
    fn compile_navigation_plan_keeps_contextual_deep_out_of_primary() {
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            intent: None,
            density: None,
        };
        let plan = compile_navigation_plan_with_generated_at(
            &test_catalog(),
            &test_space_graph(),
            &layout,
            &context,
            "1000",
        );
        assert!(plan.surfacing.primary_core.contains(&"/spaces".to_string()));
        assert!(
            plan.surfacing
                .contextual_deep
                .contains(&"/logs".to_string())
        );
        assert!(!plan.surfacing.primary_core.contains(&"/logs".to_string()));
    }

    #[test]
    fn default_shell_layout_spec_nav_slots_are_valid() {
        for entry in default_shell_layout_spec().navigation_graph.entries {
            if let Some(slot) = entry.nav_slot.as_deref() {
                assert!(
                    valid_nav_slot(slot),
                    "invalid nav_slot for route {}: {}",
                    entry.route_id,
                    slot
                );
            }
        }
    }

    #[test]
    fn compile_navigation_plan_prioritizes_primary_attention_slot() {
        let catalog = PlatformCapabilityCatalog {
            schema_version: "1.0.0".to_string(),
            catalog_version: "test-v1".to_string(),
            catalog_hash: Some("hash-test".to_string()),
            nodes: vec![
                test_node(
                    "cap.inbox",
                    "Inbox",
                    "/inbox",
                    "operator",
                    "core",
                    "IN",
                    SurfacingHeuristic::PrimaryCore,
                    OperationalFrequency::Continuous,
                ),
                test_node(
                    "cap.spaces",
                    "Spaces",
                    "/spaces",
                    "viewer",
                    "core",
                    "SP",
                    SurfacingHeuristic::PrimaryCore,
                    OperationalFrequency::Continuous,
                ),
            ],
            edges: vec![],
        };

        let layout = ShellLayoutSpec {
            layout_id: "layout.test".to_string(),
            navigation_graph: NavigationGraphSpec {
                entries: vec![
                    NavigationEntrySpec {
                        route_id: "/spaces".to_string(),
                        label: "Spaces".to_string(),
                        icon: "SP".to_string(),
                        category: "core".to_string(),
                        required_role: "viewer".to_string(),
                        nav_slot: None,
                        nav_meta: None,
                    },
                    NavigationEntrySpec {
                        route_id: "/inbox".to_string(),
                        label: "Inbox".to_string(),
                        icon: "IN".to_string(),
                        category: "core".to_string(),
                        required_role: "operator".to_string(),
                        nav_slot: None,
                        nav_meta: Some(nav_meta(
                            Some(1),
                            "critical",
                            true,
                            Some("Approvals"),
                            "expanded",
                        )),
                    },
                ],
            },
        };

        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            intent: None,
            density: None,
        };
        let plan = compile_navigation_plan_with_generated_at(
            &catalog,
            &test_space_graph(),
            &layout,
            &context,
            "1000",
        );
        assert_eq!(
            plan.entries.first().map(|e| e.route_id.as_str()),
            Some("/inbox")
        );
        assert_eq!(
            plan.entries.first().map(|e| e.nav_slot.as_str()),
            Some("primary_attention")
        );
    }

    #[test]
    fn compile_navigation_plan_is_deterministic_for_same_inputs() {
        let catalog = test_catalog();
        let layout = test_layout();
        let context = CompilationContext {
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            intent: Some("navigate".to_string()),
            density: Some("comfortable".to_string()),
        };

        let first = compile_navigation_plan_with_generated_at(
            &catalog,
            &test_space_graph(),
            &layout,
            &context,
            "1000",
        );
        let second = compile_navigation_plan_with_generated_at(
            &catalog,
            &test_space_graph(),
            &layout,
            &context,
            "1000",
        );
        assert_eq!(first.plan_hash, second.plan_hash);
        assert_eq!(first.entries, second.entries);
    }

    #[test]
    fn compile_action_plan_respects_requested_zones() {
        let request = test_action_plan_request(
            vec![SurfaceZone::HeapPageBar, SurfaceZone::HeapCardMenu],
            1,
        );

        let plan = compile_action_plan(
            &PlatformCapabilityCatalog::default(),
            &test_space_graph(),
            &default_shell_layout_spec(),
            &request,
        );

        let zone_set: HashSet<SurfaceZone> = plan.zones.iter().map(|zone| zone.zone.clone()).collect();
        assert_eq!(zone_set.len(), 2);
        assert!(zone_set.contains(&SurfaceZone::HeapPageBar));
        assert!(zone_set.contains(&SurfaceZone::HeapCardMenu));
        assert!(!zone_set.contains(&SurfaceZone::HeapSelectionBar));
    }

    #[test]
    fn compile_action_plan_exposes_card_menu_actions_for_single_selection() {
        let request = test_action_plan_request(vec![SurfaceZone::HeapCardMenu], 1);

        let plan = compile_action_plan(
            &PlatformCapabilityCatalog::default(),
            &test_space_graph(),
            &default_shell_layout_spec(),
            &request,
        );

        let card_zone = plan
            .zones
            .iter()
            .find(|zone| zone.zone == SurfaceZone::HeapCardMenu)
            .expect("card menu zone");
        let action_ids: Vec<String> = card_zone.actions.iter().map(|action| action.action.clone()).collect();
        assert_eq!(
            action_ids,
            vec![
                "view_discussion".to_string(),
                "history".to_string(),
                "pin".to_string(),
                "delete".to_string(),
            ]
        );
        assert!(card_zone.actions.iter().all(|action| action.visible));
        assert!(
            card_zone
                .actions
                .iter()
                .filter(|action| action.action != "view_discussion")
                .all(|action| action.enabled)
        );
    }

    #[test]
    fn compile_action_plan_disables_single_select_card_actions_for_multi_selection() {
        let request = test_action_plan_request(vec![SurfaceZone::HeapCardMenu], 2);

        let plan = compile_action_plan(
            &PlatformCapabilityCatalog::default(),
            &test_space_graph(),
            &default_shell_layout_spec(),
            &request,
        );

        let card_zone = plan
            .zones
            .iter()
            .find(|zone| zone.zone == SurfaceZone::HeapCardMenu)
            .expect("card menu zone");
        let history_action = card_zone
            .actions
            .iter()
            .find(|action| action.action == "history")
            .expect("history action");
        assert!(!history_action.enabled);
        assert_eq!(
            history_action.disabled_reason.as_deref(),
            Some("Requires exactly one selected block.")
        );
    }

    fn test_catalog() -> PlatformCapabilityCatalog {
        PlatformCapabilityCatalog {
            schema_version: "1.0.0".to_string(),
            catalog_version: "test-v1".to_string(),
            catalog_hash: Some("hash-test".to_string()),
            nodes: vec![
                test_node(
                    "cap.spaces",
                    "Spaces",
                    "/spaces",
                    "viewer",
                    "core",
                    "SP",
                    SurfacingHeuristic::PrimaryCore,
                    OperationalFrequency::Daily,
                ),
                test_node(
                    "cap.logs",
                    "Logs",
                    "/logs",
                    "operator",
                    "system",
                    "LG",
                    SurfacingHeuristic::ContextualDeep,
                    OperationalFrequency::Continuous,
                ),
            ],
            edges: vec![],
        }
    }

    fn test_node(
        id: &str,
        name: &str,
        route_id: &str,
        required_role: &str,
        category: &str,
        icon: &str,
        surfacing_heuristic: SurfacingHeuristic,
        operational_frequency: OperationalFrequency,
    ) -> CapabilityNode {
        CapabilityNode {
            id: CapabilityId(id.to_string()),
            resource_ref: None,
            name: name.to_string(),
            description: format!("{name} lane"),
            intent_type: Default::default(),
            route_id: Some(route_id.to_string()),
            category: Some(category.to_string()),
            required_role: Some(required_role.to_string()),
            required_claims: vec![],
            icon: Some(icon.to_string()),
            surfacing_heuristic,
            operational_frequency,
            domain_entities: vec![],
            placement_constraint: None,
            root_path: None,
            invariant_violations: vec![],
        }
    }

    fn test_space_graph() -> SpaceCapabilityGraph {
        SpaceCapabilityGraph {
            schema_version: "1.0.0".to_string(),
            space_id: "space.alpha".to_string(),
            base_catalog_version: "test-v1".to_string(),
            base_catalog_hash: "hash-test".to_string(),
            nodes: vec![],
            edges: vec![],
            updated_at: "1000".to_string(),
            updated_by: "steward".to_string(),
            lineage_ref: Some("decision:test".to_string()),
        }
    }

    fn test_action_plan_request(
        zones: Vec<SurfaceZone>,
        selected_count: u32,
    ) -> CompiledActionPlanRequest {
        CompiledActionPlanRequest {
            schema_version: "1.0.0".to_string(),
            space_id: "space.alpha".to_string(),
            actor_role: "operator".to_string(),
            route_id: "/heap".to_string(),
            page_type: PageType::HeapBoard,
            intent: Some("manage_heap".to_string()),
            density: Some("comfortable".to_string()),
            zones,
            selection: ActionSelectionContext {
                selected_artifact_ids: if selected_count > 0 {
                    vec!["artifact-1".to_string()]
                } else {
                    Vec::new()
                },
                active_artifact_id: if selected_count > 0 {
                    Some("artifact-1".to_string())
                } else {
                    None
                },
                selected_count,
                selected_block_types: Some(vec!["note".to_string()]),
            },
            active_filters: None,
            feature_flags: Some(ActionFeatureFlags {
                heap_create_flow_enabled: Some(true),
                heap_parity_enabled: Some(true),
            }),
        }
    }

    fn test_layout() -> ShellLayoutSpec {
        ShellLayoutSpec {
            layout_id: "layout.test".to_string(),
            navigation_graph: NavigationGraphSpec {
                entries: vec![
                    NavigationEntrySpec {
                        route_id: "/spaces".to_string(),
                        label: "Spaces".to_string(),
                        icon: "SP".to_string(),
                        category: "core".to_string(),
                        required_role: "viewer".to_string(),
                        nav_slot: None,
                        nav_meta: None,
                    },
                    NavigationEntrySpec {
                        route_id: "/logs".to_string(),
                        label: "Logs".to_string(),
                        icon: "LG".to_string(),
                        category: "system".to_string(),
                        required_role: "operator".to_string(),
                        nav_slot: None,
                        nav_meta: None,
                    },
                ],
            },
        }
    }
}
