// mock_embedding.rs
// Mock Embedding Provider for testing

use crate::embedding_provider::{EmbeddingError, EmbeddingProvider};
use async_trait::async_trait;

const NOSTRA_STANDARD_DIM: usize = 384; // DEC-042-002

/// Mock Embedding Generator
///
/// Generates deterministic embeddings based on input text hash.
/// Useful for testing without API calls.
pub struct MockEmbeddingGenerator {
    dimension: usize,
}

impl MockEmbeddingGenerator {
    pub fn new() -> Self {
        Self {
            dimension: NOSTRA_STANDARD_DIM,
        }
    }

    #[allow(dead_code)]
    pub fn with_dimension(dimension: usize) -> Self {
        Self { dimension }
    }

    /// Generate deterministic embedding from text hash
    fn hash_to_embedding(&self, text: &str) -> Vec<f32> {
        // Simple deterministic hash
        let mut hash: u64 = 0;
        for (i, b) in text.bytes().enumerate() {
            hash = hash.wrapping_add((b as u64).wrapping_mul((i + 1) as u64));
            hash = hash.rotate_left(7);
        }

        // Generate embedding from hash
        let mut embedding = Vec::with_capacity(self.dimension);
        let mut current = hash;

        for _ in 0..self.dimension {
            // Simple LCG for pseudo-random distribution
            current = current.wrapping_mul(6364136223846793005).wrapping_add(1);
            let value = ((current >> 32) as f32) / (u32::MAX as f32) * 2.0 - 1.0;
            embedding.push(value);
        }

        // Normalize to unit length (cosine similarity friendly)
        let magnitude: f32 = embedding.iter().map(|x| x * x).sum::<f32>().sqrt();
        if magnitude > 0.0 {
            for v in &mut embedding {
                *v /= magnitude;
            }
        }

        embedding
    }
}

impl Default for MockEmbeddingGenerator {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for MockEmbeddingGenerator {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }
        Ok(self.hash_to_embedding(text))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> String {
        "mock/deterministic-v1".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_deterministic_embedding() {
        let generator = MockEmbeddingGenerator::new();

        let emb1 = generator.embed("hello").await.unwrap();
        let emb2 = generator.embed("hello").await.unwrap();

        // Same input should produce same output
        assert_eq!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_different_inputs() {
        let generator = MockEmbeddingGenerator::new();

        let emb1 = generator.embed("hello").await.unwrap();
        let emb2 = generator.embed("world").await.unwrap();

        // Different inputs should produce different outputs
        assert_ne!(emb1, emb2);
    }

    #[tokio::test]
    async fn test_dimension() {
        let generator = MockEmbeddingGenerator::new();
        let emb = generator.embed("test").await.unwrap();

        assert_eq!(emb.len(), 384);
    }

    #[tokio::test]
    async fn test_normalized() {
        let generator = MockEmbeddingGenerator::new();
        let emb = generator.embed("test").await.unwrap();

        let magnitude: f32 = emb.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_empty_input() {
        let generator = MockEmbeddingGenerator::new();
        let result = generator.embed("").await;

        assert!(result.is_err());
    }

    #[test]
    fn test_model_id() {
        let generator = MockEmbeddingGenerator::new();
        assert_eq!(generator.model_id(), "mock/deterministic-v1");
    }
}
