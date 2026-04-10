pub mod error;
pub mod event_bus;
pub mod ports;
#[cfg(feature = "baml-policy-experiments")]
pub mod policy_experiments;

use async_trait::async_trait;
use event_bus::{RuntimeSessionUpdateRequest, RuntimeSessionUpdateResult};
use serde::{Deserialize, Serialize};

pub use error::RuntimeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub event_source: String,
    pub event_type_prefix: String,
    pub remote_endpoint: Option<String>,
    pub fail_on_network_error: bool,
    pub shadow_mode: bool,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            event_source: "nostra://cortex-runtime".to_string(),
            event_type_prefix: "nostra.acp".to_string(),
            remote_endpoint: None,
            fail_on_network_error: false,
            shadow_mode: false,
        }
    }
}

#[async_trait]
pub trait CortexRuntime: Send + Sync {
    async fn publish_session_update(
        &self,
        request: RuntimeSessionUpdateRequest,
    ) -> Result<RuntimeSessionUpdateResult, RuntimeError>;
}
