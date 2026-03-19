pub mod agents;
pub mod error;
pub mod event_bus;
pub mod gateway;
pub mod governance;
pub mod memory_fs;
pub mod policy;
#[cfg(feature = "baml-policy-experiments")]
pub mod policy_experiments;
pub mod ports;
pub mod resilience;
pub mod streaming;
pub mod ux;
pub mod viewspec;
pub mod workflow;

use async_trait::async_trait;
use event_bus::{RuntimeSessionUpdateRequest, RuntimeSessionUpdateResult};
use gateway::types::{GatewayRequestEnvelope, GatewayResponseEnvelope};
use serde::{Deserialize, Serialize};
use serde_json::json;

pub use error::RuntimeError;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub event_source: String,
    pub event_type_prefix: String,
    pub remote_endpoint: Option<String>,
    pub fail_on_network_error: bool,
    pub shadow_mode: bool,
    #[serde(default)]
    pub gateway: GatewayRuntimeConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayRuntimeConfig {
    pub gateway_port: u16,
    pub legacy_dispatch_mode: GatewayLegacyDispatchMode,
    pub collab_realtime_enabled: bool,
    pub closeout_tasks_path: Option<String>,
    pub testing_log_dir: Option<String>,
    pub decision_surface_log_dir: Option<String>,
    pub motoko_graph_log_dir: Option<String>,
    pub active_space_id: Option<String>,
    pub decision_signed_mode: String,
    pub decision_require_signed_principal: bool,
    pub decision_signature_max_skew_secs: u64,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GatewayLegacyDispatchMode {
    InProcess,
    HttpLoopback,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self {
            event_source: "nostra://cortex-runtime".to_string(),
            event_type_prefix: "nostra.acp".to_string(),
            remote_endpoint: None,
            fail_on_network_error: false,
            shadow_mode: false,
            gateway: GatewayRuntimeConfig::default(),
        }
    }
}

impl Default for GatewayRuntimeConfig {
    fn default() -> Self {
        Self {
            gateway_port: 3000,
            legacy_dispatch_mode: GatewayLegacyDispatchMode::InProcess,
            collab_realtime_enabled: true,
            closeout_tasks_path: None,
            testing_log_dir: None,
            decision_surface_log_dir: None,
            motoko_graph_log_dir: None,
            active_space_id: None,
            decision_signed_mode: "off".to_string(),
            decision_require_signed_principal: false,
            decision_signature_max_skew_secs: 600,
        }
    }
}

#[async_trait]
pub trait CortexRuntime: Send + Sync {
    async fn publish_session_update(
        &self,
        request: RuntimeSessionUpdateRequest,
    ) -> Result<RuntimeSessionUpdateResult, RuntimeError>;

    async fn handle_gateway_request(
        &self,
        request: GatewayRequestEnvelope,
    ) -> Result<GatewayResponseEnvelope, RuntimeError> {
        Ok(GatewayResponseEnvelope::not_implemented(
            request.method,
            request.path,
            json!({
                "reason": "gateway request handling is not configured for this runtime implementation"
            }),
        ))
    }
}
