use async_trait::async_trait;
use cortex_runtime::RuntimeError;
use cortex_runtime::ports::StorageAdapter;
use serde_json::Value;
use std::path::PathBuf;
use tokio::fs;

pub struct DesktopStorageAdapter {
    root_dir: PathBuf,
}

impl DesktopStorageAdapter {
    pub fn new(root_dir: PathBuf) -> Self {
        Self { root_dir }
    }

    fn get_path(&self, key: &str) -> PathBuf {
        let mut path = self.root_dir.clone();
        for component in key.split('/') {
            path.push(component);
        }
        path
    }
}

#[async_trait]
impl StorageAdapter for DesktopStorageAdapter {
    async fn put(&self, key: &str, value: &Value) -> Result<(), RuntimeError> {
        let path = self.get_path(key);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                RuntimeError::Storage(format!("Failed to create directories: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(value).map_err(|e| {
            RuntimeError::Serialization(format!("Failed to serialize value: {}", e))
        })?;

        fs::write(&path, content).await.map_err(|e| {
            RuntimeError::Storage(format!("Failed to write file {}: {}", path.display(), e))
        })?;
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Value>, RuntimeError> {
        let path = self.get_path(key);
        if !path.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&path).await.map_err(|e| {
            RuntimeError::Storage(format!("Failed to read file {}: {}", path.display(), e))
        })?;

        let value: Value = serde_json::from_str(&content).map_err(|e| {
            RuntimeError::Serialization(format!(
                "Failed to deserialize value from {}: {}",
                path.display(),
                e
            ))
        })?;

        Ok(Some(value))
    }
}
