use dioxus::prelude::*;
use futures::StreamExt;
use gloo_net::websocket::{Message, futures::WebSocket};
use gloo_timers::future::TimeoutFuture;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayEvent {
    pub topic: String,
    pub payload: serde_json::Value,
    pub source: String,
    pub timestamp: u64,
}

// Global Signal for connection status
pub static GATEWAY_CONNECTED: GlobalSignal<bool> = Signal::global(|| false);
// Global Signal for latest event
pub static LATEST_EVENT: GlobalSignal<Option<GatewayEvent>> = Signal::global(|| None);

pub fn use_gateway_client() {
    // Start the connection loop
    use_future(move || async move {
        loop {
            // Attempt connection
            let ws_url = "ws://localhost:3003/gateway";

            match WebSocket::open(ws_url) {
                Ok(ws) => {
                    println!("[Gateway] Connecting to {}", ws_url);
                    let (_write, mut read) = ws.split();

                    // Connection successful
                    GATEWAY_CONNECTED.signal().set(true);
                    println!("[Gateway] Connected!");

                    // Reader Loop
                    while let Some(msg) = read.next().await {
                        match msg {
                            Ok(Message::Text(text)) => {
                                println!("[Gateway] Received: {}", text);
                                if let Ok(event) = serde_json::from_str::<GatewayEvent>(&text) {
                                    LATEST_EVENT.signal().set(Some(event));
                                }
                            }
                            Ok(Message::Bytes(_)) => {}
                            Err(e) => {
                                println!("[Gateway] Error: {:?}", e);
                                break;
                            }
                        }
                    }

                    // Connection lost
                    GATEWAY_CONNECTED.signal().set(false);
                    println!("[Gateway] Disconnected.");
                }
                Err(e) => {
                    println!("[Gateway] Connection failed: {:?}", e);
                }
            }

            // Retry delay
            TimeoutFuture::new(5000).await;
        }
    });
}
