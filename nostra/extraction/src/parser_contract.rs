use crate::{ExtractionBlockV1, ExtractionBoundingBoxV1, ExtractionPageV1};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ParserAdapterPayloadV1 {
    #[serde(default)]
    pub job_id: Option<String>,
    pub source_ref: String,
    pub source_type: String,
    #[serde(default)]
    pub content_ref: Option<String>,
    #[serde(default)]
    pub artifact_path: Option<String>,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub file_size: Option<u64>,
    #[serde(default)]
    pub parser_profile: Option<String>,
    pub resolved_content: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ParserAdapterOutputV1 {
    #[serde(default)]
    pub parser_backend: Option<String>,
    #[serde(default)]
    pub parser_profile: Option<String>,
    #[serde(default)]
    pub pages: Vec<ExtractionPageV1>,
    #[serde(default)]
    pub flags: Vec<String>,
    #[serde(default)]
    pub model_id: Option<String>,
}

pub fn build_local_parser_output(
    parser_backend: &str,
    parser_profile: Option<&str>,
    resolved_content: &str,
    content_ref: Option<&str>,
    mut flags: Vec<String>,
    block_confidence: f32,
    model_id: impl Into<String>,
) -> ParserAdapterOutputV1 {
    let pages = split_pages(resolved_content)
        .iter()
        .enumerate()
        .map(|(page_idx, page_text)| ExtractionPageV1 {
            page_number: page_idx as u32 + 1,
            page_image_ref: content_ref.map(|value| format!("{value}#page={}", page_idx + 1)),
            blocks: page_text
                .lines()
                .enumerate()
                .filter_map(|(line_idx, line)| {
                    let trimmed = line.trim();
                    if trimmed.is_empty() {
                        return None;
                    }
                    Some(ExtractionBlockV1 {
                        block_type: if trimmed.starts_with('|') {
                            "table".to_string()
                        } else {
                            "text".to_string()
                        },
                        text: trimmed.to_string(),
                        bbox: Some(ExtractionBoundingBoxV1 {
                            x: 0.0,
                            y: line_idx as f32 * 18.0,
                            width: (trimmed.len().min(120) as f32) * 6.0,
                            height: 16.0,
                        }),
                        reading_order: Some(line_idx as u32 + 1),
                        confidence: Some(block_confidence),
                    })
                })
                .collect(),
        })
        .collect::<Vec<_>>();

    dedupe_flags(&mut flags);
    ParserAdapterOutputV1 {
        parser_backend: Some(parser_backend.to_string()),
        parser_profile: parser_profile
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(str::to_string),
        pages,
        flags,
        model_id: Some(model_id.into()),
    }
}

pub fn sanitize_flag_component(value: &str) -> String {
    let sanitized = value
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect::<String>();
    let collapsed = sanitized
        .split('_')
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join("_");
    if collapsed.is_empty() {
        "unknown".to_string()
    } else {
        collapsed.chars().take(96).collect()
    }
}

fn split_pages(content: &str) -> Vec<String> {
    let pages = content
        .split('\x0c')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if pages.is_empty() {
        vec![content.trim().to_string()]
    } else {
        pages
    }
}

fn dedupe_flags(flags: &mut Vec<String>) {
    let mut deduped = Vec::with_capacity(flags.len());
    for flag in flags.drain(..) {
        if !deduped.iter().any(|existing| existing == &flag) {
            deduped.push(flag);
        }
    }
    *flags = deduped;
}
