use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::response::IntoResponse;
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GatewayEvent {
    pub topic: String,
    pub payload: serde_json::Value,
    pub source: String,
    pub timestamp: u64,
}

#[derive(Clone)]
pub struct GatewayService {
    tx: broadcast::Sender<GatewayEvent>,
}

impl Default for GatewayService {
    fn default() -> Self {
        Self::new()
    }
}

impl GatewayService {
    pub fn new() -> Self {
        let (tx, _rx) = broadcast::channel(100);
        Self { tx }
    }

    pub fn broadcast(&self, event: GatewayEvent) {
        let _ = self.tx.send(event);
    }

    pub async fn handle_ws_upgrade(
        ws: WebSocketUpgrade,
        service: Arc<GatewayService>,
    ) -> impl IntoResponse {
        ws.on_upgrade(move |socket| handle_socket(socket, service))
    }
}

async fn handle_socket(socket: WebSocket, service: Arc<GatewayService>) {
    let mut rx = service.tx.subscribe();

    // Start a task to forward broadcast messages to the websocket
    let (mut sender, mut receiver) = socket.split();

    let mut send_task = tokio::spawn(async move {
        while let Ok(event) = rx.recv().await {
            if let Ok(msg) = serde_json::to_string(&event) {
                if sender.send(Message::Text(msg)).await.is_err() {
                    break;
                }
            }
        }
    });

    // Handle incoming messages (keep connection alive)
    // For now, we just log them or ignore, maybe implement "Client to Server" commands later
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Close(_) => break,
                Message::Text(t) => {
                    println!("   [Gateway] Received: {}", t);
                }
                _ => {}
            }
        }
    });

    // If either task fails/ends, abort the other
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{Router, routing::get};
    use futures::{SinkExt, StreamExt};
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio_tungstenite::connect_async;

    #[tokio::test]
    async fn test_gateway_connection_and_broadcast() {
        // 1. Setup Service
        let service = Arc::new(GatewayService::new());
        let service_clone = service.clone();

        // 2. Setup Server
        let app = Router::new().route(
            "/gateway",
            get(move |ws| GatewayService::handle_ws_upgrade(ws, service_clone)),
        );

        let listener = match TcpListener::bind("127.0.0.1:0").await {
            Ok(listener) => listener,
            Err(e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                // Some sandboxed environments disallow local socket binds.
                return;
            }
            Err(e) => panic!("failed to bind test socket: {e}"),
        };
        let addr = listener.local_addr().unwrap();

        tokio::spawn(async move {
            axum::serve(listener, app).await.unwrap();
        });

        // 3. Client Connection
        let url = format!("ws://{}/gateway", addr);
        let (mut ws_stream, _) = connect_async(url).await.expect("Failed to connect");

        // 4. Test Broadcast
        let test_event = GatewayEvent {
            topic: "test".to_string(),
            payload: serde_json::json!({"foo": "bar"}),
            source: "test_runner".to_string(),
            timestamp: 12345,
        };

        service.broadcast(test_event.clone());

        // 5. Verify Receipt
        if let Some(Ok(msg)) = ws_stream.next().await {
            if let tokio_tungstenite::tungstenite::Message::Text(text) = msg {
                let received: GatewayEvent = serde_json::from_str(&text).unwrap();
                assert_eq!(received.topic, "test");
                assert_eq!(received.payload, test_event.payload);
            } else {
                panic!("Expected text message");
            }
        } else {
            panic!("Stream closed unexpectedly");
        }
    }
}
