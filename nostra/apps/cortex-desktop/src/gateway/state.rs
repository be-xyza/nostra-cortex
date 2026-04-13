use crate::services::acp_adapter::{AcpAdapter, AcpPolicyConfig};
use crate::services::file_system_service::FileSystemService;
use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct GatewayState {
    adapter: Arc<AcpAdapter>,
    workspace_root: PathBuf,
}

impl GatewayState {
    pub fn load_default() -> Result<Self, String> {
        let workspace_root = FileSystemService::get_root_path();
        let cfg = AcpPolicyConfig::baseline(vec![workspace_root.clone()]);
        let adapter = AcpAdapter::new(cfg).map_err(|err| err.to_string())?;
        Ok(Self {
            adapter: Arc::new(adapter),
            workspace_root,
        })
    }

    pub fn adapter(&self) -> Arc<AcpAdapter> {
        Arc::clone(&self.adapter)
    }

    pub fn workspace_root(&self) -> &PathBuf {
        &self.workspace_root
    }
}
