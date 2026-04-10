// -- Monitor Types --

#[derive(Clone, Debug, Deserialize, Serialize, CandidType, PartialEq)]
pub enum MonitorLevel {
    Critical,
    Warning,
    Healthy,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType, PartialEq)]
pub enum CanisterState {
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "stopping")]
    Stopping,
    #[serde(rename = "stopped")]
    Stopped,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct CanisterStatus {
    #[serde(rename = "canister_id")]
    pub canister_id: String,
    pub name: String,
    pub status: CanisterState,
    #[serde(rename = "memory_size")]
    pub memory_size: candid::Nat,
    pub cycles: candid::Nat,
    #[serde(rename = "module_hash")]
    pub module_hash: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct SystemMetrics {
    #[serde(rename = "active_workflows")]
    pub active_workflows: candid::Nat,
    #[serde(rename = "error_count_24h")]
    pub error_count_24h: candid::Nat,
    #[serde(rename = "active_users_24h")]
    pub active_users_24h: candid::Nat,
}

#[derive(Clone, Debug, Deserialize, Serialize, CandidType)]
pub struct SystemStatus {
    pub version: String,
    pub status: MonitorLevel,
    #[serde(rename = "uptime_seconds")]
    pub uptime_seconds: candid::Int,
    pub canisters: Vec<CanisterStatus>,
    pub metrics: SystemMetrics,
    #[serde(rename = "last_updated")]
    pub last_updated: candid::Int,
}
