use crate::{Document, Stage};
use anyhow::Result;

pub struct IngestStage;

impl Stage for IngestStage {
    fn name(&self) -> &'static str {
        "Ingest"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;
        let source_type = doc.request.source_type.to_lowercase();
        let parser_hint = if source_type.contains("pdf") {
            "docling+ocrmypdf"
        } else if source_type.contains("markdown") || source_type.contains("md") {
            "markdown_normalizer"
        } else if source_type.contains("csv") {
            "csv_normalizer"
        } else {
            "plain_text"
        };

        doc.metadata = serde_json::json!({
            "status": "ingested",
            "parser_hint": parser_hint,
            "source_type": doc.request.source_type,
            "source_ref": doc.request.source_ref,
            "content_len": doc.request.content.len()
        });

        if doc.request.content.trim().is_empty() {
            doc.flags.push("empty_content".to_string());
        }

        Ok(doc)
    }
}
