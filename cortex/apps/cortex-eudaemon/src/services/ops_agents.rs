use crate::gateway::server::workspace_root;
use chrono::{DateTime, Utc};
use cortex_domain::agent::contracts::{ActionTarget, AgentRun, AgentRunEvent};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentContributionApprovalRequest {
    pub decision: String,
    #[serde(default)]
    pub rationale: Option<String>,
    pub actor: String,
    #[serde(default)]
    #[serde(alias = "decision_ref")]
    pub decision_ref: Option<String>,
    #[serde(default)]
    #[serde(alias = "actor_principal")]
    pub actor_principal: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentRunRecord {
    #[serde(flatten)]
    pub run: AgentRun,
    #[serde(default)]
    pub events: Vec<AgentRunEvent>,
    #[serde(default)]
    pub pending_action_target: Option<ActionTarget>,
    #[serde(default)]
    pub approval: Option<AgentContributionApprovalRequest>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct AgentRunSummaryResponse {
    pub run_id: String,
    pub workflow_id: String,
    pub space_id: String,
    pub contribution_id: String,
    pub agent_id: Option<String>,
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub authority_level: Option<String>,
    pub requires_review: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkRouterStatusResponse {
    pub service: String,
    pub mode: String,
    pub max_dispatch_level: String,
    pub mutation_allowed: bool,
    pub live_transport_enabled: bool,
    pub health: String,
    pub pending_count: usize,
    pub exported_count: usize,
    pub outbox_envelope_count: usize,
    pub unknown_response_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_observed_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evidence_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evidence_status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_evidence_finished_at: Option<String>,
    pub authority: WorkRouterAuthoritySummary,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkRouterAuthoritySummary {
    pub source_mutation_allowed: bool,
    pub runtime_mutation_allowed: bool,
    pub forbidden_actions_confirmed: Vec<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkRouterDispatchQueueResponse {
    pub pending: Vec<WorkRouterPendingDispatchSummary>,
    pub unknowns: Vec<WorkRouterUnknownResponseSummary>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkRouterPendingDispatchSummary {
    pub run_id: String,
    pub status: String,
    pub task_ref: Option<String>,
    pub route: Option<String>,
    pub risk_level: Option<String>,
    pub max_level: Option<String>,
    pub transport_kind: Option<String>,
    pub request_id: Option<String>,
    pub channel_ref: Option<String>,
    pub created_at: Option<String>,
    pub started_at: Option<String>,
    pub message_preview: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkRouterUnknownResponseSummary {
    pub unknown_id: String,
    pub raw_text: String,
    pub normalized_text: String,
    pub status: String,
    pub created_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proposed_mapping: Option<serde_json::Value>,
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

fn agent_runs_dir() -> PathBuf {
    decision_surface_log_dir().join("agent_runs")
}

fn work_router_log_root() -> PathBuf {
    std::env::var("WORK_ROUTER_LOG_ROOT")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("work_router"))
}

fn count_json_files(dir: PathBuf) -> usize {
    fs::read_dir(dir)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("json"))
        .count()
}

fn count_pending_work_router_runs(root: &PathBuf) -> usize {
    let runs_dir = root.join("runs");
    let Ok(entries) = fs::read_dir(runs_dir) else {
        return 0;
    };
    entries
        .flatten()
        .filter_map(|entry| fs::read_to_string(entry.path().join("run.json")).ok())
        .filter_map(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
        .filter(|value| value.get("status").and_then(|status| status.as_str()) == Some("pending_decision"))
        .count()
}

fn read_json_value(path: PathBuf) -> Option<serde_json::Value> {
    fs::read_to_string(path)
        .ok()
        .and_then(|raw| serde_json::from_str::<serde_json::Value>(&raw).ok())
}

fn as_bool(value: &serde_json::Value, key: &str) -> bool {
    value.get(key).and_then(|item| item.as_bool()).unwrap_or(false)
}

fn as_string(value: &serde_json::Value, key: &str) -> Option<String> {
    value
        .get(key)
        .and_then(|item| item.as_str())
        .map(ToString::to_string)
}

fn as_usize(value: &serde_json::Value, key: &str) -> Option<usize> {
    value
        .get(key)
        .and_then(|item| item.as_u64())
        .and_then(|item| usize::try_from(item).ok())
}

fn nested_string(value: &serde_json::Value, keys: &[&str]) -> Option<String> {
    let mut current = value;
    for key in keys {
        current = current.get(*key)?;
    }
    current.as_str().map(ToString::to_string)
}

fn truncate_preview(raw: &str, max_chars: usize) -> String {
    let trimmed = raw.trim();
    let mut preview = trimmed.chars().take(max_chars).collect::<String>();
    if trimmed.chars().count() > max_chars {
        preview.push_str("...");
    }
    preview
}

fn work_router_health(last_observed_at: Option<&str>) -> String {
    let Some(raw) = last_observed_at else {
        return "unknown".to_string();
    };
    let Ok(observed_at) = DateTime::parse_from_rfc3339(raw).map(|value| value.with_timezone(&Utc)) else {
        return "unknown".to_string();
    };
    let age_seconds = Utc::now().signed_duration_since(observed_at).num_seconds();
    if age_seconds <= 900 {
        "active".to_string()
    } else {
        "stale".to_string()
    }
}

fn agent_run_path(space_id: &str, run_id: &str) -> PathBuf {
    agent_runs_dir().join(format!(
        "{}__{}.json",
        sanitize_fs_component(space_id),
        sanitize_fs_component(run_id)
    ))
}

fn stringify_serde_enum<T: Serialize>(value: &T) -> Option<String> {
    serde_json::to_value(value)
        .ok()
        .and_then(|value| value.as_str().map(|v| v.to_string()))
}

pub(crate) fn load_agent_run(space_id: &str, run_id: &str) -> Result<AgentRunRecord, String> {
    let path = agent_run_path(space_id, run_id);
    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("failed_to_read_agent_run:{}:{err}", path.display()))?;
    serde_json::from_str::<AgentRunRecord>(&raw)
        .map_err(|err| format!("failed_to_parse_agent_run:{}:{err}", path.display()))
}

pub(crate) fn list_agent_runs(
    space_id: &str,
    limit: usize,
) -> Result<Vec<AgentRunSummaryResponse>, String> {
    let normalized_space_id = space_id.trim();
    if normalized_space_id.is_empty() {
        return Err("space_id is required".to_string());
    }

    let clamped_limit = limit.clamp(1, 200);
    let prefix = format!("{}__", sanitize_fs_component(normalized_space_id));
    let mut summaries = Vec::new();
    let entries = fs::read_dir(agent_runs_dir()).map_err(|err| err.to_string())?;
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let filename = path
            .file_name()
            .and_then(|value| value.to_str())
            .unwrap_or_default();
        if !filename.starts_with(&prefix) {
            continue;
        }
        let raw = match fs::read_to_string(&path) {
            Ok(raw) => raw,
            Err(_) => continue,
        };
        let record = match serde_json::from_str::<AgentRunRecord>(&raw) {
            Ok(record) => record,
            Err(_) => continue,
        };
        summaries.push(AgentRunSummaryResponse {
            run_id: record.run.run_id.clone(),
            workflow_id: record.run.workflow_id.clone(),
            space_id: record.run.space_id.clone(),
            contribution_id: record.run.contribution_id.clone(),
            agent_id: record.run.agent_id.clone(),
            status: stringify_serde_enum(&record.run.status)
                .unwrap_or_else(|| "unknown".to_string()),
            started_at: record.run.started_at.clone(),
            updated_at: record.run.updated_at.clone(),
            authority_level: record
                .run
                .authority_level
                .as_ref()
                .and_then(stringify_serde_enum),
            requires_review: record.approval.is_some(),
        });
    }

    summaries.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| b.run_id.cmp(&a.run_id))
    });
    summaries.truncate(clamped_limit);
    Ok(summaries)
}

pub(crate) fn get_work_router_status() -> Result<WorkRouterStatusResponse, String> {
    let root = work_router_log_root();
    let heartbeat_path = root.join("service").join("heartbeat.json");
    let evidence_path = root
        .join("agent_run_evidence")
        .join("workrouter-observe-loop-latest.json");
    let heartbeat = read_json_value(heartbeat_path)
        .ok_or_else(|| "work_router_heartbeat_unavailable".to_string())?;
    let evidence = read_json_value(evidence_path).unwrap_or_else(|| serde_json::json!({}));
    let last_observed_at = as_string(&heartbeat, "observedAt");

    let forbidden_actions_confirmed = evidence
        .get("forbiddenActionsConfirmed")
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.as_str().map(ToString::to_string))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let authority = evidence
        .get("authority")
        .cloned()
        .unwrap_or_else(|| serde_json::json!({}));

    Ok(WorkRouterStatusResponse {
        service: as_string(&heartbeat, "service").unwrap_or_else(|| "cortex-workrouter".to_string()),
        mode: as_string(&heartbeat, "mode").unwrap_or_else(|| "unknown".to_string()),
        max_dispatch_level: as_string(&heartbeat, "maxDispatchLevel")
            .unwrap_or_else(|| "unknown".to_string()),
        mutation_allowed: as_bool(&heartbeat, "mutationAllowed"),
        live_transport_enabled: as_bool(&heartbeat, "liveTransportEnabled"),
        health: work_router_health(last_observed_at.as_deref()),
        pending_count: as_usize(&heartbeat, "pendingCount")
            .unwrap_or_else(|| count_pending_work_router_runs(&root)),
        exported_count: as_usize(&heartbeat, "exportedCount").unwrap_or(0),
        outbox_envelope_count: count_json_files(root.join("outbox")),
        unknown_response_count: count_json_files(root.join("unknown")),
        last_observed_at,
        last_evidence_id: as_string(&evidence, "evidenceId"),
        last_evidence_status: as_string(&evidence, "status"),
        last_evidence_finished_at: as_string(&evidence, "finishedAt"),
        authority: WorkRouterAuthoritySummary {
            source_mutation_allowed: as_bool(&authority, "sourceMutationAllowed"),
            runtime_mutation_allowed: as_bool(&authority, "runtimeMutationAllowed"),
            forbidden_actions_confirmed,
        },
    })
}

pub(crate) fn get_work_router_dispatch_queue() -> Result<WorkRouterDispatchQueueResponse, String> {
    let root = work_router_log_root();
    let mut pending = Vec::new();
    let runs_dir = root.join("runs");
    if let Ok(entries) = fs::read_dir(&runs_dir) {
        for entry in entries.flatten() {
            let run_path = entry.path().join("run.json");
            let Some(run) = read_json_value(run_path) else {
                continue;
            };
            if run.get("status").and_then(|status| status.as_str()) != Some("pending_decision") {
                continue;
            }
            let artifact_refs = run
                .get("artifactRefs")
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let router_bundle_path = as_string(&artifact_refs, "routerBundle").map(PathBuf::from);
            let router_bundle = router_bundle_path.and_then(read_json_value);
            let dispatch_request = router_bundle
                .as_ref()
                .and_then(|bundle| bundle.get("dispatchRequest"))
                .cloned()
                .unwrap_or_else(|| serde_json::json!({}));
            let message_preview = as_string(&artifact_refs, "message")
                .and_then(|path| fs::read_to_string(PathBuf::from(path)).ok())
                .map(|raw| truncate_preview(&raw, 280));

            pending.push(WorkRouterPendingDispatchSummary {
                run_id: as_string(&run, "runId").unwrap_or_else(|| "unknown".to_string()),
                status: as_string(&run, "status").unwrap_or_else(|| "unknown".to_string()),
                task_ref: nested_string(&run, &["summary", "taskRef"]),
                route: nested_string(&run, &["summary", "route"]),
                risk_level: nested_string(&run, &["summary", "riskLevel"]),
                max_level: nested_string(&run, &["authority", "maxLevel"]),
                transport_kind: nested_string(&dispatch_request, &["transport", "kind"])
                    .or_else(|| nested_string(&run, &["authority", "transportKind"])),
                request_id: as_string(&dispatch_request, "requestId"),
                channel_ref: nested_string(&dispatch_request, &["transport", "channelRef"]),
                created_at: as_string(&dispatch_request, "createdAt"),
                started_at: as_string(&run, "startedAt"),
                message_preview,
            });
        }
    }
    pending.sort_by(|a, b| {
        b.started_at
            .cmp(&a.started_at)
            .then_with(|| b.run_id.cmp(&a.run_id))
    });

    let mut unknowns = Vec::new();
    if let Ok(entries) = fs::read_dir(root.join("unknown")) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            let Some(unknown) = read_json_value(path.clone()) else {
                continue;
            };
            let unknown_id = path
                .file_stem()
                .and_then(|value| value.to_str())
                .unwrap_or("unknown")
                .to_string();
            unknowns.push(WorkRouterUnknownResponseSummary {
                unknown_id,
                raw_text: as_string(&unknown, "rawText").unwrap_or_default(),
                normalized_text: as_string(&unknown, "normalizedText").unwrap_or_default(),
                status: as_string(&unknown, "status").unwrap_or_else(|| "needs_routing_review".to_string()),
                created_at: as_string(&unknown, "createdAt"),
                proposed_classification: as_string(&unknown, "recommendedClassification"),
                proposed_mapping: unknown.get("proposedMapping").cloned(),
            });
        }
    }
    unknowns.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.unknown_id.cmp(&a.unknown_id))
    });

    Ok(WorkRouterDispatchQueueResponse { pending, unknowns })
}
