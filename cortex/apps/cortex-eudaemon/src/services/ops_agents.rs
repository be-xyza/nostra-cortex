use crate::gateway::server::workspace_root;
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
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub auth_mode: Option<String>,
    #[serde(default)]
    pub response_id: Option<String>,
    #[serde(default)]
    pub prompt_template_artifact_id: Option<String>,
    #[serde(default)]
    pub prompt_template_revision_id: Option<String>,
    #[serde(default)]
    pub prompt_execution_artifact_id: Option<String>,
    #[serde(default)]
    pub parent_run_id: Option<String>,
    #[serde(default)]
    pub child_run_ids: Vec<String>,
    #[serde(default)]
    pub provider_trace_summary: Option<serde_json::Value>,
    pub status: String,
    pub started_at: String,
    pub updated_at: String,
    pub authority_level: Option<String>,
    pub requires_review: bool,
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
    let runs_dir = agent_runs_dir();
    if !runs_dir.exists() {
        return Ok(summaries);
    }
    let entries = fs::read_dir(runs_dir).map_err(|err| err.to_string())?;
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
            provider: record.run.provider.clone(),
            model: record.run.model.clone(),
            auth_mode: record.run.auth_mode.clone(),
            response_id: record.run.response_id.clone(),
            prompt_template_artifact_id: record.run.prompt_template_artifact_id.clone(),
            prompt_template_revision_id: record.run.prompt_template_revision_id.clone(),
            prompt_execution_artifact_id: record.run.prompt_execution_artifact_id.clone(),
            parent_run_id: record.run.parent_run_id.clone(),
            child_run_ids: record.run.child_run_ids.clone(),
            provider_trace_summary: record.run.provider_trace_summary.clone(),
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
