use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct ActorRecord {
    pub actor_id: String,
    pub actor_type: String,
    #[serde(default)]
    pub roles: Vec<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, Eq)]
pub struct ActorRegistry {
    pub actors: BTreeMap<String, ActorRecord>,
}

fn backup_path_for(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.bak", path.display()))
}

impl ActorRegistry {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read actor registry: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse actor registry: {}", e))
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create actor registry directory: {}", e))?;
        }
        if path.exists() {
            fs::copy(path, backup_path_for(path))
                .map_err(|e| format!("Failed to back up actor registry: {}", e))?;
        }

        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize actor registry: {}", e))?;
        let tmp_path = PathBuf::from(format!("{}.tmp", path.display()));
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create actor registry temp file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write actor registry temp file: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to fsync actor registry temp file: {}", e))?;
        fs::rename(&tmp_path, path)
            .map_err(|e| format!("Failed to install actor registry temp file: {}", e))?;
        Ok(())
    }

    pub fn upsert(&mut self, record: ActorRecord) {
        self.actors.insert(record.actor_id.clone(), record);
    }

    pub fn contains(&self, actor_id: &str) -> bool {
        self.actors.contains_key(actor_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_path(prefix: &str) -> PathBuf {
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        std::env::temp_dir().join(format!("{prefix}-{stamp}.json"))
    }

    #[test]
    fn actor_registry_round_trip_and_contains() {
        let path = temp_path("actor-registry");
        let mut registry = ActorRegistry::default();
        registry.upsert(ActorRecord {
            actor_id: "agent:cortex-worker-01".to_string(),
            actor_type: "agent".to_string(),
            roles: vec!["operator".to_string()],
            status: "active".to_string(),
        });
        registry.save_to_path(&path).expect("save actor registry");

        let loaded = ActorRegistry::load_from_path(&path).expect("load actor registry");
        assert!(loaded.contains("agent:cortex-worker-01"));

        let _ = fs::remove_file(path);
    }
}
