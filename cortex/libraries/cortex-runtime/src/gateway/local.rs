use crate::RuntimeError;
use crate::ports::LocalGatewayOrchestrationAdapter;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalGatewayMutationRecord {
    pub mutation_id: String,
    pub idempotency_key: String,
    #[serde(default)]
    pub space_id: Option<String>,
    pub kip_command: String,
    pub timestamp: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
    #[serde(default)]
    pub conflict_state: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalGatewayProbe {
    pub queue_size: usize,
    pub queue_export_ok: bool,
    pub gateway_online: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LocalGatewayQueueAction {
    Retry,
    Discard,
    Fork,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct LocalGatewayMutationSubmit {
    pub mutation_id: String,
    pub idempotency_key: String,
    #[serde(default)]
    pub space_id: Option<String>,
    pub kip_command: String,
    pub timestamp: u64,
    #[serde(default)]
    pub attempts: u32,
    #[serde(default)]
    pub last_error: Option<String>,
    #[serde(default)]
    pub last_attempt_at: Option<u64>,
}

pub fn queue_snapshot(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
) -> Result<Vec<LocalGatewayMutationRecord>, RuntimeError> {
    adapter.queue_snapshot()
}

pub fn export_queue_json(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
) -> Result<String, RuntimeError> {
    adapter.export_queue_json()
}

pub fn apply_queue_action(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
    mutation_id: &str,
    action: &str,
) -> Result<(), RuntimeError> {
    let mutation_id = mutation_id.trim();
    if mutation_id.is_empty() {
        return Err(RuntimeError::Domain("mutation_id is required".to_string()));
    }
    adapter.apply_queue_action(mutation_id, parse_queue_action(action)?)
}

pub fn probe(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
) -> Result<LocalGatewayProbe, RuntimeError> {
    adapter.probe()
}

pub fn set_online(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
    status: bool,
) -> Result<(), RuntimeError> {
    adapter.set_online(status)
}

pub fn is_online(adapter: &dyn LocalGatewayOrchestrationAdapter) -> Result<bool, RuntimeError> {
    adapter.is_online()
}

pub fn submit_mutation(
    adapter: &dyn LocalGatewayOrchestrationAdapter,
    mutation: LocalGatewayMutationSubmit,
) -> Result<String, RuntimeError> {
    adapter.submit_mutation(mutation)
}

fn parse_queue_action(raw: &str) -> Result<LocalGatewayQueueAction, RuntimeError> {
    match raw.trim() {
        "retry" => Ok(LocalGatewayQueueAction::Retry),
        "discard" => Ok(LocalGatewayQueueAction::Discard),
        "fork" => Ok(LocalGatewayQueueAction::Fork),
        other => Err(RuntimeError::Domain(format!(
            "unsupported queue action: {other}"
        ))),
    }
}
