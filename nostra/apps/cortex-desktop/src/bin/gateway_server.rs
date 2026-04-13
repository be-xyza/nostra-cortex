use cortex_desktop::gateway::server::GatewayService;

fn gateway_port() -> u16 {
    std::env::var("CORTEX_GATEWAY_PORT")
        .ok()
        .and_then(|raw| raw.parse::<u16>().ok())
        .unwrap_or(4943)
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let port = gateway_port();
    tracing::info!("starting cortex-desktop gateway server on port {}", port);
    GatewayService::start(port).await;
}
