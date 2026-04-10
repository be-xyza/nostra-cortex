use crate::vector_service::{
    SearchOptions as VectorSearchOptions, SearchResult as VectorSearchResult, VectorService,
};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f64,
    pub source: String,
    pub content: String,
}

pub struct SearchOptions {
    pub limit: usize,
}

/// Legacy search facade retained for compatibility.
///
/// Retrieval authority is delegated to `VectorService` so semantic and hybrid
/// search behavior remains consistent across worker code paths.
pub struct NostraSearch {
    vector_service: Arc<VectorService>,
}

impl NostraSearch {
    pub fn new(vector_service: Arc<VectorService>) -> Self {
        Self { vector_service }
    }

    pub async fn search(&self, query: &str, opts: SearchOptions) -> Result<Vec<SearchResult>> {
        let results = self
            .vector_service
            .search_with_options(query, opts.limit as i32, VectorSearchOptions::default())
            .await?;
        Ok(results.into_iter().map(Self::map_result).collect())
    }

    fn map_result(item: VectorSearchResult) -> SearchResult {
        SearchResult {
            id: item.id,
            score: item.score as f64,
            source: "vector_service".to_string(),
            content: item.content.unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock_embedding::MockEmbeddingGenerator;
    use crate::vector_service::IndexDocument;
    use ic_agent::identity::AnonymousIdentity;

    #[tokio::test]
    async fn test_legacy_search_adapter_delegates_to_vector_service() {
        let agent = Arc::new(
            ic_agent::Agent::builder()
                .with_url("http://127.0.0.1:4943")
                .with_identity(AnonymousIdentity)
                .build()
                .expect("agent"),
        );

        let service = Arc::new(VectorService::new(
            Arc::new(MockEmbeddingGenerator::new()),
            agent,
            "legacy_search_adapter".to_string(),
        ));

        service
            .index_documents(
                vec![IndexDocument {
                    id: "legacy-doc-1".to_string(),
                    text: "Nostra hybrid retrieval adapter".to_string(),
                    label: "legacy".to_string(),
                    space_id: "space-legacy".to_string(),
                    source_ref: "urn:legacy:1".to_string(),
                    source_type: "note".to_string(),
                    tags: vec!["legacy".to_string()],
                    timestamp_ms: Some(1_700_000_100_000),
                    cei_metadata: None,
                    modality: None,
                }],
                Some("legacy-adapter-key"),
            )
            .await
            .expect("index");

        let search = NostraSearch::new(service);
        let found = search
            .search("hybrid retrieval", SearchOptions { limit: 5 })
            .await
            .expect("search");
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].id, "legacy-doc-1");
    }
}
