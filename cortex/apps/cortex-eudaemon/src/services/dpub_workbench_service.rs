use crate::services::gateway_config::gateway_base;
pub use cortex_workbench_contracts::{
    DpubEditionEntry, DpubEditionTrendResponse, DpubLensEvaluateRequest, DpubLensOverlayResponse,
    DpubLensSummaryResponse, DpubPipelineQueryRequest, DpubPipelineRunReport,
    DpubPipelineRunRequest, DpubRunHistoryItem, DpubSimulationArtifact, DpubSystemBuildResponse,
    DpubSystemReadyResponse,
};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

pub struct DpubWorkbenchService;

impl DpubWorkbenchService {
    pub async fn get_system_ready() -> Result<DpubSystemReadyResponse, String> {
        get_json("/api/system/ready").await
    }

    pub async fn get_system_build() -> Result<DpubSystemBuildResponse, String> {
        get_json("/api/system/build").await
    }

    pub async fn get_overview(space_id: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/overview"
        ))
        .await
    }

    pub async fn get_graph(space_id: &str, exploration_mode: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/graph?mode={exploration_mode}"
        ))
        .await
    }

    pub async fn get_path_assessment(space_id: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/path-assessment"
        ))
        .await
    }

    pub async fn get_doctor(space_id: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/doctor"
        ))
        .await
    }

    pub async fn get_simulations(space_id: &str) -> Result<Vec<DpubSimulationArtifact>, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/simulations"
        ))
        .await
    }

    pub async fn get_editions(space_id: &str) -> Result<Vec<DpubEditionEntry>, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/editions"
        ))
        .await
    }

    pub async fn get_runs(space_id: &str, limit: usize) -> Result<Vec<DpubRunHistoryItem>, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/runs?limit={limit}"
        ))
        .await
    }

    pub async fn get_run(space_id: &str, run_id: &str) -> Result<DpubPipelineRunReport, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/runs/{run_id}"
        ))
        .await
    }

    pub async fn get_edition_diff(space_id: &str, from: &str, to: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/edition-diff?from={from}&to={to}"
        ))
        .await
    }

    pub async fn get_lens_summary(space_id: &str) -> Result<DpubLensSummaryResponse, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/lens-summary"
        ))
        .await
    }

    pub async fn get_edition_trends(
        space_id: &str,
        goal: &str,
        window: usize,
    ) -> Result<DpubEditionTrendResponse, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/edition-trends?goal={goal}&window={window}"
        ))
        .await
    }

    pub async fn evaluate_lenses(
        space_id: &str,
        req: &DpubLensEvaluateRequest,
    ) -> Result<DpubLensOverlayResponse, String> {
        post_json(
            &format!("/api/kg/spaces/{space_id}/contribution-graph/lens/evaluate"),
            req,
        )
        .await
    }

    pub async fn get_violations_by_node(space_id: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/violations/by-node"
        ))
        .await
    }

    pub async fn run_pipeline(
        space_id: &str,
        actor_role: &str,
        actor_id: &str,
        req: &DpubPipelineRunRequest,
    ) -> Result<DpubPipelineRunReport, String> {
        post_json_with_actor(
            &format!("/api/kg/spaces/{space_id}/contribution-graph/pipeline/run"),
            actor_role,
            actor_id,
            req,
        )
        .await
    }

    pub async fn query(space_id: &str, req: &DpubPipelineQueryRequest) -> Result<Value, String> {
        post_json(
            &format!("/api/kg/spaces/{space_id}/contribution-graph/pipeline/query"),
            req,
        )
        .await
    }

    pub async fn launch_agent_contribution(
        space_id: &str,
        contribution_id: &str,
    ) -> Result<Value, String> {
        #[derive(Serialize)]
        #[serde(rename_all = "camelCase")]
        struct AgentContributionRequest {
            contribution_id: String,
        }
        let req = AgentContributionRequest {
            contribution_id: contribution_id.to_string(),
        };
        post_json(
            &format!("/api/kg/spaces/{space_id}/agents/contributions"),
            &req,
        )
        .await
    }

    pub async fn get_blast_radius(space_id: &str, contribution_id: &str) -> Result<Value, String> {
        get_json(&format!(
            "/api/kg/spaces/{space_id}/contribution-graph/blast-radius?contributionId={contribution_id}"
        ))
        .await
    }

    pub async fn export_steward_packet(
        space_id: &str,
        actor_role: &str,
        actor_id: &str,
        payload: &Value,
    ) -> Result<Value, String> {
        post_json_with_actor(
            &format!("/api/kg/spaces/{space_id}/contribution-graph/steward-packet/export"),
            actor_role,
            actor_id,
            payload,
        )
        .await
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

async fn post_json<TReq: Serialize, TResp: DeserializeOwned>(
    path: &str,
    payload: &TReq,
) -> Result<TResp, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", gateway_base(), path);
    let response = client
        .post(&url)
        .json(payload)
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
        .json::<TResp>()
        .await
        .map_err(|err| format!("invalid response for {}: {}", path, err))
}

async fn post_json_with_actor<TReq: Serialize, TResp: DeserializeOwned>(
    path: &str,
    actor_role: &str,
    actor_id: &str,
    payload: &TReq,
) -> Result<TResp, String> {
    let client = reqwest::Client::new();
    let url = format!("{}{}", gateway_base(), path);
    let response = client
        .post(&url)
        .header("x-cortex-role", actor_role)
        .header("x-cortex-actor", actor_id)
        .json(payload)
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
        .json::<TResp>()
        .await
        .map_err(|err| format!("invalid response for {}: {}", path, err))
}
