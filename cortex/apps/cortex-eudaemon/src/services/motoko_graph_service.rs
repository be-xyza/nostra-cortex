use crate::services::gateway_config::gateway_base;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphStatus {
    pub gate_result: String,
    pub posture: String,
    pub authority_mode: String,
    pub runtime_dependency_promotion: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphWorkload {
    pub path: String,
    pub workload: i64,
    pub edge_workload: i64,
    pub seconds_per_edge: f64,
    pub cycles_per_edge: f64,
    pub memory_per_edge_bytes: f64,
    pub walk_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphStability {
    pub path: String,
    pub steps_total: i64,
    pub steps_pass: i64,
    pub steps_blocked: i64,
    pub first_attempt_pass: i64,
    pub retries_consumed: i64,
    pub port_conflicts: i64,
    pub reliability_percent: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphWorkflowStage {
    pub stage: String,
    pub status: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphEvidence {
    pub gate_file: String,
    pub m4_metrics_file: String,
    pub m8_metrics_file: String,
    pub stability_file: String,
    pub analysis_file: String,
    pub m8_pass_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphSnapshot {
    pub schema_version: String,
    pub generated_at: String,
    pub contribution_id: String,
    pub status: MotokoGraphStatus,
    pub workloads: Vec<MotokoGraphWorkload>,
    pub stability: Vec<MotokoGraphStability>,
    pub workflow_stages: Vec<MotokoGraphWorkflowStage>,
    pub evidence: MotokoGraphEvidence,
    pub history_event_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MotokoGraphDecisionEvent {
    pub schema_version: String,
    pub decision_event_id: String,
    pub captured_at: String,
    pub contribution: String,
    pub decision_date: String,
    pub selected_option: String,
    pub rationale: String,
    pub posture_before: String,
    pub posture_after: String,
    pub authority_mode: String,
    pub evidence_refs: Vec<String>,
    pub steward: String,
    pub owner: String,
    pub follow_up_actions: Vec<String>,
    pub source: String,
    pub status: Option<String>,
    pub applied_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringCheck {
    pub name: String,
    pub required: bool,
    pub status: String,
    pub details: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringRunSummary {
    pub schema_version: String,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub gateway_base: String,
    pub overall_status: String,
    pub required_failures: u64,
    pub warnings: u64,
    pub checks: Vec<MonitoringCheck>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringWindowSummary {
    pub total_runs: u64,
    pub pass_runs: u64,
    pub warn_runs: u64,
    pub fail_runs: u64,
    pub reliability_percent: f64,
    pub warning_rate_percent: f64,
    pub required_failure_rate_percent: f64,
    pub gateway_warning_rate_percent: f64,
    pub mean_duration_seconds: f64,
    pub p95_duration_seconds: f64,
    pub last_success_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringWindows {
    #[serde(rename = "7d")]
    pub days_7: MonitoringWindowSummary,
    #[serde(rename = "30d")]
    pub days_30: MonitoringWindowSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringLatest {
    pub run_id: String,
    pub overall_status: String,
    pub required_failures: u64,
    pub warnings: u64,
    pub duration_seconds: f64,
    pub started_at: String,
    pub finished_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MonitoringTrendPayload {
    pub schema_version: String,
    pub generated_at: String,
    pub windows: MonitoringWindows,
    pub latest: MonitoringLatest,
    pub last_applied_decision_event_id: Option<String>,
    pub next_action: String,
    pub advisory_recommendation: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct MotokoGraphHealthResponse {
    pub status: String,
    pub schema_version: String,
    pub kg_log_dir: String,
    pub snapshot_exists: bool,
    pub history_count: usize,
    pub pending_count: usize,
    pub snapshot_last_modified: Option<u64>,
    pub snapshot_fresh: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct DecisionCaptureRequest {
    pub schema_version: String,
    pub contribution: String,
    pub decision_date: String,
    pub selected_option: String,
    pub rationale: String,
    pub posture_before: String,
    pub posture_after: String,
    pub authority_mode: String,
    pub evidence_refs: Vec<String>,
    pub steward: String,
    pub owner: String,
    pub follow_up_actions: Vec<String>,
    pub source: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct DecisionCaptureResponse {
    pub decision_event_id: String,
    pub status: String,
    pub path: String,
}

pub struct MotokoGraphService;

impl MotokoGraphService {
    pub async fn get_snapshot() -> Result<MotokoGraphSnapshot, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/kg/motoko-graph/snapshot", gateway_base()))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch motoko-graph snapshot: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Motoko-graph snapshot request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<MotokoGraphSnapshot>()
            .await
            .map_err(|e| format!("Failed to parse motoko-graph snapshot: {}", e))
    }

    pub async fn get_decision_history() -> Result<Vec<MotokoGraphDecisionEvent>, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/kg/motoko-graph/decision-history",
                gateway_base()
            ))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch decision history: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Motoko-graph decision history request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<Vec<MotokoGraphDecisionEvent>>()
            .await
            .map_err(|e| format!("Failed to parse decision history: {}", e))
    }

    pub async fn get_health() -> Result<MotokoGraphHealthResponse, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/kg/motoko-graph/health", gateway_base()))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch motoko-graph health: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Motoko-graph health request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<MotokoGraphHealthResponse>()
            .await
            .map_err(|e| format!("Failed to parse motoko-graph health: {}", e))
    }

    pub async fn get_monitoring_trends() -> Result<MonitoringTrendPayload, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/kg/motoko-graph/monitoring-trends",
                gateway_base()
            ))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch monitoring trends: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Motoko-graph monitoring trends request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<MonitoringTrendPayload>()
            .await
            .map_err(|e| format!("Failed to parse monitoring trends: {}", e))
    }

    pub async fn get_monitoring_runs(limit: usize) -> Result<Vec<MonitoringRunSummary>, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!(
                "{}/api/kg/motoko-graph/monitoring-runs?limit={}",
                gateway_base(),
                limit
            ))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch monitoring runs: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Motoko-graph monitoring runs request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<Vec<MonitoringRunSummary>>()
            .await
            .map_err(|e| format!("Failed to parse monitoring runs: {}", e))
    }

    pub async fn capture_decision(
        payload: &DecisionCaptureRequest,
    ) -> Result<DecisionCaptureResponse, String> {
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}/api/kg/motoko-graph/decision-capture",
                gateway_base()
            ))
            .json(payload)
            .send()
            .await
            .map_err(|e| format!("Failed to submit decision capture: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Decision capture request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<DecisionCaptureResponse>()
            .await
            .map_err(|e| format!("Failed to parse decision capture response: {}", e))
    }
}
