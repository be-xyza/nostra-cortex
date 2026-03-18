use cortex_domain::memory_fs::{Blob, Oid, Tree, TreeEntry};
use serde::{Deserialize, Serialize};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use crate::RuntimeError;
use crate::ports::{StorageAdapter, TimeProvider};

use super::store::ContextFs;

/// Handle to an isolated sandbox namespace within ContextFs.
/// All objects written through a sandboxed ContextFs are prefixed
/// with `fs/sandboxes/{sandbox_id}/` to prevent cross-contamination.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SandboxHandle {
    pub sandbox_id: String,
    pub root_tree_oid: Option<Oid>,
    pub created_at: u64,
    pub status: SandboxStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SandboxStatus {
    Created,
    Ingested,
    Evaluated,
    Destroyed,
}

/// Configuration for sandbox ingestion limits.
#[derive(Debug, Clone)]
pub struct SandboxConfig {
    pub max_file_size_bytes: u64,
    pub max_depth: usize,
    pub max_total_files: usize,
}

impl Default for SandboxConfig {
    fn default() -> Self {
        Self {
            max_file_size_bytes: 10 * 1024 * 1024, // 10 MB
            max_depth: 20,
            max_total_files: 10_000,
        }
    }
}

/// A scoped ContextFs that prefixes all keys with a sandbox namespace.
/// This enforces the Temporal Boundary: the Engine never sees raw
/// filesystem paths, only content-addressable objects within a namespace.
pub struct SandboxFs {
    inner: ContextFs,
    pub handle: SandboxHandle,
}

impl SandboxFs {
    /// Create a new sandbox namespace.
    pub fn create(
        sandbox_id: impl Into<String>,
        storage: Arc<dyn StorageAdapter>,
        time: Arc<dyn TimeProvider>,
    ) -> Self {
        let sandbox_id = sandbox_id.into();
        let now = time.now_unix_secs();

        // Wrap the storage adapter to prefix all keys
        let prefixed_storage = Arc::new(PrefixedStorage {
            inner: storage,
            prefix: format!("fs/sandboxes/{}/", sandbox_id),
        });

        Self {
            inner: ContextFs::new(prefixed_storage, time),
            handle: SandboxHandle {
                sandbox_id,
                root_tree_oid: None,
                created_at: now,
                status: SandboxStatus::Created,
            },
        }
    }

    /// Write a blob into the sandbox.
    pub async fn write_blob(&self, content: Vec<u8>) -> Result<Oid, RuntimeError> {
        self.inner.write_blob(content).await
    }

    /// Write a tree into the sandbox.
    pub async fn write_tree(&self, tree: &Tree) -> Result<Oid, RuntimeError> {
        self.inner.write_tree(tree).await
    }

    /// Read a blob from the sandbox.
    pub async fn read_blob(&self, oid: &Oid) -> Result<Blob, RuntimeError> {
        self.inner.read_blob(oid).await
    }

    /// Read a tree from the sandbox.
    pub async fn read_tree(&self, oid: &Oid) -> Result<Tree, RuntimeError> {
        self.inner.read_tree(oid).await
    }

    /// Ingest a directory tree from a pre-built tree of (name, content) pairs.
    /// This is the adapter-neutral ingestion entry point: the caller (application
    /// layer) has already read the filesystem and passes content as byte vectors.
    ///
    /// Returns the root tree Oid.
    pub async fn ingest_entries(
        &mut self,
        entries: Vec<FsEntry>,
        config: &SandboxConfig,
    ) -> Result<Oid, RuntimeError> {
        let root_oid = self
            .build_tree_recursive(&entries, 0, config, &mut 0)
            .await?;
        self.handle.root_tree_oid = Some(root_oid.clone());
        self.handle.status = SandboxStatus::Ingested;
        Ok(root_oid)
    }

    /// Recursively build a tree from filesystem entries.
    fn build_tree_recursive<'a>(
        &'a self,
        entries: &'a [FsEntry],
        depth: usize,
        config: &'a SandboxConfig,
        file_count: &'a mut usize,
    ) -> Pin<Box<dyn Future<Output = Result<Oid, RuntimeError>> + Send + 'a>> {
        Box::pin(async move {
            if depth > config.max_depth {
                return Err(RuntimeError::Storage(format!(
                    "Sandbox ingestion exceeded max depth: {}",
                    config.max_depth
                )));
            }

            let mut tree = Tree::new();

            for entry in entries {
                match entry {
                    FsEntry::File { name, content } => {
                        *file_count += 1;
                        if *file_count > config.max_total_files {
                            return Err(RuntimeError::Storage(format!(
                                "Sandbox ingestion exceeded max file count: {}",
                                config.max_total_files
                            )));
                        }
                        if content.len() as u64 > config.max_file_size_bytes {
                            continue; // skip oversized files
                        }
                        let blob_oid = self.write_blob(content.clone()).await?;
                        tree.add_entry(name.clone(), TreeEntry::Blob(blob_oid));
                    }
                    FsEntry::Directory { name, children } => {
                        let subtree_oid = self
                            .build_tree_recursive(children, depth + 1, config, file_count)
                            .await?;
                        tree.add_entry(name.clone(), TreeEntry::Tree(subtree_oid));
                    }
                }
            }

            self.write_tree(&tree).await
        })
    }
}

