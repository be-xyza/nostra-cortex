use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CreationMode {
    Blank,
    Import,
    Template,
    Preview,
    Observed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SpaceStatus {
    Provisioning,
    Quarantine,
    Active,
    Archived,
    Tombstoned,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRecord {
    pub space_id: String,
    pub creation_mode: CreationMode,
    pub reference_uri: Option<String>,
    pub template_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub draft_source_mode: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub lineage_note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub governance_scope: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility_state: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_graph_uri: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_graph_version: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability_graph_hash: Option<String>,
    pub status: SpaceStatus,
    pub created_at: String,
    pub owner: String,
    #[serde(default)]
    pub members: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub archetype: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SpaceRegistry {
    pub spaces: BTreeMap<String, SpaceRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpaceAgentRoutingOverride {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRoutingSettings {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub adapter_set_ref: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub provider_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_model: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth_binding_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent_routing_policy: Option<String>,
    #[serde(default)]
    pub agent_overrides: BTreeMap<String, SpaceAgentRoutingOverride>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub updated_by: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "camelCase")]
pub struct SpaceRuntimeSettingsRegistry {
    #[serde(default)]
    pub spaces: BTreeMap<String, SpaceRoutingSettings>,
}

impl SpaceRegistry {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read space registry: {}", e))?;
        serde_json::from_str(&content).map_err(|e| format!("Failed to parse space registry: {}", e))
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create registry directory: {}", e))?;
        }
        if path.exists() {
            fs::copy(path, Self::backup_path_for(path))
                .map_err(|e| format!("Failed to create registry backup: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize space registry: {}", e))?;
        let tmp_path = Self::temp_path_for(path);
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create registry temp file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write space registry temp file: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync space registry temp file: {}", e))?;
        fs::rename(&tmp_path, path)
            .map_err(|e| format!("Failed to install space registry temp file: {}", e))?;
        Ok(())
    }

    fn temp_path_for(path: &Path) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("{}.tmp", path.display()))
    }

    fn backup_path_for(path: &Path) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("{}.bak", path.display()))
    }

    pub fn upsert(&mut self, record: SpaceRecord) {
        self.spaces.insert(record.space_id.clone(), record);
    }

    pub fn get(&self, space_id: &str) -> Option<&SpaceRecord> {
        self.spaces.get(space_id)
    }

    pub fn list_active(&self) -> Vec<&SpaceRecord> {
        self.spaces
            .values()
            .filter(|r| r.status == SpaceStatus::Active)
            .collect()
    }
}

