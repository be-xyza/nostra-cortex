use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HumanApprovalEvent {
    pub scenario_id: String,
    pub space_id: String,
    pub actor: String,
    pub decision: ApprovalDecision,
    pub rationale: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ApprovalDecision {
    Approved,
    Rejected,
    NeedsModification,
}
