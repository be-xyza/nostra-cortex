use crate::services::viewspec::{
    ComponentRef, LayoutEdge, LayoutGraph, LayoutNode, ViewSpecA11y, ViewSpecConfidence,
    ViewSpecLineage, ViewSpecProvenance, ViewSpecScope, ViewSpecV1,
    compile_viewspec_to_render_surface, default_viewspec_policy, now_iso,
};
use axum::{Json, extract::Query, http::HeaderMap, response::IntoResponse};
use chrono::Utc;
use cortex_ic_adapter::workflow::WorkflowEngineCanisterExecutionAdapter;
use cortex_domain::workflow::{
    WORKFLOW_INDEX_KEY, WorkflowCompileResult, WorkflowDefinitionV1, WorkflowDraftV1,
    WorkflowExecutionAdapterKind, WorkflowProposalEnvelope, WorkflowScope,
    WorkflowScopeAdoptionRecord, WorkflowSnapshotV1, scope_key as workflow_scope_key,
};
use cortex_runtime::{
    RuntimeError,
    ports::TimeProvider,
    workflow::{adapter::WorkflowExecutionAdapter, local_durable_worker::LocalDurableWorkerAdapter},
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;
use tokio::time::timeout;

#[derive(Deserialize)]
pub struct WorkbenchQuery {
    pub space_id: Option<String>,
    pub route: Option<String>,
    pub intent: Option<String>,
    pub density: Option<String>,
    pub node_id: Option<String>,
    pub run_id: Option<String>,
    pub contribution_id: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WorkbenchSurfaceKind {
    Labs,
    ExecutionCanvas,
    System,
    Siq,
    Testing,
    Logs,
    Agents,
    Contributions,
    Artifacts,
    Spaces,
    Flows,
    Initiatives,
    Studio,
    Heap,
    Synthesis,
    Generic,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct WorkbenchSurfaceRegistration {
    route_id: &'static str,
    kind: WorkbenchSurfaceKind,
    allow_generic_surface: bool,
}

const WORKBENCH_SURFACE_REGISTRY: &[WorkbenchSurfaceRegistration] = &[
    WorkbenchSurfaceRegistration {
        route_id: "/labs",
        kind: WorkbenchSurfaceKind::Labs,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/labs/execution-canvas",
        kind: WorkbenchSurfaceKind::ExecutionCanvas,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/system",
        kind: WorkbenchSurfaceKind::System,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/system/siq",
        kind: WorkbenchSurfaceKind::Siq,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/testing",
        kind: WorkbenchSurfaceKind::Testing,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/logs",
        kind: WorkbenchSurfaceKind::Logs,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/agents",
        kind: WorkbenchSurfaceKind::Agents,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/contributions",
        kind: WorkbenchSurfaceKind::Contributions,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/artifacts",
        kind: WorkbenchSurfaceKind::Artifacts,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/spaces",
        kind: WorkbenchSurfaceKind::Spaces,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/flows",
        kind: WorkbenchSurfaceKind::Flows,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/workflows",
        kind: WorkbenchSurfaceKind::Flows,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/initiatives",
        kind: WorkbenchSurfaceKind::Initiatives,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/system/contribution-graph",
        kind: WorkbenchSurfaceKind::Initiatives,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/studio",
        kind: WorkbenchSurfaceKind::Studio,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/heap",
        kind: WorkbenchSurfaceKind::Heap,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/synthesis",
        kind: WorkbenchSurfaceKind::Synthesis,
        allow_generic_surface: false,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/vfs",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/settings",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/inbox",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/discovery",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/metrics",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/memory",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
    WorkbenchSurfaceRegistration {
        route_id: "/simulation",
        kind: WorkbenchSurfaceKind::Generic,
        allow_generic_surface: true,
    },
];

fn registered_workbench_surface(route: &str) -> Option<&'static WorkbenchSurfaceRegistration> {
    WORKBENCH_SURFACE_REGISTRY
        .iter()
        .find(|entry| entry.route_id == route)
}

const WORKFLOW_DRAFT_INDEX_KEY: &str = "/cortex/workflows/drafts/current/index.json";
const WORKFLOW_PROPOSAL_INDEX_KEY: &str = "/cortex/workflows/drafts/proposals/index.json";
const WORKFLOW_ACTIVE_SCOPE_INDEX_KEY: &str = "/cortex/workflows/definitions/active/index.json";
const WORKFLOW_INSTANCE_INDEX_KEY: &str = "/cortex/workflows/instances/index.json";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkbenchWorkflowDraftIndexEntry {
    workflow_draft_id: String,
    scope_key: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkbenchWorkflowProposalIndexEntry {
    proposal_id: String,
    scope_key: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkbenchWorkflowInstanceIndexEntry {
    instance_id: String,
    adapter: WorkflowExecutionAdapterKind,
    scope_key: String,
    definition_id: String,
    updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct WorkbenchWorkflowDefinitionArtifact {
    definition: WorkflowDefinitionV1,
    compile_result: WorkflowCompileResult,
}

#[derive(Debug, Clone, PartialEq)]
struct WorkbenchWorkflowState {
    drafts: Vec<(WorkbenchWorkflowDraftIndexEntry, WorkflowDraftV1)>,
    proposals: Vec<(WorkbenchWorkflowProposalIndexEntry, WorkflowProposalEnvelope)>,
    definitions: Vec<(WorkbenchWorkflowDraftIndexEntry, WorkbenchWorkflowDefinitionArtifact)>,
    active_scopes: Vec<WorkflowScopeAdoptionRecord>,
    instances: Vec<WorkflowSnapshotV1>,
    degraded_messages: Vec<String>,
}

#[derive(Debug, Clone, Copy)]
struct WorkbenchWorkflowTimeProvider;

impl TimeProvider for WorkbenchWorkflowTimeProvider {
    fn now_unix_secs(&self) -> u64 {
        Utc::now().timestamp().max(0) as u64
    }

    fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError> {
        chrono::DateTime::<Utc>::from_timestamp(unix_secs as i64, 0)
            .map(|value| value.to_rfc3339())
            .ok_or_else(|| RuntimeError::Storage(format!("invalid unix seconds: {unix_secs}")))
    }
}

fn workspace_root() -> PathBuf {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    manifest_dir
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .map(Path::to_path_buf)
        .unwrap_or(manifest_dir)
}

fn cortex_ux_store_root() -> PathBuf {
    std::env::var("NOSTRA_CORTEX_UX_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("cortex").join("ux"))
}

fn decision_surface_log_dir() -> PathBuf {
    std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            workspace_root()
                .join("logs")
                .join("system")
                .join("decision_surfaces")
        })
}

fn workbench_temporal_runtime_root() -> PathBuf {
    decision_surface_log_dir().join("temporal_bridge_runtime")
}

fn read_json_file<T: DeserializeOwned>(path: &Path) -> Result<Option<T>, String> {
    match fs::read_to_string(path) {
        Ok(raw) => serde_json::from_str::<T>(&raw)
            .map(Some)
            .map_err(|err| err.to_string()),
        Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(err) => Err(err.to_string()),
    }
}

fn workflow_store_path(key: &str) -> PathBuf {
    cortex_ux_store_root().join(key.trim_start_matches('/'))
}

fn read_workflow_store_json<T: DeserializeOwned>(key: &str) -> Result<Option<T>, String> {
    read_json_file(workflow_store_path(key).as_path())
}

fn workflow_scope_prefix(space_id: &str) -> String {
    workflow_scope_key(&WorkflowScope {
        space_id: Some(space_id.to_string()),
        route_id: None,
        role: None,
    })
}

fn workflow_scope_matches(scope_key: &str, space_prefix: &str) -> bool {
    scope_key == space_prefix || scope_key.starts_with(&format!("{space_prefix}__"))
}

async fn workbench_workflow_adapter(
    adapter: WorkflowExecutionAdapterKind,
) -> Result<Box<dyn WorkflowExecutionAdapter>, String> {
    match adapter {
        WorkflowExecutionAdapterKind::LocalDurableWorkerV1 => Ok(Box::new(
            LocalDurableWorkerAdapter::new(
                workbench_temporal_runtime_root(),
                Arc::new(WorkbenchWorkflowTimeProvider),
            ),
        )),
        WorkflowExecutionAdapterKind::WorkflowEngineCanisterV1 => Ok(Box::new(
            WorkflowEngineCanisterExecutionAdapter::from_env()
                .await
                .map_err(|err| err.to_string())?,
        )),
    }
}

async fn load_workbench_workflow_instances(space_id: &str) -> Result<Vec<WorkflowSnapshotV1>, String> {
    let mut snapshots = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    let scope_prefix = workflow_scope_prefix(space_id);

    let registry = read_workflow_store_json::<BTreeMap<String, WorkbenchWorkflowInstanceIndexEntry>>(
        WORKFLOW_INSTANCE_INDEX_KEY,
    )?
    .unwrap_or_default();

    for entry in registry.values() {
        if !workflow_scope_matches(&entry.scope_key, &scope_prefix) {
            continue;
        }
        let adapter = workbench_workflow_adapter(entry.adapter.clone()).await?;
        let snapshot = adapter
            .snapshot(entry.instance_id.as_str())
            .await
            .map_err(|err| err.to_string())?;
        if snapshot.instance.scope.space_id.as_deref() == Some(space_id) {
            seen.insert(snapshot.instance.instance_id.clone());
            snapshots.push(snapshot);
        }
    }

    let instances_dir = workbench_temporal_runtime_root().join("instances");
    if instances_dir.exists() {
        let adapter = LocalDurableWorkerAdapter::new(
            workbench_temporal_runtime_root(),
            Arc::new(WorkbenchWorkflowTimeProvider),
        );
        for entry in fs::read_dir(instances_dir).map_err(|err| err.to_string())? {
            let entry = entry.map_err(|err| err.to_string())?;
            let path = entry.path();
            if path.extension().and_then(|value| value.to_str()) != Some("json") {
                continue;
            }
            let Some(instance_id) = path
                .file_stem()
                .and_then(|value| value.to_str())
                .map(|value| value.to_string()) else {
                continue;
            };
            if seen.contains(&instance_id) {
                continue;
            }
            let snapshot = adapter
                .snapshot(instance_id.as_str())
                .await
                .map_err(|err| err.to_string())?;
            if snapshot.instance.scope.space_id.as_deref() == Some(space_id) {
                snapshots.push(snapshot);
            }
        }
    }
    snapshots.sort_by(|left, right| right.instance.updated_at.cmp(&left.instance.updated_at));
    Ok(snapshots)
}

async fn load_workbench_workflow_state(space_id: &str) -> WorkbenchWorkflowState {
    let scope_prefix = workflow_scope_prefix(space_id);
    let mut degraded_messages = Vec::new();

    let draft_index = match read_workflow_store_json::<
        BTreeMap<String, WorkbenchWorkflowDraftIndexEntry>,
    >(WORKFLOW_DRAFT_INDEX_KEY)
    {
        Ok(Some(index)) => index,
        Ok(None) => BTreeMap::new(),
        Err(err) => {
            degraded_messages.push(format!("workflow_draft_index={err}"));
            BTreeMap::new()
        }
    };
    let mut drafts = Vec::new();
    for entry in draft_index.values() {
        if !workflow_scope_matches(&entry.scope_key, &scope_prefix) {
            continue;
        }
        let key = format!(
            "/cortex/workflows/drafts/current/{}/{}.json",
            entry.scope_key, entry.workflow_draft_id
        );
        match read_workflow_store_json::<WorkflowDraftV1>(key.as_str()) {
            Ok(Some(draft)) => drafts.push((entry.clone(), draft)),
            Ok(None) => degraded_messages.push(format!(
                "workflow_draft_missing={}:{}",
                entry.scope_key, entry.workflow_draft_id
            )),
            Err(err) => degraded_messages.push(format!(
                "workflow_draft_load={}:{}:{err}",
                entry.scope_key, entry.workflow_draft_id
            )),
        }
    }
    drafts.sort_by(|left, right| right.0.updated_at.cmp(&left.0.updated_at));

    let proposal_index = match read_workflow_store_json::<
        BTreeMap<String, WorkbenchWorkflowProposalIndexEntry>,
    >(WORKFLOW_PROPOSAL_INDEX_KEY)
    {
        Ok(Some(index)) => index,
        Ok(None) => BTreeMap::new(),
        Err(err) => {
            degraded_messages.push(format!("workflow_proposal_index={err}"));
            BTreeMap::new()
        }
    };
    let mut proposals = Vec::new();
    for entry in proposal_index.values() {
        if !workflow_scope_matches(&entry.scope_key, &scope_prefix) {
            continue;
        }
        let key = format!(
            "/cortex/workflows/drafts/proposals/{}/{}.json",
            entry.scope_key, entry.proposal_id
        );
        match read_workflow_store_json::<WorkflowProposalEnvelope>(key.as_str()) {
            Ok(Some(proposal)) => proposals.push((entry.clone(), proposal)),
            Ok(None) => degraded_messages.push(format!(
                "workflow_proposal_missing={}:{}",
                entry.scope_key, entry.proposal_id
            )),
            Err(err) => degraded_messages.push(format!(
                "workflow_proposal_load={}:{}:{err}",
                entry.scope_key, entry.proposal_id
            )),
        }
    }
    proposals.sort_by(|left, right| right.0.updated_at.cmp(&left.0.updated_at));

    let definition_index = match read_workflow_store_json::<
        BTreeMap<String, WorkbenchWorkflowDraftIndexEntry>,
    >(WORKFLOW_INDEX_KEY)
    {
        Ok(Some(index)) => index,
        Ok(None) => BTreeMap::new(),
        Err(err) => {
            degraded_messages.push(format!("workflow_definition_index={err}"));
            BTreeMap::new()
        }
    };
    let mut definitions = Vec::new();
    for entry in definition_index.values() {
        if !workflow_scope_matches(&entry.scope_key, &scope_prefix) {
            continue;
        }
        let key = format!(
            "/cortex/workflows/definitions/current/{}/{}.json",
            entry.scope_key, entry.workflow_draft_id
        );
        match read_workflow_store_json::<WorkbenchWorkflowDefinitionArtifact>(key.as_str()) {
            Ok(Some(definition)) => definitions.push((entry.clone(), definition)),
            Ok(None) => degraded_messages.push(format!(
                "workflow_definition_missing={}:{}",
                entry.scope_key, entry.workflow_draft_id
            )),
            Err(err) => degraded_messages.push(format!(
                "workflow_definition_load={}:{}:{err}",
                entry.scope_key, entry.workflow_draft_id
            )),
        }
    }
    definitions.sort_by(|left, right| right.0.updated_at.cmp(&left.0.updated_at));

    let active_scope_index = match read_workflow_store_json::<
        BTreeMap<String, WorkflowScopeAdoptionRecord>,
    >(WORKFLOW_ACTIVE_SCOPE_INDEX_KEY)
    {
        Ok(Some(index)) => index,
        Ok(None) => BTreeMap::new(),
        Err(err) => {
            degraded_messages.push(format!("workflow_active_scope_index={err}"));
            BTreeMap::new()
        }
    };
    let mut active_scopes = active_scope_index
        .into_values()
        .filter(|record| workflow_scope_matches(&record.scope_key, &scope_prefix))
        .collect::<Vec<_>>();
    active_scopes.sort_by(|left, right| right.adopted_at.cmp(&left.adopted_at));

    let instances = match load_workbench_workflow_instances(space_id).await {
        Ok(records) => records,
        Err(err) => {
            degraded_messages.push(format!("workflow_instances={err}"));
            Vec::new()
        }
    };

    WorkbenchWorkflowState {
        drafts,
        proposals,
        definitions,
        active_scopes,
        instances,
        degraded_messages,
    }
}

async fn render_registered_workbench_surface(
    registration: &WorkbenchSurfaceRegistration,
    _headers: &HeaderMap,
    space_id: &str,
    actor_id: &str,
    role: &str,
    node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    match registration.kind {
        WorkbenchSurfaceKind::Labs => generate_labs_directory_viewspec(),
        WorkbenchSurfaceKind::ExecutionCanvas => generate_execution_canvas_viewspec(space_id),
        WorkbenchSurfaceKind::System => generate_system_viewspec(node_id, intent, density),
        WorkbenchSurfaceKind::Siq => generate_siq_viewspec(space_id, intent, density).await,
        WorkbenchSurfaceKind::Testing => generate_testing_viewspec(space_id, intent, density).await,
        WorkbenchSurfaceKind::Logs => generate_logs_viewspec(role, node_id, intent, density).await,
        WorkbenchSurfaceKind::Agents => {
            generate_agents_viewspec(space_id, node_id, intent, density).await
        }
        WorkbenchSurfaceKind::Contributions => {
            generate_contributions_viewspec(space_id, node_id, intent, density).await
        }
        WorkbenchSurfaceKind::Artifacts => {
            generate_artifacts_viewspec(node_id, intent, density).await
        }
        WorkbenchSurfaceKind::Spaces => generate_spaces_viewspec(space_id, actor_id, role).await,
        WorkbenchSurfaceKind::Flows => {
            generate_flows_viewspec(registration.route_id, space_id, node_id, intent, density).await
        }
        WorkbenchSurfaceKind::Initiatives => generate_initiatives_viewspec(),
        WorkbenchSurfaceKind::Studio => generate_studio_viewspec(),
        WorkbenchSurfaceKind::Heap => generate_heap_viewspec(space_id),
        WorkbenchSurfaceKind::Synthesis => generate_synthesis_viewspec(),
        WorkbenchSurfaceKind::Generic => generate_generic_workbench_viewspec(registration.route_id),
    }
}

pub async fn get_workbench_ux_viewspec(
    headers: HeaderMap,
    Query(query): Query<WorkbenchQuery>,
) -> impl IntoResponse {
    let route = query.route.clone().unwrap_or_else(|| "/".to_string());
    let space_id = query
        .space_id
        .unwrap_or_else(|| "nostra-governance-v0".to_string());
    let intent = query.intent.unwrap_or_else(|| "navigate".to_string());
    let density = query.density.unwrap_or_else(|| "comfortable".to_string());
    let node_id = query.node_id.clone().or_else(|| {
        let route = query.route.as_deref().unwrap_or("/");
        if route != "/contributions" {
            return None;
        }
        query
            .run_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|run_id| format!("graph_run:{run_id}"))
            .or_else(|| {
                query
                    .contribution_id
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|contribution_id| format!("contribution:{contribution_id}"))
            })
    });

    let actor_id = headers
        .get("x-cortex-actor")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("anon")
        .to_string();

    let role = headers
        .get("x-cortex-role")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("operator")
        .to_string();

    let view_spec = match registered_workbench_surface(route.as_str()) {
        Some(registration) => {
            render_registered_workbench_surface(
                registration,
                &headers,
                &space_id,
                &actor_id,
                &role,
                node_id.as_deref(),
                &intent,
                &density,
            )
            .await
        }
        None => generate_generic_workbench_viewspec(&route),
    };

    match compile_viewspec_to_render_surface(&view_spec) {
        Ok(surface) => (StatusCode::OK, Json(surface)),
        Err(validation) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({
                "error": "viewspec_compilation_failed",
                "validation": validation
            })),
        ),
    }
}

fn generate_heap_viewspec(space_id: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![ComponentRef {
        component_id: "heap_canvas".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([(
            "widgetType".to_string(),
            Value::String("HeapCanvas".to_string()),
        )]),
        a11y: None,
        children: vec![],
    }];

    let layout_graph = LayoutGraph {
        nodes: vec![LayoutNode {
            node_id: "node_1".to_string(),
            role: "content".to_string(),
            component_ref_id: "heap_canvas".to_string(),
        }],
        edges: vec![],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "cortex_workbench_heap".to_string(),
        scope: ViewSpecScope {
            space_id: Some(space_id.to_string()),
            route_id: Some("/heap".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Navigate the infinitely expansive native spatial heap canvas.".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.99,
            rationale: "Deterministic heap canvas scaffolding".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_synthesis_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "synthesis_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Synthesis (Deprecated Surface)".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "synthesis_banner".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("AlertBanner".to_string()),
                ),
                (
                    "title".to_string(),
                    Value::String("Deprecated Surface".to_string()),
                ),
                ("severity".to_string(), Value::String("warning".to_string())),
                (
                    "message".to_string(),
                    Value::String(
                        "Synthesis is no longer a primary workbench route. Use Heap, Labs, or Studio."
                            .to_string(),
                    ),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "synthesis_open_heap".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::from([
                ("label".to_string(), Value::String("Open Heap".to_string())),
                ("href".to_string(), Value::String("/heap".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "synthesis_open_labs".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::from([
                ("label".to_string(), Value::String("Open Labs".to_string())),
                ("href".to_string(), Value::String("/labs".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "synthesis_open_studio".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::from([
                ("label".to_string(), Value::String("Open Studio".to_string())),
                ("href".to_string(), Value::String("/studio".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "synthesis_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "status".to_string(),
                component_ref_id: "synthesis_banner".to_string(),
            },
            LayoutNode {
                node_id: "node_3".to_string(),
                role: "actions".to_string(),
                component_ref_id: "synthesis_open_heap".to_string(),
            },
            LayoutNode {
                node_id: "node_4".to_string(),
                role: "actions".to_string(),
                component_ref_id: "synthesis_open_labs".to_string(),
            },
            LayoutNode {
                node_id: "node_5".to_string(),
                role: "actions".to_string(),
                component_ref_id: "synthesis_open_studio".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "node_1".to_string(),
                to: "node_2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_2".to_string(),
                to: "node_3".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_3".to_string(),
                to: "node_4".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_4".to_string(),
                to: "node_5".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "cortex_workbench_synthesis".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/synthesis".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Deprecated synthesis placeholder using A2UI primitives.".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.99,
            rationale: "Deterministic synthesis deprecation placeholder".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_labs_directory_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "labs_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Labs".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "labs_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Draft and compare execution-oriented surfaces here before they become durable operator tools."
                        .to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "space_studio_heading".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Space Studio".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "space_studio_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Draft a new space, test its shape, and decide later whether it should become a live space or a reusable template."
                        .to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "space_studio_open".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::from([
                (
                    "label".to_string(),
                    Value::String("Open Space Studio".to_string()),
                ),
                ("href".to_string(), Value::String("/labs/space-studio".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "execution_canvas_heading".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Execution Canvas".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "execution_canvas_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Prototype execution flows on a governed spatial canvas before they become workflow-backed projections."
                        .to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "execution_canvas_open".to_string(),
            component_type: "Button".to_string(),
            props: BTreeMap::from([
                (
                    "label".to_string(),
                    Value::String("Open Execution Canvas".to_string()),
                ),
                (
                    "href".to_string(),
                    Value::String("/labs/execution-canvas".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "labs_promotion".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Drafts stay in Labs until a steward promotes the pattern into a broader operating surface."
                        .to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "labs_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "content".to_string(),
                component_ref_id: "labs_desc".to_string(),
            },
            LayoutNode {
                node_id: "node_3".to_string(),
                role: "content".to_string(),
                component_ref_id: "space_studio_heading".to_string(),
            },
            LayoutNode {
                node_id: "node_4".to_string(),
                role: "content".to_string(),
                component_ref_id: "space_studio_desc".to_string(),
            },
            LayoutNode {
                node_id: "node_5".to_string(),
                role: "actions".to_string(),
                component_ref_id: "space_studio_open".to_string(),
            },
            LayoutNode {
                node_id: "node_6".to_string(),
                role: "content".to_string(),
                component_ref_id: "execution_canvas_heading".to_string(),
            },
            LayoutNode {
                node_id: "node_7".to_string(),
                role: "content".to_string(),
                component_ref_id: "execution_canvas_desc".to_string(),
            },
            LayoutNode {
                node_id: "node_8".to_string(),
                role: "actions".to_string(),
                component_ref_id: "execution_canvas_open".to_string(),
            },
            LayoutNode {
                node_id: "node_9".to_string(),
                role: "status".to_string(),
                component_ref_id: "labs_promotion".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "node_1".to_string(),
                to: "node_2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_2".to_string(),
                to: "node_3".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_3".to_string(),
                to: "node_4".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_4".to_string(),
                to: "node_5".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_5".to_string(),
                to: "node_6".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_6".to_string(),
                to: "node_7".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_7".to_string(),
                to: "node_8".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_8".to_string(),
                to: "node_9".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-labs".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/labs".to_string()),
            role: Some("viewer".to_string()),
        },
        intent: "Display UX Labs Directory".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.95,
            rationale: "Deterministic labs directory layout".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_execution_canvas_viewspec(space_id: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "execution_canvas_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Execution Canvas".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "execution_canvas_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(
                    "Labs-local execution authoring surface with governed spatial primitives and optional workflow lineage."
                        .to_string(),
                ),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "execution_canvas_plane".to_string(),
            component_type: "SpatialPlane".to_string(),
            props: BTreeMap::from([
                ("plane_id".to_string(), json!("labs-execution-canvas")),
                ("surface_class".to_string(), json!("execution")),
                (
                    "focus_bounds".to_string(),
                    json!({ "x": 0, "y": 0, "w": 1280, "h": 760 }),
                ),
                (
                    "view_state".to_string(),
                    json!({ "zoom": 1.0, "pan_x": 0, "pan_y": 0 }),
                ),
                (
                    "layout_ref".to_string(),
                    json!({
                        "space_id": space_id,
                        "view_spec_id": "workbench-labs-execution-canvas",
                    }),
                ),
                (
                    "commands".to_string(),
                    json!([
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "lane-labs",
                                "kind": "group",
                                "x": 40,
                                "y": 44,
                                "w": 1180,
                                "h": 460,
                                "label": "Labs flow",
                                "member_ids": ["node-input", "node-tool", "node-procedure", "node-output"],
                                "collapsed": false
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "annotation-1",
                                "kind": "annotation",
                                "x": 88,
                                "y": 530,
                                "w": 360,
                                "h": 92,
                                "text": "Use Labs-local canvases for topology experiments. Workflow-backed overlays become read-only topology surfaces."
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "node-input",
                                "kind": "node",
                                "node_class": "input",
                                "status": "done",
                                "x": 100,
                                "y": 138,
                                "w": 220,
                                "h": 132,
                                "text": "Intent",
                                "ports": [
                                    { "id": "out", "side": "right", "direction": "out", "label": "intent" }
                                ]
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "node-tool",
                                "kind": "node",
                                "node_class": "tool",
                                "status": "running",
                                "x": 392,
                                "y": 138,
                                "w": 248,
                                "h": 152,
                                "text": "Worker Tool",
                                "ports": [
                                    { "id": "in", "side": "left", "direction": "in", "label": "context" },
                                    { "id": "out", "side": "right", "direction": "out", "label": "result" }
                                ]
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "node-procedure",
                                "kind": "node",
                                "node_class": "procedure",
                                "status": "idle",
                                "x": 734,
                                "y": 130,
                                "w": 248,
                                "h": 164,
                                "text": "Procedure",
                                "ports": [
                                    { "id": "in", "side": "left", "direction": "in", "label": "input" },
                                    { "id": "ok", "side": "right", "direction": "out", "label": "success" },
                                    { "id": "branch", "side": "bottom", "direction": "out", "label": "branch" }
                                ]
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "node-output",
                                "kind": "node",
                                "node_class": "output",
                                "status": "blocked",
                                "x": 1028,
                                "y": 142,
                                "w": 170,
                                "h": 132,
                                "text": "Projection",
                                "ports": [
                                    { "id": "in", "side": "left", "direction": "in", "label": "surface" }
                                ]
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "edge-input-tool",
                                "kind": "edge",
                                "edge_class": "data",
                                "x": 320,
                                "y": 196,
                                "from_shape_id": "node-input",
                                "to_shape_id": "node-tool",
                                "from_port_id": "out",
                                "to_port_id": "in",
                                "text": "Context"
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "edge-tool-procedure",
                                "kind": "edge",
                                "edge_class": "control",
                                "x": 640,
                                "y": 204,
                                "from_shape_id": "node-tool",
                                "to_shape_id": "node-procedure",
                                "from_port_id": "out",
                                "to_port_id": "in",
                                "text": "Invoke"
                            }
                        },
                        {
                            "op": "create_shape",
                            "shape": {
                                "id": "edge-procedure-output",
                                "kind": "edge",
                                "edge_class": "branch",
                                "x": 976,
                                "y": 210,
                                "from_shape_id": "node-procedure",
                                "to_shape_id": "node-output",
                                "from_port_id": "ok",
                                "to_port_id": "in",
                                "text": "Success"
                            }
                        },
                        {
                            "op": "set_selection",
                            "shape_ids": ["node-tool"]
                        }
                    ]),
                ),
            ]),
            a11y: Some(ViewSpecA11y {
                label: Some("Execution Canvas".to_string()),
                ..ViewSpecA11y::default()
            }),
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "execution_canvas_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "content".to_string(),
                component_ref_id: "execution_canvas_desc".to_string(),
            },
            LayoutNode {
                node_id: "node_3".to_string(),
                role: "content".to_string(),
                component_ref_id: "execution_canvas_plane".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "node_1".to_string(),
                to: "node_2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "node_2".to_string(),
                to: "node_3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-labs-execution-canvas".to_string(),
        scope: ViewSpecScope {
            space_id: Some(space_id.to_string()),
            route_id: Some("/labs/execution-canvas".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Display a governed Labs execution canvas using canonical SpatialPlane primitives."
            .to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.98,
            rationale: "Deterministic Labs execution canvas scaffold".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn generate_system_viewspec(
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let inspector_text = selected_node_id
        .map(|node_id| {
            format!(
                "Selected node: {node_id}. intent={intent}; density={density}. Use the inspector to drill into route-bound capabilities."
            )
        })
        .unwrap_or_else(|| {
            format!(
                "Select a node on the canvas to inspect metadata. intent={intent}; density={density}."
            )
        });

    let component_refs = vec![
        ComponentRef {
            component_id: "sys_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("System Capability Graph".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "sys_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Visualizing dynamic Nostra Core runtime capabilities.".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "capability_map".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("CapabilityMap".to_string()),
                ),
                (
                    "dataSourceUrl".to_string(),
                    Value::String("/api/system/capability-graph".to_string()),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "inspector_heading".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Inspector".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "inspector_text".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String(inspector_text))]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "sys_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "sys_desc".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "visualization".to_string(),
                component_ref_id: "capability_map".to_string(),
            },
            LayoutNode {
                node_id: "4".to_string(),
                role: "sidebar".to_string(),
                component_ref_id: "inspector_heading".to_string(),
            },
            LayoutNode {
                node_id: "5".to_string(),
                role: "sidebar".to_string(),
                component_ref_id: "inspector_text".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "3".to_string(),
                to: "4".to_string(),
                relation: "inspector_panel".to_string(),
            },
            LayoutEdge {
                from: "4".to_string(),
                to: "5".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-system".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/system".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "System Capabilities Insight".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Precise system mapping layer".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

async fn generate_spaces_viewspec(space_id: &str, actor_id: &str, role: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    // 1. Context Hydration: Query the KG via DpubWorkbenchService
    let overview_result =
        crate::services::dpub_workbench_service::DpubWorkbenchService::get_overview(space_id).await;
    let ready_result =
        crate::services::dpub_workbench_service::DpubWorkbenchService::get_system_ready().await;

    // 2. Intent Formulation / A2UI Schema Synthesis
    let mut component_refs = vec![ComponentRef {
        component_id: "spaces_title".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([(
            "text".to_string(),
            Value::String(format!("Organizational Space: {}", space_id)),
        )]),
        a11y: None,
        children: vec![],
    }];

    // Space Creation Wizard widget (drives the frontend wizard component)
    // Space creation is now a sequenced pure A2UI form
    component_refs.push(ComponentRef {
        component_id: "space_creation_wizard".to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([("widgetType".to_string(), Value::String("Card".to_string()))]),
        a11y: None,
        children: vec![
            "wizard_title".to_string(),
            "wizard_input_name".to_string(),
            "wizard_submit".to_string(),
        ],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_title".to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([(
            "text".to_string(),
            Value::String("Create New Space".to_string()),
        )]),
        a11y: None,
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_input_name".to_string(),
        component_type: "TextField".to_string(),
        props: BTreeMap::from([(
            "label".to_string(),
            Value::String("Space Identifier".to_string()),
        )]),
        a11y: Some(ViewSpecA11y {
            label: Some("Space Identifier Input".to_string()),
            description: None,
            role: None,
            live: None,
            required: None,
            invalid: None,
        }),
        children: vec![],
    });

    component_refs.push(ComponentRef {
        component_id: "wizard_submit".to_string(),
        component_type: "Button".to_string(),
        props: BTreeMap::from([
            (
                "label".to_string(),
                Value::String("Provision Space".to_string()),
            ),
            (
                "action".to_string(),
                Value::String("provisionSpace".to_string()),
            ),
        ]),
        a11y: Some(ViewSpecA11y {
            label: Some("Provision Space Button".to_string()),
            description: None,
            role: Some("button".to_string()),
            live: None,
            required: None,
            invalid: None,
        }),
        children: vec![],
    });

    let mut nodes = vec![
        LayoutNode {
            node_id: "1".to_string(),
            role: "header".to_string(),
            component_ref_id: "spaces_title".to_string(),
        },
        LayoutNode {
            node_id: "wizard".to_string(),
            role: "content".to_string(),
            component_ref_id: "space_creation_wizard".to_string(),
        },
    ];
    let mut edges = vec![LayoutEdge {
        from: "1".to_string(),
        to: "wizard".to_string(),
        relation: "flows_to".to_string(),
    }];
    let mut current_node_idx = 2;

    // Build Health Status component
    let health_c_id = "spaces_health_crdt".to_string();
    let (health_title, health_sev, health_msg) = match ready_result {
        Ok(ready) => {
            let title = if ready.ready {
                "System Operational"
            } else {
                "System Degraded"
            };
            let sev = if ready.ready { "success" } else { "error" };
            let msg = format!(
                "ICP Local Network: {} (CRDT Stream Connected)",
                if ready.icp_network_healthy {
                    "Healthy"
                } else {
                    "Unhealthy"
                }
            );
            (title.to_string(), sev.to_string(), msg)
        }
        Err(_) => {
            let title = "System Unknown".to_string();
            let sev = "warning".to_string();
            let msg = format!(
                "Temporal integrity verified. The operational graph for `{}` is healthy and determinism checks have passed.",
                space_id
            );
            (title, sev, msg)
        }
    };

    let health_props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("AlertBanner".to_string()),
        ),
        ("title".to_string(), Value::String(health_title.to_string())),
        (
            "severity".to_string(),
            Value::String(health_sev.to_string()),
        ),
        ("message".to_string(), Value::String(health_msg)),
        (
            "crdtDocument".to_string(),
            Value::String(format!("spaces:{}", space_id)),
        ),
        ("crdtSubscribe".to_string(), Value::Bool(true)),
    ]);

    component_refs.push(ComponentRef {
        component_id: health_c_id.clone(),
        component_type: "Container".to_string(),
        props: health_props,
        a11y: None,
        children: vec![],
    });

    current_node_idx += 1;
    let health_node_id = current_node_idx.to_string();
    nodes.push(LayoutNode {
        node_id: health_node_id.clone(),
        role: "status".to_string(),
        component_ref_id: health_c_id,
    });
    edges.push(LayoutEdge {
        from: (current_node_idx - 1).to_string(),
        to: health_node_id.clone(),
        relation: "flows_to".to_string(),
    });

    match overview_result {
        Ok(Value::Object(map)) => {
            // Found data. Synthesize real, interactive A2UI primitives for each metric.
            let mut grid_children = vec![];

            for (key, value) in map {
                let card_id = format!("metric_{}", key.replace(" ", "_").to_lowercase());
                if let Value::Object(inner) = value {
                    // Object values become detail cards (HeapBlockCard)
                    component_refs.push(ComponentRef {
                        component_id: card_id.clone(),
                        component_type: "Container".to_string(),
                        props: BTreeMap::from([
                            (
                                "widgetType".to_string(),
                                Value::String("HeapBlockCard".to_string()),
                            ),
                            ("title".to_string(), Value::String(key.clone())),
                            ("attributes".to_string(), Value::Object(inner)),
                        ]),
                        a11y: None,
                        children: vec![],
                    });
                } else {
                    // Scalar values become MetricCards
                    let val_str = match value {
                        Value::Array(arr) => format!("{} items", arr.len()),
                        Value::String(s) => s,
                        Value::Number(n) => n.to_string(),
                        Value::Bool(b) => b.to_string(),
                        _ => "Unknown".to_string(),
                    };
                    component_refs.push(ComponentRef {
                        component_id: card_id.clone(),
                        component_type: "Container".to_string(),
                        props: BTreeMap::from([
                            (
                                "widgetType".to_string(),
                                Value::String("MetricCard".to_string()),
                            ),
                            ("label".to_string(), Value::String(key.clone())),
                            ("value".to_string(), Value::String(val_str)),
                        ]),
                        a11y: None,
                        children: vec![],
                    });
                }
                grid_children.push(card_id);
            }

            // Group all the metrics into a single A2UI Container/Grid
            let grid_c_id = "spaces_overview_grid".to_string();
            component_refs.push(ComponentRef {
                component_id: grid_c_id.clone(),
                component_type: "Container".to_string(),
                props: BTreeMap::new(), // The frontend will lay out these children (e.g., using masonry layout class if we pass one)
                a11y: None,
                children: grid_children,
            });

            current_node_idx += 1;
            let grid_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: grid_node_id.clone(),
                role: "content".to_string(),
                component_ref_id: grid_c_id,
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: grid_node_id.clone(),
                relation: "flows_to".to_string(),
            });

            // Interactive Action Controls
            let action_c_id = "spaces_action_run".to_string();
            component_refs.push(ComponentRef {
                component_id: action_c_id.clone(),
                component_type: "Button".to_string(),
                props: BTreeMap::from([
                    (
                        "label".to_string(),
                        Value::String("Trigger Agent Initiative".to_string()),
                    ),
                    (
                        "action".to_string(),
                        Value::String(format!("startAgentInitiative?spaceId={}", space_id)),
                    ),
                ]),
                a11y: Some(ViewSpecA11y {
                    label: Some("Trigger Agent Initiative button".to_string()),
                    description: None,
                    role: None,
                    live: None,
                    required: None,
                    invalid: None,
                }),
                children: vec![],
            });

            current_node_idx += 1;
            let action_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: action_node_id.clone(),
                role: "actions".to_string(),
                component_ref_id: action_c_id,
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: action_node_id.clone(),
                relation: "flows_to".to_string(),
            });

            // 2C: Contextual Navigation Menu
            let nav_c_id = "spaces_nav_tabs".to_string();
            component_refs.push(ComponentRef {
                component_id: nav_c_id.clone(),
                component_type: "Tabs".to_string(),
                props: BTreeMap::from([(
                    "tabItems".to_string(),
                    Value::Array(vec![
                        json!({"title": "Active Flows", "child": "flows_placeholder"}),
                        json!({"title": "Initiative Graph", "child": "graph_placeholder"}),
                    ]),
                )]),
                a11y: Some(ViewSpecA11y {
                    label: Some("Spaces Navigation Tabs".to_string()),
                    description: None,
                    role: None,
                    live: None,
                    required: None,
                    invalid: None,
                }),
                children: vec![],
            });

            current_node_idx += 1;
            let nav_node_id = current_node_idx.to_string();
            nodes.push(LayoutNode {
                node_id: nav_node_id.clone(),
                role: "navigation".to_string(),
                component_ref_id: nav_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: (current_node_idx - 1).to_string(),
                to: nav_node_id.clone(),
                relation: "flows_to".to_string(),
            });
        }
        Ok(_) => {
            // Fallback for empty overview
            let m_c_id = "spaces_msg".to_string();
            component_refs.push(ComponentRef {
                component_id: m_c_id.clone(),
                component_type: "Text".to_string(),
                props: BTreeMap::from([(
                    "text".to_string(),
                    Value::String("No overview metrics found.".to_string()),
                )]),
                a11y: None,
                children: vec![],
            });
            nodes.push(LayoutNode {
                node_id: "msg".to_string(),
                role: "content".to_string(),
                component_ref_id: m_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: "1".to_string(),
                to: "msg".to_string(),
                relation: "flows_to".to_string(),
            });
        }
        Err(e) => {
            let error_c_id = "spaces_error".to_string();
            component_refs.push(ComponentRef {
                component_id: error_c_id.clone(),
                component_type: "Text".to_string(),
                props: BTreeMap::from([(
                    "text".to_string(),
                    Value::String(format!("Failed to hydrate space {}: {}", space_id, e)),
                )]),
                a11y: None,
                children: vec![],
            });
            nodes.push(LayoutNode {
                node_id: "err".to_string(),
                role: "content".to_string(),
                component_ref_id: error_c_id.clone(),
            });
            edges.push(LayoutEdge {
                from: "1".to_string(),
                to: "err".to_string(),
                relation: "flows_to".to_string(),
            });
        }
    }

    let layout_graph = LayoutGraph { nodes, edges };

    let mut spec = ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: format!("spaces:{}:{}", space_id, actor_id),
        scope: ViewSpecScope {
            space_id: Some(space_id.to_string()),
            route_id: Some("/spaces".to_string()),
            role: Some(role.to_string()),
        },
        intent: format!("Interactive settings and context for space {}", space_id),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.98,
            rationale: "Graph-hydrated localized context".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    };

    mutate_space_a2ui_viewspec(&mut spec, space_id);
    spec
}

/// Agent-driven injection hook used by the Space Auditor to apply transient
/// priority banners and ViewSpec mutations based on SIQ health metrics.
/// In production, this reads pending `ViewSpecProposalEnvelope` records from
/// the proposals store and applies only those with `Ratified` status.
fn mutate_space_a2ui_viewspec(spec: &mut ViewSpecV1, space_id: &str) {
    let registry_path = crate::gateway::server::workspace_root()
        .join("_spaces")
        .join("registry.json");
    if let Ok(registry) = cortex_domain::spaces::SpaceRegistry::load_from_path(&registry_path) {
        if let Some(record) = registry.get(space_id) {
            if record.status == cortex_domain::spaces::SpaceStatus::Quarantine {
                let alert_id = format!("auditor_quarantine_{}", space_id);
                spec.component_refs.push(ComponentRef {
                    component_id: alert_id.clone(),
                    component_type: "Container".to_string(),
                    props: BTreeMap::from([
                        ("widgetType".to_string(), Value::String("AlertBanner".to_string())),
                        ("title".to_string(), Value::String("Space Quarantined".to_string())),
                        ("severity".to_string(), Value::String("warning".to_string())),
                        ("message".to_string(), Value::String(
                            format!("Space '{}' is in quarantine pending import validation. Agent proposals require ratification.", space_id),
                        )),
                    ]),
                    a11y: None,
                    children: vec![],
                });
                spec.layout_graph.nodes.push(LayoutNode {
                    node_id: "agent_quarantine_node".to_string(),
                    role: "alert".to_string(),
                    component_ref_id: alert_id,
                });
            }
        }
    }
}

fn generate_initiatives_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "initiatives_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String("Strategic Initiatives".to_string()),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "initiatives_metrics_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
            a11y: None,
            children: vec![
                "init_stat_active".to_string(),
                "init_stat_planned".to_string(),
            ],
        },
        ComponentRef {
            component_id: "init_stat_active".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Active".to_string())),
                ("value".to_string(), Value::String("4".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "init_stat_planned".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("MetricCard".to_string()),
                ),
                ("label".to_string(), Value::String("Planned".to_string())),
                ("value".to_string(), Value::String("12".to_string())),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "initiatives_grid".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
            a11y: None,
            children: vec!["init_card_1".to_string(), "init_card_2".to_string()],
        },
        ComponentRef {
            component_id: "init_card_1".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("HeapBlockCard".to_string()),
                ),
                (
                    "title".to_string(),
                    Value::String("Deploy Nexus Core".to_string()),
                ),
                ("status".to_string(), Value::String("Active".to_string())),
                (
                    "attributes".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![
                        (
                            "layer".to_string(),
                            Value::String("infrastructure".to_string()),
                        ),
                        ("role".to_string(), Value::String("critical".to_string())),
                    ])),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "init_card_2".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                (
                    "widgetType".to_string(),
                    Value::String("HeapBlockCard".to_string()),
                ),
                (
                    "title".to_string(),
                    Value::String("Refactor Auth".to_string()),
                ),
                ("status".to_string(), Value::String("Planned".to_string())),
                (
                    "attributes".to_string(),
                    Value::Object(serde_json::Map::from_iter(vec![(
                        "layer".to_string(),
                        Value::String("application".to_string()),
                    )])),
                ),
            ]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "initiatives_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "initiatives_metrics_grid".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "content".to_string(),
                component_ref_id: "initiatives_grid".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-initiatives".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/initiatives".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Initiative Tracking".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Data directory".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}
fn generate_studio_viewspec() -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());

    let component_refs = vec![
        ComponentRef {
            component_id: "studio_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Studio Canvas".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "studio_desc".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([("text".to_string(), Value::String("Agentic Code Generation & A2UI Live Editing".to_string()))]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "markdown_card".to_string(),
            component_type: "Card".to_string(),
            props: BTreeMap::new(),
            a11y: None,
            children: vec!["code_block".to_string()],
        },
        ComponentRef {
            component_id: "code_block".to_string(),
            component_type: "Markdown".to_string(),
            props: BTreeMap::from([("content".to_string(), Value::String("```rust\n// Welcome to Cortex Studio\n\nfn main() {\n    println!(\"A2UI Live Editor Online\");\n}\n```".to_string()))]),
            a11y: None,
            children: vec![],
        }
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "1".to_string(),
                role: "header".to_string(),
                component_ref_id: "studio_title".to_string(),
            },
            LayoutNode {
                node_id: "2".to_string(),
                role: "content".to_string(),
                component_ref_id: "studio_desc".to_string(),
            },
            LayoutNode {
                node_id: "3".to_string(),
                role: "content".to_string(),
                component_ref_id: "markdown_card".to_string(),
            },
        ],
        edges: vec![
            LayoutEdge {
                from: "1".to_string(),
                to: "2".to_string(),
                relation: "flows_to".to_string(),
            },
            LayoutEdge {
                from: "2".to_string(),
                to: "3".to_string(),
                relation: "flows_to".to_string(),
            },
        ],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-studio".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/studio".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Authoring Environment".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 1.0,
            rationale: "Studio Layer".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn heading_component(id: &str, text: String) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Heading".to_string(),
        props: BTreeMap::from([("text".to_string(), Value::String(text.clone()))]),
        a11y: Some(ViewSpecA11y {
            label: Some(text),
            ..ViewSpecA11y::default()
        }),
        children: vec![],
    }
}

fn text_component(id: &str, text: String) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Text".to_string(),
        props: BTreeMap::from([("text".to_string(), Value::String(text))]),
        a11y: None,
        children: vec![],
    }
}

fn button_component(
    id: &str,
    label: &str,
    action: Option<String>,
    href: Option<String>,
    a11y_label: Option<String>,
) -> ComponentRef {
    let mut props = BTreeMap::from([("label".to_string(), Value::String(label.to_string()))]);
    if let Some(action) = action {
        props.insert("action".to_string(), Value::String(action));
    }
    if let Some(href) = href {
        props.insert("href".to_string(), Value::String(href));
    }
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Button".to_string(),
        props,
        a11y: Some(ViewSpecA11y {
            label: Some(a11y_label.unwrap_or_else(|| label.to_string())),
            description: None,
            role: Some("button".to_string()),
            live: None,
            required: None,
            invalid: None,
        }),
        children: vec![],
    }
}

fn metric_card_component(
    id: &str,
    label: &str,
    value: String,
    trend: Option<String>,
) -> ComponentRef {
    let mut props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("MetricCard".to_string()),
        ),
        ("label".to_string(), Value::String(label.to_string())),
        ("value".to_string(), Value::String(value.clone())),
    ]);
    if let Some(trend) = trend {
        props.insert("trend".to_string(), Value::String(trend));
    }
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Card".to_string(),
        props,
        a11y: Some(ViewSpecA11y {
            label: Some(format!("Metric: {} = {}", label, value)),
            ..ViewSpecA11y::default()
        }),
        children: vec![],
    }
}

fn data_table_component(id: &str, columns: Vec<&str>, rows: Vec<Value>) -> ComponentRef {
    data_table_component_with_options(id, columns, rows, None)
}

fn data_table_component_with_options(
    id: &str,
    columns: Vec<&str>,
    rows: Vec<Value>,
    hidden_columns: Option<Vec<&str>>,
) -> ComponentRef {
    let mut props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("DataTable".to_string()),
        ),
        (
            "columns".to_string(),
            Value::Array(
                columns
                    .into_iter()
                    .map(|column| Value::String(column.to_string()))
                    .collect(),
            ),
        ),
        ("rows".to_string(), Value::Array(rows)),
    ]);
    if let Some(hidden_columns) = hidden_columns {
        props.insert(
            "hiddenColumns".to_string(),
            Value::Array(
                hidden_columns
                    .into_iter()
                    .map(|column| Value::String(column.to_string()))
                    .collect(),
            ),
        );
    }
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props,
        a11y: Some(ViewSpecA11y {
            label: Some(format!("Data table: {}", id)),
            ..ViewSpecA11y::default()
        }),
        children: vec![],
    }
}

