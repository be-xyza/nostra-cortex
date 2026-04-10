// openai_embedder.rs
// OpenAI Embedding Provider (Tier 3: Cloud)

use crate::embedding_provider::{EmbeddingError, EmbeddingProvider};
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

const OPENAI_EMBEDDINGS_URL: &str = "https://api.openai.com/v1/embeddings";
const DEFAULT_MODEL: &str = "text-embedding-3-small";
const NOSTRA_STANDARD_DIM: usize = 384; // DEC-042-002

#[derive(Serialize)]
struct EmbeddingRequest {
    input: String,
    model: String,
    dimensions: usize,
}

#[derive(Deserialize)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
}

#[derive(Deserialize)]
struct EmbeddingData {
    embedding: Vec<f32>,
}

/// OpenAI Embedding Provider
///
/// Uses `text-embedding-3-small` by default, quantized to 384 dimensions
/// to match the Nostra standard (DEC-042-002).
pub struct OpenAIEmbedder {
    api_key: String,
    model: String,
    dimension: usize,
    client: Client,
}

impl OpenAIEmbedder {
    /// Create a new OpenAI embedder with default settings
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            model: DEFAULT_MODEL.to_string(),
            dimension: NOSTRA_STANDARD_DIM,
            client: Client::new(),
        }
    }

    /// Create with custom model and dimension
    pub fn with_config(api_key: String, model: String, dimension: usize) -> Self {
        Self {
            api_key,
            model,
            dimension,
            client: Client::new(),
        }
    }
}

#[async_trait]
impl EmbeddingProvider for OpenAIEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }

        let request = EmbeddingRequest {
            input: text.to_string(),
            model: self.model.clone(),
            dimensions: self.dimension,
        };

        let response = self
            .client
            .post(OPENAI_EMBEDDINGS_URL)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request)
            .send()
            .await
            .map_err(|e| EmbeddingError::ApiError(e.to_string()))?;

        if response.status() == 429 {
            return Err(EmbeddingError::RateLimited);
        }

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err(EmbeddingError::ApiError(format!(
                "HTTP {}: {}",
                status, body
            )));
        }

        let embedding_response: EmbeddingResponse = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("Failed to parse response: {}", e)))?;

        embedding_response
            .data
            .into_iter()
            .next()
            .map(|d| d.embedding)
            .ok_or_else(|| EmbeddingError::ApiError("No embedding in response".to_string()))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> String {
        format!("openai/{}", self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let embedder = OpenAIEmbedder::new("test-key".to_string());
        assert_eq!(embedder.dimension(), 384);
        assert_eq!(embedder.model_id(), "openai/text-embedding-3-small");
    }

    #[test]
    fn test_custom_config() {
        let embedder = OpenAIEmbedder::with_config(
            "test-key".to_string(),
            "text-embedding-3-large".to_string(),
            1536,
        );
        assert_eq!(embedder.dimension(), 1536);
        assert_eq!(embedder.model_id(), "openai/text-embedding-3-large");
    }

    // Integration test - requires OPENAI_API_KEY env var
    #[tokio::test]
    #[ignore]
    async fn test_real_embedding() {
        let api_key = std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
        let embedder = OpenAIEmbedder::new(api_key);

        let result = embedder.embed("Hello, world!").await;
        assert!(result.is_ok());

        let embedding = result.unwrap();
        assert_eq!(embedding.len(), 384);
    }
}
