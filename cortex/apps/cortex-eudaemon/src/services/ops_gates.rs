use crate::gateway::server::workspace_root;
use crate::services::siq_types::SiqGateSummary;
use crate::services::testing_service::TestGateSummaryArtifact;
use serde::de::DeserializeOwned;
use std::fs;
use std::path::{Path, PathBuf};

fn resolve_testing_log_dir() -> PathBuf {
    std::env::var("NOSTRA_TESTING_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("testing"))
}

fn resolve_siq_log_dir() -> PathBuf {
    std::env::var("NOSTRA_SIQ_LOG_DIR")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| workspace_root().join("logs").join("siq"))
}

fn testing_gate_summary_path() -> PathBuf {
    resolve_testing_log_dir().join("test_gate_summary_latest.json")
}

fn siq_gate_summary_path() -> PathBuf {
    resolve_siq_log_dir().join("siq_gate_summary_latest.json")
}

fn read_json_artifact<T: DeserializeOwned>(path: &Path) -> Result<T, String> {
    let raw = fs::read_to_string(path)
        .map_err(|err| format!("failed_to_read_artifact:{}:{err}", path.display()))?;
    serde_json::from_str::<T>(&raw)
        .map_err(|err| format!("failed_to_parse_artifact:{}:{err}", path.display()))
}

pub(crate) fn load_testing_gate_summary() -> Result<TestGateSummaryArtifact, String> {
    read_json_artifact(&testing_gate_summary_path())
}

pub(crate) fn load_siq_gate_summary() -> Result<SiqGateSummary, String> {
    read_json_artifact(&siq_gate_summary_path())
}