fn alert_banner_component(id: &str, title: &str, severity: &str, message: String) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([
            (
                "widgetType".to_string(),
                Value::String("AlertBanner".to_string()),
            ),
            ("title".to_string(), Value::String(title.to_string())),
            ("severity".to_string(), Value::String(severity.to_string())),
            ("message".to_string(), Value::String(message)),
        ]),
        a11y: Some(ViewSpecA11y {
            label: Some(format!("Alert: {}", title)),
            ..ViewSpecA11y::default()
        }),
        children: vec![],
    }
}

fn grid_component(id: &str, children: Vec<&str>) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([("widgetType".to_string(), Value::String("Grid".to_string()))]),
        a11y: None,
        children: children
            .into_iter()
            .map(|value| value.to_string())
            .collect(),
    }
}

fn workflow_summary_strip_component(
    id: &str,
    eyebrow: &str,
    title: &str,
    description: String,
    metrics: Vec<Value>,
) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([
            (
                "widgetType".to_string(),
                Value::String("WorkflowSummaryStrip".to_string()),
            ),
            ("eyebrow".to_string(), Value::String(eyebrow.to_string())),
            ("title".to_string(), Value::String(title.to_string())),
            ("description".to_string(), Value::String(description)),
            ("metrics".to_string(), Value::Array(metrics)),
        ]),
        a11y: None,
        children: vec![],
    }
}

