/// A structural facade representing the Temporal SDK concepts.
/// This allows the worker to be architected for Durable Execution
/// without linking the heavy C++ core during initial hardening slices.
use async_trait::async_trait;
use cortex_domain::simulation::feedback::{ApprovalDecision, HumanApprovalEvent};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};
use tokio::sync::oneshot;
use tokio::time::Duration;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeMode {
    GatewayPrimary,
    TemporalShadow,
    TemporalPrimary,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TemporalExecutionBackend {
    Bridge,
    Sdk,
}

impl TemporalExecutionBackend {
    pub fn from_env() -> Self {
        match std::env::var("CORTEX_TEMPORAL_EXECUTION_BACKEND")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("sdk") => Self::Sdk,
            Some("bridge") | None => Self::Bridge,
            Some(_) => Self::Bridge,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Bridge => "bridge",
            Self::Sdk => "sdk",
        }
    }
}

impl RuntimeMode {
    pub fn from_env() -> Self {
        match std::env::var("CORTEX_AGENT_RUNTIME_MODE")
            .ok()
            .map(|value| value.trim().to_ascii_lowercase())
            .as_deref()
        {
            Some("gateway_primary") => Self::GatewayPrimary,
            Some("temporal_primary") => Self::TemporalPrimary,
            Some("temporal_shadow") | None => Self::TemporalShadow,
            Some(_) => Self::TemporalShadow,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::GatewayPrimary => "gateway_primary",
            Self::TemporalShadow => "temporal_shadow",
            Self::TemporalPrimary => "temporal_primary",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowOptions {
    pub task_queue: String,
    pub workflow_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityOptions {
    pub task_queue: String,
}

#[derive(Clone)]
pub struct MockWorkflowContext;

static APPROVAL_WAITERS: LazyLock<
    Arc<Mutex<HashMap<String, oneshot::Sender<HumanApprovalEvent>>>>,
> = LazyLock::new(|| Arc::new(Mutex::new(HashMap::new())));

fn approval_waiter_key(space_id: &str, scenario_id: &str) -> String {
    format!("{}::{}", space_id.trim(), scenario_id.trim())
}

fn sanitize_fs_component(value: &str) -> String {
    let mut sanitized = String::with_capacity(value.len());
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_') {
            sanitized.push(ch);
        } else {
            sanitized.push('_');
        }
    }
    if sanitized.is_empty() {
        "unknown".to_string()
    } else {
        sanitized
    }
}

fn decision_surface_log_dir() -> PathBuf {
    std::env::var("NOSTRA_DECISION_SURFACE_LOG_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            std::env::var("CORTEX_IC_PROJECT_ROOT")
                .map(|root| PathBuf::from(root).join("logs/system/decision_surfaces"))
        })
        .unwrap_or_else(|_| PathBuf::from("logs/system/decision_surfaces"))
}

fn scenario_signal_path(scenario_id: &str) -> PathBuf {
    decision_surface_log_dir()
        .join("temporal_signals")
        .join(format!(
            "scenario__{}.json",
            sanitize_fs_component(scenario_id)
        ))
}

fn parse_external_signal(space_id: &str, scenario_id: &str) -> Option<HumanApprovalEvent> {
    let raw = fs::read_to_string(scenario_signal_path(scenario_id)).ok()?;
    let parsed: serde_json::Value = serde_json::from_str(&raw).ok()?;
    let payload_space = parsed.get("spaceId").and_then(|value| value.as_str())?;
    let payload_scenario = parsed.get("scenarioId").and_then(|value| value.as_str())?;
    if payload_space != space_id || payload_scenario != scenario_id {
        return None;
    }
    let decision = match parsed
        .get("decision")
        .and_then(|value| value.as_str())
        .unwrap_or("")
        .trim()
        .to_ascii_lowercase()
        .as_str()
    {
        "approved" => ApprovalDecision::Approved,
        "rejected" => ApprovalDecision::Rejected,
        "needs_modification" => ApprovalDecision::NeedsModification,
        _ => return None,
    };
    Some(HumanApprovalEvent {
        scenario_id: payload_scenario.to_string(),
        space_id: payload_space.to_string(),
        actor: parsed
            .get("actor")
            .and_then(|value| value.as_str())
            .unwrap_or("temporal-bridge")
            .to_string(),
        decision,
        rationale: parsed
            .get("rationale")
            .and_then(|value| value.as_str())
            .map(|value| value.to_string()),
    })
}

impl MockWorkflowContext {
    pub async fn execute_activity<A, Input, Output>(
        &self,
        activity: A,
        input: Input,
        _options: ActivityOptions,
    ) -> Result<Output, String>
    where
        A: Activity<Input, Output> + Send + Sync,
        Input: Serialize + Send + Sync,
        Output: for<'de> Deserialize<'de> + Send + Sync,
    {
        activity.execute(input).await
    }

    pub async fn wait_for_human_approval(
        &self,
        space_id: &str,
        scenario_id: &str,
        timeout: Duration,
    ) -> Result<HumanApprovalEvent, String> {
        if let Some(event) = parse_external_signal(space_id, scenario_id) {
            return Ok(event);
        }

        let key = approval_waiter_key(space_id, scenario_id);
        let (tx, mut rx) = oneshot::channel::<HumanApprovalEvent>();
        if let Ok(mut waiters) = APPROVAL_WAITERS.lock() {
            waiters.insert(key.clone(), tx);
        }
        let mut poll_interval = tokio::time::interval(Duration::from_millis(250));
        let deadline = tokio::time::sleep(timeout);
        tokio::pin!(deadline);

        let result = loop {
            tokio::select! {
                _ = &mut deadline => break Err("approval_timeout".to_string()),
                _ = poll_interval.tick() => {
                    if let Some(event) = parse_external_signal(space_id, scenario_id) {
                        break Ok(event);
                    }
                }
                signal = &mut rx => {
                    match signal {
                        Ok(event) => break Ok(event),
                        Err(_) => break Err("approval_signal_channel_closed".to_string()),
                    }
                }
            }
        };

        if let Ok(mut waiters) = APPROVAL_WAITERS.lock() {
            waiters.remove(&key);
        }

        result
    }

    pub fn signal_human_approval(event: HumanApprovalEvent) -> Result<(), String> {
        let key = approval_waiter_key(&event.space_id, &event.scenario_id);
        let sender = APPROVAL_WAITERS
            .lock()
            .ok()
            .and_then(|mut waiters| waiters.remove(&key));
        let Some(sender) = sender else {
            return Err("approval_waiter_missing".to_string());
        };
        sender
            .send(event)
            .map_err(|_| "approval_waiter_closed".to_string())
    }
}

#[async_trait]
pub trait Workflow<Input, Output> {
    const NAME: &'static str;
    async fn execute(&self, ctx: &MockWorkflowContext, input: Input) -> Result<Output, String>;
}

#[async_trait]
pub trait Activity<Input, Output> {
    const NAME: &'static str;
    async fn execute(&self, input: Input) -> Result<Output, String>;
}
