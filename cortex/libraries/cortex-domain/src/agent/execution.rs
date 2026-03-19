use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpaceExecutionHook {
    pub space_id: String,
    pub contribution_id: String,
    pub requested_by: String,
    pub status: ExecutionHookStatus,
    pub started_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ExecutionHookStatus {
    Pending,
    Dispatched,
    RunningSimulation,
    AwaitingHumanFeedback,
    Completed,
    Failed,
}

impl SpaceExecutionHook {
    pub fn new(space_id: &str, contribution_id: &str, user_id: &str, timestamp: u64) -> Self {
        Self {
            space_id: space_id.to_string(),
            contribution_id: contribution_id.to_string(),
            requested_by: user_id.to_string(),
            status: ExecutionHookStatus::Pending,
            started_at: timestamp,
        }
    }

    pub fn transition_to(&mut self, status: ExecutionHookStatus) {
        self.status = status;
    }
}
