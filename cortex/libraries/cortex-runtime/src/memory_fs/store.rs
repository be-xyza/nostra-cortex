use serde_json::{Value, json};
use std::sync::Arc;

use crate::RuntimeError;
use crate::ports::{StorageAdapter, TimeProvider};
use cortex_domain::memory_fs::{Blob, Commit, Oid, Tree};

/// Prefix for all objects (blobs, trees, commits) addressed by Oid
const OBJECT_PREFIX: &str = "fs/objects/";
/// Prefix for branch references
const REF_PREFIX: &str = "fs/refs/heads/";

pub struct ContextFs {
    storage: Arc<dyn StorageAdapter>,
    time: Arc<dyn TimeProvider>,
}

impl ContextFs {
    pub fn new(storage: Arc<dyn StorageAdapter>, time: Arc<dyn TimeProvider>) -> Self {
        Self { storage, time }
    }

    // --- Object Storage ---

    async fn put_object(&self, oid: &Oid, value: &Value) -> Result<(), RuntimeError> {
        let key = format!("{}{}", OBJECT_PREFIX, oid.as_str());
        self.storage.put(&key, value).await
    }

    async fn get_object(&self, oid: &Oid) -> Result<Option<Value>, RuntimeError> {
        let key = format!("{}{}", OBJECT_PREFIX, oid.as_str());
        self.storage.get(&key).await
    }

    // --- References ---

    async fn put_ref(&self, branch_name: &str, oid: &Oid) -> Result<(), RuntimeError> {
        let key = format!("{}{}", REF_PREFIX, branch_name);
        self.storage.put(&key, &json!(oid.as_str())).await
    }

    async fn get_ref(&self, branch_name: &str) -> Result<Option<Oid>, RuntimeError> {
        let key = format!("{}{}", REF_PREFIX, branch_name);
        if let Some(val) = self.storage.get(&key).await? {
            if let Some(s) = val.as_str() {
                return Ok(Some(Oid::new(s.to_string())));
            }
        }
        Ok(None)
    }

    // --- Core Operations ---

    /// Write a blob to the store
    pub async fn write_blob(&self, content: Vec<u8>) -> Result<Oid, RuntimeError> {
        let blob = Blob::new(content);
        let oid = blob.oid();
        let val =
            serde_json::to_value(&blob).map_err(|e| RuntimeError::Serialization(e.to_string()))?;
        self.put_object(&oid, &val).await?;
        Ok(oid)
    }

    /// Read a blob from the store
    pub async fn read_blob(&self, oid: &Oid) -> Result<Blob, RuntimeError> {
        let val = self
            .get_object(oid)
            .await?
            .ok_or_else(|| RuntimeError::Storage(format!("Blob not found: {}", oid)))?;
        serde_json::from_value(val)
            .map_err(|e| RuntimeError::Storage(format!("Failed to deserialize blob: {}", e)))
    }

    /// Write a tree to the store
    pub async fn write_tree(&self, tree: &Tree) -> Result<Oid, RuntimeError> {
        let oid = tree.oid();
        let val =
            serde_json::to_value(tree).map_err(|e| RuntimeError::Serialization(e.to_string()))?;
        self.put_object(&oid, &val).await?;
        Ok(oid)
    }

    /// Read a tree from the store
    pub async fn read_tree(&self, oid: &Oid) -> Result<Tree, RuntimeError> {
        let val = self
            .get_object(oid)
            .await?
            .ok_or_else(|| RuntimeError::Storage(format!("Tree not found: {}", oid)))?;
        serde_json::from_value(val)
            .map_err(|e| RuntimeError::Storage(format!("Failed to deserialize tree: {}", e)))
    }

    /// Read a commit from the store
    pub async fn read_commit(&self, oid: &Oid) -> Result<Commit, RuntimeError> {
        let val = self
            .get_object(oid)
            .await?
            .ok_or_else(|| RuntimeError::Storage(format!("Commit not found: {}", oid)))?;
        serde_json::from_value(val)
            .map_err(|e| RuntimeError::Storage(format!("Failed to deserialize commit: {}", e)))
    }

    // --- High Level Operations ---

    /// Create a commit on a branch
    pub async fn commit(
        &self,
        branch_name: &str,
        tree_oid: Oid,
        author: String,
        message: String,
    ) -> Result<Oid, RuntimeError> {
        // Resolve parent
        let parents = match self.get_ref(branch_name).await? {
            Some(parent_oid) => vec![parent_oid],
            None => vec![], // Initial commit
        };

        let commit = Commit {
            tree: tree_oid,
            parents,
            author,
            message,
            timestamp: self.time.now_unix_secs(),
        };

        let commit_oid = commit.oid();
        let val = serde_json::to_value(&commit)
            .map_err(|e| RuntimeError::Serialization(e.to_string()))?;

        self.put_object(&commit_oid, &val).await?;
        self.put_ref(branch_name, &commit_oid).await?;

        Ok(commit_oid)
    }

    /// Create a new branch pointing to a specific commit
    pub async fn create_branch(
        &self,
        branch_name: &str,
        commit_oid: Oid,
    ) -> Result<(), RuntimeError> {
        // Ensure commit exists
        let _ = self.read_commit(&commit_oid).await?;
        self.put_ref(branch_name, &commit_oid).await?;
        Ok(())
    }

    /// Get the commit Oid a branch points to
    pub async fn resolve_branch(&self, branch_name: &str) -> Result<Oid, RuntimeError> {
        self.get_ref(branch_name)
            .await?
            .ok_or_else(|| RuntimeError::Storage(format!("Branch not found: {}", branch_name)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::memory_fs::test_support::{MockStorage, MockTime};
    use cortex_domain::memory_fs::TreeEntry;
    use std::sync::Arc;

    #[test]
    fn test_context_fs_commit_flow() {
        futures::executor::block_on(async {
            let storage = Arc::new(MockStorage::new());
            let time = Arc::new(MockTime::new());
            let fs = ContextFs::new(storage.clone(), time.clone());

            // 1. Write blobs
            let blob1_oid = fs.write_blob(b"hello".to_vec()).await.unwrap();
            let blob2_oid = fs.write_blob(b"world".to_vec()).await.unwrap();

            // 2. Build tree
            let mut tree = Tree::new();
            tree.add_entry("hello.txt".to_string(), TreeEntry::Blob(blob1_oid));
            tree.add_entry("world.txt".to_string(), TreeEntry::Blob(blob2_oid));
            let tree_oid = fs.write_tree(&tree).await.unwrap();

            // 3. Commit to main branch
            let commit1_oid = fs
                .commit(
                    "main",
                    tree_oid.clone(),
                    "Agent 1".to_string(),
                    "Initial context".to_string(),
                )
                .await
                .unwrap();

            // 4. Verify branch updated
            let resolved_oid = fs.resolve_branch("main").await.unwrap();
            assert_eq!(resolved_oid, commit1_oid);

            // 5. Read back
            let commit_back = fs.read_commit(&commit1_oid).await.unwrap();
            assert_eq!(commit_back.tree, tree_oid);
            assert_eq!(commit_back.message, "Initial context");
        })
    }
}
