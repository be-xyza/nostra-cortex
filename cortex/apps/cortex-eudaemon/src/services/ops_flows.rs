use axum::{body::to_bytes, extract::Path, response::IntoResponse};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkflowAutomationDescriptor {
    pub automation_key: String,
    pub enabled: bool,
    pub paused: bool,
    pub interval_secs: u64,
    pub active_workflow_id: Option<String>,
    pub last_workflow_id: Option<String>,
    pub last_run_at: Option<String>,
    pub last_status: Option<String>,
    pub can_run_now: bool,
    pub can_pause: bool,
    pub can_resume: bool,
    pub pause_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct WorkflowCatalogEntry {
    pub name: String,
    pub path: String,
    pub source: String,
    pub status: String,
    pub description: Option<String>,
    pub launch_template: Option<String>,
    pub read_only: bool,
    pub automation: Option<WorkflowAutomationDescriptor>,
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

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecisionSurfaceEnvelope {
    pub surface_id: String,
    pub workflow_id: String,
    pub mutation_id: String,
    pub status: String,
    pub required_actions: Vec<String>,
    pub evidence_refs: Vec<String>,
    pub last_updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_of_truth: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub policy_version: Option<u64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub degraded_reason: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_reason: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub payload: Option<serde_json::Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DecisionPlaneResponse {
    pub space_id: String,
    pub surfaces: Vec<DecisionSurfaceEnvelope>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digest: Option<DecisionSurfaceEnvelope>,
}

async fn response_to_typed<T: DeserializeOwned>(
    response: axum::response::Response,
) -> Result<T, String> {
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 2 * 1024 * 1024)
        .await
        .map_err(|err| err.to_string())?;
    if !status.is_success() {
        let body = String::from_utf8_lossy(&bytes);
        return Err(format!("http_{}:{body}", status.as_u16()));
    }
    serde_json::from_slice::<T>(&bytes).map_err(|err| err.to_string())
}

async fn fetch_worker_acp_status() -> Option<WorkerAcpAutomationStatus> {
    let response = reqwest::Client::new()
        .get("http://127.0.0.1:3003/automations/acp/status")
        .timeout(Duration::from_millis(250))
        .send()
        .await
        .ok()?;
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
        Some(s) => Some(WorkflowAutomationDescriptor {
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
            can_run_now: s.enabled && !s.paused,
            can_pause: s.enabled && !s.paused,
            can_resume: s.enabled && s.paused,
            pause_reason: if s.paused {
                Some("Paused by operator or policy.".to_string())
            } else {
                None
            },
        }),
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

pub(crate) async fn load_workflow_catalog() -> Result<Vec<WorkflowCatalogEntry>, String> {
    let mut catalog = crate::services::file_system_service::FileSystemService::list_workflows()
        .into_iter()
        .map(|flow| WorkflowCatalogEntry {
            name: flow.name,
            path: flow.path,
            source: "filesystem".to_string(),
            status: "available".to_string(),
            description: Some("Local workflow file".to_string()),
            launch_template: None,
            read_only: false,
            automation: None,
        })
        .collect::<Vec<_>>();

    let worker_status = fetch_worker_acp_status().await;
    catalog.push(build_acp_native_entry(worker_status));
    Ok(catalog)
}

pub(crate) async fn load_decision_plane(space_id: &str) -> Result<DecisionPlaneResponse, String> {
    response_to_typed(
        crate::gateway::server::get_system_decision_plane(Path(space_id.to_string()))
            .await
            .into_response(),
    )
    .await
}
