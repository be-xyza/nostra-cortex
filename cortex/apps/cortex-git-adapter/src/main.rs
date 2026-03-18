mod adapter;
mod config;
mod endpoints;
mod github;
mod metrics;
mod nostra;
mod projector;
mod reconcile;
mod state;
mod workspace;

use crate::adapter::AppState;
use axum::extract::DefaultBodyLimit;
use axum::{Router, routing::get, routing::post};
use std::net::SocketAddr;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let config = config::AppConfig::load()?;
    let registry = config.load_registry()?;
    let state_dir = config.state_dir.clone();
    state::ensure_state_dirs(&state_dir).map_err(anyhow::Error::msg)?;

    let sink = nostra::build_sink(&config).await?;
    let github_api = github::api::GithubApi::new(&config).await?;
    let metrics = metrics::Metrics::new();

    let app_state = AppState {
        config,
        registry,
        sink,
        github_api,
        metrics,
    };

    reconcile::spawn_reconciler(app_state.clone());

    let max_body = app_state.config.max_request_body_bytes.max(1024);
    let bind = app_state.config.bind.clone();
    let port = app_state.config.port;

    let app = Router::new()
        .route("/webhooks/github", post(adapter::github_webhook))
        .route("/health", get(endpoints::health))
        .route("/ready", get(endpoints::ready))
        .route("/metrics", get(endpoints::metrics))
        .layer(DefaultBodyLimit::max(max_body))
        .with_state(app_state);

    let addr: SocketAddr = format!("{}:{}", bind, port)
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid bind/port: {e}"))?;
    tracing::info!("cortex-git-adapter listening on http://{}", addr);
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