/// Adapter-neutral representation of a filesystem entry.
/// Built by the application layer (cortex-eudaemon) from `walkdir`,
/// then passed to `SandboxFs::ingest_entries`.
#[derive(Debug, Clone)]
pub enum FsEntry {
    File {
        name: String,
        content: Vec<u8>,
    },
    Directory {
        name: String,
        children: Vec<FsEntry>,
    },
}

/// A `StorageAdapter` wrapper that prefixes all keys for sandbox isolation.
struct PrefixedStorage {
    inner: Arc<dyn StorageAdapter>,
    prefix: String,
}

#[async_trait::async_trait]
impl StorageAdapter for PrefixedStorage {
    async fn put(&self, key: &str, value: &serde_json::Value) -> Result<(), RuntimeError> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        self.inner.put(&prefixed_key, value).await
    }

    async fn get(&self, key: &str) -> Result<Option<serde_json::Value>, RuntimeError> {
        let prefixed_key = format!("{}{}", self.prefix, key);
        self.inner.get(&prefixed_key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_fs::test_support::{MockStorage, MockTime};

    #[test]
    fn sandbox_create_and_ingest() {
        futures::executor::block_on(async {
            let storage = Arc::new(MockStorage::new());
            let time = Arc::new(MockTime::new());

            let mut sandbox = SandboxFs::create("test-sandbox-1", storage.clone(), time.clone());
            assert_eq!(sandbox.handle.status, SandboxStatus::Created);
            assert!(sandbox.handle.root_tree_oid.is_none());

            // Build a simple directory tree
            let entries = vec![
                FsEntry::File {
                    name: "README.md".to_string(),
                    content: b"# Test Repo".to_vec(),
                },
                FsEntry::Directory {
                    name: "src".to_string(),
                    children: vec![FsEntry::File {
                        name: "main.rs".to_string(),
                        content: b"fn main() {}".to_vec(),
                    }],
                },
            ];

            let config = SandboxConfig::default();
            let root_oid = sandbox.ingest_entries(entries, &config).await.unwrap();

            assert_eq!(sandbox.handle.status, SandboxStatus::Ingested);
            assert_eq!(sandbox.handle.root_tree_oid, Some(root_oid.clone()));

            // Verify we can read the root tree back
            let root_tree = sandbox.read_tree(&root_oid).await.unwrap();
            assert!(root_tree.entries.contains_key("README.md"));
            assert!(root_tree.entries.contains_key("src"));

            // Verify keys are prefixed
            let raw_keys: Vec<String> = storage.data.read().unwrap().keys().cloned().collect();
            for key in &raw_keys {
                assert!(
                    key.starts_with("fs/sandboxes/test-sandbox-1/"),
                    "Key should be prefixed: {}",
                    key
                );
            }
        })
    }

    #[test]
    fn sandbox_enforces_file_count_limit() {
        futures::executor::block_on(async {
            let storage = Arc::new(MockStorage::new());
            let time = Arc::new(MockTime::new());

            let mut sandbox = SandboxFs::create("limit-test", storage, time);

            let entries: Vec<FsEntry> = (0..5)
                .map(|i| FsEntry::File {
                    name: format!("file_{}.txt", i),
                    content: b"data".to_vec(),
                })
                .collect();

            let config = SandboxConfig {
                max_total_files: 3,
                ..SandboxConfig::default()
            };

            let result = sandbox.ingest_entries(entries, &config).await;
            assert!(result.is_err());
        })
    }

    #[test]
    fn sandbox_enforces_depth_limit() {
        futures::executor::block_on(async {
            let storage = Arc::new(MockStorage::new());
            let time = Arc::new(MockTime::new());

            let mut sandbox = SandboxFs::create("depth-test", storage, time);

            // Build deeply nested structure
            let mut current = vec![FsEntry::File {
                name: "leaf.txt".to_string(),
                content: b"leaf".to_vec(),
            }];
            for i in (0..5).rev() {
                current = vec![FsEntry::Directory {
                    name: format!("d{}", i),
                    children: current,
                }];
            }

            let config = SandboxConfig {
                max_depth: 3,
                ..SandboxConfig::default()
            };

            let result = sandbox.ingest_entries(current, &config).await;
            assert!(result.is_err());
        })
    }
}
