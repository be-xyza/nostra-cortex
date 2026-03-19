use crate::adapter::AppState;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct HealthResponse<'a> {
    name: &'a str,
    version: &'a str,
    uptime_secs: f64,
}

pub async fn health(State(state): State<AppState>) -> impl IntoResponse {
    let body = HealthResponse {
        name: "cortex-git-adapter",
        version: env!("CARGO_PKG_VERSION"),
        uptime_secs: state.metrics.uptime().as_secs_f64(),
    };
    (StatusCode::OK, axum::Json(body))
}

#[derive(Debug, Serialize)]
struct ReadyResponse {
    ready: bool,
    reasons: Vec<String>,
}

pub async fn ready(State(state): State<AppState>) -> Response {
    let mut reasons = Vec::new();

    if state.config.webhook_secret.as_deref().unwrap_or("").is_empty() {
        reasons.push("missing CORTEX_GIT_ADAPTER_WEBHOOK_SECRET".to_string());
    }

    if state.registry.repos.is_empty() {
        reasons.push("registry has no repos configured".to_string());
    }

    if let Err(err) = crate::state::ensure_state_dirs(&state.config.state_dir) {
        reasons.push(format!("state_dir not writable: {err}"));
    }

    if !state.config.nostra_sink_is_configured() {
        reasons.push("nostra sink not configured".to_string());
    }

    if !state.config.github_auth_is_configured() {
        reasons.push("github auth not configured (required for reconciliation)".to_string());
    }

    let ready = reasons.is_empty();
    let status = if ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };

    (status, axum::Json(ReadyResponse { ready, reasons })).into_response()
}

pub async fn metrics(State(state): State<AppState>) -> Response {
    let rendered = state.metrics.render_prometheus();
    (
        StatusCode::OK,
        [("content-type", "text/plain; version=0.0.4")],
        rendered,
    )
        .into_response()
}

