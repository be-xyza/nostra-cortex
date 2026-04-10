use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum LabStatus {
    #[serde(rename = "alpha")]
    Alpha,
    #[serde(rename = "beta")]
    Beta,
    #[serde(rename = "experimental")]
    Experimental,
    #[serde(rename = "deprecated")]
    Deprecated,
    #[serde(rename = "prototype")]
    Prototype,
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize, Default)]
pub enum ActivityStatus {
    #[default]
    Active, // Activity within 7 days
    Dormant,  // Activity within 30 days
    Archived, // No activity > 30 days
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct LabManifest {
    pub id: String,
    pub name: String,
    pub description: String,
    pub version: String,
    pub thumbnail_url: Option<String>,
    pub route_path: Option<String>,
    pub status: LabStatus,
    pub supported_contexts: Vec<String>,

    // v2 Constitution Metadata
    #[serde(default = "default_hypothesis")]
    pub hypothesis: String,
    #[serde(skip)]
    pub activity_status: ActivityStatus,
    #[serde(default = "default_last_activity")]
    pub last_activity: String, // ISO 8601 timestamp
}

fn default_hypothesis() -> String {
    "Exploring new patterns for the Nostra network.".to_string()
}

fn default_last_activity() -> String {
    "2026-01-01T00:00:00Z".to_string()
}

// Frontend-only state mirroring the backend or local overrides
#[derive(Clone, Debug, PartialEq)]
pub struct UserLabState {
    pub lab_id: String,
    pub enabled: bool,
    pub is_favorite: bool,
    pub active_contexts: Vec<String>,
    // current_config: Option<String>, // JSON string
}

pub mod contexts {
    pub const CORTEX_GRAPH_TAB: &str = "cortex:graph-tab";
    pub const SPACES_NETWORK_VIEW: &str = "spaces:network-view";
    pub const AGENT_CHAT: &str = "agent:chat-view";
    pub const SYSTEM_GLOBAL: &str = "system:global";
    pub const CORTEX_LABS: &str = "cortex:labs-view";
}
