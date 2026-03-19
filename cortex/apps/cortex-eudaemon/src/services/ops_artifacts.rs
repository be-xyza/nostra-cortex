use crate::gateway::server::workspace_root;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactIndexItem {
    pub artifact_id: String,
    pub title: String,
    pub status: String,
    pub updated_at: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub published_at: Option<String>,
    pub head_revision_id: String,
    pub version: u64,
    pub route_id: String,
    pub owner_role: String,
    pub source_of_truth: String,
    pub fallback_active: bool,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ArtifactIndexResponse {
    pub schema_version: String,
    pub generated_at: String,
    pub count: usize,
    pub items: Vec<ArtifactIndexItem>,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ArtifactStoreRecord {
    artifact_id: String,
    title: String,
    status: String,
    updated_at: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    published_at: Option<String>,
    head_revision_id: String,
    version: u64,
    route_id: String,
    owner_role: String,
    source_of_truth: String,
    fallback_active: bool,
}

fn workspace_logs_dir() -> PathBuf {
    workspace_root().join("logs")
}

fn cortex_ux_log_dir() -> PathBuf {
    std::env::var("NOSTRA_CORTEX_UX_LOG_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| workspace_logs_dir().join("cortex").join("ux"))
}

fn cortex_ux_artifacts_store_path() -> PathBuf {
    cortex_ux_log_dir().join("artifacts_store.json")
}

fn read_artifacts_store() -> Result<Vec<ArtifactStoreRecord>, String> {
    let path = cortex_ux_artifacts_store_path();
    let raw = fs::read_to_string(&path)
        .map_err(|err| format!("failed_to_read_artifacts_store:{}:{err}", path.display()))?;
    serde_json::from_str::<Vec<ArtifactStoreRecord>>(&raw)
        .map_err(|err| format!("failed_to_parse_artifacts_store:{}:{err}", path.display()))
}

pub(crate) fn list_artifacts(limit: usize) -> Result<ArtifactIndexResponse, String> {
    let mut artifacts = read_artifacts_store()?;
    artifacts.sort_by(|a, b| {
        b.updated_at
            .cmp(&a.updated_at)
            .then_with(|| b.artifact_id.cmp(&a.artifact_id))
    });
    let count = artifacts.len();
    let items = artifacts
        .into_iter()
        .take(limit.clamp(1, 200))
        .map(|artifact| ArtifactIndexItem {
            artifact_id: artifact.artifact_id,
            title: artifact.title,
            status: artifact.status,
            updated_at: artifact.updated_at,
            published_at: artifact.published_at,
            head_revision_id: artifact.head_revision_id,
            version: artifact.version,
            route_id: artifact.route_id,
            owner_role: artifact.owner_role,
            source_of_truth: artifact.source_of_truth,
            fallback_active: artifact.fallback_active,
        })
        .collect::<Vec<_>>();

    Ok(ArtifactIndexResponse {
        schema_version: "1.0.0".to_string(),
        generated_at: crate::services::viewspec::now_iso(),
        count,
        items,
    })
}
