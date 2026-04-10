use crate::services::workflow_service::WorkflowService;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SignalMetrics {
    pub active_workflows: usize,
    pub processed_signals: usize,
    pub system_load: f32, // 0.0 to 1.0
}

pub struct CortexService {
    // Wrapper around existing services
    workflow_service: WorkflowService,
}

impl CortexService {
    pub fn new() -> Self {
        Self {
            workflow_service: WorkflowService::new(),
        }
    }

    pub async fn get_metrics(&self) -> Result<SignalMetrics, String> {
        // In the future, this comes from a dedicated backend canister
        // For now, we aggregate from workflow service + mock
        let pending = self
            .workflow_service
            .get_pending_tasks()
            .await
            .unwrap_or_default()
            .len();

        Ok(SignalMetrics {
            active_workflows: pending, // Mock mapping
            processed_signals: 124,    // Mock
            system_load: 0.42,         // Mock
        })
    }

    pub async fn submit_vote(&self, task_id: &str, approved: bool) -> Result<(), String> {
        // Logic to find instance ID from step ID would fail here in a real app
        // without more context, but assuming we pass the full context later.
        // For the mock, we just log.
        web_sys::console::log_1(&format!("Submitting Vote: {} -> {}", task_id, approved).into());
        Ok(())
    }
}
