pub mod brand_policy;
pub mod dfx;
pub mod governance;
pub mod ic_cli;
pub mod streaming;
pub mod workflow;

use async_trait::async_trait;
use cortex_runtime::RuntimeError;
use cortex_runtime::ports::{NetworkAdapter, StorageAdapter};
use serde::{Deserialize, Serialize};
use serde_json::Value;

pub trait AdapterIdentity {
    fn adapter_name(&self) -> &str;
    fn adapter_version(&self) -> &str;
    fn authority_namespace(&self) -> &str;
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct IcAdapterDescriptor {
    pub name: String,
    pub version: String,
    pub authority_namespace: String,
}

impl AdapterIdentity for IcAdapterDescriptor {
    fn adapter_name(&self) -> &str {
        &self.name
    }

    fn adapter_version(&self) -> &str {
        &self.version
    }

    fn authority_namespace(&self) -> &str {
        &self.authority_namespace
    }
}

#[derive(Default)]
pub struct NoopIcStorageAdapter;

#[async_trait]
impl StorageAdapter for NoopIcStorageAdapter {
    async fn put(&self, _key: &str, _value: &Value) -> Result<(), RuntimeError> {
        Ok(())
    }

    async fn get(&self, _key: &str) -> Result<Option<Value>, RuntimeError> {
        Ok(None)
    }
}

#[derive(Default)]
pub struct NoopIcNetworkAdapter;

#[async_trait]
impl NetworkAdapter for NoopIcNetworkAdapter {
    async fn post_json(
        &self,
        _endpoint: &str,
        _idempotency_key: &str,
        _body: &Value,
    ) -> Result<(), RuntimeError> {
        Ok(())
    }
}