fn workflow_status_badge_component(
    id: &str,
    label: &str,
    status: String,
    emphasis: &str,
    href: Option<String>,
) -> ComponentRef {
    let mut props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("WorkflowStatusBadge".to_string()),
        ),
        ("label".to_string(), Value::String(label.to_string())),
        ("status".to_string(), Value::String(status)),
        ("emphasis".to_string(), Value::String(emphasis.to_string())),
    ]);
    if let Some(href) = href {
        props.insert("href".to_string(), Value::String(href));
    }
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props,
        a11y: None,
        children: vec![],
    }
}

fn workflow_projection_preview_component(
    id: &str,
    eyebrow: &str,
    definition_id: String,
    definition_href: Option<String>,
    motif: String,
    digest: String,
    node_count: usize,
    projections: Vec<Value>,
) -> ComponentRef {
    let mut props = BTreeMap::from([
        (
            "widgetType".to_string(),
            Value::String("WorkflowProjectionPreview".to_string()),
        ),
        ("eyebrow".to_string(), Value::String(eyebrow.to_string())),
        ("definitionId".to_string(), Value::String(definition_id)),
        ("motif".to_string(), Value::String(motif)),
        ("digest".to_string(), Value::String(digest)),
        ("nodeCount".to_string(), Value::String(node_count.to_string())),
        ("projections".to_string(), Value::Array(projections)),
    ]);
    if let Some(definition_href) = definition_href {
        props.insert(
            "definitionHref".to_string(),
            Value::String(definition_href),
        );
    }
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props,
        a11y: None,
        children: vec![],
    }
}

fn workflow_instance_timeline_component(
    id: &str,
    eyebrow: &str,
    title: &str,
    entries: Vec<Value>,
) -> ComponentRef {
    ComponentRef {
        component_id: id.to_string(),
        component_type: "Container".to_string(),
        props: BTreeMap::from([
            (
                "widgetType".to_string(),
                Value::String("WorkflowInstanceTimeline".to_string()),
            ),
            ("eyebrow".to_string(), Value::String(eyebrow.to_string())),
            ("title".to_string(), Value::String(title.to_string())),
            ("entries".to_string(), Value::Array(entries)),
        ]),
        a11y: None,
        children: vec![],
    }
}

fn linear_layout(component_refs: &[ComponentRef]) -> LayoutGraph {
    let ids: Vec<String> = component_refs.iter().map(|c| c.component_id.clone()).collect();
    linear_layout_from_ids(ids)
}

