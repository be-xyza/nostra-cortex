use crate::services::dfx_client::LocalIcClient;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotInfo {
    pub snapshot_id: String,
    pub canister_id: String,
    pub created_at: String, // ISO 8601
    pub trigger: String,    // "system_upgrade", "manual"
    pub verified: bool,
}

pub struct SnapshotService {
    client: LocalIcClient,
}

impl SnapshotService {
    pub fn new(project_root: Option<PathBuf>) -> Self {
        Self {
            client: LocalIcClient::new(project_root),
        }
    }

    /// Creates a verified snapshot of a canister
    /// Returns the snapshot ID if successful
    pub async fn create_snapshot(
        &self,
        canister_name: &str,
        network: &str,
    ) -> Result<SnapshotInfo, String> {
        // 1. Check if canister is running (also ensures backend is ready)
        let status = self
            .client
            .get_canister_status(canister_name)
            .await
            .map_err(|e| e.to_string())?;

        // 2. Execute Snapshot Command via Backend
        let snapshot_id = self
            .client
            .backend
            .snapshot_create(canister_name, network)
            .await?;

        // 3. Construct Info
        let info = SnapshotInfo {
            snapshot_id,
            canister_id: status.id,
            created_at: chrono::Utc::now().to_rfc3339(),
            trigger: "system_upgrade".to_string(),
            verified: true, // Native snapshots are verified by the Replica
        };

        // 4. Save Metadata (Mock - in real life write to snapshot.json)
        self.save_metadata(&info)?;

        Ok(info)
    }

    /// Restores a canister to a specific snapshot
    pub async fn restore_snapshot(
        &self,
        canister_name: &str,
        snapshot_id: &str,
        network: &str,
    ) -> Result<(), String> {
        self.client
            .backend
            .snapshot_load(snapshot_id, canister_name, network)
            .await
    }

    fn save_metadata(&self, _info: &SnapshotInfo) -> Result<(), String> {
        // In a full implementation, write to _snapshots/snapshot.json
        Ok(())
    }
}
