use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityType {
    ReadinessAudit,
    PhaseTransition,
    SimulatedWork,
    SystemSignal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActivityStatus {
    Proposed,
    Active,
    Resolved,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: String,
    pub activity_type: ActivityType,
    pub status: ActivityStatus,
    pub description: String,
    pub timestamp: u64,
    pub metadata: HashMap<String, String>,
}

#[derive(Clone)]
pub struct ActivityService {
    // In-memory store for now, in a real app this would be backed by a DB or Canister
    activities: std::sync::Arc<tokio::sync::RwLock<Vec<Activity>>>,
}

impl Default for ActivityService {
    fn default() -> Self {
        Self::new()
    }
}

impl ActivityService {
    pub fn new() -> Self {
        Self {
            activities: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
        }
    }

    pub async fn record_activity(
        &self,
        activity_type: ActivityType,
        status: ActivityStatus,
        description: String,
        metadata: HashMap<String, String>,
    ) -> String {
        let id = format!("act_{}", Uuid::new_v4());
        let activity = Activity {
            id: id.clone(),
            activity_type,
            status,
            description,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            metadata,
        };

        let mut activities = self.activities.write().await;
        activities.push(activity);
        id
    }

    pub async fn list_activities(&self) -> Vec<Activity> {
        self.activities.read().await.clone()
    }
}