fn linear_layout_from_ids(ids: Vec<String>) -> LayoutGraph {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    for (index, id) in ids.iter().enumerate() {
        let node_id = format!("node_{}", index + 1);
        let role = if index == 0 {
            "header"
        } else {
            "content"
        };
        nodes.push(LayoutNode {
            node_id: node_id.clone(),
            role: role.to_string(),
            component_ref_id: id.clone(),
        });
        if index > 0 {
            edges.push(LayoutEdge {
                from: format!("node_{}", index),
                to: node_id,
                relation: "flows_to".to_string(),
            });
        }
    }
    LayoutGraph { nodes, edges }
}

fn make_workbench_viewspec(
    view_spec_id: &str,
    route: &str,
    role: &str,
    intent_text: String,
    style_tokens: BTreeMap<String, String>,
    component_refs: Vec<ComponentRef>,
) -> ViewSpecV1 {
    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: view_spec_id.to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some(route.to_string()),
            role: Some(role.to_string()),
        },
        intent: intent_text,
        constraints: vec![],
        layout_graph: linear_layout(&component_refs),
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.95,
            rationale: "Live operational workbench projection".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

fn parse_log_node_id(node_id: Option<&str>, default_stream: &str) -> (String, u64) {
    if let Some(node_id) = node_id.and_then(|value| value.strip_prefix("log_stream:")) {
        if let Some((stream_id, cursor_raw)) = node_id.split_once(":cursor:") {
            let cursor = cursor_raw.trim().parse::<u64>().unwrap_or(0);
            if !stream_id.trim().is_empty() {
                return (stream_id.trim().to_string(), cursor);
            }
        }
    }
    (default_stream.to_string(), 0)
}

fn parse_prefixed_node_id(node_id: Option<&str>, prefix: &str) -> Option<String> {
    node_id
        .and_then(|value| value.strip_prefix(prefix))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_string())
}

fn workbench_selection_href(route_id: &str, node_id: &str) -> String {
    format!("{route_id}?node_id={node_id}")
}

