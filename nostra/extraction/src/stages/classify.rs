use crate::{Document, Stage};
use anyhow::Result;

pub struct ClassifyStage;

impl Stage for ClassifyStage {
    fn name(&self) -> &'static str {
        "Classify (OpenSPG)"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;
        let schema_ref = doc
            .request
            .schema_ref
            .clone()
            .unwrap_or_default()
            .to_lowercase();

        for entity in &mut doc.candidate_entities {
            let label = entity.label.to_lowercase();
            entity.entity_type = if label.contains("inc")
                || label.contains("corp")
                || label.contains("llc")
                || label.contains("ltd")
                || label.contains("labs")
                || label.contains("university")
                || label.contains("foundation")
            {
                "Organization".to_string()
            } else if label.contains("api")
                || label.contains("sdk")
                || label.contains("rust")
                || label.contains("motoko")
                || label.contains("docker")
                || label.contains("kubernetes")
                || label.contains("ollama")
                || label.contains("azure")
            {
                "Technology".to_string()
            } else if schema_ref.contains("project")
                || label.contains("project")
                || label.contains("platform")
                || label.contains("engine")
                || label.contains("workflow")
                || label.contains("nostra")
                || label.contains("cortex")
            {
                "Project".to_string()
            } else {
                "Concept".to_string()
            };
        }

        doc.metadata["classification_status"] = serde_json::json!("classified");
        Ok(doc)
    }
}
