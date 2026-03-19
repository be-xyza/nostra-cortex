use crate::services::gateway_config::gateway_base;
use crate::services::siq_types::{
    SiqCoverage, SiqDependencyClosure, SiqGateSummary, SiqGraphProjection, SiqHealth,
    SiqRunArtifact,
};
use serde::de::DeserializeOwned;

pub struct SiqService;

impl SiqService {
    pub async fn get_coverage() -> Result<SiqCoverage, String> {
        get_json("/api/system/siq/coverage").await
    }

    pub async fn get_dependency_closure() -> Result<SiqDependencyClosure, String> {
        get_json("/api/system/siq/dependency-closure").await
    }

    pub async fn get_gates_latest() -> Result<SiqGateSummary, String> {
        get_json("/api/system/siq/gates/latest").await
    }

    pub async fn get_graph_projection() -> Result<SiqGraphProjection, String> {
        get_json("/api/system/siq/graph-projection").await
    }

    pub async fn get_runs(limit: usize) -> Result<Vec<SiqRunArtifact>, String> {
        get_json(&format!("/api/system/siq/runs?limit={limit}")).await
    }

    pub async fn get_run(run_id: &str) -> Result<SiqRunArtifact, String> {
        get_json(&format!("/api/system/siq/runs/{run_id}")).await
    }

    pub async fn get_health() -> Result<SiqHealth, String> {
        get_json("/api/system/siq/health").await
    }
}

async fn get_json<T: DeserializeOwned>(path: &str) -> Result<T, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", gateway_base(), path);
    let response = client
        .get(&url)
        .send()
        .await
        .map_err(|err| format!("request failed for {}: {}", path, err))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response
            .text()
            .await
            .unwrap_or_else(|_| "no body".to_string());
        return Err(format!(
            "request failed for {} ({}): {}",
            path, status, body
        ));
    }

    response
        .json::<T>()
        .await
        .map_err(|err| format!("invalid response for {}: {}", path, err))
}
