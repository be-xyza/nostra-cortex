use crate::workspace::discover_workspace_root;
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub bind: String,
    pub port: u16,
    pub registry_path: PathBuf,
    pub state_dir: PathBuf,
    pub max_request_body_bytes: usize,
    pub request_timeout_secs: u64,
    pub delivery_retention_days: u64,
    pub reconcile_per_page: usize,
    pub reconcile_max_pages: usize,
    pub projector_emit_attributes: bool,
    pub projector_store_author_email: bool,
    pub webhook_secret: Option<String>,
    pub github_api_base: String,
    pub github_token: Option<String>,
    pub github_app_id: Option<String>,
    pub github_app_installation_id: Option<String>,
    pub github_app_private_key_pem: Option<String>,
    pub github_app_private_key_path: Option<PathBuf>,
    pub nostra_ic_host: String,
    pub nostra_kip_canister_id: Option<String>,
    pub kip_method: String,
    pub use_dfx: bool,
    pub dfx_canister_name: String,
    pub dfx_project_root: Option<PathBuf>,
}

impl AppConfig {
    pub fn load() -> anyhow::Result<Self> {
        let bind = std::env::var("CORTEX_GIT_ADAPTER_BIND").unwrap_or_else(|_| "0.0.0.0".to_string());
        let port = std::env::var("CORTEX_GIT_ADAPTER_PORT")
            .ok()
            .and_then(|v| v.parse::<u16>().ok())
            .unwrap_or(8787);

        let workspace_root = std::env::var("NOSTRA_WORKSPACE_ROOT")
            .ok()
            .map(PathBuf::from)
            .or_else(|| discover_workspace_root(std::env::current_dir().ok()?))
            .ok_or_else(|| anyhow::anyhow!("unable to resolve workspace root; set NOSTRA_WORKSPACE_ROOT"))?;

        let registry_path = std::env::var("CORTEX_GIT_ADAPTER_REGISTRY_PATH")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root.join("cortex/apps/cortex-git-adapter/config/git_adapter_registry.toml"));

        let state_dir = std::env::var("CORTEX_GIT_ADAPTER_STATE_DIR")
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| workspace_root.join("logs").join("git_adapter"));

        let max_request_body_bytes = std::env::var("CORTEX_GIT_ADAPTER_MAX_REQUEST_BODY_BYTES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(1024 * 1024);

        let request_timeout_secs = std::env::var("CORTEX_GIT_ADAPTER_REQUEST_TIMEOUT_SECS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(10);

        let delivery_retention_days = std::env::var("CORTEX_GIT_ADAPTER_DELIVERY_RETENTION_DAYS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(30);

        let reconcile_per_page = std::env::var("CORTEX_GIT_ADAPTER_RECONCILE_PER_PAGE")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(50)
            .clamp(1, 100);

        let reconcile_max_pages = std::env::var("CORTEX_GIT_ADAPTER_RECONCILE_MAX_PAGES")
            .ok()
            .and_then(|v| v.parse::<usize>().ok())
            .unwrap_or(10)
            .clamp(1, 50);

        let projector_emit_attributes = std::env::var("CORTEX_GIT_ADAPTER_KIP_EMIT_ATTRIBUTES")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let projector_store_author_email = std::env::var("CORTEX_GIT_ADAPTER_STORE_AUTHOR_EMAIL")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let webhook_secret = std::env::var("CORTEX_GIT_ADAPTER_WEBHOOK_SECRET").ok();

        let github_api_base = std::env::var("GITHUB_API_BASE").unwrap_or_else(|_| "https://api.github.com".to_string());
        let github_token = std::env::var("GITHUB_TOKEN").ok();
        let github_app_id = std::env::var("GITHUB_APP_ID").ok();
        let github_app_installation_id = std::env::var("GITHUB_APP_INSTALLATION_ID").ok();
        let github_app_private_key_pem = std::env::var("GITHUB_APP_PRIVATE_KEY_PEM").ok();
        let github_app_private_key_path = std::env::var("GITHUB_APP_PRIVATE_KEY_PATH").ok().map(PathBuf::from);

        let nostra_ic_host = std::env::var("NOSTRA_IC_HOST")
            .or_else(|_| std::env::var("IC_HOST"))
            .unwrap_or_else(|_| "http://127.0.0.1:4943".to_string());

        let nostra_kip_canister_id = std::env::var("CANISTER_ID_NOSTRA_KIP")
            .ok()
            .or_else(|| std::env::var("NOSTRA_KIP_CANISTER_ID").ok());

        let nostra_backend_canister_id = std::env::var("CANISTER_ID_NOSTRA_BACKEND")
            .ok()
            .or_else(|| std::env::var("NOSTRA_BACKEND_CANISTER_ID").ok())
            .or_else(|| std::env::var("CANISTER_ID_BACKEND").ok());

        let kip_method = std::env::var("CORTEX_GIT_ADAPTER_KIP_METHOD")
            .unwrap_or_else(|_| "execute_kip_mutation".to_string());

        let use_dfx = std::env::var("CORTEX_GIT_ADAPTER_USE_DFX")
            .map(|v| v.trim() == "1" || v.trim().eq_ignore_ascii_case("true"))
            .unwrap_or(false);
        let dfx_canister_name = std::env::var("CORTEX_GIT_ADAPTER_DFX_CANISTER")
            .unwrap_or_else(|_| "nostra_backend".to_string());
        let dfx_project_root = std::env::var("CORTEX_GIT_ADAPTER_DFX_PROJECT_ROOT")
            .ok()
            .map(PathBuf::from);

        Ok(Self {
            bind,
            port,
            registry_path,
            state_dir,
            max_request_body_bytes,
            request_timeout_secs,
            delivery_retention_days,
            reconcile_per_page,
            reconcile_max_pages,
            projector_emit_attributes,
            projector_store_author_email,
            webhook_secret,
            github_api_base,
            github_token,
            github_app_id,
            github_app_installation_id,
            github_app_private_key_pem,
            github_app_private_key_path,
            nostra_ic_host,
            nostra_kip_canister_id: nostra_kip_canister_id.or_else(|| nostra_backend_canister_id.clone()),
            kip_method,
            use_dfx,
            dfx_canister_name,
            dfx_project_root,
        })
    }

    pub fn load_registry(&self) -> anyhow::Result<Registry> {
        let raw = std::fs::read_to_string(&self.registry_path)
            .map_err(|e| anyhow::anyhow!("failed to read registry {}: {e}", self.registry_path.display()))?;
        let parsed: RegistryFile = toml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("failed to parse registry {}: {e}", self.registry_path.display()))?;
        Ok(Registry::from_file(parsed))
    }
}

