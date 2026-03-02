use cortex_eudaemon::services::gateway_config::gateway_port_with_note;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (port, note) = gateway_port_with_note();
    if let Some(note) = note {
        tracing::warn!("{}", note);
    }
    tracing::info!("starting cortex gateway server on port {}", port);
    cortex_eudaemon::gateway::server::GatewayService::start(port).await;
}
