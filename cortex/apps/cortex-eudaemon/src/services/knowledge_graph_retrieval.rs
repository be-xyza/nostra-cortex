use crate::services::knowledge_graph_query::{
    InMemoryTriple, TripleMatch, TripleQueryAdapter, TripleQueryFilters, TripleQueryRequest,
    TripleQueryScope,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const DEFAULT_SCHEMA_VERSION: &str = "1.0.0";
const DEFAULT_ORDERING_STRATEGY: &str = "canonical";
const RRF_K: f64 = 60.0;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VectorRetrievalHit {
    pub id: String,
    pub source_ref: String,
    pub content: String,
    pub score: f32,
    #[serde(default)]
    pub metadata: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRetrievalRequest {
    pub query_id: String,
    pub query_text: String,
    pub retrieval_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub triple_request: Option<TripleQueryRequest>,
    pub graph_limit: u64,
    pub vector_limit: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRetrievalCitation {
    pub source_ref: String,
    pub kind: String,
    pub score: f64,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub excerpt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRetrievalResponse {
    pub query_id: String,
    pub retrieval_mode: String,
    pub answer_summary: String,
    pub graph_matches: Vec<TripleMatch>,
    pub vector_matches: Vec<VectorRetrievalHit>,
    pub citations: Vec<GraphRetrievalCitation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRetrievalBenchmarkCase {
    pub case_id: String,
    pub query_class: String,
    pub request: GraphRetrievalRequest,
    pub relevant_source_refs: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GraphRetrievalBenchmarkResult {
    pub case_id: String,
    pub graph_only_score: f64,
    pub vector_only_score: f64,
    pub hybrid_graph_embedding_score: f64,
}

pub struct GraphRetrievalHarness;

impl GraphRetrievalHarness {
    pub fn execute(
        triples: &[InMemoryTriple],
        vector_hits: &[VectorRetrievalHit],
        request: &GraphRetrievalRequest,
    ) -> Result<GraphRetrievalResponse, String> {
        if !matches!(
            request.retrieval_mode.as_str(),
            "graph_only" | "vector_only" | "hybrid_graph_embedding"
        ) {
            return Err(format!(
                "unsupported retrieval_mode {}",
                request.retrieval_mode
            ));
        }

        let graph_request = request
            .triple_request
            .clone()
            .unwrap_or_else(|| heuristic_graph_request(request));

        let graph_matches = if request.retrieval_mode == "vector_only" {
            Vec::new()
        } else {
            let mut response = TripleQueryAdapter::execute(triples, &graph_request)?;
            response
                .triples
                .truncate(request.graph_limit.try_into().unwrap_or(usize::MAX));
            response.triples
        };

        let vector_matches = if request.retrieval_mode == "graph_only" {
            Vec::new()
        } else {
            let mut ordered = vector_hits.to_vec();
            ordered.sort_by(|left, right| {
                right
                    .score
                    .total_cmp(&left.score)
                    .then_with(|| left.source_ref.cmp(&right.source_ref))
            });
            ordered.truncate(request.vector_limit);
            ordered
        };

        let citations = merge_citations(&graph_matches, &vector_matches);
        let answer_summary = format!(
            "{} retrieval for '{}' returned {} graph matches, {} vector hits, and {} citations.",
            request.retrieval_mode,
            request.query_text,
            graph_matches.len(),
            vector_matches.len(),
            citations.len()
        );

        Ok(GraphRetrievalResponse {
            query_id: request.query_id.clone(),
            retrieval_mode: request.retrieval_mode.clone(),
            answer_summary,
            graph_matches,
            vector_matches,
            citations,
        })
    }

    pub fn benchmark(
        triples: &[InMemoryTriple],
        vector_hits: &[VectorRetrievalHit],
        cases: &[GraphRetrievalBenchmarkCase],
    ) -> Result<Vec<GraphRetrievalBenchmarkResult>, String> {
        cases
            .iter()
            .map(|case| {
                let graph_request = GraphRetrievalRequest {
                    retrieval_mode: "graph_only".to_string(),
                    ..case.request.clone()
                };
                let vector_request = GraphRetrievalRequest {
                    retrieval_mode: "vector_only".to_string(),
                    ..case.request.clone()
                };
                let hybrid_request = GraphRetrievalRequest {
                    retrieval_mode: "hybrid_graph_embedding".to_string(),
                    ..case.request.clone()
                };

                let graph = Self::execute(triples, vector_hits, &graph_request)?;
                let vector = Self::execute(triples, vector_hits, &vector_request)?;
                let hybrid = Self::execute(triples, vector_hits, &hybrid_request)?;

                Ok(GraphRetrievalBenchmarkResult {
                    case_id: case.case_id.clone(),
                    graph_only_score: recall_score(&graph, &case.relevant_source_refs),
                    vector_only_score: recall_score(&vector, &case.relevant_source_refs),
                    hybrid_graph_embedding_score: recall_score(&hybrid, &case.relevant_source_refs),
                })
            })
            .collect()
    }
}

fn heuristic_graph_request(request: &GraphRetrievalRequest) -> TripleQueryRequest {
    let query = request.query_text.to_ascii_lowercase();
    let predicate = if query.contains("capability") {
        Some("has_capability".to_string())
    } else if query.contains("evidence") {
        Some("research:evidences".to_string())
    } else if query.contains("hypothesis") {
        Some("research:tests_hypothesis".to_string())
    } else {
        None
    };

    TripleQueryRequest {
        schema_version: DEFAULT_SCHEMA_VERSION.to_string(),
        query_id: format!("{}-planned", request.query_id),
        ordering_strategy: DEFAULT_ORDERING_STRATEGY.to_string(),
        scope: TripleQueryScope {
            named_graph_scope: "any".to_string(),
            scope_ref: None,
        },
        filters: TripleQueryFilters {
            subject: None,
            predicate,
            object: None,
            include_provenance: true,
            limit: request.graph_limit,
            offset: 0,
        },
    }
}

fn merge_citations(
    graph_matches: &[TripleMatch],
    vector_matches: &[VectorRetrievalHit],
) -> Vec<GraphRetrievalCitation> {
    let mut merged: BTreeMap<String, GraphRetrievalCitation> = BTreeMap::new();

    for (index, item) in graph_matches.iter().enumerate() {
        let score = rrf_score(index);
        merged
            .entry(item.source_ref.clone())
            .and_modify(|citation| {
                citation.score += score;
                citation.kind = if citation.kind == "vector" {
                    "hybrid".to_string()
                } else {
                    citation.kind.clone()
                };
            })
            .or_insert(GraphRetrievalCitation {
                source_ref: item.source_ref.clone(),
                kind: "graph".to_string(),
                score,
                excerpt: Some(format!(
                    "{} {} {}",
                    item.subject, item.predicate, item.object
                )),
            });
    }

    for (index, item) in vector_matches.iter().enumerate() {
        let score = rrf_score(index);
        merged
            .entry(item.source_ref.clone())
            .and_modify(|citation| {
                citation.score += score;
                citation.kind = if citation.kind == "graph" {
                    "hybrid".to_string()
                } else {
                    citation.kind.clone()
                };
                if citation.excerpt.is_none() {
                    citation.excerpt = Some(item.content.clone());
                }
            })
            .or_insert(GraphRetrievalCitation {
                source_ref: item.source_ref.clone(),
                kind: "vector".to_string(),
                score,
                excerpt: Some(item.content.clone()),
            });
    }

    let mut citations: Vec<GraphRetrievalCitation> = merged.into_values().collect();
    citations.sort_by(|left, right| {
        right
            .score
            .total_cmp(&left.score)
            .then_with(|| left.source_ref.cmp(&right.source_ref))
    });
    citations
}

fn rrf_score(index: usize) -> f64 {
    1.0 / (RRF_K + (index + 1) as f64)
}

fn recall_score(response: &GraphRetrievalResponse, relevant_source_refs: &[String]) -> f64 {
    let relevant: std::collections::BTreeSet<&str> =
        relevant_source_refs.iter().map(String::as_str).collect();
    let matched = response
        .citations
        .iter()
        .filter(|citation| relevant.contains(citation.source_ref.as_str()))
        .count();
    if relevant.is_empty() {
        0.0
    } else {
        matched as f64 / relevant.len() as f64
    }
}
