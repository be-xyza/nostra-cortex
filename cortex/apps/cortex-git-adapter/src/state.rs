use chrono::Utc;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub fn ensure_state_dirs(root: &Path) -> Result<(), String> {
    fs::create_dir_all(root.join("deliveries"))
        .map_err(|e| format!("failed to create deliveries dir: {e}"))?;
    fs::create_dir_all(root.join("repos"))
        .map_err(|e| format!("failed to create repos dir: {e}"))?;
    Ok(())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeliveryRecord {
    pub received_at: String,
    pub delivery_id: String,
    pub event: String,
    pub repo_full_name: String,
    pub payload_sha256: String,
}

impl DeliveryRecord {
    pub fn new(delivery_id: &str, event: &str, repo_full_name: &str, payload: &[u8]) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let digest = hasher.finalize();
        Self {
            received_at: Utc::now().to_rfc3339(),
            delivery_id: delivery_id.to_string(),
            event: event.to_string(),
            repo_full_name: repo_full_name.to_string(),
            payload_sha256: hex::encode(digest),
        }
    }
}

pub struct IdempotencyStore {
    root: PathBuf,
}

impl IdempotencyStore {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    fn path_for(&self, delivery_id: &str) -> PathBuf {
        self.root
            .join("deliveries")
            .join(format!("{}.json", sanitize_token(delivery_id)))
    }

    pub fn is_seen(&self, delivery_id: &str) -> Result<bool, String> {
        Ok(self.path_for(delivery_id).exists())
    }

    pub fn mark_seen(&self, record: &DeliveryRecord) -> Result<(), String> {
        let path = self.path_for(&record.delivery_id);
        let payload = serde_json::to_string_pretty(record)
            .map_err(|e| format!("failed to encode delivery record: {e}"))?;
        atomic_write_text(&path, payload.as_str())
    }
}

fn sanitize_token(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

pub fn atomic_write_text(path: &Path, content: &str) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or_else(|| "path has no parent".to_string())?;
    fs::create_dir_all(parent).map_err(|e| format!("mkdir {}: {e}", parent.display()))?;

    let tmp = parent.join(format!(
        ".tmp-{}-{}",
        sanitize_token(
            path.file_name()
                .and_then(|v| v.to_str())
                .unwrap_or("file")
        ),
        uuid::Uuid::new_v4()
    ));
    fs::write(&tmp, content).map_err(|e| format!("write {}: {e}", tmp.display()))?;
    fs::rename(&tmp, path).map_err(|e| format!("rename {} -> {}: {e}", tmp.display(), path.display()))?;
    Ok(())
}

pub fn prune_deliveries(root: &Path, retention_days: u64) -> Result<usize, String> {
    let retention = Duration::from_secs(retention_days.saturating_mul(24 * 60 * 60));
    let cutoff = SystemTime::now()
        .checked_sub(retention)
        .unwrap_or(SystemTime::UNIX_EPOCH);

    let dir = root.join("deliveries");
    let mut removed = 0usize;
    let entries = fs::read_dir(&dir).map_err(|e| format!("read_dir {}: {e}", dir.display()))?;
    for entry in entries {
        let entry = entry.map_err(|e| format!("read_dir entry: {e}"))?;
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if modified < cutoff {
            if fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}
