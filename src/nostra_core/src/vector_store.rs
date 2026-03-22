use crate::types::Entity;
use async_trait::async_trait;

#[async_trait]
pub trait VectorStore: Send + Sync {
    /// Index an entity for semantic search.
    async fn upsert(&self, entity: &Entity) -> Result<(), String>;

    /// Search for relevant entities by query string.
    /// Returns a list of Entity IDs.
    async fn search(&self, query: &str) -> Result<Vec<String>, String>;
}
