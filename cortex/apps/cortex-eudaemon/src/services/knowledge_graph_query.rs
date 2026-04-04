use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::convert::TryFrom;

const SUPPORTED_SCHEMA_VERSION: &str = "1.0.0";
const SUPPORTED_ORDERING_STRATEGY: &str = "canonical";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripleQueryScope {
    pub named_graph_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripleQueryFilters {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub subject: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub predicate: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub object: Option<String>,
    pub include_provenance: bool,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripleQueryRequest {
    pub schema_version: String,
    pub query_id: String,
    pub ordering_strategy: String,
    pub scope: TripleQueryScope,
    pub filters: TripleQueryFilters,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripleMatch {
    pub ordinal: u64,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub graph_scope: String,
    pub provenance_scope: String,
    pub source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TripleQueryResponse {
    pub schema_version: String,
    pub query_id: String,
    pub ordering_strategy: String,
    pub scope: TripleQueryScope,
    pub result_count: u64,
    pub triples: Vec<TripleMatch>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct InMemoryTriple {
    pub subject: String,
    pub predicate: String,
    pub object: String,
    pub graph_scope: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_ref: Option<String>,
    pub provenance_scope: String,
    pub source_ref: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

pub struct TripleQueryAdapter;

impl TripleQueryAdapter {
    pub fn execute(
        triples: &[InMemoryTriple],
        request: &TripleQueryRequest,
    ) -> Result<TripleQueryResponse, String> {
        if request.schema_version != SUPPORTED_SCHEMA_VERSION {
            return Err(format!(
                "schema_version must be {}",
                SUPPORTED_SCHEMA_VERSION
            ));
        }
        if request.ordering_strategy != SUPPORTED_ORDERING_STRATEGY {
            return Err("ordering_strategy must stay canonical".to_string());
        }

        validate_scope(&request.scope)?;

        let mut filtered: Vec<(usize, &InMemoryTriple)> = triples
            .iter()
            .enumerate()
            .filter(|(_, triple)| triple_matches_scope(triple, &request.scope))
            .filter(|(_, triple)| triple_matches_filters(triple, &request.filters))
            .collect();

        filtered.sort_by(|left, right| canonical_cmp(left.0, left.1, right.0, right.1));

        let offset = usize::try_from(request.filters.offset)
            .map_err(|_| "offset exceeds platform limits".to_string())?;
        let limit = usize::try_from(request.filters.limit)
            .map_err(|_| "limit exceeds platform limits".to_string())?;

        let matches: Vec<TripleMatch> = filtered
            .into_iter()
            .skip(offset)
            .take(limit)
            .enumerate()
            .map(|(ordinal, (_, triple))| TripleMatch {
                ordinal: ordinal as u64,
                subject: triple.subject.clone(),
                predicate: triple.predicate.clone(),
                object: triple.object.clone(),
                graph_scope: triple.graph_scope.clone(),
                provenance_scope: triple.provenance_scope.clone(),
                source_ref: triple.source_ref.clone(),
                confidence: request
                    .filters
                    .include_provenance
                    .then_some(triple.confidence)
                    .flatten(),
            })
            .collect();

        Ok(TripleQueryResponse {
            schema_version: request.schema_version.clone(),
            query_id: request.query_id.clone(),
            ordering_strategy: request.ordering_strategy.clone(),
            scope: request.scope.clone(),
            result_count: matches.len() as u64,
            triples: matches,
        })
    }
}

fn validate_scope(scope: &TripleQueryScope) -> Result<(), String> {
    match scope.named_graph_scope.as_str() {
        "actor" | "agent" => {
            if scope
                .scope_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .is_none()
            {
                return Err(format!(
                    "scope_ref is required for {} scope",
                    scope.named_graph_scope
                ));
            }
        }
        "system" => {
            if scope.scope_ref.is_some() {
                return Err("scope_ref is forbidden for system scope".to_string());
            }
        }
        "any" => {
            if scope.scope_ref.is_some() {
                return Err("scope_ref must be omitted for any scope".to_string());
            }
        }
        _ => {
            return Err(format!(
                "unsupported named_graph_scope {}",
                scope.named_graph_scope
            ));
        }
    }
    Ok(())
}

fn triple_matches_scope(triple: &InMemoryTriple, scope: &TripleQueryScope) -> bool {
    match scope.named_graph_scope.as_str() {
        "system" => triple.graph_scope == "system",
        "actor" | "agent" => {
            triple.graph_scope == scope.named_graph_scope
                && triple.scope_ref.as_deref() == scope.scope_ref.as_deref()
        }
        "any" => true,
        _ => false,
    }
}

fn triple_matches_filters(triple: &InMemoryTriple, filters: &TripleQueryFilters) -> bool {
    if let Some(subject) = &filters.subject {
        if &triple.subject != subject {
            return false;
        }
    }
    if let Some(predicate) = &filters.predicate {
        if &triple.predicate != predicate {
            return false;
        }
    }
    if let Some(object) = &filters.object {
        if &triple.object != object {
            return false;
        }
    }
    true
}

fn scope_rank(scope: &str) -> u8 {
    match scope {
        "system" => 0,
        "actor" => 1,
        "agent" => 2,
        _ => 255,
    }
}

fn canonical_cmp(
    left_index: usize,
    left: &InMemoryTriple,
    right_index: usize,
    right: &InMemoryTriple,
) -> Ordering {
    scope_rank(&left.graph_scope)
        .cmp(&scope_rank(&right.graph_scope))
        .then_with(|| left.subject.cmp(&right.subject))
        .then_with(|| left.predicate.cmp(&right.predicate))
        .then_with(|| left.object.cmp(&right.object))
        .then_with(|| scope_rank(&left.provenance_scope).cmp(&scope_rank(&right.provenance_scope)))
        .then_with(|| left.source_ref.cmp(&right.source_ref))
        .then_with(|| left.scope_ref.cmp(&right.scope_ref))
        .then_with(|| {
            left.confidence
                .map(f64::to_bits)
                .cmp(&right.confidence.map(f64::to_bits))
        })
        .then_with(|| left_index.cmp(&right_index))
}
