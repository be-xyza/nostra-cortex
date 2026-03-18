pub mod contracts;
pub mod execution;
pub mod mcp;
pub mod provider;
pub mod roles;
pub mod runner;
pub mod tools;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AgentState {
    pub identity: String,
    pub role: Option<String>,
    pub knowledge_pointers: Vec<String>,
    pub active_intention: Option<AgentIntention>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentIntention {
    pub id: String,
    pub description: String,
    pub target_resource: String,
    pub status: IntentionStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IntentionStatus {
    Pending,
    Executing,
    Simulating,
    Failed,
    Completed,
}

impl AgentState {
    pub fn new(identity: &str) -> Self {
        Self {
            identity: identity.to_string(),
            role: None,
            knowledge_pointers: Vec::new(),
            active_intention: None,
        }
    }

    pub fn set_intention(&mut self, intention: AgentIntention) {
        self.active_intention = Some(intention);
    }
}
