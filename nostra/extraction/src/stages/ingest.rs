use crate::{
    Document, ExtractionRequestV1, NormalizedDocumentV1, Stage, adapters::run_parser_adapter,
};
use anyhow::Result;

pub struct IngestStage;

impl Stage for IngestStage {
    fn name(&self) -> &'static str {
        "Ingest"
    }

    fn process(&self, input: Document) -> Result<Document> {
        let mut doc = input;
        let parser_execution = run_parser_adapter(&doc.request, &doc.resolved_content)?;
        let parser_backend = parser_execution.normalized_document.parser_backend.clone();
        let parser_hint = parser_execution.parser_hint.clone();
        let page_count = parser_execution.normalized_document.pages.len();
        let synthesized = synthesize_resolved_content(&parser_execution.normalized_document);
        if synthesized.trim().is_empty() {
            if doc.resolved_content.trim().is_empty() {
                doc.flags.push("empty_content".to_string());
            }
        } else if should_replace_resolved_content_from_parser(&doc.request, &doc.resolved_content) {
            let had_existing_content = !doc.resolved_content.trim().is_empty();
            doc.resolved_content = synthesized;
            doc.flags.push(if had_existing_content {
                "resolved_content_replaced_by_parser".to_string()
            } else {
                "resolved_content_synthesized_from_parser".to_string()
            });
        } else if doc.resolved_content.trim().is_empty() {
            doc.resolved_content = synthesized;
            doc.flags
                .push("resolved_content_synthesized_from_parser".to_string());
        }

        doc.metadata = serde_json::json!({
            "status": "ingested",
            "parser_hint": parser_hint,
            "parser_backend": parser_backend,
            "source_type": doc.request.source_type,
            "source_ref": doc.request.source_ref,
            "content_ref": doc.request.content_ref,
            "artifact_path": doc.request.artifact_path,
            "content_len": doc.resolved_content.len(),
            "mime_type": doc.request.mime_type,
            "page_count": page_count
        });
        doc.provenance.extraction_backend =
            parser_execution.normalized_document.parser_backend.clone();
        doc.provenance.model_id = parser_execution.model_id;
        extend_unique_flags(&mut doc.flags, parser_execution.flags);
        doc.normalized_document = Some(parser_execution.normalized_document);

        Ok(doc)
    }
}

fn should_replace_resolved_content_from_parser(
    request: &ExtractionRequestV1,
    current_content: &str,
) -> bool {
    if current_content.trim().is_empty() {
        return true;
    }

    let source_type = request.source_type.trim().to_ascii_lowercase();
    if matches!(source_type.as_str(), "pdf" | "image") {
        return true;
    }

    let mime_type = request
        .mime_type
        .as_deref()
        .unwrap_or_default()
        .trim()
        .to_ascii_lowercase();
    if mime_type.is_empty() {
        return false;
    }

    !is_textual_mime_type(&mime_type)
        && (request.artifact_path.is_some() || request.content_ref.is_some())
}

fn is_textual_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("text/")
        || matches!(
            mime_type,
            "application/json" | "application/xml" | "text/csv" | "text/html"
        )
}

fn synthesize_resolved_content(document: &NormalizedDocumentV1) -> String {
    document
        .pages
        .iter()
        .map(|page| {
            page.blocks
                .iter()
                .map(|block| block.text.trim())
                .filter(|text| !text.is_empty())
                .collect::<Vec<_>>()
                .join("\n")
        })
        .filter(|page_text| !page_text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\x0c")
}

fn extend_unique_flags(target: &mut Vec<String>, incoming: Vec<String>) {
    for flag in incoming {
        if !target.iter().any(|existing| existing == &flag) {
            target.push(flag);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{should_replace_resolved_content_from_parser, synthesize_resolved_content};
    use crate::{ExtractionBlockV1, ExtractionPageV1, ExtractionRequestV1, NormalizedDocumentV1};

    #[test]
    fn synthesized_content_joins_blocks_and_pages() {
        let document = NormalizedDocumentV1 {
            parser_backend: "docling".to_string(),
            parser_profile: Some("docling".to_string()),
            pages: vec![
                ExtractionPageV1 {
                    page_number: 1,
                    page_image_ref: None,
                    blocks: vec![
                        ExtractionBlockV1 {
                            block_type: "text".to_string(),
                            text: "Alpha line".to_string(),
                            ..Default::default()
                        },
                        ExtractionBlockV1 {
                            block_type: "text".to_string(),
                            text: "Beta line".to_string(),
                            ..Default::default()
                        },
                    ],
                },
                ExtractionPageV1 {
                    page_number: 2,
                    page_image_ref: None,
                    blocks: vec![ExtractionBlockV1 {
                        block_type: "text".to_string(),
                        text: "Gamma line".to_string(),
                        ..Default::default()
                    }],
                },
            ],
        };

        assert_eq!(
            synthesize_resolved_content(&document),
            "Alpha line\nBeta line\x0cGamma line"
        );
    }

    #[test]
    fn binary_artifacts_prefer_parser_synthesized_content() {
        let request = ExtractionRequestV1 {
            source_ref: "cortex://upload?id=upload-1".to_string(),
            source_type: "pdf".to_string(),
            content_ref: Some("cortex://upload?id=upload-1".to_string()),
            artifact_path: Some("/tmp/upload.pdf".to_string()),
            mime_type: Some("application/pdf".to_string()),
            ..Default::default()
        };

        assert!(should_replace_resolved_content_from_parser(
            &request,
            "%PDF-1.7 nonsense"
        ));
    }

    #[test]
    fn textual_inputs_keep_existing_resolved_content() {
        let request = ExtractionRequestV1 {
            source_ref: "cortex://upload?id=upload-2".to_string(),
            source_type: "text".to_string(),
            content_ref: Some("cortex://upload?id=upload-2".to_string()),
            artifact_path: Some("/tmp/upload.md".to_string()),
            mime_type: Some("text/markdown".to_string()),
            ..Default::default()
        };

        assert!(!should_replace_resolved_content_from_parser(
            &request,
            "# Existing markdown"
        ));
    }
}
