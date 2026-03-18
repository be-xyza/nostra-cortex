use crate::services::gateway_config::gateway_base;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCatalogEntry {
    pub test_id: String,
    pub name: String,
    pub layer: String,
    pub stack: String,
    pub owner: String,
    pub path: String,
    pub command: String,
    pub artifacts: Vec<String>,
    pub gate_level: String,
    pub destructive: bool,
    pub tags: Vec<String>,
    pub last_seen_commit: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestCatalogArtifact {
    pub schema_version: String,
    pub generated_at: String,
    pub tests: Vec<TestCatalogEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestRunResult {
    pub test_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub error_summary: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestRunArtifact {
    pub schema_version: String,
    pub run_id: String,
    pub started_at: String,
    pub finished_at: String,
    pub agent_id: String,
    pub environment: String,
    pub git_commit: String,
    pub results: Vec<TestRunResult>,
    pub artifacts: Vec<String>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestGateFailure {
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestGateCounts {
    pub pass: u64,
    pub fail: u64,
    pub warn: u64,
    pub pending: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TestGateSummaryArtifact {
    pub schema_version: String,
    pub generated_at: String,
    pub mode: String,
    pub catalog_valid: bool,
    pub run_artifacts_valid: bool,
    pub required_blockers_pass: bool,
    pub overall_verdict: String,
    pub latest_run_id: Option<String>,
    pub failures: Vec<TestGateFailure>,
    pub counts: TestGateCounts,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestGateLatestResponse {
    pub summary: TestGateSummaryArtifact,
    pub surface: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TestingHealthResponse {
    pub status: String,
    pub testing_log_dir: String,
    pub schema_version: String,
    pub catalog_exists: bool,
    pub gate_exists: bool,
    pub runs_count: usize,
    pub catalog_last_modified: Option<u64>,
    pub gate_last_modified: Option<u64>,
    pub latest_run_last_modified: Option<u64>,
    pub catalog_fresh: bool,
    pub gate_fresh: bool,
}

pub struct TestingService;

impl TestingService {
    pub async fn get_catalog() -> Result<TestCatalogArtifact, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/testing/catalog", gateway_base()))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch testing catalog: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Testing catalog request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<TestCatalogArtifact>()
            .await
            .map_err(|e| format!("Failed to parse testing catalog: {}", e))
    }

    pub async fn get_runs(
        limit: usize,
        status: Option<&str>,
    ) -> Result<Vec<TestRunArtifact>, String> {
        let client = reqwest::Client::new();
        let mut request = client
            .get(format!("{}/api/testing/runs", gateway_base()))
            .query(&[("limit", limit)]);

        if let Some(status) = status {
            request = request.query(&[("status", status)]);
        }

        let response = request
            .send()
            .await
            .map_err(|e| format!("Failed to fetch testing runs: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Testing runs request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<Vec<TestRunArtifact>>()
            .await
            .map_err(|e| format!("Failed to parse testing runs: {}", e))
    }

    pub async fn get_run(run_id: &str) -> Result<TestRunArtifact, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/testing/runs/{}", gateway_base(), run_id))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch testing run: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!("Testing run request failed ({}): {}", status, body));
        }

        response
            .json::<TestRunArtifact>()
            .await
            .map_err(|e| format!("Failed to parse testing run: {}", e))
    }

    pub async fn get_gates_latest() -> Result<TestGateLatestResponse, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/testing/gates/latest", gateway_base()))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch testing gate summary: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Testing gate summary request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<TestGateLatestResponse>()
            .await
            .map_err(|e| format!("Failed to parse testing gate summary: {}", e))
    }

    pub async fn get_health() -> Result<TestingHealthResponse, String> {
        let client = reqwest::Client::new();
        let response = client
            .get(format!("{}/api/testing/health", gateway_base()))
            .send()
            .await
            .map_err(|e| format!("Failed to fetch testing health: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "no body".to_string());
            return Err(format!(
                "Testing health request failed ({}): {}",
                status, body
            ));
        }

        response
            .json::<TestingHealthResponse>()
            .await
            .map_err(|e| format!("Failed to parse testing health: {}", e))
    }
}
