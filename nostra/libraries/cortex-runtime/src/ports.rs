use crate::RuntimeError;
use async_trait::async_trait;
use cortex_domain::events::ProjectedEvent;
use serde_json::Value;

#[async_trait]
pub trait StorageAdapter: Send + Sync {
    async fn put(&self, key: &str, value: &Value) -> Result<(), RuntimeError>;
    async fn get(&self, key: &str) -> Result<Option<Value>, RuntimeError>;
}

#[async_trait]
pub trait NetworkAdapter: Send + Sync {
    async fn post_json(
        &self,
        endpoint: &str,
        idempotency_key: &str,
        body: &Value,
    ) -> Result<(), RuntimeError>;
}

pub trait TimeProvider: Send + Sync {
    fn now_unix_secs(&self) -> u64;
    fn to_rfc3339(&self, unix_secs: u64) -> Result<String, RuntimeError>;
}

pub trait LogAdapter: Send + Sync {
    fn info(&self, message: &str);
    fn warn(&self, message: &str);
    fn error(&self, message: &str);
}

#[async_trait]
pub trait EventBus: Send + Sync {
    async fn append_projected_event(&self, event: &ProjectedEvent) -> Result<(), RuntimeError>;
}
