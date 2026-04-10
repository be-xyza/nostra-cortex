use crate::api::{
    WorkerKnowledgeAskRequest, WorkerKnowledgeSearchRequest, worker_ask_knowledge,
    worker_search_knowledge,
};
use crate::types::{KnowledgeAskResponse, KnowledgeSearchResult, SearchFilters};

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceSearchInput {
    pub query: String,
    pub limit: i32,
    pub retrieval_mode: String,
    pub diagnostics: bool,
    pub rerank_enabled: bool,
    pub filters: Option<SearchFilters>,
    pub modalities: Vec<String>,
}

impl Default for SurfaceSearchInput {
    fn default() -> Self {
        Self {
            query: String::new(),
            limit: 8,
            retrieval_mode: "hybrid".to_string(),
            diagnostics: false,
            rerank_enabled: false,
            filters: None,
            modalities: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceAskInput {
    pub question: String,
    pub limit: i32,
    pub retrieval_mode: String,
    pub diagnostics: bool,
    pub rerank_enabled: bool,
    pub filters: Option<SearchFilters>,
    pub modalities: Vec<String>,
    pub max_context_chunks: usize,
    pub require_provenance: bool,
}

impl Default for SurfaceAskInput {
    fn default() -> Self {
        Self {
            question: String::new(),
            limit: 8,
            retrieval_mode: "hybrid".to_string(),
            diagnostics: false,
            rerank_enabled: false,
            filters: None,
            modalities: Vec::new(),
            max_context_chunks: 4,
            require_provenance: true,
        }
    }
}

fn merge_filters_with_modalities(
    filters: Option<SearchFilters>,
    modalities: Vec<String>,
) -> Option<SearchFilters> {
    let normalized_modalities: Vec<String> = modalities
        .into_iter()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
        .collect();

    if normalized_modalities.is_empty() {
        return filters;
    }

    let mut out = filters.unwrap_or_default();
    out.modalities = normalized_modalities;
    Some(out)
}

pub fn build_surface_search_request(
    ui_surface: &str,
    input: SurfaceSearchInput,
) -> WorkerKnowledgeSearchRequest {
    let filters = merge_filters_with_modalities(input.filters, input.modalities);
    WorkerKnowledgeSearchRequest {
        query: input.query,
        limit: input.limit,
        retrieval_mode: input.retrieval_mode,
        diagnostics: input.diagnostics,
        rerank_enabled: input.rerank_enabled,
        ui_surface: Some(ui_surface.to_string()),
        filters,
        fusion_weights: None,
    }
}

pub fn build_surface_ask_request(
    ui_surface: &str,
    input: SurfaceAskInput,
) -> WorkerKnowledgeAskRequest {
    let filters = merge_filters_with_modalities(input.filters, input.modalities);
    WorkerKnowledgeAskRequest {
        question: input.question,
        limit: input.limit,
        retrieval_mode: input.retrieval_mode,
        diagnostics: input.diagnostics,
        rerank_enabled: input.rerank_enabled,
        ui_surface: Some(ui_surface.to_string()),
        filters,
        fusion_weights: None,
        max_context_chunks: input.max_context_chunks,
        require_provenance: input.require_provenance,
    }
}

pub async fn search_knowledge_for_surface(
    ui_surface: &str,
    input: SurfaceSearchInput,
) -> Result<Vec<KnowledgeSearchResult>, String> {
    let request = build_surface_search_request(ui_surface, input);
    worker_search_knowledge(&request).await
}

pub async fn ask_knowledge_for_surface(
    ui_surface: &str,
    input: SurfaceAskInput,
) -> Result<KnowledgeAskResponse, String> {
    let request = build_surface_ask_request(ui_surface, input);
    worker_ask_knowledge(&request).await
}

pub fn parse_csv_field(input: &str) -> Vec<String> {
    input
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(ToString::to_string)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn search_payload_builder_serializes_surface_and_filters() {
        let mut filters = SearchFilters::default();
        filters.space_id = Some("space-alpha".to_string());
        filters.tags = vec!["idea".to_string(), "priority".to_string()];

        let request = build_surface_search_request(
            "main-nav",
            SurfaceSearchInput {
                query: "semantic orchestration".to_string(),
                diagnostics: true,
                filters: Some(filters),
                ..SurfaceSearchInput::default()
            },
        );
        let payload = serde_json::to_value(request).expect("serialize search request");

        assert_eq!(payload["ui_surface"], "main-nav");
        assert_eq!(payload["diagnostics"], true);
        assert_eq!(payload["filters"]["space_id"], "space-alpha");
        assert_eq!(payload["filters"]["tags"][0], "idea");
    }

    #[test]
    fn ask_payload_builder_requires_provenance_by_default() {
        let request = build_surface_ask_request(
            "ideation",
            SurfaceAskInput {
                question: "What supports this idea?".to_string(),
                ..SurfaceAskInput::default()
            },
        );
        let payload = serde_json::to_value(request).expect("serialize ask request");

        assert_eq!(payload["ui_surface"], "ideation");
        assert_eq!(payload["require_provenance"], true);
        assert_eq!(payload["retrieval_mode"], "hybrid");
    }

    #[test]
    fn payload_builder_respects_keyword_mode_override() {
        let search_request = build_surface_search_request(
            "projects",
            SurfaceSearchInput {
                query: "keyword only".to_string(),
                retrieval_mode: "lexical".to_string(),
                ..SurfaceSearchInput::default()
            },
        );
        let ask_request = build_surface_ask_request(
            "projects",
            SurfaceAskInput {
                question: "keyword answer".to_string(),
                retrieval_mode: "lexical".to_string(),
                ..SurfaceAskInput::default()
            },
        );

        let search_payload = serde_json::to_value(search_request).expect("serialize search request");
        let ask_payload = serde_json::to_value(ask_request).expect("serialize ask request");

        assert_eq!(search_payload["retrieval_mode"], "lexical");
        assert_eq!(ask_payload["retrieval_mode"], "lexical");
    }

    #[test]
    fn csv_parser_handles_empty_values() {
        let values = parse_csv_field("alpha, beta ,,gamma");
        assert_eq!(values, vec!["alpha", "beta", "gamma"]);
    }

    #[test]
    fn payload_builder_serializes_modalities() {
        let search_request = build_surface_search_request(
            "main-nav",
            SurfaceSearchInput {
                query: "modality scoped search".to_string(),
                modalities: vec!["image".to_string(), " video ".to_string()],
                ..SurfaceSearchInput::default()
            },
        );
        let ask_request = build_surface_ask_request(
            "main-nav",
            SurfaceAskInput {
                question: "modality scoped ask".to_string(),
                modalities: vec!["audio".to_string()],
                ..SurfaceAskInput::default()
            },
        );

        let search_payload = serde_json::to_value(search_request).expect("serialize search request");
        let ask_payload = serde_json::to_value(ask_request).expect("serialize ask request");

        assert_eq!(search_payload["filters"]["modalities"][0], "image");
        assert_eq!(search_payload["filters"]["modalities"][1], "video");
        assert_eq!(ask_payload["filters"]["modalities"][0], "audio");
    }
}
