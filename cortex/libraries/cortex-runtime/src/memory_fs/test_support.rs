use crate::RuntimeError;
use crate::memory_fs::sandbox::{FsEntry, SandboxConfig, SandboxFs};
use crate::ports::{StorageAdapter, TimeProvider};
use async_trait::async_trait;
use cortex_domain::memory_fs::Oid;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, RwLock};

pub struct MockStorage {
    pub data: RwLock<HashMap<String, Value>>,
}

impl MockStorage {
    pub fn new() -> Self {
        Self {
            data: RwLock::new(HashMap::new()),
        }
    }
}

#[async_trait]
impl StorageAdapter for MockStorage {
    async fn put(&self, key: &str, value: &Value) -> Result<(), RuntimeError> {
        self.data
            .write()
            .unwrap()
            .insert(key.to_string(), value.clone());
        Ok(())
    }
    async fn get(&self, key: &str) -> Result<Option<Value>, RuntimeError> {
        Ok(self.data.read().unwrap().get(key).cloned())
    }
}

pub struct MockTime {
    pub current: AtomicU64,
}

impl MockTime {
    pub fn new() -> Self {
        Self {
            current: AtomicU64::new(1000),
        }
    }
}

impl TimeProvider for MockTime {
    fn now_unix_secs(&self) -> u64 {
        self.current.fetch_add(10, Ordering::SeqCst)
    }
    fn to_rfc3339(&self, _unix_secs: u64) -> Result<String, RuntimeError> {
        Ok("2026-02-26T00:00:00Z".to_string())
    }
}

pub fn make_sandbox_with_entries(
    entries: Vec<FsEntry>,
) -> (
    Arc<MockStorage>,
    futures::executor::LocalPool,
    SandboxFs,
    Oid,
) {
    let storage = Arc::new(MockStorage::new());
    let time = Arc::new(MockTime::new());
    let mut sandbox = SandboxFs::create("mat-test", storage.clone(), time);

    let root_oid =
        futures::executor::block_on(sandbox.ingest_entries(entries, &SandboxConfig::default()))
            .unwrap();

    let pool = futures::executor::LocalPool::new();
    (storage, pool, sandbox, root_oid)
}
