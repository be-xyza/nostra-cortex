use crate::embedding_provider::EmbeddingProvider;
use crate::mock_embedding::MockEmbeddingGenerator;
use crate::ollama_embedder::OllamaEmbedder;
use crate::openai_embedder::OpenAIEmbedder;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbeddingRoute {
    Ollama,
    OpenAI,
    Mock,
}

pub fn resolve_embedding_route(
    requested_provider: &str,
    openai_key_present: bool,
    local_probe_ok: bool,
) -> EmbeddingRoute {
    match requested_provider.to_lowercase().as_str() {
        "mock" => EmbeddingRoute::Mock,
        "openai" => {
            if openai_key_present {
                EmbeddingRoute::OpenAI
            } else {
                EmbeddingRoute::Mock
            }
        }
        "ollama" | "local" => {
            if local_probe_ok {
                EmbeddingRoute::Ollama
            } else if openai_key_present {
                EmbeddingRoute::OpenAI
            } else {
                EmbeddingRoute::Mock
            }
        }
        _ => {
            if local_probe_ok {
                EmbeddingRoute::Ollama
            } else if openai_key_present {
                EmbeddingRoute::OpenAI
            } else {
                EmbeddingRoute::Mock
            }
        }
    }
}

pub fn build_provider(
    route: EmbeddingRoute,
    local_base: String,
    local_model: String,
    local_dim: usize,
    openai_api_key: String,
) -> Arc<dyn EmbeddingProvider> {
    match route {
        EmbeddingRoute::Ollama => Arc::new(OllamaEmbedder::with_config(
            local_base,
            local_model,
            local_dim,
        )),
        EmbeddingRoute::OpenAI => Arc::new(OpenAIEmbedder::new(openai_api_key)),
        EmbeddingRoute::Mock => Arc::new(MockEmbeddingGenerator::new()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resolve_auto_local_available_prefers_ollama() {
        let route = resolve_embedding_route("auto", false, true);
        assert_eq!(route, EmbeddingRoute::Ollama);
    }

    #[test]
    fn test_resolve_auto_local_unavailable_falls_back_to_openai_when_key_present() {
        let route = resolve_embedding_route("auto", true, false);
        assert_eq!(route, EmbeddingRoute::OpenAI);
    }

    #[test]
    fn test_resolve_auto_local_unavailable_no_key_falls_back_to_mock() {
        let route = resolve_embedding_route("auto", false, false);
        assert_eq!(route, EmbeddingRoute::Mock);
    }

    #[test]
    fn test_resolve_openai_without_key_falls_back_to_mock() {
        let route = resolve_embedding_route("openai", false, true);
        assert_eq!(route, EmbeddingRoute::Mock);
    }
}
