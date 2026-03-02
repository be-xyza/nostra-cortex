use axum::extract::ws::Message;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;

#[derive(Debug)]
pub struct AgentApprovalSignal {
    pub decision: String,
    pub rationale: Option<String>,
    pub actor: String,
    pub decision_ref: Option<String>,
    pub actor_principal: Option<String>,
}

#[derive(Clone)]
pub struct GatewayState {
    pub broadcast_tx: broadcast::Sender<Message>,
    pub approval_waiters: Arc<Mutex<HashMap<String, oneshot::Sender<AgentApprovalSignal>>>>,
    pub temporal_projectors: Arc<Mutex<HashMap<String, JoinHandle<()>>>>,
}

impl GatewayState {
    pub fn new() -> Self {
        let (broadcast_tx, _) = broadcast::channel(100);
        Self {
            broadcast_tx,
            approval_waiters: Arc::new(Mutex::new(HashMap::new())),
            temporal_projectors: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}
