// embedding_provider.rs
// Canonical Embedding Interface (DEC-042-004, DEC-042-010)

use async_trait::async_trait;
use thiserror::Error;

/// Errors that can occur during embedding generation
#[allow(dead_code)]
#[derive(Error, Debug)]
pub enum EmbeddingError {
    #[error("API error: {0}")]
    ApiError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

/// Metadata about an embedding model
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct EmbeddingMetadata {
    pub model_family: String,  // e.g., "Jina", "MiniLM", "OpenAI"
    pub model_version: String, // e.g., "text-embedding-3-small"
    pub dimension: usize,      // e.g., 384
}

/// The core trait for embedding providers (DEC-042-004)
///
/// All embedding providers (OpenAI, Local, ICP-Native) must implement this trait.
/// This enables:
/// - Model swapping without architectural changes
/// - Mock embedders for unit tests
/// - Seamless transition to on-chain when ready
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// Generate embedding for a single text input
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError>;

    /// Batch embed multiple texts
    /// Default implementation: sequential calls to `embed`
    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// Return the output dimension of this provider
    fn dimension(&self) -> usize;

    /// Return a unique identifier for this model (e.g., "openai/text-embedding-3-small")
    fn model_id(&self) -> String;

    /// Return full metadata about the model
    #[allow(dead_code)]
    fn metadata(&self) -> EmbeddingMetadata {
        EmbeddingMetadata {
            model_family: self
                .model_id()
                .split('/')
                .next()
                .unwrap_or("unknown")
                .to_string(),
            model_version: self.model_id(),
            dimension: self.dimension(),
        }
    }
}

#[async_trait]
impl<T: ?Sized + EmbeddingProvider> EmbeddingProvider for std::sync::Arc<T> {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        (**self).embed(text).await
    }

    async fn embed_batch(&self, texts: &[&str]) -> Result<Vec<Vec<f32>>, EmbeddingError> {
        (**self).embed_batch(texts).await
    }

    fn dimension(&self) -> usize {
        (**self).dimension()
    }

    fn model_id(&self) -> String {
        (**self).model_id()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestEmbedder;

    #[async_trait]
    impl EmbeddingProvider for TestEmbedder {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
            Ok(vec![0.1, 0.2, 0.3])
        }

        fn dimension(&self) -> usize {
            3
        }

        fn model_id(&self) -> String {
            "test/mock-v1".to_string()
        }
    }

    #[tokio::test]
    async fn test_embed_batch() {
        let embedder = TestEmbedder;
        let texts = vec!["hello", "world"];
        let results = embedder.embed_batch(&texts).await.unwrap();
        assert_eq!(results.len(), 2);
        assert_eq!(results[0].len(), 3);
    }

    #[test]
    fn test_metadata() {
        let embedder = TestEmbedder;
        let meta = embedder.metadata();
        assert_eq!(meta.model_family, "test");
        assert_eq!(meta.dimension, 3);
    }
}