async fn generate_gate_viewspec(
    route: &str,
    view_spec_id: &str,
    kind: &str,
    space_id: &str,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let (summary, degraded_reason) = if kind == "testing" {
        match crate::services::ops_gates::load_testing_gate_summary() {
            Ok(value) => (
                serde_json::to_value(value).unwrap_or_else(|_| json!({})),
                Option::<String>::None,
            ),
            Err(err) => (json!({}), Some(err)),
        }
    } else {
        match crate::services::ops_gates::load_siq_gate_summary() {
            Ok(value) => (
                serde_json::to_value(value).unwrap_or_else(|_| json!({})),
                Option::<String>::None,
            ),
            Err(err) => (json!({}), Some(err)),
        }
    };

    let overall_verdict = summary
        .get("overallVerdict")
        .or_else(|| summary.get("overall_verdict"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let latest_run_id = summary
        .get("latestRunId")
        .or_else(|| summary.get("latest_run_id"))
        .and_then(|v| v.as_str())
        .unwrap_or("unknown")
        .to_string();
    let required_pass = if kind == "testing" {
        summary
            .get("requiredBlockersPass")
            .or_else(|| summary.get("required_blockers_pass"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    } else {
        summary
            .get("requiredGatesPass")
            .or_else(|| summary.get("required_gates_pass"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    };
    let failures = summary
        .get("failures")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let failure_rows = failures
        .iter()
        .map(|row| {
            if let Some(obj) = row.as_object() {
                let code = obj
                    .get("code")
                    .or_else(|| obj.get("failureCode"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let message = obj
                    .get("message")
                    .or_else(|| obj.get("failureMessage"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                json!({
                    "Code": code,
                    "Message": message,
                })
            } else {
                json!({
                    "Code": "unknown",
                    "Message": row.to_string(),
                })
            }
        })
        .collect::<Vec<_>>();

    let stream_id = if kind == "testing" {
        "testing_gate_summary_latest"
    } else {
        "siq_gate_summary_latest"
    };
    let heap_focus = if kind == "testing" {
        "gate_summary_testing_latest"
    } else {
        "gate_summary_siq_latest"
    };

    let mut component_refs = vec![
        heading_component(
            &format!("{kind}_title"),
            if kind == "testing" {
                "Testing Gate Summary".to_string()
            } else {
                "SIQ Gate Summary".to_string()
            },
        ),
        alert_banner_component(
            &format!("{kind}_status"),
            if degraded_reason.is_some() {
                "Gate surface degraded"
            } else {
                "Gate surface ready"
            },
            if degraded_reason.is_some() {
                "warning"
            } else if required_pass {
                "success"
            } else {
                "error"
            },
            degraded_reason.unwrap_or_else(|| {
                format!(
                    "latest_run_id={} overall_verdict={} required_gates_pass={}",
                    latest_run_id, overall_verdict, required_pass
                )
            }),
        ),
        grid_component(
            &format!("{kind}_metrics_grid"),
            vec![
                &format!("{kind}_metric_verdict"),
                &format!("{kind}_metric_required"),
                &format!("{kind}_metric_run"),
                &format!("{kind}_metric_failures"),
            ],
        ),
        metric_card_component(
            &format!("{kind}_metric_verdict"),
            "Overall Verdict",
            overall_verdict.clone(),
            None,
        ),
        metric_card_component(
            &format!("{kind}_metric_required"),
            "Required Gates Pass",
            required_pass.to_string(),
            None,
        ),
        metric_card_component(
            &format!("{kind}_metric_run"),
            "Latest Run",
            latest_run_id,
            None,
        ),
        metric_card_component(
            &format!("{kind}_metric_failures"),
            "Failure Count",
            failures.len().to_string(),
            None,
        ),
        heading_component(&format!("{kind}_failures_heading"), "Failures".to_string()),
        data_table_component(
            &format!("{kind}_failures_table"),
            vec!["Code", "Message"],
            failure_rows,
        ),
        button_component(
            &format!("{kind}_save_to_heap"),
            "Save to Heap",
            Some(format!(
                "emitGateSummaryToHeap?kind={kind}&workspaceId={}",
                space_id
            )),
            None,
            None,
        ),
        button_component(
            &format!("{kind}_open_heap"),
            "Open in Heap",
            None,
            Some(format!("/heap?focus={heap_focus}")),
            None,
        ),
        button_component(
            &format!("{kind}_open_logs"),
            "Open Logs",
            None,
            Some(format!("/logs?node_id=log_stream:{stream_id}:cursor:0")),
            None,
        ),
    ];

    if failures.is_empty() {
        component_refs.push(text_component(
            &format!("{kind}_failures_empty"),
            "No failures found in latest gate summary.".to_string(),
        ));
    }

    make_workbench_viewspec(
        view_spec_id,
        route,
        "operator",
        format!("Gate summary workbench projection for {}", kind),
        style_tokens,
        component_refs,
    )
}

async fn generate_siq_viewspec(space_id: &str, intent: &str, density: &str) -> ViewSpecV1 {
    generate_gate_viewspec(
        "/system/siq",
        "workbench-system-siq",
        "siq",
        space_id,
        intent,
        density,
    )
    .await
}

async fn generate_testing_viewspec(space_id: &str, intent: &str, density: &str) -> ViewSpecV1 {
    generate_gate_viewspec(
        "/testing",
        "workbench-testing",
        "testing",
        space_id,
        intent,
        density,
    )
    .await
}

async fn generate_logs_viewspec(
    role: &str,
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let streams = crate::services::system_logs::list_streams()
        .into_iter()
        .filter(|stream| crate::services::system_logs::role_allows(role, &stream.required_role))
        .collect::<Vec<_>>();

    let default_stream = streams
        .first()
        .map(|stream| stream.stream_id.as_str())
        .unwrap_or("siq_gate_summary_latest");
    let (mut selected_stream_id, mut cursor) = parse_log_node_id(selected_node_id, default_stream);
    if !streams
        .iter()
        .any(|stream| stream.stream_id == selected_stream_id)
    {
        selected_stream_id = default_stream.to_string();
        cursor = 0;
    }

    let (tail, degraded_reason) =
        match crate::services::system_logs::tail_stream(&selected_stream_id, cursor, 100) {
            Ok((_def, response)) => (Some(response), None),
            Err(err) => (None, Some(err)),
        };

    let stream_rows = streams
        .iter()
        .map(|stream| {
            json!({
                "_row_id": stream.stream_id,
                "_href": format!("/logs?node_id=log_stream:{}:cursor:0", stream.stream_id),
                "Stream ID": stream.stream_id,
                "Label": stream.label,
                "Format": stream.format,
                "Required Role": stream.required_role,
                "Description": stream.description
            })
        })
        .collect::<Vec<_>>();

    let event_rows = tail
        .as_ref()
        .map(|response| {
            response
                .events
                .iter()
                .map(|event| {
                    json!({
                        "Timestamp": event.ts.clone().unwrap_or_else(|| "-".to_string()),
                        "Level": event.level,
                        "Subsystem": event.subsystem,
                        "Message": event.message,
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let next_cursor = tail
        .as_ref()
        .map(|response| response.next_cursor)
        .unwrap_or(cursor);

    let component_refs = vec![
        heading_component("logs_title", "System Logs".to_string()),
        alert_banner_component(
            "logs_status",
            if degraded_reason.is_some() {
                "Logs degraded"
            } else {
                "Logs ready"
            },
            if degraded_reason.is_some() {
                "warning"
            } else {
                "success"
            },
            degraded_reason.unwrap_or_else(|| {
                format!(
                    "selected_stream={} cursor={} next_cursor={}",
                    selected_stream_id, cursor, next_cursor
                )
            }),
        ),
        grid_component(
            "logs_metrics_grid",
            vec![
                "logs_metric_streams",
                "logs_metric_events",
                "logs_metric_cursor",
                "logs_metric_next_cursor",
                "logs_metric_selected_stream",
            ],
        ),
        metric_card_component(
            "logs_metric_streams",
            "Visible Streams",
            streams.len().to_string(),
            None,
        ),
        metric_card_component(
            "logs_metric_events",
            "Events (page)",
            event_rows.len().to_string(),
            None,
        ),
        metric_card_component("logs_metric_cursor", "Cursor", cursor.to_string(), None),
        metric_card_component(
            "logs_metric_next_cursor",
            "Next Cursor",
            next_cursor.to_string(),
            None,
        ),
        metric_card_component(
            "logs_metric_selected_stream",
            "Selected Stream",
            selected_stream_id.clone(),
            None,
        ),
        data_table_component_with_options(
            "logs_streams_table",
            vec![
                "Stream ID",
                "Label",
                "Format",
                "Required Role",
                "Description",
            ],
            stream_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component(
            "logs_events_table",
            vec!["Timestamp", "Level", "Subsystem", "Message"],
            event_rows,
        ),
        button_component(
            "logs_refresh",
            "Refresh",
            None,
            Some(format!(
                "/logs?node_id=log_stream:{}:cursor:{}",
                selected_stream_id, cursor
            )),
            None,
        ),
        button_component(
            "logs_next",
            "Next page",
            None,
            Some(format!(
                "/logs?node_id=log_stream:{}:cursor:{}",
                selected_stream_id, next_cursor
            )),
            None,
        ),
        button_component(
            "logs_reset",
            "Reset",
            None,
            Some(format!(
                "/logs?node_id=log_stream:{}:cursor:0",
                selected_stream_id
            )),
            None,
        ),
    ];

    make_workbench_viewspec(
        "workbench-logs",
        "/logs",
        "operator",
        "Curated structured log streams with deterministic cursor tailing.".to_string(),
        style_tokens,
        component_refs,
    )
}

async fn generate_agents_viewspec(
    space_id: &str,
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let (runs, degraded_reason) = match crate::services::ops_agents::list_agent_runs(space_id, 25) {
        Ok(value) => (
            value
                .into_iter()
                .filter_map(|row| serde_json::to_value(row).ok())
                .collect::<Vec<_>>(),
            None,
        ),
        Err(err) => (Vec::new(), Some(err)),
    };

    let selected_run_id = parse_prefixed_node_id(selected_node_id, "agent_run:");
    let selected_record = if let Some(run_id) = selected_run_id.as_ref() {
        crate::services::ops_agents::load_agent_run(space_id, run_id)
            .ok()
            .and_then(|record| serde_json::to_value(record).ok())
    } else {
        None
    };

    let pending_reviews = runs
        .iter()
        .filter(|row| {
            row.get("requiresReview")
                .and_then(|v| v.as_bool())
                .unwrap_or(false)
        })
        .count();
    let run_rows = runs
        .iter()
        .map(|row| {
            let run_id = row.get("runId").and_then(|v| v.as_str()).unwrap_or("unknown");
            json!({
                "_row_id": run_id,
                "_href": format!("/agents?node_id=agent_run:{run_id}"),
                "Run ID": run_id,
                "Workflow ID": row.get("workflowId").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "Status": row.get("status").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "Authority": row.get("authorityLevel").and_then(|v| v.as_str()).unwrap_or("-"),
                "Started At": row.get("startedAt").and_then(|v| v.as_str()).unwrap_or("-"),
                "Requires Review": row.get("requiresReview").and_then(|v| v.as_bool()).unwrap_or(false).to_string(),
            })
        })
        .collect::<Vec<_>>();

    let event_rows = selected_record
        .as_ref()
        .and_then(|record| record.get("events"))
        .and_then(|events| events.as_array())
        .map(|events| {
            events
                .iter()
                .take(200)
                .map(|event| {
                    json!({
                        "Timestamp": event.get("timestamp").and_then(|v| v.as_str()).unwrap_or("-"),
                        "Type": event.get("eventType").or_else(|| event.get("type")).and_then(|v| v.as_str()).unwrap_or("event"),
                        "Message": event.get("message").and_then(|v| v.as_str()).unwrap_or("-"),
                    })
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let mut component_refs = vec![
        heading_component("agents_title", "Agents".to_string()),
        alert_banner_component(
            "agents_status",
            if degraded_reason.is_some() {
                "Agents degraded"
            } else {
                "Agents ready"
            },
            if degraded_reason.is_some() {
                "warning"
            } else {
                "success"
            },
            degraded_reason
                .unwrap_or_else(|| format!("space_id={} recent_runs={}", space_id, runs.len())),
        ),
        grid_component(
            "agents_metrics_grid",
            vec![
                "agents_metric_recent",
                "agents_metric_pending",
                "agents_metric_selected",
            ],
        ),
        metric_card_component(
            "agents_metric_recent",
            "Recent Runs",
            runs.len().to_string(),
            None,
        ),
        metric_card_component(
            "agents_metric_pending",
            "Pending Review",
            pending_reviews.to_string(),
            None,
        ),
        metric_card_component(
            "agents_metric_selected",
            "Selected Run",
            selected_run_id
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            None,
        ),
        data_table_component_with_options(
            "agents_runs_table",
            vec![
                "Run ID",
                "Workflow ID",
                "Status",
                "Authority",
                "Started At",
                "Requires Review",
            ],
            run_rows,
            Some(vec!["_row_id", "_href"]),
        ),
    ];

    if let Some(record) = selected_record.as_ref() {
        component_refs.push(data_table_component(
            "agents_selected_table",
            vec!["Field", "Value"],
            vec![
                json!({"Field":"Run ID","Value": record.get("runId").and_then(|v| v.as_str()).unwrap_or("-")}),
                json!({"Field":"Workflow ID","Value": record.get("workflowId").and_then(|v| v.as_str()).unwrap_or("-")}),
                json!({"Field":"Status","Value": record.get("status").and_then(|v| v.as_str()).unwrap_or("-")}),
                json!({"Field":"Requires Review","Value": record.get("approval").map(|_| "true").unwrap_or("false")}),
                json!({"Field":"Started At","Value": record.get("startedAt").and_then(|v| v.as_str()).unwrap_or("-")}),
                json!({"Field":"Updated At","Value": record.get("updatedAt").and_then(|v| v.as_str()).unwrap_or("-")}),
                json!({"Field":"Finished At","Value": record.get("finishedAt").and_then(|v| v.as_str()).unwrap_or("-")}),
            ],
        ));
        component_refs.push(heading_component(
            "agents_events_heading",
            "Selected Run Events".to_string(),
        ));
        component_refs.push(data_table_component(
            "agents_events_table",
            vec!["Timestamp", "Type", "Message"],
            event_rows,
        ));
    } else {
        component_refs.push(text_component(
            "agents_selection_hint",
            "Select a run using node_id=agent_run:<runId> to inspect event details.".to_string(),
        ));
    }

    make_workbench_viewspec(
        "workbench-agents",
        "/agents",
        "operator",
        "Live agent runs with deterministic detail drill-ins.".to_string(),
        style_tokens,
        component_refs,
    )
}

fn contribution_graph_runs_dir(space_id: &str) -> PathBuf {
    crate::gateway::server::workspace_root()
        .join("_spaces")
        .join(space_id)
        .join(".cortex")
        .join("logs")
        .join("contribution_graph")
        .join("runs")
}

fn load_contribution_graph_runs(space_id: &str, limit: usize) -> Result<Vec<Value>, String> {
    let mut runs = Vec::new();
    let dir = contribution_graph_runs_dir(space_id);
    let entries = fs::read_dir(&dir)
        .map_err(|err| format!("failed_to_read_contribution_runs:{}:{err}", dir.display()))?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(value) => value,
            Err(_) => continue,
        };
        let value = match serde_json::from_str::<Value>(&raw) {
            Ok(value) => value,
            Err(_) => continue,
        };
        runs.push(value);
    }
    runs.sort_by(|left, right| {
        let right_started = right
            .get("startedAt")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        let left_started = left
            .get("startedAt")
            .and_then(|value| value.as_str())
            .unwrap_or_default();
        right_started.cmp(left_started)
    });
    runs.truncate(limit.min(200));
    Ok(runs)
}

fn load_contribution_graph_run(space_id: &str, run_id: &str) -> Option<Value> {
    let path = contribution_graph_runs_dir(space_id).join(format!("{run_id}.json"));
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<Value>(&raw).ok())
}

async fn generate_contributions_viewspec(
    space_id: &str,
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let (graph_runs, graph_degraded) = match load_contribution_graph_runs(space_id, 25) {
        Ok(value) => (value, None),
        Err(err) => (Vec::new(), Some(err)),
    };
    let (agent_runs, agent_degraded) = match crate::services::ops_agents::list_agent_runs(space_id, 25)
    {
        Ok(value) => (
            value
                .into_iter()
                .filter_map(|row| serde_json::to_value(row).ok())
                .collect::<Vec<_>>(),
            None,
        ),
        Err(err) => (Vec::new(), Some(err)),
    };

    let selected_graph_run_id = parse_prefixed_node_id(selected_node_id, "graph_run:");
    let selected_agent_run_id = parse_prefixed_node_id(selected_node_id, "agent_run:");
    let selected_contribution_id = parse_prefixed_node_id(selected_node_id, "contribution:");

    let selected_graph_run = selected_graph_run_id
        .as_deref()
        .and_then(|run_id| load_contribution_graph_run(space_id, run_id));
    let selected_agent_run = selected_agent_run_id
        .as_deref()
        .and_then(|run_id| crate::services::ops_agents::load_agent_run(space_id, run_id).ok())
        .and_then(|record| serde_json::to_value(record).ok());

    let graph_rows = graph_runs
        .iter()
        .map(|row| {
            let run_id = row.get("runId").and_then(|value| value.as_str()).unwrap_or("unknown");
            json!({
                "_row_id": run_id,
                "_href": format!("/contributions?run_id={run_id}"),
                "Run ID": run_id,
                "Mode": row.get("mode").and_then(|value| value.as_str()).unwrap_or("unknown"),
                "Status": row.get("status").and_then(|value| value.as_str()).unwrap_or("unknown"),
                "Started At": row.get("startedAt").and_then(|value| value.as_str()).unwrap_or("-"),
            })
        })
        .collect::<Vec<_>>();
    let agent_rows = agent_runs
        .iter()
        .map(|row| {
            let run_id = row.get("runId").and_then(|value| value.as_str()).unwrap_or("unknown");
            json!({
                "_row_id": run_id,
                "_href": format!("/contributions?node_id=agent_run:{run_id}"),
                "Run ID": run_id,
                "Contribution ID": row.get("contributionId").and_then(|value| value.as_str()).unwrap_or("-"),
                "Status": row.get("status").and_then(|value| value.as_str()).unwrap_or("unknown"),
                "Requires Review": row.get("requiresReview").and_then(|value| value.as_bool()).unwrap_or(false).to_string(),
            })
        })
        .collect::<Vec<_>>();
    let degraded_messages = [graph_degraded, agent_degraded]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();

    let mut component_refs = vec![
        heading_component("contributions_title", "Contributions Cockpit".to_string()),
        alert_banner_component(
            "contributions_status",
            if degraded_messages.is_empty() {
                "Contribution lifecycle ready"
            } else {
                "Contribution lifecycle degraded"
            },
            if degraded_messages.is_empty() { "success" } else { "warning" },
            if degraded_messages.is_empty() {
                format!(
                    "space_id={} graph_runs={} agent_runs={}",
                    space_id,
                    graph_runs.len(),
                    agent_runs.len()
                )
            } else {
                degraded_messages.join(" | ")
            },
        ),
        grid_component(
            "contributions_metrics_grid",
            vec![
                "contributions_metric_graph_runs",
                "contributions_metric_agent_runs",
                "contributions_metric_focus",
            ],
        ),
        metric_card_component(
            "contributions_metric_graph_runs",
            "Graph Runs",
            graph_runs.len().to_string(),
            None,
        ),
        metric_card_component(
            "contributions_metric_agent_runs",
            "Agent Runs",
            agent_runs.len().to_string(),
            None,
        ),
        metric_card_component(
            "contributions_metric_focus",
            "Focus",
            selected_agent_run_id
                .clone()
                .or(selected_contribution_id.clone())
                .or(selected_graph_run_id.clone())
                .unwrap_or_else(|| "none".to_string()),
            None,
        ),
        data_table_component_with_options(
            "contributions_graph_runs_table",
            vec!["Run ID", "Mode", "Status", "Started At"],
            graph_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component_with_options(
            "contributions_agent_runs_table",
            vec!["Run ID", "Contribution ID", "Status", "Requires Review"],
            agent_rows,
            Some(vec!["_row_id", "_href"]),
        ),
    ];

    if let Some(run) = selected_graph_run.as_ref() {
        component_refs.push(data_table_component(
            "contributions_selected_graph_run_table",
            vec!["Field", "Value"],
            vec![
                json!({"Field":"Run ID","Value": run.get("runId").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Mode","Value": run.get("mode").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Status","Value": run.get("status").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Started At","Value": run.get("startedAt").and_then(|value| value.as_str()).unwrap_or("-")}),
            ],
        ));
    }

    if let Some(record) = selected_agent_run.as_ref() {
        component_refs.push(data_table_component(
            "contributions_selected_run_table",
            vec!["Field", "Value"],
            vec![
                json!({"Field":"Run ID","Value": record.get("runId").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Workflow ID","Value": record.get("workflowId").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Contribution ID","Value": record.get("contributionId").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Status","Value": record.get("status").and_then(|value| value.as_str()).unwrap_or("-")}),
                json!({"Field":"Updated At","Value": record.get("updatedAt").and_then(|value| value.as_str()).unwrap_or("-")}),
            ],
        ));
    }

    if let Some(contribution_id) = selected_contribution_id.as_ref() {
        component_refs.push(data_table_component(
            "contributions_selected_contribution_table",
            vec!["Field", "Value"],
            vec![
                json!({"Field":"Contribution ID","Value": contribution_id}),
                json!({"Field":"Route Contract","Value": format!("/contributions?node_id=contribution:{contribution_id}")}),
            ],
        ));
    } else if selected_graph_run.is_none() && selected_agent_run.is_none() {
        component_refs.push(text_component(
            "contributions_selection_hint",
            "Select a graph run with run_id=<runId>, an agent run with node_id=agent_run:<runId>, or a contribution with node_id=contribution:<id>.".to_string(),
        ));
    }

    make_workbench_viewspec(
        "workbench-contributions",
        "/contributions",
        "operator",
        "Steward-facing contribution lifecycle cockpit with graph history and live agent drill-ins.".to_string(),
        style_tokens,
        component_refs,
    )
}

async fn generate_artifacts_viewspec(
    selected_node_id: Option<&str>,
    intent: &str,
    _density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), "compact".to_string());
    style_tokens.insert("padding".to_string(), "0".to_string());
    style_tokens.insert("spacing".to_string(), "0".to_string());
    style_tokens.insert("edgeToEdge".to_string(), "true".to_string());

    let (items, _degraded_reason) = match crate::services::ops_artifacts::list_artifacts(200) {
        Ok(value) => (
            value
                .items
                .into_iter()
                .filter_map(|item| serde_json::to_value(item).ok())
                .collect::<Vec<_>>(),
            None,
        ),
        Err(err) => (Vec::new(), Some(err)),
    };

    let selected_artifact_id = parse_prefixed_node_id(selected_node_id, "artifact:");

    let rows = items
        .iter()
        .map(|row| {
            let artifact_id = row
                .get("artifactId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");

            let title = row.get("title").and_then(|v| v.as_str()).unwrap_or("").to_lowercase();
            let aid = artifact_id.to_lowercase();

            let block_type = row
                .get("heapBlockType")
                .or_else(|| row.get("heap_block_type"))
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
                .unwrap_or_else(|| {
                    let route = row.get("routeId").and_then(|v| v.as_str()).unwrap_or("");
                    if aid.contains("gate_summary") || title.contains("gate summary") {
                        "gate_summary".to_string()
                    } else if aid.ends_with(".pdf") || title.ends_with(".pdf") {
                        "document".to_string()
                    } else if route == "/heap" {
                        "heap_block".to_string()
                    } else if route == "/contributions" {
                        "contribution".to_string()
                    } else if aid.ends_with(".md") || title.ends_with(".md") {
                        "markdown".to_string()
                    } else if aid.contains("workflow") {
                        "workflow".to_string()
                    } else if aid.contains("plan") {
                        "plan".to_string()
                    } else if aid.contains("analysis") {
                        "analysis".to_string()
                    } else {
                        "artifact".to_string()
                    }
                });

            let updated_at = row.get("updatedAt").and_then(|v| v.as_str()).unwrap_or("-");
            let clean_updated = if let Some(idx) = updated_at.find('.') {
                format!("{}Z", &updated_at[..idx])
            } else {
                updated_at.to_string()
            };

            json!({
                "_row_id": artifact_id,
                "_href": format!("/artifacts?node_id=artifact:{artifact_id}"),
                "Artifact ID": artifact_id,
                "Title": row.get("title").and_then(|v| v.as_str()).unwrap_or("untitled"),
                "Type": block_type,
                "Status": row.get("status").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "Updated At": clean_updated,
            })
        })
        .collect::<Vec<_>>();

    let published_count = items
        .iter()
        .filter(|item| {
            item.get("status")
                .and_then(|v| v.as_str())
                .map(|status| status == "published")
                .unwrap_or(false)
        })
        .count();

    let component_refs = vec![
        heading_component("artifacts_title", "Artifacts".to_string()),
        ComponentRef {
            component_id: "artifacts_filter_row".to_string(),
            component_type: "Container".to_string(),
            props: BTreeMap::from([
                ("widgetType".to_string(), Value::String("Row".to_string())),
            ]),
            a11y: Some(ViewSpecA11y { label: Some("Filter options".to_string()), ..ViewSpecA11y::default() }),
            children: vec![
                "artifacts_filter_all".to_string(),
                "artifacts_filter_published".to_string(),
                "artifacts_filter_drafts".to_string(),
                "artifacts_filter_selected".to_string(),
            ],
        },
        button_component(
            "artifacts_filter_all",
            &format!("All ({})", items.len()),
            None,
            Some("/artifacts".to_string()),
            Some("Filter artifacts by all items".to_string()),
        ),
        button_component(
            "artifacts_filter_published",
            &format!("Published ({})", published_count),
            None,
            Some("/artifacts?intent=published".to_string()),
            Some("Filter artifacts by published items".to_string()),
        ),
        button_component(
            "artifacts_filter_drafts",
            &format!("Drafts ({})", items.len() - published_count),
            None,
            Some("/artifacts?intent=draft".to_string()),
            Some("Filter artifacts by draft items".to_string()),
        ),
        button_component(
            "artifacts_filter_selected",
            &format!("Selected: {}", selected_artifact_id.as_deref().unwrap_or("none")),
            None,
            None,
            Some("Show selected artifact details".to_string()),
        ),
        data_table_component_with_options(
            "artifacts_table",
            vec![
                "Artifact ID",
                "Title",
                "Type",
                "Status",
                "Updated At",
            ],
            rows,
            Some(vec!["_row_id", "_href", "Artifact ID"]), // pass Artifact ID as a hidden option so WidgetRegistry can render it under Title without its own column
        ),
    ];

    let layout_graph = linear_layout_from_ids(vec![
        "artifacts_title".to_string(),
        "artifacts_filter_row".to_string(),
        "artifacts_table".to_string(),
    ]);

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: "workbench-artifacts".to_string(),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some("/artifacts".to_string()),
            role: Some("operator".to_string()),
        },
        intent: "Artifact inventory with deterministic selection drill-ins.".to_string(),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.95,
            rationale: "Live operational workbench projection".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

async fn generate_flows_viewspec(
    route_id: &str,
    space_id: &str,
    selected_node_id: Option<&str>,
    intent: &str,
    density: &str,
) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());
    style_tokens.insert("intent".to_string(), intent.to_string());
    style_tokens.insert("density".to_string(), density.to_string());

    let workflow_focus = route_id == "/workflows"
        || selected_node_id
            .map(|value| value.starts_with("workflow_"))
            .unwrap_or(false);
    let (decision_plane, plane_error, workflow_catalog, catalog_error) = if workflow_focus {
        (json!({}), None, Vec::new(), None)
    } else {
        let (decision_plane, plane_error) = match timeout(
            Duration::from_millis(250),
            crate::services::ops_flows::load_decision_plane(space_id),
        )
        .await
        {
            Ok(Ok(value)) => (
                serde_json::to_value(value).unwrap_or_else(|_| json!({})),
                None,
            ),
            Ok(Err(err)) => (json!({}), Some(err)),
            Err(_) => (json!({}), Some("decision_plane_timeout".to_string())),
        };
        let (workflow_catalog, catalog_error) = match timeout(
            Duration::from_millis(250),
            crate::services::ops_flows::load_workflow_catalog(),
        )
        .await
        {
            Ok(Ok(value)) => (
                value
                    .into_iter()
                    .filter_map(|entry| serde_json::to_value(entry).ok())
                    .collect::<Vec<_>>(),
                None,
            ),
            Ok(Err(err)) => (Vec::new(), Some(err)),
            Err(_) => (Vec::new(), Some("workflow_catalog_timeout".to_string())),
        };
        (decision_plane, plane_error, workflow_catalog, catalog_error)
    };
    let workflow_state = load_workbench_workflow_state(space_id).await;

    let surfaces = decision_plane
        .get("surfaces")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let digest = decision_plane
        .get("digest")
        .cloned()
        .unwrap_or_else(|| json!({}));
    let blocked = digest
        .get("status")
        .and_then(|v| v.as_str())
        .map(|status| status == "blocked")
        .unwrap_or(false);
    let required_actions = digest
        .get("requiredActions")
        .and_then(|v| v.as_array())
        .map(|items| items.len())
        .unwrap_or(0);
    let degraded_count = surfaces
        .iter()
        .filter(|surface| {
            surface
                .get("degradedReason")
                .and_then(|value| value.as_str())
                .map(|value| !value.trim().is_empty())
                .unwrap_or(false)
                || matches!(
                    surface.get("status").and_then(|value| value.as_str()),
                    Some("blocked" | "missing" | "require_simulation")
                )
        })
        .count();

    let surface_rows = surfaces
        .iter()
        .map(|surface| {
            let surface_id = surface
                .get("surfaceId")
                .and_then(|v| v.as_str())
                .unwrap_or("unknown");
            json!({
                "_row_id": surface_id,
                "_href": workbench_selection_href(route_id, &format!("flow_surface:{surface_id}")),
                "Surface ID": surface_id,
                "Status": surface.get("status").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "Source": surface.get("sourceOfTruth").and_then(|v| v.as_str()).unwrap_or("-"),
                "Required Actions": surface.get("requiredActions").and_then(|v| v.as_array()).map(|actions| actions.len()).unwrap_or(0).to_string(),
                "Degraded": surface.get("degradedReason").and_then(|v| v.as_str()).unwrap_or("-"),
            })
        })
        .collect::<Vec<_>>();

    let workflow_rows = workflow_catalog
        .iter()
        .map(|workflow| {
            json!({
                "Name": workflow.get("name").and_then(|v| v.as_str()).unwrap_or("unknown"),
                "Path": workflow.get("path").and_then(|v| v.as_str()).unwrap_or("-"),
                "Status": workflow.get("status").and_then(|v| v.as_str()).unwrap_or("-"),
                "Description": workflow.get("description").and_then(|v| v.as_str()).unwrap_or("-"),
            })
        })
        .collect::<Vec<_>>();

    let selected_surface_id = parse_prefixed_node_id(selected_node_id, "flow_surface:");
    let selected_workflow_draft_id = parse_prefixed_node_id(selected_node_id, "workflow_draft:");
    let selected_workflow_proposal_id =
        parse_prefixed_node_id(selected_node_id, "workflow_proposal:");
    let selected_workflow_definition_id =
        parse_prefixed_node_id(selected_node_id, "workflow_definition:");
    let selected_workflow_active_scope =
        parse_prefixed_node_id(selected_node_id, "workflow_active:");
    let selected_workflow_instance_id =
        parse_prefixed_node_id(selected_node_id, "workflow_instance:");
    let mut degraded_messages = Vec::new();
    if let Some(err) = plane_error {
        degraded_messages.push(format!("decision_plane={err}"));
    }
    if let Some(err) = catalog_error {
        degraded_messages.push(format!("workflow_catalog={err}"));
    }
    degraded_messages.extend(workflow_state.degraded_messages.clone());

    let workflow_draft_rows = workflow_state
        .drafts
        .iter()
        .map(|(entry, draft)| {
            json!({
                "_row_id": draft.workflow_draft_id,
                "_href": workbench_selection_href(
                    route_id,
                    &format!("workflow_draft:{}", draft.workflow_draft_id),
                ),
                "Draft ID": draft.workflow_draft_id,
                "Motif": format!("{:?}", draft.motif_kind).to_ascii_lowercase(),
                "Scope": entry.scope_key,
                "Nodes": draft.graph.nodes.len().to_string(),
                "Updated At": entry.updated_at,
            })
        })
        .collect::<Vec<_>>();
    let workflow_proposal_rows = workflow_state
        .proposals
        .iter()
        .map(|(entry, proposal)| {
            json!({
                "_row_id": proposal.proposal_id,
                "_href": workbench_selection_href(
                    route_id,
                    &format!("workflow_proposal:{}", proposal.proposal_id),
                ),
                "Proposal ID": proposal.proposal_id,
                "Status": format!("{:?}", proposal.status).to_ascii_lowercase(),
                "Definition ID": proposal.definition_id,
                "Scope": entry.scope_key,
                "Updated At": entry.updated_at,
            })
        })
        .collect::<Vec<_>>();
    let workflow_definition_rows = workflow_state
        .definitions
        .iter()
        .map(|(entry, artifact)| {
            json!({
                "_row_id": artifact.definition.definition_id,
                "_href": workbench_selection_href(
                    route_id,
                    &format!(
                        "workflow_definition:{}",
                        artifact.definition.definition_id
                    ),
                ),
                "Definition ID": artifact.definition.definition_id,
                "Motif": format!("{:?}", artifact.definition.motif_kind).to_ascii_lowercase(),
                "Scope": entry.scope_key,
                "Nodes": artifact.definition.graph.nodes.len().to_string(),
                "Digest": artifact.definition.digest.clone().unwrap_or_else(|| artifact.compile_result.digest.clone()),
            })
        })
        .collect::<Vec<_>>();
    let workflow_active_rows = workflow_state
        .active_scopes
        .iter()
        .map(|record| {
            json!({
                "_row_id": record.scope_key,
                "_href": workbench_selection_href(
                    route_id,
                    &format!("workflow_active:{}", record.scope_key),
                ),
                "Scope": record.scope_key,
                "Definition ID": record.active_definition_id,
                "Proposal ID": record.adopted_from_proposal_id,
                "Adopted At": record.adopted_at,
            })
        })
        .collect::<Vec<_>>();
    let workflow_instance_rows = workflow_state
        .instances
        .iter()
        .map(|snapshot| {
            json!({
                "_row_id": snapshot.instance.instance_id,
                "_href": workbench_selection_href(
                    route_id,
                    &format!("workflow_instance:{}", snapshot.instance.instance_id),
                ),
                "Instance ID": snapshot.instance.instance_id,
                "Status": format!("{:?}", snapshot.instance.status).to_ascii_lowercase(),
                "Definition ID": snapshot.instance.definition_id,
                "Updated At": snapshot.instance.updated_at,
                "Checkpoints": snapshot.checkpoints.len().to_string(),
                "Outcome": snapshot.outcome.as_ref().map(|outcome| format!("{:?}", outcome.status).to_ascii_lowercase()).unwrap_or_else(|| "-".to_string()),
                "Source": snapshot.instance.source_of_truth,
            })
        })
        .collect::<Vec<_>>();
    let workflow_summary_metrics = vec![
        json!({"label":"Drafts","value":workflow_state.drafts.len().to_string(),"tone":"default"}),
        json!({"label":"Proposals","value":workflow_state.proposals.len().to_string(),"tone":"default"}),
        json!({"label":"Active","value":workflow_state.active_scopes.len().to_string(),"tone":"success"}),
        json!({"label":"Instances","value":workflow_state.instances.len().to_string(),"tone":"default"}),
        json!({"label":"Blocked","value":blocked.to_string(),"tone":if blocked { "warning" } else { "success" }}),
        json!({"label":"Degraded","value":degraded_count.to_string(),"tone":if degraded_count > 0 { "warning" } else { "success" }}),
    ];
    let workflow_timeline_entries = workflow_state
        .instances
        .iter()
        .map(|snapshot| {
            json!({
                "instanceId": snapshot.instance.instance_id,
                "status": format!("{:?}", snapshot.instance.status).to_ascii_lowercase(),
                "updatedAt": snapshot.instance.updated_at,
                "checkpoints": snapshot.checkpoints.len().to_string(),
                "outcome": snapshot.outcome.as_ref().map(|outcome| format!("{:?}", outcome.status).to_ascii_lowercase()).unwrap_or_else(|| "-".to_string()),
                "href": format!(
                    "/api/cortex/workflow-instances/{}/trace",
                    snapshot.instance.instance_id
                ),
            })
        })
        .collect::<Vec<_>>();

    let mut component_refs = vec![
        heading_component(
            "flows_title",
            if workflow_focus {
                "Workflow Orchestration".to_string()
            } else {
                "Decision Flows".to_string()
            },
        ),
        alert_banner_component(
            "flows_status",
            if degraded_messages.is_empty() {
                "Flows ready"
            } else {
                "Flows degraded"
            },
            if degraded_messages.is_empty() {
                "success"
            } else {
                "warning"
            },
            if degraded_messages.is_empty() {
                format!("space_id={} loaded_surfaces={}", space_id, surfaces.len())
            } else {
                degraded_messages.join("; ")
            },
        ),
        workflow_summary_strip_component(
            "flows_workflow_summary_strip",
            "Workflow Summary",
            if workflow_focus {
                "Workflow Orchestration"
            } else {
                "Decision Flow Overview"
            },
            if degraded_messages.is_empty() {
                "Governed workflow drafts, ratified definitions, and live instances in one surface."
                    .to_string()
            } else {
                format!("Degraded signals present: {}", degraded_messages.join("; "))
            },
            workflow_summary_metrics,
        ),
        grid_component(
            "flows_metrics_grid",
            vec![
                "flows_metric_blocked",
                "flows_metric_escalation",
                "flows_metric_degraded",
                "flows_metric_surfaces",
                "flows_metric_workflow_drafts",
                "flows_metric_workflow_proposals",
                "flows_metric_workflow_definitions",
                "flows_metric_workflow_instances",
            ],
        ),
        metric_card_component("flows_metric_blocked", "Blocked", blocked.to_string(), None),
        metric_card_component(
            "flows_metric_escalation",
            "Require Escalation",
            required_actions.to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_degraded",
            "Degraded Surfaces",
            degraded_count.to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_surfaces",
            "Loaded Surfaces",
            surfaces.len().to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_workflow_drafts",
            "Workflow Drafts",
            workflow_state.drafts.len().to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_workflow_proposals",
            "Workflow Proposals",
            workflow_state.proposals.len().to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_workflow_definitions",
            "Active Definitions",
            workflow_state.active_scopes.len().to_string(),
            None,
        ),
        metric_card_component(
            "flows_metric_workflow_instances",
            "Workflow Instances",
            workflow_state.instances.len().to_string(),
            None,
        ),
        heading_component(
            "flows_surfaces_heading",
            if workflow_focus {
                "Decision Plane Surfaces".to_string()
            } else {
                "Decision Plane Surfaces".to_string()
            },
        ),
        data_table_component_with_options(
            "flows_surfaces_table",
            vec![
                "Surface ID",
                "Status",
                "Source",
                "Required Actions",
                "Degraded",
            ],
            surface_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        heading_component("flows_catalog_heading", "Workflow Catalog".to_string()),
        data_table_component(
            "flows_catalog_table",
            vec!["Name", "Path", "Status", "Description"],
            workflow_rows,
        ),
        heading_component("flows_workflow_heading", "Workflow Governance".to_string()),
        data_table_component_with_options(
            "flows_workflow_drafts_table",
            vec!["Draft ID", "Motif", "Scope", "Nodes", "Updated At"],
            workflow_draft_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component_with_options(
            "flows_workflow_proposals_table",
            vec!["Proposal ID", "Status", "Definition ID", "Scope", "Updated At"],
            workflow_proposal_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component_with_options(
            "flows_workflow_definitions_table",
            vec!["Definition ID", "Motif", "Scope", "Nodes", "Digest"],
            workflow_definition_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component_with_options(
            "flows_workflow_active_table",
            vec!["Scope", "Definition ID", "Proposal ID", "Adopted At"],
            workflow_active_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        data_table_component_with_options(
            "flows_workflow_instances_table",
            vec![
                "Instance ID",
                "Status",
                "Definition ID",
                "Updated At",
                "Checkpoints",
                "Outcome",
                "Source",
            ],
            workflow_instance_rows,
            Some(vec!["_row_id", "_href"]),
        ),
        workflow_instance_timeline_component(
            "flows_workflow_instance_timeline",
            "Runtime Timeline",
            "Recent Workflow Instances",
            workflow_timeline_entries,
        ),
    ];

    if workflow_focus {
        component_refs.retain(|component| {
            !matches!(
                component.component_id.as_str(),
                "flows_surfaces_heading" | "flows_surfaces_table" | "flows_catalog_heading"
                    | "flows_catalog_table"
            )
        });
    }

    if let Some(surface_id) = selected_surface_id.as_ref() {
        let selected = surfaces.iter().find(|surface| {
            surface
                .get("surfaceId")
                .and_then(|v| v.as_str())
                .map(|value| value == surface_id)
                .unwrap_or(false)
        });
        if let Some(selected) = selected {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Surface Detail".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Surface ID","Value": selected.get("surfaceId").and_then(|v|v.as_str()).unwrap_or("-")}),
                    json!({"Field":"Status","Value": selected.get("status").and_then(|v|v.as_str()).unwrap_or("-")}),
                    json!({"Field":"Source Of Truth","Value": selected.get("sourceOfTruth").and_then(|v|v.as_str()).unwrap_or("-")}),
                    json!({"Field":"Required Actions","Value": selected.get("requiredActions").and_then(|v|v.as_array()).map(|actions| actions.len()).unwrap_or(0).to_string()}),
                ],
            ));
        }
    } else if let Some(workflow_draft_id) = selected_workflow_draft_id.as_ref() {
        if let Some((entry, draft)) = workflow_state
            .drafts
            .iter()
            .find(|(_, draft)| draft.workflow_draft_id == *workflow_draft_id)
        {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Workflow Draft".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Draft ID","Value": draft.workflow_draft_id}),
                    json!({"Field":"Scope","Value": entry.scope_key}),
                    json!({"Field":"Intent","Value": draft.intent}),
                    json!({"Field":"Motif","Value": format!("{:?}", draft.motif_kind).to_ascii_lowercase()}),
                    json!({"Field":"Nodes","Value": draft.graph.nodes.len().to_string()}),
                    json!({"Field":"Edges","Value": draft.graph.edges.len().to_string()}),
                    json!({"Field":"Updated At","Value": entry.updated_at}),
                ],
            ));
        }
    } else if let Some(proposal_id) = selected_workflow_proposal_id.as_ref() {
        if let Some((_, proposal)) = workflow_state
            .proposals
            .iter()
            .find(|(_, proposal)| proposal.proposal_id == *proposal_id)
        {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Workflow Proposal".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Proposal ID","Value": proposal.proposal_id}),
                    json!({"Field":"Status","Value": format!("{:?}", proposal.status).to_ascii_lowercase()}),
                    json!({"Field":"Draft ID","Value": proposal.workflow_draft_id}),
                    json!({"Field":"Definition ID","Value": proposal.definition_id}),
                    json!({"Field":"Proposed By","Value": proposal.proposed_by}),
                    json!({"Field":"Rationale","Value": proposal.rationale}),
                    json!({"Field":"Created At","Value": proposal.created_at}),
                ],
            ));
            component_refs.push(workflow_status_badge_component(
                "flows_selected_status_badge",
                "Proposal",
                format!("{:?}", proposal.status).to_ascii_lowercase(),
                match proposal.status {
                    cortex_domain::workflow::WorkflowProposalStatus::Ratified
                    | cortex_domain::workflow::WorkflowProposalStatus::Approved => "success",
                    cortex_domain::workflow::WorkflowProposalStatus::Rejected => "error",
                    _ => "warning",
                },
                Some(workbench_selection_href(
                    route_id,
                    &format!("workflow_proposal:{}", proposal.proposal_id),
                )),
            ));
            component_refs.push(button_component(
                "flows_selected_proposal_replay",
                "Replay Artifact",
                None,
                Some(format!(
                    "/api/cortex/workflow-drafts/proposals/{}/replay",
                    proposal.proposal_id
                )),
                None,
            ));
            component_refs.push(button_component(
                "flows_selected_proposal_digest",
                "Digest Artifact",
                None,
                Some(format!(
                    "/api/cortex/workflow-drafts/proposals/{}/digest",
                    proposal.proposal_id
                )),
                None,
            ));
        }
    } else if let Some(definition_id) = selected_workflow_definition_id.as_ref() {
        if let Some((_, artifact)) = workflow_state
            .definitions
            .iter()
            .find(|(_, artifact)| artifact.definition.definition_id == *definition_id)
        {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Workflow Definition".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Definition ID","Value": artifact.definition.definition_id}),
                    json!({"Field":"Intent","Value": artifact.definition.intent}),
                    json!({"Field":"Motif","Value": format!("{:?}", artifact.definition.motif_kind).to_ascii_lowercase()}),
                    json!({"Field":"Nodes","Value": artifact.definition.graph.nodes.len().to_string()}),
                    json!({"Field":"Edges","Value": artifact.definition.graph.edges.len().to_string()}),
                    json!({"Field":"Digest","Value": artifact.definition.digest.clone().unwrap_or_else(|| artifact.compile_result.digest.clone())}),
                ],
            ));
            component_refs.push(workflow_status_badge_component(
                "flows_selected_status_badge",
                "Definition",
                "ratified".to_string(),
                "success",
                Some(workbench_selection_href(
                    route_id,
                    &format!(
                        "workflow_definition:{}",
                        artifact.definition.definition_id
                    ),
                )),
            ));
            component_refs.push(workflow_projection_preview_component(
                "flows_workflow_projection_preview",
                "Definition Preview",
                artifact.definition.definition_id.clone(),
                Some(workbench_selection_href(
                    route_id,
                    &format!(
                        "workflow_definition:{}",
                        artifact.definition.definition_id
                    ),
                )),
                format!("{:?}", artifact.definition.motif_kind).to_ascii_lowercase(),
                artifact
                    .definition
                    .digest
                    .clone()
                    .unwrap_or_else(|| artifact.compile_result.digest.clone()),
                artifact.definition.graph.nodes.len(),
                vec![
                    json!({"label":"Graph","kind":"flow_graph_v1","href":format!("/api/cortex/workflow-definitions/{}/projections/flow_graph_v1", artifact.definition.definition_id)}),
                    json!({"label":"A2UI","kind":"a2ui_surface_v1","href":format!("/api/cortex/workflow-definitions/{}/projections/a2ui_surface_v1", artifact.definition.definition_id)}),
                    json!({"label":"SW","kind":"serverless_workflow_v0_8","href":format!("/api/cortex/workflow-definitions/{}/projections/serverless_workflow_v0_8", artifact.definition.definition_id)}),
                    json!({"label":"Normalized","kind":"normalized_graph_v1","href":format!("/api/cortex/workflow-definitions/{}/projections/normalized_graph_v1", artifact.definition.definition_id)}),
                ],
            ));
            component_refs.push(button_component(
                "flows_selected_definition_graph",
                "Flow Graph",
                None,
                Some(format!(
                    "/api/cortex/workflow-definitions/{}/projections/flow_graph_v1",
                    artifact.definition.definition_id
                )),
                None,
            ));
            component_refs.push(button_component(
                "flows_selected_definition_a2ui",
                "A2UI Projection",
                None,
                Some(format!(
                    "/api/cortex/workflow-definitions/{}/projections/a2ui_surface_v1",
                    artifact.definition.definition_id
                )),
                None,
            ));
        }
    } else if let Some(scope_key) = selected_workflow_active_scope.as_ref() {
        if let Some(record) = workflow_state
            .active_scopes
            .iter()
            .find(|record| record.scope_key == *scope_key)
        {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Active Workflow Scope".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Scope","Value": record.scope_key}),
                    json!({"Field":"Definition ID","Value": record.active_definition_id}),
                    json!({"Field":"Proposal ID","Value": record.adopted_from_proposal_id}),
                    json!({"Field":"Adopted At","Value": record.adopted_at}),
                    json!({"Field":"Adopted By","Value": record.adopted_by}),
                ],
            ));
            component_refs.push(workflow_status_badge_component(
                "flows_selected_status_badge",
                "Active Scope",
                "active".to_string(),
                "success",
                Some(workbench_selection_href(
                    route_id,
                    &format!("workflow_active:{}", record.scope_key),
                )),
            ));
        }
    } else if let Some(instance_id) = selected_workflow_instance_id.as_ref() {
        if let Some(snapshot) = workflow_state
            .instances
            .iter()
            .find(|snapshot| snapshot.instance.instance_id == *instance_id)
        {
            component_refs.push(heading_component(
                "flows_selected_heading",
                "Selected Workflow Instance".to_string(),
            ));
            component_refs.push(data_table_component(
                "flows_selected_table",
                vec!["Field", "Value"],
                vec![
                    json!({"Field":"Instance ID","Value": snapshot.instance.instance_id}),
                    json!({"Field":"Status","Value": format!("{:?}", snapshot.instance.status).to_ascii_lowercase()}),
                    json!({"Field":"Definition ID","Value": snapshot.instance.definition_id}),
                    json!({"Field":"Binding ID","Value": snapshot.instance.binding_id}),
                    json!({"Field":"Updated At","Value": snapshot.instance.updated_at}),
                    json!({"Field":"Checkpoints","Value": snapshot.checkpoints.len().to_string()}),
                    json!({"Field":"Outcome","Value": snapshot.outcome.as_ref().map(|outcome| format!("{:?}", outcome.status).to_ascii_lowercase()).unwrap_or_else(|| "-".to_string())}),
                ],
            ));
            component_refs.push(workflow_status_badge_component(
                "flows_selected_status_badge",
                "Instance",
                format!("{:?}", snapshot.instance.status).to_ascii_lowercase(),
                match snapshot.instance.status {
                    cortex_domain::workflow::WorkflowInstanceStatus::Completed => "success",
                    cortex_domain::workflow::WorkflowInstanceStatus::Failed
                    | cortex_domain::workflow::WorkflowInstanceStatus::Cancelled => "error",
                    _ => "warning",
                },
                Some(workbench_selection_href(
                    route_id,
                    &format!("workflow_instance:{}", snapshot.instance.instance_id),
                )),
            ));
            component_refs.push(button_component(
                "flows_selected_instance_trace",
                "Trace",
                None,
                Some(format!(
                    "/api/cortex/workflow-instances/{}/trace",
                    snapshot.instance.instance_id
                )),
                None,
            ));
            component_refs.push(button_component(
                "flows_selected_instance_checkpoints",
                "Checkpoints",
                None,
                Some(format!(
                    "/api/cortex/workflow-instances/{}/checkpoints",
                    snapshot.instance.instance_id
                )),
                None,
            ));
            component_refs.push(button_component(
                "flows_selected_instance_outcome",
                "Outcome",
                None,
                Some(format!(
                    "/api/cortex/workflow-instances/{}/outcome",
                    snapshot.instance.instance_id
                )),
                None,
            ));
        }
    } else {
        component_refs.push(text_component(
            "flows_selection_hint",
            "Select a decision surface or workflow artifact using node_id=flow_surface:<surfaceId>, workflow_draft:<id>, workflow_proposal:<id>, workflow_definition:<id>, workflow_active:<scopeKey>, or workflow_instance:<id>."
                .to_string(),
        ));
    }

    make_workbench_viewspec(
        "workbench-flows-live",
        route_id,
        "operator",
        if workflow_focus {
            "Workflow governance and runtime projection.".to_string()
        } else {
            "Live decision-plane and workflow catalog projection.".to_string()
        },
        style_tokens,
        component_refs,
    )
}

