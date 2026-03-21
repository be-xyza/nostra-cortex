#![allow(non_snake_case)]

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum StartupMode {
    Full,
    Router,
    Smoke,
}

impl StartupMode {
    fn from_env() -> Self {
        match std::env::var("CORTEX_STARTUP_MODE")
            .unwrap_or_else(|_| "full".to_string())
            .to_lowercase()
            .as_str()
        {
            "smoke" => StartupMode::Smoke,
            "router" => StartupMode::Router,
            _ => StartupMode::Full,
        }
    }
}

use tracing_subscriber::fmt::writer::MakeWriterExt;

#[tokio::main]
async fn main() {
    cortex_eudaemon::services::telemetry_stream::init_broadcast();
    let broadcast_writer = cortex_eudaemon::services::telemetry_stream::BroadcastWriter;
    tracing_subscriber::fmt()
        .with_writer(std::io::stdout.and(broadcast_writer))
        .init();

    let startup_mode = StartupMode::from_env();
    let build_id = option_env!("GIT_SHA")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    let build_time = option_env!("BUILD_TIME_UTC")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .unwrap_or("unknown");
    tracing::info!(
        "starting cortex headless daemon mode={:?} build_id={} build_time_utc={}",
        startup_mode,
        build_id,
        build_time
    );

    let gateway_port = cortex_eudaemon::services::gateway_config::gateway_port();
    let gateway_mode =
        match cortex_eudaemon::services::gateway_config::gateway_legacy_dispatch_mode() {
            cortex_runtime::GatewayLegacyDispatchMode::HttpLoopback => "http_loopback",
            cortex_runtime::GatewayLegacyDispatchMode::InProcess => "in_process",
        };
    tracing::info!("Starting Gateway Service on port {}", gateway_port);
    tracing::info!("Gateway Mode: {}", gateway_mode);

    tokio::spawn(async move {
        cortex_eudaemon::gateway::server::GatewayService::start(gateway_port).await;
    });

    // Initialize Gateway
    cortex_eudaemon::services::local_gateway::get_gateway().init();

    if cortex_eudaemon::services::acp_protocol::is_acp_pilot_enabled() {
        if let Ok(endpoint) = std::env::var("CORTEX_ACP_LOG_REGISTRY_URL") {
            match cortex_eudaemon::services::local_gateway::get_gateway()
                .flush_observability_events(&endpoint)
                .await
            {
                Ok(count) => tracing::info!(
                    "ACP pilot startup flush completed, emitted {} observability events",
                    count
                ),
                Err(err) => tracing::warn!("ACP pilot startup flush failed: {}", err),
            }
        }
    }

    // Start Shell PTY processes
    cortex_eudaemon::services::terminal_service::TerminalService::spawn_shell();

    // Start the Vector Agent
    cortex_eudaemon::services::agent_service::AgentService::spawn_vector_agent().await;

    // Start Space Auditor (Event-Driven sync and autonomous SIQ metric evaluation)
    cortex_eudaemon::services::space_auditor::SpaceAuditor::spawn();

    // Wait indefinitely (Headless daemon loop)
    tokio::signal::ctrl_c()
        .await
        .expect("failed to listen for event");
    tracing::info!("cortex headless daemon shutting down gracefully.");
}
