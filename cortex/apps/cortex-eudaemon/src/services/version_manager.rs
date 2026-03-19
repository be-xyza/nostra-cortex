use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionManifest {
    pub versions: HashMap<String, ReleaseInfo>, // key: "v0.9.0"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseInfo {
    pub version: String,
    pub canisters: HashMap<String, String>, // key: "nostra_backend", value: "sha256:..."
}

pub struct VersionManager {
    manifest: VersionManifest,
}

impl VersionManager {
    pub fn new() -> Self {
        // In a real app, load this from a baked-in JSON or remote URL
        Self {
            manifest: VersionManifest {
                versions: HashMap::new(),
            },
        }
    }

    pub fn load_manifest(&mut self, json_content: &str) -> Result<(), String> {
        self.manifest = serde_json::from_str(json_content).map_err(|e| e.to_string())?;
        Ok(())
    }

    pub fn verify_hash(&self, version: &str, canister: &str, active_hash: &str) -> bool {
        if let Some(release) = self.manifest.versions.get(version) {
            if let Some(expected_hash) = release.canisters.get(canister) {
                return expected_hash == active_hash;
            }
        }
        false
    }

    pub fn get_expected_hash(&self, version: &str, canister: &str) -> Option<String> {
        self.manifest
            .versions
            .get(version)
            .and_then(|r| r.canisters.get(canister).cloned())
    }
}