fn generate_generic_workbench_viewspec(route: &str) -> ViewSpecV1 {
    let mut style_tokens = BTreeMap::new();
    style_tokens.insert("theme".to_string(), "cortex".to_string());
    style_tokens.insert("context".to_string(), "workbench".to_string());

    let route_title = if route.starts_with('/') {
        let t = route[1..].to_string();
        if t.is_empty() {
            "Home".to_string()
        } else {
            let mut c = t.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    } else {
        route.to_string()
    };

    let component_refs = vec![
        ComponentRef {
            component_id: "route_title".to_string(),
            component_type: "Heading".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(format!("{} View", route_title)),
            )]),
            a11y: None,
            children: vec![],
        },
        ComponentRef {
            component_id: "route_content".to_string(),
            component_type: "Text".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                Value::String(format!(
                    "This A2UI surface for '{}' is under construction.",
                    route
                )),
            )]),
            a11y: None,
            children: vec![],
        },
    ];

    let layout_graph = LayoutGraph {
        nodes: vec![
            LayoutNode {
                node_id: "node_1".to_string(),
                role: "header".to_string(),
                component_ref_id: "route_title".to_string(),
            },
            LayoutNode {
                node_id: "node_2".to_string(),
                role: "content".to_string(),
                component_ref_id: "route_content".to_string(),
            },
        ],
        edges: vec![LayoutEdge {
            from: "node_1".to_string(),
            to: "node_2".to_string(),
            relation: "flows_to".to_string(),
        }],
    };

    ViewSpecV1 {
        schema_version: "1.0.0".to_string(),
        view_spec_id: format!("workbench-{}", route_title.to_lowercase()),
        scope: ViewSpecScope {
            space_id: Some("cortex-web".to_string()),
            route_id: Some(route.to_string()),
            role: Some("operator".to_string()),
        },
        intent: format!("Dynamic UI for {}", route),
        constraints: vec![],
        layout_graph,
        style_tokens,
        component_refs,
        confidence: ViewSpecConfidence {
            score: 0.85,
            rationale: "Server-generated placeholder layout".to_string(),
        },
        lineage: ViewSpecLineage::default(),
        policy: default_viewspec_policy(),
        provenance: ViewSpecProvenance {
            created_by: "cortex-eudaemon".to_string(),
            created_at: now_iso(),
            source_mode: "agent".to_string(),
        },
        lock: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::services::ops_agents::AgentRunRecord;
    use cortex_domain::agent::contracts::{AgentRun, AgentRunStatus};
    use std::path::{Path, PathBuf};

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
        path: PathBuf,
    }

    impl TestTempDir {
        fn new() -> Self {
            let nonce = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("system time")
                .as_nanos();
            let path = std::env::temp_dir().join(format!(
                "workbench-ux-tests-{}-{}",
                std::process::id(),
                nonce
            ));
            std::fs::create_dir_all(&path).expect("create temp dir");
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

    fn component<'a>(view_spec: &'a ViewSpecV1, id: &str) -> &'a ComponentRef {
        view_spec
            .component_refs
            .iter()
            .find(|component| component.component_id == id)
            .expect("component exists")
    }

    fn table_rows(view_spec: &ViewSpecV1, id: &str) -> Vec<Value> {
        component(view_spec, id)
            .props
            .get("rows")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default()
    }

    fn widget_type(component: &ComponentRef) -> Option<&str> {
        component
            .props
            .get("widgetType")
            .and_then(|value| value.as_str())
    }

    async fn render_registered_route(route: &str) -> ViewSpecV1 {
        let headers = HeaderMap::new();
        let registration = registered_workbench_surface(route).expect("registered surface");
        render_registered_workbench_surface(
            registration,
            &headers,
            "nostra-governance-v0",
            "test-actor",
            "operator",
            None,
            "navigate",
            "comfortable",
        )
        .await
    }

    fn write_agent_fixture(root: &Path, space_id: &str, run_id: &str) {
        let runs_dir = root.join("agent_runs");
        std::fs::create_dir_all(&runs_dir).expect("agent runs dir");
        let record = AgentRunRecord {
            run: AgentRun {
                run_id: run_id.to_string(),
                workflow_id: "wf-test".to_string(),
                space_id: space_id.to_string(),
                contribution_id: "contribution-test".to_string(),
                agent_id: Some("agent:test".to_string()),
                status: AgentRunStatus::Completed,
                started_at: "2026-03-10T00:00:00Z".to_string(),
                updated_at: "2026-03-10T00:05:00Z".to_string(),
                stream_channel: None,
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
                simulation: None,
                surface_update: None,
                authority_outcome: None,
                authority_level: None,
                execution_id: None,
                attempt_id: None,
                temporal_binding: None,
                shadow_summary: None,
                approval_timeout_seconds: None,
            },
            events: Vec::new(),
            pending_action_target: None,
            approval: None,
        };
        std::fs::write(
            runs_dir.join(format!("{space_id}__{run_id}.json")),
            serde_json::to_vec_pretty(&record).expect("agent record json"),
        )
        .expect("write agent record");
    }

    fn write_artifacts_fixture(root: &Path) {
        std::fs::create_dir_all(root).expect("artifacts root");
        let payload = json!([
            {
                "artifactId": "artifact-alpha",
                "title": "Artifact Alpha",
                "status": "published",
                "updatedAt": "2026-03-10T00:00:00Z",
                "publishedAt": "2026-03-10T00:10:00Z",
                "headRevisionId": "rev-alpha",
                "version": 3,
                "routeId": "/studio",
                "ownerRole": "operator",
                "sourceOfTruth": "store",
                "fallbackActive": false
            }
        ]);
        std::fs::write(
            root.join("artifacts_store.json"),
            serde_json::to_vec_pretty(&payload).expect("artifacts json"),
        )
        .expect("write artifacts store");
    }

    fn write_json_fixture<T: Serialize>(path: &Path, payload: &T) {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).expect("create parent");
        }
        std::fs::write(path, serde_json::to_vec_pretty(payload).expect("json fixture"))
            .expect("write fixture");
    }

    fn write_contribution_run_fixture(root: &Path, space_id: &str, run_id: &str) {
        let runs_dir = root
            .join("_spaces")
            .join(space_id)
            .join(".cortex")
            .join("logs")
            .join("contribution_graph")
            .join("runs");
        std::fs::create_dir_all(&runs_dir).expect("contribution runs dir");
        write_json_fixture(
            &runs_dir.join(format!("{run_id}.json")),
            &json!({
                "schemaVersion": "nostra.dpub.pipeline_run.v1",
                "runId": run_id,
                "mode": "simulate",
                "actorRole": "steward",
                "actorId": "steward:test",
                "startedAt": "2026-03-20T00:00:00Z",
                "finishedAt": "2026-03-20T00:02:00Z",
                "status": "success",
                "durationMs": 120000,
                "graphRootHashAfter": "hash-alpha",
                "artifacts": {
                    "simulate": {
                        "sessionId": "sim-123"
                    }
                }
            }),
        );
    }

    fn write_workflow_fixture(ux_root: &Path, decision_root: &Path, space_id: &str) {
        let scope = cortex_domain::workflow::WorkflowScope {
            space_id: Some(space_id.to_string()),
            route_id: Some("/flows".to_string()),
            role: Some("operator".to_string()),
        };
        let scope_key = cortex_domain::workflow::scope_key(&scope);
        let draft = cortex_domain::workflow::WorkflowDraftV1 {
            schema_version: "1.0.0".to_string(),
            workflow_draft_id: "workflow-draft-alpha".to_string(),
            scope: scope.clone(),
            intent_ref: Some("workflow_intent_alpha".to_string()),
            intent: "Compare two solvers and gate with evaluation".to_string(),
            motif_kind: cortex_domain::workflow::WorkflowMotifKind::ParallelCompare,
            constraints: Vec::new(),
            graph: cortex_domain::workflow::WorkflowGraphV1 {
                nodes: vec![
                    cortex_domain::workflow::WorkflowNodeV1 {
                        node_id: "parallel".to_string(),
                        label: "Parallel Branches".to_string(),
                        kind: cortex_domain::workflow::WorkflowNodeKind::Parallel,
                        reads: vec!["inputs.prompt".to_string()],
                        writes: Vec::new(),
                        evidence_outputs: Vec::new(),
                        authority_requirements: Vec::new(),
                        checkpoint_policy: None,
                        loop_policy: None,
                        subflow_ref: None,
                        config: json!({}),
                    },
                    cortex_domain::workflow::WorkflowNodeV1 {
                        node_id: "solver_a".to_string(),
                        label: "Solver A".to_string(),
                        kind: cortex_domain::workflow::WorkflowNodeKind::CapabilityCall,
                        reads: vec!["inputs.prompt".to_string()],
                        writes: vec!["artifacts.solver_a".to_string()],
                        evidence_outputs: vec!["evidence.solver_a".to_string()],
                        authority_requirements: vec!["capability:solver".to_string()],
                        checkpoint_policy: None,
                        loop_policy: None,
                        subflow_ref: None,
                        config: json!({ "agent": "solver-a" }),
                    },
                    cortex_domain::workflow::WorkflowNodeV1 {
                        node_id: "solver_b".to_string(),
                        label: "Solver B".to_string(),
                        kind: cortex_domain::workflow::WorkflowNodeKind::CapabilityCall,
                        reads: vec!["inputs.prompt".to_string()],
                        writes: vec!["artifacts.solver_b".to_string()],
                        evidence_outputs: vec!["evidence.solver_b".to_string()],
                        authority_requirements: vec!["capability:solver".to_string()],
                        checkpoint_policy: None,
                        loop_policy: None,
                        subflow_ref: None,
                        config: json!({ "agent": "solver-b" }),
                    },
                    cortex_domain::workflow::WorkflowNodeV1 {
                        node_id: "judge".to_string(),
                        label: "Judge".to_string(),
                        kind: cortex_domain::workflow::WorkflowNodeKind::EvaluationGate,
                        reads: vec![
                            "artifacts.solver_a".to_string(),
                            "artifacts.solver_b".to_string(),
                        ],
                        writes: vec!["evaluation.best".to_string()],
                        evidence_outputs: vec!["evidence.judge".to_string()],
                        authority_requirements: Vec::new(),
                        checkpoint_policy: None,
                        loop_policy: None,
                        subflow_ref: None,
                        config: json!({ "benchmarkProfile": "compare-alpha" }),
                    },
                    cortex_domain::workflow::WorkflowNodeV1 {
                        node_id: "terminal".to_string(),
                        label: "Terminal".to_string(),
                        kind: cortex_domain::workflow::WorkflowNodeKind::Terminal,
                        reads: vec!["evaluation.best".to_string()],
                        writes: Vec::new(),
                        evidence_outputs: Vec::new(),
                        authority_requirements: Vec::new(),
                        checkpoint_policy: None,
                        loop_policy: None,
                        subflow_ref: None,
                        config: json!({}),
                    },
                ],
                edges: vec![
                    cortex_domain::workflow::WorkflowEdgeV1 {
                        edge_id: "edge_parallel_a".to_string(),
                        from: "parallel".to_string(),
                        to: "solver_a".to_string(),
                        relation: "fans_out".to_string(),
                    },
                    cortex_domain::workflow::WorkflowEdgeV1 {
                        edge_id: "edge_parallel_b".to_string(),
                        from: "parallel".to_string(),
                        to: "solver_b".to_string(),
                        relation: "fans_out".to_string(),
                    },
                    cortex_domain::workflow::WorkflowEdgeV1 {
                        edge_id: "edge_a".to_string(),
                        from: "solver_a".to_string(),
                        to: "judge".to_string(),
                        relation: "flows_to".to_string(),
                    },
                    cortex_domain::workflow::WorkflowEdgeV1 {
                        edge_id: "edge_b".to_string(),
                        from: "solver_b".to_string(),
                        to: "judge".to_string(),
                        relation: "flows_to".to_string(),
                    },
                    cortex_domain::workflow::WorkflowEdgeV1 {
                        edge_id: "edge_terminal".to_string(),
                        from: "judge".to_string(),
                        to: "terminal".to_string(),
                        relation: "flows_to".to_string(),
                    },
                ],
            },
            context_contract: cortex_domain::workflow::ContextContractV1::default(),
            confidence: cortex_domain::workflow::WorkflowConfidence {
                score: 0.92,
                rationale: "Deterministic fixture".to_string(),
            },
            lineage: cortex_domain::workflow::WorkflowLineage::default(),
            policy: cortex_domain::workflow::WorkflowDraftPolicyV1 {
                recommendation_only: true,
                require_review: true,
                allow_shadow_execution: true,
            },
            provenance: cortex_domain::workflow::WorkflowProvenance {
                created_by: "fixture".to_string(),
                created_at: "2026-03-11T10:00:00Z".to_string(),
                source_mode: "hybrid".to_string(),
            },
        };
        let compile_result =
            cortex_domain::workflow::compile_workflow_draft(&draft).expect("compile fixture");
        let definition = cortex_domain::workflow::WorkflowDefinitionV1 {
            schema_version: draft.schema_version.clone(),
            definition_id: "workflow-definition-alpha".to_string(),
            scope: draft.scope.clone(),
            intent_ref: draft.intent_ref.clone(),
            intent: draft.intent.clone(),
            motif_kind: draft.motif_kind.clone(),
            constraints: draft.constraints.clone(),
            graph: draft.graph.clone(),
            context_contract: draft.context_contract.clone(),
            confidence: draft.confidence.clone(),
            lineage: draft.lineage.clone(),
            policy: draft.policy.clone(),
            provenance: draft.provenance.clone(),
            governance_ref: Some(cortex_domain::workflow::WorkflowGovernanceRef {
                gate_level: "release_blocker".to_string(),
                gate_status: "approved".to_string(),
                decision_gate_id: "gate:workflow:test".to_string(),
                replay_contract_ref: "replay:workflow:test".to_string(),
                source_of_truth: "local_mirror".to_string(),
                lineage_id: "lineage:workflow:test".to_string(),
                degraded_reason: None,
                definition_digest: compile_result.digest.clone(),
                binding_digest: "binding-digest-alpha".to_string(),
            }),
            digest: Some(compile_result.digest.clone()),
        };
        let proposal = cortex_domain::workflow::WorkflowProposalEnvelope {
            proposal_id: "workflow-proposal-alpha".to_string(),
            workflow_draft_id: draft.workflow_draft_id.clone(),
            definition_id: definition.definition_id.clone(),
            scope_key: scope_key.clone(),
            proposed_by: "fixture".to_string(),
            rationale: "Promote tested draft".to_string(),
            created_at: "2026-03-11T10:05:00Z".to_string(),
            status: cortex_domain::workflow::WorkflowProposalStatus::Ratified,
            review: Some(cortex_domain::workflow::WorkflowProposalReviewRecord {
                reviewed_by: "reviewer".to_string(),
                reviewed_at: "2026-03-11T10:06:00Z".to_string(),
                summary: "Looks sound".to_string(),
                checks: vec!["digest-stable".to_string()],
                approved: true,
            }),
            decision: Some(cortex_domain::workflow::WorkflowProposalDecisionRecord {
                decided_by: "operator".to_string(),
                decided_at: "2026-03-11T10:07:00Z".to_string(),
                decision: "ratified".to_string(),
                rationale: "Make active".to_string(),
            }),
            governance_ref: definition.governance_ref.clone(),
        };
        let active_scope = cortex_domain::workflow::WorkflowScopeAdoptionRecord {
            scope_key: scope_key.clone(),
            active_definition_id: definition.definition_id.clone(),
            adopted_from_proposal_id: proposal.proposal_id.clone(),
            adopted_at: "2026-03-11T10:08:00Z".to_string(),
            adopted_by: "operator".to_string(),
        };

        write_json_fixture(
            &ux_root.join("cortex/workflows/drafts/current/index.json"),
            &json!({
                draft.workflow_draft_id.clone(): {
                    "workflowDraftId": draft.workflow_draft_id,
                    "scopeKey": scope_key,
                    "updatedAt": "2026-03-11T10:00:00Z"
                }
            }),
        );
        write_json_fixture(
            &ux_root.join(format!(
                "cortex/workflows/drafts/current/{}/workflow-draft-alpha.json",
                cortex_domain::workflow::scope_key(&scope)
            )),
            &draft,
        );
        write_json_fixture(
            &ux_root.join("cortex/workflows/drafts/proposals/index.json"),
            &json!({
                proposal.proposal_id.clone(): {
                    "proposalId": proposal.proposal_id,
                    "scopeKey": cortex_domain::workflow::scope_key(&scope),
                    "updatedAt": "2026-03-11T10:07:00Z"
                }
            }),
        );
        write_json_fixture(
            &ux_root.join(format!(
                "cortex/workflows/drafts/proposals/{}/workflow-proposal-alpha.json",
                cortex_domain::workflow::scope_key(&scope)
            )),
            &proposal,
        );
        write_json_fixture(
            &ux_root.join("cortex/workflows/definitions/current/index.json"),
            &json!({
                definition.definition_id.clone(): {
                    "workflowDraftId": definition.definition_id,
                    "scopeKey": cortex_domain::workflow::scope_key(&scope),
                    "updatedAt": "2026-03-11T10:08:00Z"
                }
            }),
        );
        write_json_fixture(
            &ux_root.join(format!(
                "cortex/workflows/definitions/current/{}/workflow-definition-alpha.json",
                cortex_domain::workflow::scope_key(&scope)
            )),
            &json!({
                "definition": definition,
                "compileResult": compile_result
            }),
        );
        write_json_fixture(
            &ux_root.join("cortex/workflows/definitions/active/index.json"),
            &json!({
                cortex_domain::workflow::scope_key(&scope): active_scope
            }),
        );

        let snapshot = cortex_domain::agent::contracts::TemporalBridgeRunSnapshot {
            schema_version: "1.0.0".to_string(),
            run_id: "workflow-instance-alpha".to_string(),
            workflow_id: "workflow-definition-alpha".to_string(),
            space_id: space_id.to_string(),
            contribution_id: "workflow-definition:workflow-definition-alpha".to_string(),
            status: "waiting_approval".to_string(),
            started_at: "2026-03-11T10:09:00Z".to_string(),
            updated_at: "2026-03-11T10:10:00Z".to_string(),
            sequence: 1,
            events: vec![cortex_domain::agent::contracts::AgentRunEvent {
                event_type: "workflow_started".to_string(),
                run_id: "workflow-instance-alpha".to_string(),
                space_id: space_id.to_string(),
                timestamp: "2026-03-11T10:09:00Z".to_string(),
                sequence: 1,
                payload: json!({ "definitionId": "workflow-definition-alpha" }),
            }],
            simulation: None,
            surface_update: None,
            authority_outcome: None,
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
            provider_trace: None,
            approval_timeout_seconds: 1800,
            terminal: false,
            error: None,
        };
        let instance = cortex_domain::workflow::WorkflowInstanceV1 {
            schema_version: "1.0.0".to_string(),
            instance_id: "workflow-instance-alpha".to_string(),
            definition_id: "workflow-definition-alpha".to_string(),
            binding_id: "workflow-binding-alpha".to_string(),
            status: cortex_domain::workflow::WorkflowInstanceStatus::Queued,
            scope,
            created_at: "2026-03-11T10:09:00Z".to_string(),
            updated_at: "2026-03-11T10:09:00Z".to_string(),
            definition_digest: "definition-digest-alpha".to_string(),
            binding_digest: "binding-digest-alpha".to_string(),
            source_of_truth: "local_durable_worker_v1".to_string(),
            replay_contract_ref: Some("replay:workflow:test".to_string()),
            lineage_id: Some("lineage:workflow:test".to_string()),
            degraded_reason: None,
        };
        write_json_fixture(
            &decision_root.join("temporal_bridge_runtime/instances/workflow-instance-alpha.json"),
            &json!({
                "instance": instance,
                "checkpointPolicy": cortex_domain::workflow::WorkflowCheckpointPolicyV1 {
                    resume_allowed: true,
                    cancel_allowed: true,
                    pause_allowed: true,
                    timeout_seconds: Some(1800),
                }
            }),
        );
        write_json_fixture(
            &decision_root.join("temporal_bridge_runtime/snapshots/workflow-instance-alpha.json"),
            &snapshot,
        );
    }

    #[tokio::test]
    async fn logs_viewspec_stream_rows_include_click_targets() {
        let view_spec = generate_logs_viewspec(
            "operator",
            Some("log_stream:siq_gate_summary_latest:cursor:0"),
            "navigate",
            "comfortable",
        )
        .await;
        let rows = table_rows(&view_spec, "logs_streams_table");
        let first = rows.first().and_then(|row| row.as_object()).expect("row");
        assert!(
            first
                .get("_href")
                .and_then(|value| value.as_str())
                .is_some()
        );
    }

    #[tokio::test]
    async fn agents_viewspec_rows_include_href_metadata() {
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_DECISION_SURFACE_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        write_agent_fixture(temp.path(), "space-alpha", "run-001");

        let view_spec = generate_agents_viewspec(
            "space-alpha",
            Some("agent_run:run-001"),
            "navigate",
            "comfortable",
        )
        .await;
        let rows = table_rows(&view_spec, "agents_runs_table");
        let first = rows.first().and_then(|row| row.as_object()).expect("row");
        assert_eq!(
            first.get("_href").and_then(|value| value.as_str()),
            Some("/agents?node_id=agent_run:run-001")
        );
    }

    #[tokio::test]
    async fn contributions_viewspec_projects_graph_and_agent_deep_links() {
        let workspace = TestTempDir::new();
        let decision = TestTempDir::new();
        let _workspace_guard = EnvVarGuard::set(
            "NOSTRA_WORKSPACE_ROOT",
            workspace.path().display().to_string().as_str(),
        );
        let _decision_guard = EnvVarGuard::set(
            "NOSTRA_DECISION_SURFACE_LOG_DIR",
            decision.path().display().to_string().as_str(),
        );

        write_contribution_run_fixture(workspace.path(), "space-alpha", "graph-run-001");
        write_agent_fixture(decision.path(), "space-alpha", "run-001");

        let view_spec = generate_contributions_viewspec(
            "space-alpha",
            Some("agent_run:run-001"),
            "navigate",
            "comfortable",
        )
        .await;

        let graph_rows = table_rows(&view_spec, "contributions_graph_runs_table");
        let graph_first = graph_rows
            .first()
            .and_then(|row| row.as_object())
            .expect("graph row");
        assert_eq!(
            graph_first.get("_href").and_then(|value| value.as_str()),
            Some("/contributions?run_id=graph-run-001")
        );

        let agent_rows = table_rows(&view_spec, "contributions_agent_runs_table");
        let agent_first = agent_rows
            .first()
            .and_then(|row| row.as_object())
            .expect("agent row");
        assert_eq!(
            agent_first.get("_href").and_then(|value| value.as_str()),
            Some("/contributions?node_id=agent_run:run-001")
        );

        let selected = component(&view_spec, "contributions_selected_run_table");
        let selected_rows = selected
            .props
            .get("rows")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(selected_rows.iter().any(|row| {
            row.get("Field").and_then(|value| value.as_str()) == Some("Run ID")
                && row.get("Value").and_then(|value| value.as_str()) == Some("run-001")
        }));
    }

    #[tokio::test]
    async fn artifacts_viewspec_rows_include_href_metadata() {
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        write_artifacts_fixture(temp.path());

        let view_spec =
            generate_artifacts_viewspec(Some("artifact:artifact-alpha"), "navigate", "comfortable")
                .await;
        let rows = table_rows(&view_spec, "artifacts_table");
        let first = rows.first().and_then(|row| row.as_object()).expect("row");
        assert_eq!(
            first.get("_href").and_then(|value| value.as_str()),
            Some("/artifacts?node_id=artifact:artifact-alpha")
        );
    }

    #[tokio::test]
    async fn artifacts_viewspec_selected_artifact_compiles_with_accessible_action() {
        let temp = TestTempDir::new();
        let _guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp.path().display().to_string().as_str(),
        );
        write_artifacts_fixture(temp.path());

        let view_spec =
            generate_artifacts_viewspec(Some("artifact:artifact-alpha"), "navigate", "comfortable")
                .await;
        let compile = compile_viewspec_to_render_surface(&view_spec);
        assert!(compile.is_ok(), "selected artifact surface must compile");

        let button = component(&view_spec, "artifacts_open_studio");
        let label = button
            .a11y
            .as_ref()
            .and_then(|a11y| a11y.label.as_deref())
            .expect("artifacts button a11y label");
        assert_eq!(label, "Open selected artifact in Studio");
    }

    #[tokio::test]
    async fn flows_viewspec_rows_include_href_metadata() {
        let temp_ux = TestTempDir::new();
        let temp_decision = TestTempDir::new();
        let _ux_guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp_ux.path().display().to_string().as_str(),
        );
        let _decision_guard = EnvVarGuard::set(
            "NOSTRA_DECISION_SURFACE_LOG_DIR",
            temp_decision.path().display().to_string().as_str(),
        );
        write_workflow_fixture(temp_ux.path(), temp_decision.path(), "nostra-governance-v0");

        let view_spec = timeout(
            Duration::from_secs(2),
            generate_flows_viewspec(
                "/workflows",
                "nostra-governance-v0",
                None,
                "navigate",
                "comfortable",
            ),
        )
        .await
        .expect("flows viewspec should not block");
        let rows = table_rows(&view_spec, "flows_workflow_drafts_table");
        let first = rows.first().and_then(|row| row.as_object()).expect("row");
        assert!(
            first
                .get("_href")
                .and_then(|value| value.as_str())
                .is_some()
        );
    }

    #[tokio::test]
    async fn flows_viewspec_projects_workflow_governance_and_runtime_rows() {
        let temp_ux = TestTempDir::new();
        let temp_decision = TestTempDir::new();
        let _ux_guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp_ux.path().display().to_string().as_str(),
        );
        let _decision_guard = EnvVarGuard::set(
            "NOSTRA_DECISION_SURFACE_LOG_DIR",
            temp_decision.path().display().to_string().as_str(),
        );
        write_workflow_fixture(temp_ux.path(), temp_decision.path(), "nostra-governance-v0");

        let view_spec = timeout(
            Duration::from_secs(2),
            generate_flows_viewspec(
                "/workflows",
                "nostra-governance-v0",
                Some("workflow_instance:workflow-instance-alpha"),
                "navigate",
                "comfortable",
            ),
        )
        .await
        .expect("workflow flows viewspec should not block");

        let draft_rows = table_rows(&view_spec, "flows_workflow_drafts_table");
        let proposal_rows = table_rows(&view_spec, "flows_workflow_proposals_table");
        let definition_rows = table_rows(&view_spec, "flows_workflow_definitions_table");
        let instance_rows = table_rows(&view_spec, "flows_workflow_instances_table");

        assert_eq!(draft_rows.len(), 1);
        assert_eq!(proposal_rows.len(), 1);
        assert_eq!(definition_rows.len(), 1);
        assert_eq!(instance_rows.len(), 1);
        assert_eq!(
            draft_rows[0]
                .get("_href")
                .and_then(|value| value.as_str()),
            Some("/workflows?node_id=workflow_draft:workflow-draft-alpha")
        );
        assert_eq!(
            instance_rows[0]
                .get("Status")
                .and_then(|value| value.as_str()),
            Some("waitingcheckpoint")
        );
        let selected = component(&view_spec, "flows_selected_table");
        let selected_rows = selected
            .props
            .get("rows")
            .and_then(|value| value.as_array())
            .cloned()
            .unwrap_or_default();
        assert!(selected_rows.iter().any(|row| {
            row.get("Field").and_then(|value| value.as_str()) == Some("Binding ID")
                && row.get("Value").and_then(|value| value.as_str())
                    == Some("workflow-binding-alpha")
        }));
        assert_eq!(
            component(&view_spec, "flows_selected_instance_trace")
                .props
                .get("href")
                .and_then(|value| value.as_str()),
            Some("/api/cortex/workflow-instances/workflow-instance-alpha/trace")
        );
    }

    #[tokio::test]
    async fn workflows_viewspec_emits_workflow_specific_widgets() {
        let temp_ux = TestTempDir::new();
        let temp_decision = TestTempDir::new();
        let _ux_guard = EnvVarGuard::set(
            "NOSTRA_CORTEX_UX_LOG_DIR",
            temp_ux.path().display().to_string().as_str(),
        );
        let _decision_guard = EnvVarGuard::set(
            "NOSTRA_DECISION_SURFACE_LOG_DIR",
            temp_decision.path().display().to_string().as_str(),
        );
        write_workflow_fixture(temp_ux.path(), temp_decision.path(), "nostra-governance-v0");

        let view_spec = timeout(
            Duration::from_secs(2),
            generate_flows_viewspec(
                "/workflows",
                "nostra-governance-v0",
                Some("workflow_definition:workflow-definition-alpha"),
                "navigate",
                "comfortable",
            ),
        )
        .await
        .expect("workflow viewspec should not block");

        assert_eq!(
            widget_type(component(&view_spec, "flows_workflow_summary_strip")),
            Some("WorkflowSummaryStrip")
        );
        assert_eq!(
            widget_type(component(&view_spec, "flows_workflow_projection_preview")),
            Some("WorkflowProjectionPreview")
        );
        assert_eq!(
            widget_type(component(&view_spec, "flows_workflow_instance_timeline")),
            Some("WorkflowInstanceTimeline")
        );
        assert_eq!(
            widget_type(component(&view_spec, "flows_selected_status_badge")),
            Some("WorkflowStatusBadge")
        );
    }

    #[tokio::test]
    async fn gate_viewspecs_include_heap_and_logs_buttons() {
        let siq = generate_siq_viewspec("nostra-governance-v0", "navigate", "comfortable").await;
        let testing =
            generate_testing_viewspec("nostra-governance-v0", "navigate", "comfortable").await;

        let siq_logs = component(&siq, "siq_open_logs");
        let testing_logs = component(&testing, "testing_open_logs");
        let siq_heap = component(&siq, "siq_open_heap");
        let testing_heap = component(&testing, "testing_open_heap");

        assert_eq!(
            siq_logs
                .props
                .get("href")
                .and_then(|value: &Value| value.as_str()),
            Some("/logs?node_id=log_stream:siq_gate_summary_latest:cursor:0")
        );
        assert_eq!(
            testing_logs
                .props
                .get("href")
                .and_then(|value: &Value| value.as_str()),
            Some("/logs?node_id=log_stream:testing_gate_summary_latest:cursor:0")
        );
        assert_eq!(
            siq_heap
                .props
                .get("href")
                .and_then(|value: &Value| value.as_str()),
            Some("/heap?focus=gate_summary_siq_latest")
        );
        assert_eq!(
            testing_heap
                .props
                .get("href")
                .and_then(|value: &Value| value.as_str()),
            Some("/heap?focus=gate_summary_testing_latest")
        );
    }

    #[test]
    fn default_shell_layout_routes_are_registered_in_surface_registry() {
        for entry in cortex_domain::ux::scoring::default_shell_layout_spec()
            .navigation_graph
            .entries
        {
            assert!(
                registered_workbench_surface(&entry.route_id).is_some(),
                "missing registered workbench surface for promoted route {}",
                entry.route_id
            );
        }
    }

    #[test]
    fn production_promoted_routes_do_not_use_generic_surface_kind() {
        for route in [
            "/system",
            "/system/siq",
            "/testing",
            "/logs",
            "/agents",
            "/contributions",
            "/artifacts",
            "/spaces",
            "/flows",
            "/workflows",
            "/studio",
            "/heap",
            "/labs",
        ] {
            let registration = registered_workbench_surface(route).expect("registered surface");
            assert!(
                registration.kind != WorkbenchSurfaceKind::Generic,
                "production route {} must not use generic fallback registration",
                route
            );
            assert!(
                !registration.allow_generic_surface,
                "production route {} must not allow generic fallback",
                route
            );
        }
    }

    #[tokio::test]
    async fn execution_canvas_route_renders_spatial_plane_surface() {
        let view_spec = render_registered_route("/labs/execution-canvas").await;
        assert_eq!(view_spec.view_spec_id, "workbench-labs-execution-canvas");
        let plane = component(&view_spec, "execution_canvas_plane");
        assert_eq!(plane.component_type, "SpatialPlane");
        assert_eq!(plane.props.get("surface_class").and_then(Value::as_str), Some("execution"));
        assert_eq!(
            plane.props
                .get("layout_ref")
                .and_then(Value::as_object)
                .and_then(|layout| layout.get("view_spec_id"))
                .and_then(Value::as_str),
            Some("workbench-labs-execution-canvas")
        );
    }

    #[tokio::test]
    async fn registered_non_generic_surfaces_use_only_allowlisted_widget_types() {
        let allowed_component_types = [
            "Heading",
            "Text",
            "Button",
            "Container",
            "Card",
            "Markdown",
            "SpatialPlane",
            "TextField",
            "Tabs",
        ];
        let allowed_widget_types = [
            "AlertBanner",
            "BrandingLabsWidget",
            "CapabilityMap",
            "Card",
            "DataTable",
            "Grid",
            "HeapBlockCard",
            "HeapCanvas",
            "MetricCard",
            "WorkflowInstanceTimeline",
            "WorkflowProjectionPreview",
            "WorkflowStatusBadge",
            "WorkflowSummaryStrip",
        ];
        let forbidden_widget_types = [
            "A2UISynthesisSpace",
            "PlaygroundSurface",
            "SiqScorecard",
            "scorecard",
        ];

        for registration in WORKBENCH_SURFACE_REGISTRY
            .iter()
            .filter(|entry| !entry.allow_generic_surface)
        {
            let view_spec = render_registered_route(registration.route_id).await;
            for component in &view_spec.component_refs {
                assert!(
                    allowed_component_types.contains(&component.component_type.as_str()),
                    "route {} emitted non-allowlisted component_type {}",
                    registration.route_id,
                    component.component_type
                );
                if let Some(widget_type) = widget_type(component) {
                    assert!(
                        !forbidden_widget_types.contains(&widget_type),
                        "route {} emitted forbidden widgetType {}",
                        registration.route_id,
                        widget_type
                    );
                    assert!(
                        allowed_widget_types.contains(&widget_type),
                        "route {} emitted unknown widgetType {}",
                        registration.route_id,
                        widget_type
                    );
                }
            }
        }
    }
}
