use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqCoverage {
    pub schema_version: String,
    pub generated_at: String,
    pub integrity_set: Vec<String>,
    pub contributions: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqDependencyClosure {
    pub schema_version: String,
    pub generated_at: String,
    pub integrity_set: Vec<String>,
    pub overall_closure_state: String,
    pub rows: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqGateCounts {
    pub pass: u64,
    pub fail: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqGateSummary {
    pub schema_version: String,
    pub generated_at: String,
    pub mode: String,
    pub latest_run_id: String,
    pub overall_verdict: String,
    pub required_gates_pass: bool,
    pub counts: SiqGateCounts,
    pub failures: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqGraphProjection {
    pub schema_version: String,
    pub generated_at: String,
    pub run_id: String,
    pub graph_fingerprint: String,
    pub integrity_set: Vec<String>,
    pub edge_types: Vec<String>,
    pub entities: Value,
    pub edges: Vec<Value>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct SiqRunArtifact {
    pub schema_version: String,
    pub run_id: String,
    pub generated_at: String,
    pub mode: String,
    pub policy_path: String,
    pub policy_version: u64,
    pub overall_verdict: String,
    pub required_gates_pass: bool,
    pub counts: SiqGateCounts,
    pub failures: Vec<Value>,
    pub results: Vec<Value>,
    pub git_commit: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct SiqHealth {
    pub status: String,
    pub siq_log_dir: String,
    pub schema_version: String,
    pub coverage_exists: bool,
    pub dependency_exists: bool,
    pub gate_exists: bool,
    pub projection_exists: bool,
    pub runs_count: usize,
    pub latest_run_last_modified: Option<u64>,
    pub coverage_fresh: bool,
    pub dependency_fresh: bool,
    pub gate_fresh: bool,
    pub projection_fresh: bool,
}
