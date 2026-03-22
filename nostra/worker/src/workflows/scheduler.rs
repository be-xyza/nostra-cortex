use crate::gateway_service::{GatewayEvent, GatewayService};
use crate::workflows::acp_pilot_ops::create_acp_pilot_ops_workflow;
use crate::workflows::engine_runner::WorkflowRunner;
use anyhow::Result;
use chrono::Utc;
use nostra_workflow_core::types::WorkflowStatus;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{Duration, sleep};
use uuid::Uuid;

const DEFAULT_INTERVAL_SECS: u64 = 3600;
const DEFAULT_STATE_FILE: &str = "/tmp/cortex_acp_automation_state.json";
const ACP_AUTOMATION_KEY: &str = "acp_pilot_ops";

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SchedulerPersistedState {
    pub paused: bool,
    pub active_workflow_id: Option<String>,
    pub last_workflow_id: Option<String>,
    pub last_run_at: Option<String>,
    pub last_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchedulerSnapshot {
    pub automation_key: String,
    pub enabled: bool,
    pub paused: bool,
    pub interval_secs: u64,
    pub active_workflow_id: Option<String>,
    pub last_workflow_id: Option<String>,
    pub last_run_at: Option<String>,
    pub last_status: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum RunNowOutcome {
    Started { workflow_id: String },
    AlreadyActive { workflow_id: String },
    Disabled,
}

pub struct AcpAutomationScheduler {
    runner: Arc<WorkflowRunner>,
    gateway_service: Arc<GatewayService>,
    state: Arc<Mutex<SchedulerPersistedState>>,
    interval: Duration,
    enabled: bool,
    persist_path: PathBuf,
}

impl AcpAutomationScheduler {
    pub fn from_env(
        runner: Arc<WorkflowRunner>,
        gateway_service: Arc<GatewayService>,
    ) -> Arc<Self> {
        let enabled = env_flag("CORTEX_AUTOMATION_ACP_ENABLED");
        let interval_secs = std::env::var("CORTEX_AUTOMATION_ACP_INTERVAL_SECS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(DEFAULT_INTERVAL_SECS)
            .max(1);
        let persist_path = PathBuf::from(
            std::env::var("CORTEX_AUTOMATION_STATE_PATH")
                .unwrap_or_else(|_| DEFAULT_STATE_FILE.to_string()),
        );

        Arc::new(Self::new(
            runner,
            gateway_service,
            enabled,
            Duration::from_secs(interval_secs),
            persist_path,
        ))
    }

    pub fn new(
        runner: Arc<WorkflowRunner>,
        gateway_service: Arc<GatewayService>,
        enabled: bool,
        interval: Duration,
        persist_path: PathBuf,
    ) -> Self {
        let state = load_state(&persist_path).unwrap_or_default();
        Self {
            runner,
            gateway_service,
            state: Arc::new(Mutex::new(state)),
            interval,
            enabled,
            persist_path,
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn start(self: Arc<Self>) {
        if !self.enabled {
            return;
        }

        tokio::spawn(async move {
            loop {
                let _ = self.reconcile_active().await;

                let is_paused = {
                    let state = self.state.lock().await;
                    state.paused
                };

                if !is_paused {
                    let _ = self.start_run("interval").await;
                }

                sleep(self.interval).await;
            }
        });
    }

    pub async fn run_now(&self) -> Result<RunNowOutcome> {
        if !self.enabled {
            return Ok(RunNowOutcome::Disabled);
        }
        self.start_run("run_now").await
    }

    pub async fn pause(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.paused = true;
            persist_state(&self.persist_path, &state)?;
        }

        self.gateway_service.broadcast(GatewayEvent {
            topic: "workflow_update".to_string(),
            source: "automation_scheduler".to_string(),
            payload: serde_json::json!({
                "agent_id": "acp_automation_scheduler",
                "type": "automation_event",
                "action": "Triggered Pause",
                "automation_key": ACP_AUTOMATION_KEY,
                "status": "Paused",
            }),
            timestamp: now_millis(),
        });

        Ok(())
    }

    pub async fn resume(&self) -> Result<()> {
        {
            let mut state = self.state.lock().await;
            state.paused = false;
            persist_state(&self.persist_path, &state)?;
        }

        self.gateway_service.broadcast(GatewayEvent {
            topic: "workflow_update".to_string(),
            source: "automation_scheduler".to_string(),
            payload: serde_json::json!({
                "agent_id": "acp_automation_scheduler",
                "type": "automation_event",
                "action": "Triggered Resume",
                "automation_key": ACP_AUTOMATION_KEY,
                "status": "Resumed",
            }),
            timestamp: now_millis(),
        });

        Ok(())
    }

    pub async fn snapshot(&self) -> SchedulerSnapshot {
        let _ = self.reconcile_active().await;
        let state = self.state.lock().await;
        SchedulerSnapshot {
            automation_key: ACP_AUTOMATION_KEY.to_string(),
            enabled: self.enabled,
            paused: state.paused,
            interval_secs: self.interval.as_secs(),
            active_workflow_id: state.active_workflow_id.clone(),
            last_workflow_id: state.last_workflow_id.clone(),
            last_run_at: state.last_run_at.clone(),
            last_status: state.last_status.clone(),
        }
    }

    async fn start_run(&self, trigger: &str) -> Result<RunNowOutcome> {
        self.reconcile_active().await?;

        {
            let state = self.state.lock().await;
            if let Some(active_id) = state.active_workflow_id.as_ref() {
                return Ok(RunNowOutcome::AlreadyActive {
                    workflow_id: active_id.clone(),
                });
            }
        }

        let workflow_id = format!("{}-{}", ACP_AUTOMATION_KEY, Uuid::new_v4());
        self.runner
            .start(&workflow_id, create_acp_pilot_ops_workflow())?;

        {
            let mut state = self.state.lock().await;
            state.active_workflow_id = Some(workflow_id.clone());
            state.last_workflow_id = Some(workflow_id.clone());
            state.last_run_at = Some(Utc::now().to_rfc3339());
            state.last_status = Some("Running".to_string());
            persist_state(&self.persist_path, &state)?;
        }

        self.gateway_service.broadcast(GatewayEvent {
            topic: "workflow_update".to_string(),
            source: "automation_scheduler".to_string(),
            payload: serde_json::json!({
                "agent_id": "acp_automation_scheduler",
                "type": "automation_event",
                "action": "Triggered Run",
                "instance_id": workflow_id,
                "status": "running",
                "automation_key": ACP_AUTOMATION_KEY,
                "trigger": trigger,
            }),
            timestamp: now_millis(),
        });

        Ok(RunNowOutcome::Started { workflow_id })
    }

    async fn reconcile_active(&self) -> Result<()> {
        let active_id = {
            let state = self.state.lock().await;
            state.active_workflow_id.clone()
        };

        let Some(active_id) = active_id else {
            return Ok(());
        };

        let status = match self.runner.get_status(&active_id) {
            Ok(status) => status,
            Err(_) => {
                let mut state = self.state.lock().await;
                state.active_workflow_id = None;
                state.last_status = Some("MissingAfterRestart".to_string());
                persist_state(&self.persist_path, &state)?;
                return Ok(());
            }
        };

        if matches!(
            status,
            WorkflowStatus::Completed | WorkflowStatus::Failed(_)
        ) {
            let mut state = self.state.lock().await;
            state.active_workflow_id = None;
            state.last_status = Some(format!("{:?}", status));
            persist_state(&self.persist_path, &state)?;
        }

        Ok(())
    }
}

fn env_flag(key: &str) -> bool {
    std::env::var(key)
        .map(|value| {
            let normalized = value.trim().to_ascii_lowercase();
            normalized == "1" || normalized == "true" || normalized == "yes"
        })
        .unwrap_or(false)
}

fn load_state(path: &PathBuf) -> Result<SchedulerPersistedState> {
    if !path.exists() {
        return Ok(SchedulerPersistedState::default());
    }
    let raw = std::fs::read_to_string(path)?;
    let parsed = serde_json::from_str::<SchedulerPersistedState>(&raw)?;
    Ok(parsed)
}

fn persist_state(path: &PathBuf, state: &SchedulerPersistedState) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, serde_json::to_string_pretty(state)?)?;
    Ok(())
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflows::op_registry::{
        AdapterExecutionResult, OperationAdapter, OperationRegistry,
    };
    use anyhow::Result;
    use nostra_workflow_core::types::Context;
    use std::collections::HashMap;

    struct MockAdapter {
        outputs: HashMap<String, String>,
    }

    impl OperationAdapter for MockAdapter {
        fn execute(
            &self,
            _payload: &str,
            _context: &mut Context,
        ) -> Result<AdapterExecutionResult> {
            Ok(AdapterExecutionResult {
                outputs: self.outputs.clone(),
                message: Some("ok".to_string()),
            })
        }
    }

    fn build_runner_with_mock_registry() -> Arc<WorkflowRunner> {
        let mut registry = OperationRegistry::new();
        registry.register_adapter(
            "ops.acp.collect_metrics",
            Arc::new(MockAdapter {
                outputs: HashMap::from([(
                    "acp.metrics.snapshot_path".to_string(),
                    "/tmp/acp_metrics.jsonl".to_string(),
                )]),
            }),
        );
        registry.register_adapter(
            "ops.acp.evaluate_slo",
            Arc::new(MockAdapter {
                outputs: HashMap::from([
                    ("acp.slo.result".to_string(), "PASS".to_string()),
                    ("acp.slo.reason".to_string(), "ok".to_string()),
                ]),
            }),
        );
        registry.register_adapter(
            "ops.acp.publish_evidence",
            Arc::new(MockAdapter {
                outputs: HashMap::from([(
                    "acp.evidence.updated_files".to_string(),
                    "/tmp/evidence.md".to_string(),
                )]),
            }),
        );

        Arc::new(WorkflowRunner::new_with_registry(None, registry))
    }

    #[tokio::test]
    async fn run_now_dedupes_active_workflow() {
        let runner = build_runner_with_mock_registry();
        let gateway = Arc::new(GatewayService::new());
        let persist_path = std::env::temp_dir().join(format!(
            "acp_scheduler_state_{}.json",
            Uuid::new_v4().simple()
        ));

        let scheduler = AcpAutomationScheduler::new(
            runner,
            gateway,
            true,
            Duration::from_secs(60),
            persist_path,
        );

        let first = scheduler.run_now().await.unwrap();
        let second = scheduler.run_now().await.unwrap();

        let started = match first {
            RunNowOutcome::Started { workflow_id } => workflow_id,
            _ => panic!("expected first run to start workflow"),
        };

        match second {
            RunNowOutcome::AlreadyActive { workflow_id } => {
                assert_eq!(workflow_id, started);
            }
            _ => panic!("expected duplicate run to be deduped"),
        }
    }
}