impl SpaceRuntimeSettingsRegistry {
    pub fn load_from_path(path: &Path) -> Result<Self, String> {
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read space runtime settings: {}", e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse space runtime settings: {}", e))
    }

    pub fn save_to_path(&self, path: &Path) -> Result<(), String> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| format!("Failed to create runtime settings directory: {}", e))?;
        }
        if path.exists() {
            fs::copy(path, SpaceRegistry::backup_path_for(path))
                .map_err(|e| format!("Failed to create runtime settings backup: {}", e))?;
        }
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize runtime settings: {}", e))?;
        let tmp_path = SpaceRegistry::temp_path_for(path);
        let mut file = fs::File::create(&tmp_path)
            .map_err(|e| format!("Failed to create runtime settings temp file: {}", e))?;
        file.write_all(content.as_bytes())
            .map_err(|e| format!("Failed to write runtime settings temp file: {}", e))?;
        file.sync_all()
            .map_err(|e| format!("Failed to sync runtime settings temp file: {}", e))?;
        fs::rename(&tmp_path, path)
            .map_err(|e| format!("Failed to install runtime settings temp file: {}", e))?;
        Ok(())
    }

    pub fn get(&self, space_id: &str) -> Option<&SpaceRoutingSettings> {
        self.spaces.get(space_id)
    }

    pub fn upsert(&mut self, space_id: impl Into<String>, settings: SpaceRoutingSettings) {
        self.spaces.insert(space_id.into(), settings);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn load_legacy_space_record_without_capability_graph_fields() {
        let raw = r#"{
            "spaces": {
                "space-alpha": {
                    "spaceId": "space-alpha",
                    "creationMode": "blank",
                    "referenceUri": null,
                    "templateId": null,
                    "status": "active",
                    "createdAt": "1700000000",
                    "owner": "system"
                }
            }
        }"#;
        let registry: SpaceRegistry =
            serde_json::from_str(raw).expect("legacy parse should succeed");
        let record = registry.get("space-alpha").expect("space should exist");
        assert_eq!(record.capability_graph_uri, None);
        assert_eq!(record.capability_graph_version, None);
        assert_eq!(record.capability_graph_hash, None);
        assert_eq!(record.draft_id, None);
        assert_eq!(record.draft_source_mode, None);
        assert_eq!(record.lineage_note, None);
        assert_eq!(record.governance_scope, None);
        assert_eq!(record.visibility_state, None);
        assert!(record.members.is_empty());
        assert_eq!(record.archetype, None);
    }

    #[test]
    fn round_trip_space_record_with_capability_graph_fields() {
        let mut registry = SpaceRegistry::default();
        registry.upsert(SpaceRecord {
            space_id: "space-beta".to_string(),
            creation_mode: CreationMode::Template,
            reference_uri: Some("nostra://ref/space-beta".to_string()),
            template_id: Some("template.foundation".to_string()),
            draft_id: Some("draft-space-1".to_string()),
            draft_source_mode: Some("template".to_string()),
            lineage_note: Some("Started from the foundation template.".to_string()),
            governance_scope: Some("private".to_string()),
            visibility_state: Some("members_only".to_string()),
            capability_graph_uri: Some("_spaces/space-beta/capability_graph.json".to_string()),
            capability_graph_version: Some("catalog-v1".to_string()),
            capability_graph_hash: Some("hash-123".to_string()),
            status: SpaceStatus::Active,
            created_at: "1700000001".to_string(),
            owner: "systems-steward".to_string(),
            members: vec![
                "systems-steward".to_string(),
                "agent:cortex-worker-01".to_string(),
            ],
            archetype: Some("Research".to_string()),
        });

        let encoded = serde_json::to_string(&registry).expect("encode");
        let decoded: SpaceRegistry = serde_json::from_str(&encoded).expect("decode");
        let record = decoded.get("space-beta").expect("space should exist");
        assert_eq!(
            record.capability_graph_uri.as_deref(),
            Some("_spaces/space-beta/capability_graph.json")
        );
        assert_eq!(
            record.capability_graph_version.as_deref(),
            Some("catalog-v1")
        );
        assert_eq!(record.capability_graph_hash.as_deref(), Some("hash-123"));
        assert_eq!(record.draft_id.as_deref(), Some("draft-space-1"));
        assert_eq!(record.draft_source_mode.as_deref(), Some("template"));
        assert_eq!(
            record.lineage_note.as_deref(),
            Some("Started from the foundation template.")
        );
        assert_eq!(record.governance_scope.as_deref(), Some("private"));
        assert_eq!(record.visibility_state.as_deref(), Some("members_only"));
        assert_eq!(record.members.len(), 2);
        assert_eq!(record.archetype.as_deref(), Some("Research"));
    }

    #[test]
    fn save_to_path_creates_backup_when_overwriting() {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos();
        let path = std::env::temp_dir().join(format!("space-registry-{stamp}.json"));

        let mut registry = SpaceRegistry::default();
        registry.upsert(SpaceRecord {
            space_id: "space-gamma".to_string(),
            creation_mode: CreationMode::Blank,
            reference_uri: None,
            template_id: None,
            draft_id: None,
            draft_source_mode: None,
            lineage_note: None,
            governance_scope: None,
            visibility_state: None,
            capability_graph_uri: None,
            capability_graph_version: None,
            capability_graph_hash: None,
            status: SpaceStatus::Active,
            created_at: "1700000002".to_string(),
            owner: "systems-steward".to_string(),
            members: vec![],
            archetype: None,
        });
        registry.save_to_path(&path).expect("initial save");

        registry
            .spaces
            .get_mut("space-gamma")
            .expect("space-gamma")
            .members
            .push("agent:cortex-worker-01".to_string());
        registry.save_to_path(&path).expect("second save");

        assert!(SpaceRegistry::backup_path_for(&path).exists());

        let _ = fs::remove_file(SpaceRegistry::backup_path_for(&path));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn runtime_settings_round_trip_with_agent_overrides() {
        let mut registry = SpaceRuntimeSettingsRegistry::default();
        let mut overrides = BTreeMap::new();
        overrides.insert(
            "agent:eudaemon-alpha-01".to_string(),
            SpaceAgentRoutingOverride {
                agent_id: Some("agent:eudaemon-alpha-01".to_string()),
                adapter_set_ref: Some("adapter.primary".to_string()),
                default_model: Some("gpt-5.4".to_string()),
                auth_binding_id: Some("cred-openai".to_string()),
                auth_mode: Some("api_key".to_string()),
                provider_id: Some("openrouter_primary".to_string()),
            },
        );

        registry.upsert(
            "space-alpha",
            SpaceRoutingSettings {
                adapter_set_ref: Some("adapter.primary".to_string()),
                default_model: Some("gpt-5.4".to_string()),
                auth_binding_id: Some("cred-openai".to_string()),
                provider_id: Some("openrouter_primary".to_string()),
                agent_routing_policy: Some("space_default_with_agent_overrides".to_string()),
                agent_overrides: overrides,
                updated_at: Some("2026-03-22T00:00:00Z".to_string()),
                updated_by: Some("systems-steward".to_string()),
            },
        );

        let encoded = serde_json::to_string(&registry).expect("encode");
        let decoded: SpaceRuntimeSettingsRegistry = serde_json::from_str(&encoded).expect("decode");
        let settings = decoded.get("space-alpha").expect("space settings");
        assert_eq!(settings.default_model.as_deref(), Some("gpt-5.4"));
        assert_eq!(
            settings
                .agent_overrides
                .get("agent:eudaemon-alpha-01")
                .and_then(|entry| entry.auth_binding_id.as_deref()),
            Some("cred-openai")
        );
    }
}
