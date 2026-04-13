use crate::gateway::state::GatewayState;
use crate::services::acp_adapter::TerminalCreateRequest;
use crate::services::acp_protocol::{JsonRpcRequest, handle_rpc_request, is_acp_pilot_enabled};
use crate::services::terminal_service::{
    AcpTerminalOutputRequest, AcpTerminalWaitRequest, TerminalService,
};
use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde::Serialize;
use serde_json::{Value, json};
use std::net::SocketAddr;
use std::sync::Arc;

pub struct GatewayService;

#[derive(Clone)]
struct AppState {
    gateway: Arc<GatewayState>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct HealthResponse {
    status: &'static str,
    host_kind: &'static str,
    acp_pilot_enabled: bool,
    workspace_root: String,
}

impl GatewayService {
    pub async fn start(port: u16) {
        let gateway = GatewayState::load_default().expect("failed to initialize gateway state");
        let state = AppState {
            gateway: Arc::new(gateway),
        };

        let app = Router::new()
            .route("/api/health", get(health_check))
            .route("/api/acp/rpc", post(acp_rpc))
            .route("/api/acp/terminal/create", post(acp_terminal_create))
            .route("/api/acp/terminal/output", post(acp_terminal_output))
            .route(
                "/api/acp/terminal/wait_for_exit",
                post(acp_terminal_wait_for_exit),
            )
            .with_state(state);

        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr)
            .await
            .expect("failed to bind desktop gateway listener");

        tracing::info!("cortex-desktop gateway server listening on {}", addr);
        axum::serve(listener, app)
            .await
            .expect("desktop gateway server exited unexpectedly");
    }
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok",
        host_kind: "desktop",
        acp_pilot_enabled: is_acp_pilot_enabled(),
        workspace_root: state.gateway.workspace_root().display().to_string(),
    })
}

async fn acp_rpc(Json(request): Json<JsonRpcRequest>) -> Json<crate::services::acp_protocol::JsonRpcResponse> {
    Json(handle_rpc_request(request).await)
}

async fn acp_terminal_create(
    State(state): State<AppState>,
    Json(payload): Json<TerminalCreateRequest>,
) -> Response {
    if !is_acp_pilot_enabled() {
        return acp_disabled_response();
    }

    match state.gateway.adapter().validate_terminal_create(payload) {
        Ok(validated) => match TerminalService::acp_terminal_create(validated).await {
            Ok(response) => Json(json!(response)).into_response(),
            Err(err) => error_response(StatusCode::INTERNAL_SERVER_ERROR, err),
        },
        Err(err) => error_response(StatusCode::BAD_REQUEST, err.to_string()),
    }
}

async fn acp_terminal_output(Json(payload): Json<AcpTerminalOutputRequest>) -> Response {
    if !is_acp_pilot_enabled() {
        return acp_disabled_response();
    }

    match TerminalService::acp_terminal_output(payload).await {
        Ok(response) => Json(json!(response)).into_response(),
        Err(err) => error_response(StatusCode::BAD_REQUEST, err),
    }
}

async fn acp_terminal_wait_for_exit(Json(payload): Json<AcpTerminalWaitRequest>) -> Response {
    if !is_acp_pilot_enabled() {
        return acp_disabled_response();
    }

    match TerminalService::acp_terminal_wait_for_exit(payload).await {
        Ok(response) => Json(json!(response)).into_response(),
        Err(err) => error_response(StatusCode::BAD_REQUEST, err),
    }
}

fn acp_disabled_response() -> Response {
    error_response(
        StatusCode::SERVICE_UNAVAILABLE,
        "ACP pilot is disabled; set CORTEX_ACP_PILOT=1 to enable the desktop ACP host".to_string(),
    )
}

fn error_response(status: StatusCode, message: String) -> Response {
    let body: Value = json!({
        "error": message,
    });
    (status, Json(body)).into_response()
}
