use cortex_eudaemon::gateway::server::GatewayService;
use cortex_eudaemon::services::gateway_config::gateway_port_with_note;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (port, note) = gateway_port_with_note();
    if let Some(note) = note {
        tracing::warn!("{}", note);
    }
    tracing::info!("starting host-neutral cortex gateway on port {}", port);
    GatewayService::start(port).await;
}
