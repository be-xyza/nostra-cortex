use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;
use nostra_shared::types::provider_registry::{ProviderRecord, LlmProviderType};

// Global singleton config instance
pub static CONFIG: OnceLock<ConfigService> = OnceLock::new();

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ConfigMatrix {
    pub environment: Environment,
    pub services: ServiceConfig,
    pub governance: GovernanceConfig,
    pub canisters: CanisterConfig,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Environment {
    Local,
    Testnet,
    Mainnet,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ServiceConfig {
    pub vector: VectorConfig,
    pub llm: LlmConfig,
    pub graph: GraphConfig,
    pub providers: Vec<ProviderRecord>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorConfig {
    pub primary: VectorProviderConfig,
    pub fallback_strategy: Vec<VectorProviderType>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VectorProviderConfig {
    pub provider: VectorProviderType,
    pub endpoint: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum VectorProviderType {
    Elna,
    Bleve,
    Solr,
    Mock,
    RegexScan,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LlmConfig {
    pub primary_provider: LlmProviderType,
    pub api_base: String,
    pub fallback_chain: Vec<LlmProviderType>,
}

// Removed local LlmProviderType - now using shared version

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GraphConfig {
    pub sync_mode: GraphSyncMode,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum GraphSyncMode {
    Full,
    Spoke,
    QueryOnly,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct GovernanceConfig {
    pub deployment_gate: String,
    pub audit_level: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CanisterConfig {
    pub primary: Option<String>,
    pub streaming: Option<String>,
    pub backend: Option<String>,
}

#[derive(Clone, Debug)]
pub struct ConfigService {
    config: ConfigMatrix,
}

impl ConfigService {
    pub fn load() -> Self {
        let config_path = "nostra_config.json";
        let mut config = if Path::new(config_path).exists() {
            println!("Configuration: Loading from {}", config_path);
            let content = fs::read_to_string(config_path).unwrap_or_else(|e| {
                eprintln!("Configuration: Failed to read {}: {}", config_path, e);
                // Return a dummy content that will fail parsing and trigger default
                "{}".to_string()
            });
            serde_json::from_str(&content).unwrap_or_else(|e| {
                eprintln!(
                    "Configuration: Failed to parse file, falling back to default. Error: {}",
                    e
                );
                Self::default_config()
            })
        } else {
            println!("Configuration: File not found, using Defaults (Local)");
            Self::default_config()
        };

        // Optional runtime overrides (useful when no config file is present).
        if let Ok(endpoint) = std::env::var("NOSTRA_VECTOR_ENDPOINT") {
            if !endpoint.trim().is_empty() {
                config.services.vector.primary.endpoint = endpoint;
            }
        }
        if let Ok(provider_raw) = std::env::var("NOSTRA_VECTOR_PROVIDER") {
            let provider = match provider_raw.to_lowercase().as_str() {
                "elna" => Some(VectorProviderType::Elna),
                "bleve" => Some(VectorProviderType::Bleve),
                "solr" => Some(VectorProviderType::Solr),
                "mock" => Some(VectorProviderType::Mock),
                "regexscan" | "regex_scan" => Some(VectorProviderType::RegexScan),
                _ => None,
            };
            if let Some(p) = provider {
                config.services.vector.primary.provider = p;
            }
        }
        if let Ok(api_base) = std::env::var("NOSTRA_LLM_API_BASE") {
            if !api_base.trim().is_empty() {
                config.services.llm.api_base = api_base;
            }
        }

        // Canister ID overrides
        if let Ok(id) = std::env::var("CANISTER_ID") {
            config.canisters.primary = Some(id);
        }
        if let Ok(id) = std::env::var("CANISTER_ID_NOSTRA_STREAMING").or_else(|_| std::env::var("NOSTRA_STREAMING_CANISTER_ID")) {
            config.canisters.streaming = Some(id);
        }
        if let Ok(id) = std::env::var("CANISTER_ID_NOSTRA_BACKEND") {
            config.canisters.backend = Some(id);
        }

        Self { config }
    }

    fn default_config() -> ConfigMatrix {
        ConfigMatrix {
            environment: Environment::Local,
            services: ServiceConfig {
                vector: VectorConfig {
                    primary: VectorProviderConfig {
                        provider: VectorProviderType::Mock,
                        endpoint: "local_memory".to_string(),
                    },
                    fallback_strategy: vec![VectorProviderType::RegexScan],
                },
                llm: LlmConfig {
                    primary_provider: LlmProviderType::Ollama, // Cheap/Free local default
                    api_base: "http://localhost:11434".to_string(),
                    fallback_chain: vec![],
                },
                graph: GraphConfig {
                    sync_mode: GraphSyncMode::QueryOnly,
                },
                providers: vec![
                    ProviderRecord::new_llm("ollama-local", "Local Ollama", LlmProviderType::Ollama, "http://localhost:11434"),
                    ProviderRecord::new_llm("open-router", "OpenRouter", LlmProviderType::OpenRouter, "https://openrouter.ai/api/v1"),
                    ProviderRecord::new_llm("double-word-batch", "DoubleWord Batch", LlmProviderType::DoubleWord, "https://api.doubleword.ai/v1/batch"),
                ],
            },
            governance: GovernanceConfig {
                deployment_gate: "Open".to_string(),
                audit_level: "Minimal".to_string(),
            },
            canisters: CanisterConfig {
                primary: None,
                streaming: None,
                backend: None,
            },
        }
    }

    pub fn get() -> &'static ConfigService {
        CONFIG.get_or_init(Self::load)
    }

    pub fn get_env(&self) -> &Environment {
        &self.config.environment
    }

    pub fn get_vector_endpoints(&self) -> (String, Vec<VectorProviderType>) {
        (
            self.config.services.vector.primary.endpoint.clone(),
            self.config.services.vector.fallback_strategy.clone(),
        )
    }

    #[allow(dead_code)]
    pub fn get_llm_config(&self) -> Option<&LlmConfig> {
        Some(&self.config.services.llm)
    }

    pub fn get_providers(&self) -> &Vec<ProviderRecord> {
        &self.config.services.providers
    }

    pub fn get_canister_id(&self, name: &str) -> Option<ic_agent::export::Principal> {
        let id_str = match name {
            "primary" => self.config.canisters.primary.as_ref(),
            "streaming" => self.config.canisters.streaming.as_ref(),
            "backend" => self.config.canisters.backend.as_ref(),
            _ => return None,
        }?;

        ic_agent::export::Principal::from_text(id_str).ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config_loading() {
        let service = ConfigService::load(); // This loads default if file missing
        assert_eq!(service.config.environment, Environment::Local);
        assert_eq!(
            service.config.services.vector.primary.provider,
            VectorProviderType::Mock
        );
    }
}
