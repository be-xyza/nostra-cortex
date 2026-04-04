use crate::services::knowledge_graph_query::InMemoryTriple;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RuntimeEventSource {
    pub kind: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub scope_ref: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GlobalEventTripleRecord {
    pub event_id: String,
    pub source_ref: String,
    pub source: RuntimeEventSource,
    pub subject: String,
    pub predicate: String,
    pub object: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

pub struct RuntimeTripleProjector;

impl RuntimeTripleProjector {
    pub fn project(records: &[GlobalEventTripleRecord]) -> Result<Vec<InMemoryTriple>, String> {
        records
            .iter()
            .map(|record| {
                let (graph_scope, provenance_scope, scope_ref) =
                    normalized_scope(&record.source.kind, record.source.scope_ref.clone())?;
                Ok(InMemoryTriple {
                    subject: record.subject.clone(),
                    predicate: record.predicate.clone(),
                    object: record.object.clone(),
                    graph_scope,
                    scope_ref,
                    provenance_scope,
                    source_ref: record.source_ref.clone(),
                    confidence: record.confidence,
                })
            })
            .collect()
    }
}

fn normalized_scope(
    kind: &str,
    scope_ref: Option<String>,
) -> Result<(String, String, Option<String>), String> {
    match kind {
        "system" => {
            if scope_ref.is_some() {
                return Err("system events must not carry scope_ref".to_string());
            }
            Ok(("system".to_string(), "system".to_string(), None))
        }
        "actor" => {
            let scope = scope_ref
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "actor events require scope_ref".to_string())?;
            Ok(("actor".to_string(), "actor".to_string(), Some(scope)))
        }
        "agent" => {
            let scope = scope_ref
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| "agent events require scope_ref".to_string())?;
            Ok(("agent".to_string(), "agent".to_string(), Some(scope)))
        }
        other => Err(format!("unsupported runtime source kind {other}")),
    }
}
