use crate::{Document, Stage};
use anyhow::Result;

pub struct ReflectStage;

impl Stage for ReflectStage {
    fn name(&self) -> &'static str {
        "Reflect (Graphiti)"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;

        if doc.candidate_entities.len() < 2 {
            doc.flags.push("reflection_sparse_entities".to_string());
            doc.confidence = (doc.confidence - 0.08).max(0.0);
        }

        if doc.resolved_content.len() > 2000 && doc.candidate_entities.len() < 4 {
            doc.flags
                .push("reflection_possible_under_extraction".to_string());
            doc.confidence = (doc.confidence - 0.1).max(0.0);
        }

        if doc.resolved_content.to_lowercase().contains("unknown")
            && doc.candidate_relations.is_empty()
        {
            doc.flags
                .push("reflection_ambiguous_without_relations".to_string());
            doc.confidence = (doc.confidence - 0.05).max(0.0);
        }

        doc.metadata["reflection_status"] = serde_json::json!("reviewed");
        Ok(doc)
    }
}
