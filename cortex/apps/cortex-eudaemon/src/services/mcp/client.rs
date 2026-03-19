use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicI64, Ordering};
use tokio::sync::{Mutex, oneshot};

use cortex_domain::agent::mcp::protocol::{
    CallToolParams, CallToolResult, ClientCapabilities, ClientInfo, InitializeParams,
    InitializeResult, JsonRpcNotification, JsonRpcRequest, JsonRpcResponse, ListToolsResult,
    McpError, RequestId,
};

use crate::services::mcp::transport::{JsonRpcMessage, Transport};

type ResponseSender = oneshot::Sender<Result<JsonRpcResponse, McpError>>;

pub struct McpClient {
    transport: Arc<Box<dyn Transport>>,
    next_id: AtomicI64,
    pending_requests: Arc<Mutex<HashMap<i64, ResponseSender>>>,
}

impl McpClient {
    pub fn new(transport: Box<dyn Transport>) -> Self {
        Self {
            transport: Arc::new(transport),
            next_id: AtomicI64::new(1),
            pending_requests: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn start(&self) {
        let transport_clone = self.transport.clone();
        let pending_clone = self.pending_requests.clone();

        tokio::spawn(async move {
            loop {
                match transport_clone.receive().await {
                    Ok(JsonRpcMessage::Response(response)) => {
                        if let RequestId::Number(id) = response.id.clone() {
                            let mut pending = pending_clone.lock().await;
                            if let Some(sender) = pending.remove(&id) {
                                let _ = sender.send(Ok(response));
                            }
                        }
                    }
                    Ok(JsonRpcMessage::Notification(_notification)) => {
                        // Handle server-to-client notifications (e.g., logging) in future
                    }
                    Ok(JsonRpcMessage::Request(_req)) => {
                        // Handle server-to-client requests if needed
                    }
                    Err(_) => {
                        // Transport error (EOF or parse) -> stop listening
                        break;
                    }
                }
            }
        });
    }

    async fn send_request(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<JsonRpcResponse, McpError> {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let req = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            id: RequestId::Number(id),
            method: method.to_string(),
            params,
        };

        let (tx, rx) = oneshot::channel();
        {
            let mut pending = self.pending_requests.lock().await;
            pending.insert(id, tx);
        }

        self.transport.send(JsonRpcMessage::Request(req)).await?;

        // Wait for response
        let response = rx
            .await
            .map_err(|_| McpError::Transport("Response channel closed".to_string()))??;

        if let Some(error) = response.error {
            return Err(McpError::JsonRpc {
                code: error.code,
                message: error.message,
                data: error.data,
            });
        }

        Ok(response)
    }

    async fn send_notification(&self, method: &str, params: Option<Value>) -> Result<(), McpError> {
        let notif = JsonRpcNotification {
            jsonrpc: "2.0".to_string(),
            method: method.to_string(),
            params,
        };
        self.transport
            .send(JsonRpcMessage::Notification(notif))
            .await
    }

    pub async fn initialize(&self) -> Result<InitializeResult, McpError> {
        let params = InitializeParams {
            protocol_version: "2024-11-05".to_string(), // Current MCP spec format
            capabilities: ClientCapabilities {
                experimental: None,
                roots: None,
                sampling: None,
            },
            client_info: ClientInfo {
                name: "cortex-eudaemon".to_string(),
                version: env!("CARGO_PKG_VERSION").to_string(),
            },
        };

        let params_val =
            serde_json::to_value(params).map_err(|e| McpError::Parse(e.to_string()))?;
        let res = self.send_request("initialize", Some(params_val)).await?;

        let init_result: InitializeResult =
            serde_json::from_value(res.result.unwrap_or(Value::Null))
                .map_err(|e| McpError::Parse(e.to_string()))?;

        // Send initialized notification as per spec
        self.send_notification("notifications/initialized", None)
            .await?;

        Ok(init_result)
    }

    pub async fn list_tools(&self) -> Result<ListToolsResult, McpError> {
        let res = self.send_request("tools/list", None).await?;
        let list_res: ListToolsResult = serde_json::from_value(res.result.unwrap_or(Value::Null))
            .map_err(|e| McpError::Parse(e.to_string()))?;
        Ok(list_res)
    }

    pub async fn call_tool(
        &self,
        name: &str,
        arguments: Option<Value>,
    ) -> Result<CallToolResult, McpError> {
        let params = CallToolParams {
            name: name.to_string(),
            arguments,
        };
        let params_val =
            serde_json::to_value(params).map_err(|e| McpError::Parse(e.to_string()))?;

        let res = self.send_request("tools/call", Some(params_val)).await?;
        let call_res: CallToolResult = serde_json::from_value(res.result.unwrap_or(Value::Null))
            .map_err(|e| McpError::Parse(e.to_string()))?;
        Ok(call_res)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json::json;
    use tokio::sync::mpsc;

    struct MockTransport {
        tx: mpsc::Sender<JsonRpcMessage>,
        rx: Mutex<mpsc::Receiver<JsonRpcMessage>>,
    }

    #[async_trait]
    impl Transport for MockTransport {
        async fn send(&self, message: JsonRpcMessage) -> Result<(), McpError> {
            self.tx.send(message).await.unwrap();
            Ok(())
        }

        async fn receive(&self) -> Result<JsonRpcMessage, McpError> {
            let mut rx = self.rx.lock().await;
            rx.recv()
                .await
                .ok_or_else(|| McpError::Transport("Closed".into()))
        }
    }

    #[tokio::test]
    async fn test_client_initialization() {
        let (server_tx, client_rx) = mpsc::channel(10);
        let (client_tx, mut server_rx) = mpsc::channel(10);

        let transport = Box::new(MockTransport {
            tx: client_tx,
            rx: Mutex::new(client_rx),
        });

        let client = Arc::new(McpClient::new(transport));
        client.start();

        // Simulate server behavior in background
        tokio::spawn(async move {
            if let Some(JsonRpcMessage::Request(req)) = server_rx.recv().await {
                assert_eq!(req.method, "initialize");

                // Reply to initialize Request
                let response = JsonRpcResponse {
                    jsonrpc: "2.0".to_string(),
                    id: req.id,
                    result: Some(json!({
                        "protocolVersion": "2024-11-05",
                        "capabilities": {},
                        "serverInfo": {
                            "name": "mock-server",
                            "version": "1.0.0"
                        }
                    })),
                    error: None,
                };
                server_tx
                    .send(JsonRpcMessage::Response(response))
                    .await
                    .unwrap();
            }

            // Expect the initialized notification
            if let Some(JsonRpcMessage::Notification(notif)) = server_rx.recv().await {
                assert_eq!(notif.method, "notifications/initialized");
            }
        });

        let result = client.initialize().await.unwrap();
        assert_eq!(result.server_info.name, "mock-server");
        assert_eq!(result.protocol_version, "2024-11-05");
    }
}
