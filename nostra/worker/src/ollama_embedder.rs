// ollama_embedder.rs
// Local Embedding Provider via Ollama (Tier 1: Local)

use crate::embedding_provider::{EmbeddingError, EmbeddingProvider};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use serde_json::Value;

const DEFAULT_BASE_URL: &str = "http://localhost:11434";
const DEFAULT_MODEL: &str = "qwen3-embedding:0.6b";
const NOSTRA_STANDARD_DIM: usize = 384; // DEC-042-002

#[derive(Serialize)]
struct OllamaEmbeddingRequest {
    model: String,
    input: String,
}

/// Local embedding provider backed by an Ollama server.
///
/// Tries `/api/embed` first (new API), then falls back to `/api/embeddings`
/// for compatibility with older Ollama setups.
pub struct OllamaEmbedder {
    base_url: String,
    model: String,
    dimension: usize,
    client: Client,
}

impl OllamaEmbedder {
    pub fn new() -> Self {
        Self {
            base_url: DEFAULT_BASE_URL.to_string(),
            model: DEFAULT_MODEL.to_string(),
            dimension: NOSTRA_STANDARD_DIM,
            client: Client::new(),
        }
    }

    pub fn with_config(base_url: String, model: String, dimension: usize) -> Self {
        Self {
            base_url,
            model,
            dimension,
            client: Client::new(),
        }
    }

    async fn request_embedding(&self, path: &str, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), path);
        let request = OllamaEmbeddingRequest {
            model: self.model.clone(),
            input: text.to_string(),
        };

        let response = self
            .client
            .post(&url)
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
                "HTTP {} from {}: {}",
                status, url, body
            )));
        }

        let payload: Value = response
            .json()
            .await
            .map_err(|e| EmbeddingError::ApiError(format!("Failed to parse response: {}", e)))?;

        // New API shape: { "embeddings": [[...]], ... }
        if let Some(arr) = payload
            .get("embeddings")
            .and_then(Value::as_array)
            .and_then(|v| v.first())
            .and_then(Value::as_array)
        {
            return Self::parse_vector(arr);
        }

        // Legacy API shape: { "embedding": [...], ... }
        if let Some(arr) = payload.get("embedding").and_then(Value::as_array) {
            return Self::parse_vector(arr);
        }

        Err(EmbeddingError::ApiError(format!(
            "No embedding field found in Ollama response from {}",
            url
        )))
    }

    fn parse_vector(raw: &[Value]) -> Result<Vec<f32>, EmbeddingError> {
        let mut out = Vec::with_capacity(raw.len());
        for v in raw {
            let n = v.as_f64().ok_or_else(|| {
                EmbeddingError::ApiError("Non-numeric embedding value".to_string())
            })?;
            out.push(n as f32);
        }
        Ok(out)
    }

    fn resize_embedding(&self, embedding: Vec<f32>) -> Vec<f32> {
        if embedding.len() == self.dimension {
            return Self::normalize_l2(embedding);
        }

        if embedding.is_empty() {
            return vec![0.0; self.dimension];
        }

        // Deterministic bucketed down-projection/up-projection to maintain fixed dimensionality.
        let source_len = embedding.len();
        let mut projected = vec![0.0_f32; self.dimension];
        let mut counts = vec![0_u32; self.dimension];

        for (idx, value) in embedding.into_iter().enumerate() {
            let bucket = idx * self.dimension / source_len;
            projected[bucket] += value;
            counts[bucket] += 1;
        }

        for i in 0..self.dimension {
            if counts[i] > 0 {
                projected[i] /= counts[i] as f32;
            }
        }

        Self::normalize_l2(projected)
    }

    fn normalize_l2(mut v: Vec<f32>) -> Vec<f32> {
        let norm = v.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for x in &mut v {
                *x /= norm;
            }
        }
        v
    }
}

impl Default for OllamaEmbedder {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl EmbeddingProvider for OllamaEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, EmbeddingError> {
        if text.trim().is_empty() {
            return Err(EmbeddingError::InvalidInput("Empty text".to_string()));
        }

        // Prefer modern endpoint, then fall back for compatibility.
        let raw = match self.request_embedding("api/embed", text).await {
            Ok(v) => v,
            Err(_) => self.request_embedding("api/embeddings", text).await?,
        };

        Ok(self.resize_embedding(raw))
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn model_id(&self) -> String {
        format!("ollama/{}", self.model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let embedder = OllamaEmbedder::new();
        assert_eq!(embedder.dimension(), 384);
        assert_eq!(embedder.model_id(), "ollama/qwen3-embedding:0.6b");
    }

    #[test]
    fn test_resize_downprojects_and_normalizes() {
        let embedder = OllamaEmbedder::with_config(
            "http://localhost:11434".to_string(),
            "test-model".to_string(),
            4,
        );
        let input = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0];
        let out = embedder.resize_embedding(input);
        assert_eq!(out.len(), 4);

        let magnitude: f32 = out.iter().map(|x| x * x).sum::<f32>().sqrt();
        assert!((magnitude - 1.0).abs() < 0.001);
    }
}