#[derive(Clone, Debug)]
pub struct ProjectorSettings {
    pub emit_attributes: bool,
    pub store_author_email: bool,
}

impl AppConfig {
    pub fn github_auth_is_configured(&self) -> bool {
        self.github_token.as_deref().unwrap_or("").trim().len() > 0
            || (self.github_app_id.as_deref().unwrap_or("").trim().len() > 0
                && self.github_app_installation_id
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .len()
                    > 0
                && (self
                    .github_app_private_key_pem
                    .as_deref()
                    .unwrap_or("")
                    .trim()
                    .len()
                    > 0
                    || self.github_app_private_key_path.is_some()))
    }

    pub fn nostra_sink_is_configured(&self) -> bool {
        if self.use_dfx {
            return self.dfx_project_root.is_some();
        }
        self.nostra_kip_canister_id
            .as_deref()
            .unwrap_or("")
            .trim()
            .len()
            > 0
    }

    pub fn projector_settings(&self) -> ProjectorSettings {
        ProjectorSettings {
            emit_attributes: self.projector_emit_attributes,
            store_author_email: self.projector_store_author_email,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct RegistryFile {
    pub repos: BTreeMap<String, RepoConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepoConfig {
    pub enabled: bool,
    pub repo_full_name: String,
    #[serde(default)]
    pub branch: Option<String>,
    pub space_id: String,
    pub ingest_push: bool,
    pub ingest_pull_request: bool,
    pub ingest_issues: bool,
    pub interval_secs: u64,
    pub lookback_secs: u64,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Registry {
    pub repos: Vec<RepoConfig>,
}

impl Registry {
    fn from_file(file: RegistryFile) -> Self {
        let mut repos = file.repos.into_values().collect::<Vec<_>>();
        repos.sort_by(|a, b| a.repo_full_name.cmp(&b.repo_full_name));
        Self { repos }
    }

    pub fn repo_by_full_name(&self, name: &str) -> Option<&RepoConfig> {
        let needle = name.trim();
        self.repos.iter().find(|r| r.repo_full_name == needle)
    }

    pub fn iter_enabled(&self) -> impl Iterator<Item = &RepoConfig> {
        self.repos.iter().filter(|r| r.enabled)
    }
}

impl RepoConfig {
    pub fn branch(&self) -> &str {
        self.branch.as_deref().unwrap_or("main")
    }
}

pub fn sanitize_path_component(raw: &str) -> String {
    raw.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}
