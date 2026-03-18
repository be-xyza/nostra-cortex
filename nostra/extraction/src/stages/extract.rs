use crate::{Document, ExtractionEntityV1, ExtractionRelationV1, Stage};
use anyhow::Result;
use std::collections::BTreeSet;

pub struct ExtractStage;

impl Stage for ExtractStage {
    fn name(&self) -> &'static str {
        "Extract (OneKE)"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;

        let mut names = BTreeSet::new();
        for token in doc.request.content.split_whitespace() {
            let clean = token
                .trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '_')
                .trim()
                .to_string();
            if clean.len() < 3 {
                continue;
            }
            if clean
                .chars()
                .next()
                .map(|c| c.is_ascii_uppercase())
                .unwrap_or(false)
            {
                names.insert(clean);
            }
        }

        doc.candidate_entities = names
            .into_iter()
            .take(16)
            .enumerate()
            .map(|(idx, label)| ExtractionEntityV1 {
                id: format!("ent_{:03}", idx + 1),
                label,
                entity_type: "Unknown".to_string(),
            })
            .collect();

        let lc = doc.request.content.to_lowercase();
        if doc.candidate_entities.len() >= 2 {
            if lc.contains(" works at ") || lc.contains(" employed by ") {
                doc.candidate_relations.push(ExtractionRelationV1 {
                    id: "rel_001".to_string(),
                    source_id: doc.candidate_entities[0].id.clone(),
                    target_id: doc.candidate_entities[1].id.clone(),
                    relation_type: "works_at".to_string(),
                });
            } else if lc.contains(" uses ") || lc.contains(" built with ") {
                doc.candidate_relations.push(ExtractionRelationV1 {
                    id: "rel_001".to_string(),
                    source_id: doc.candidate_entities[0].id.clone(),
                    target_id: doc.candidate_entities[1].id.clone(),
                    relation_type: "uses".to_string(),
                });
            } else if lc.contains(" depends on ") || lc.contains(" integrates with ") {
                doc.candidate_relations.push(ExtractionRelationV1 {
                    id: "rel_001".to_string(),
                    source_id: doc.candidate_entities[0].id.clone(),
                    target_id: doc.candidate_entities[1].id.clone(),
                    relation_type: "depends_on".to_string(),
                });
            }
        }

        let entity_factor = (doc.candidate_entities.len() as f32 * 0.03).min(0.33);
        let relation_factor = (doc.candidate_relations.len() as f32 * 0.07).min(0.21);
        doc.confidence = (0.45 + entity_factor + relation_factor).min(0.95);
        if doc.candidate_entities.is_empty() {
            doc.confidence = 0.2;
            doc.flags.push("no_candidate_entities".to_string());
        }

        doc.metadata["extraction_status"] = serde_json::json!("extracted");
        doc.metadata["entities_found"] = serde_json::json!(doc.candidate_entities.len());
        doc.metadata["relations_found"] = serde_json::json!(doc.candidate_relations.len());
        Ok(doc)
    }
}
