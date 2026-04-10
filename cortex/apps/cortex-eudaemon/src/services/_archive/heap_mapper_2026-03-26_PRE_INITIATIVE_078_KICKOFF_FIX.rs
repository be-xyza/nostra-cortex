use chrono::{DateTime, Utc};
use cortex_domain::collaboration::crdt::AguiCrdtMutation;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

const HEAP_SCHEMA_VERSION: &str = "1.0.0";
const HEAP_MODE: &str = "heap";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeapMapperError {
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<Value>,
}

impl HeapMapperError {
    fn new(code: &str, message: impl Into<String>, details: Option<Value>) -> Self {
        Self {
            code: code.to_string(),
            message: message.into(),
            details,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct EmitHeapBlockRequest {
    pub schema_version: String,
    pub mode: String,
    pub workspace_id: String,
    pub source: HeapSource,
    pub block: HeapBlock,
    pub content: HeapContent,
    #[serde(default)]
    pub relations: HeapRelations,
    #[serde(default)]
    pub files: Vec<HeapFileRef>,
    #[serde(default)]
    pub apps: Vec<HeapAppConfig>,
    #[serde(default)]
    pub meta: Option<HeapMeta>,
    #[serde(default)]
    pub projection_hints: HeapProjectionHints,
    #[serde(default)]
    pub crdt_projection: HeapCrdtProjection,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapSource {
    pub agent_id: String,
    #[serde(default)]
    pub session_id: Option<String>,
    #[serde(default)]
    pub request_id: Option<String>,
    pub emitted_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapBlock {
    #[serde(default)]
    pub id: Option<String>,
    pub r#type: String,
    pub title: String,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub icon_type: Option<String>,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub main_tag: Option<String>,
    #[serde(default)]
    pub attributes: Option<std::collections::BTreeMap<String, String>>,
    #[serde(default)]
    pub behaviors: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapContent {
    pub payload_type: HeapPayloadType,
    #[serde(default)]
    pub a2ui: Option<HeapA2uiContent>,
    #[serde(default)]
    pub rich_text: Option<HeapRichTextContent>,
    #[serde(default)]
    pub media: Option<HeapMediaContent>,
    #[serde(default)]
    pub structured_data: Option<Value>,
    #[serde(default)]
    pub pointer: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HeapPayloadType {
    A2ui,
    RichText,
    Media,
    StructuredData,
    Pointer,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapRichTextContent {
    pub plain_text: String,
    #[serde(default)]
    pub title_doc: Option<Value>,
    #[serde(default)]
    pub text_doc: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapMediaContent {
    pub hash: String,
    pub mime_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapA2uiContent {
    pub surface_id: String,
    pub protocol_version: String,
    #[serde(default)]
    pub renderer: Option<String>,
    #[serde(default)]
    pub view_type: Option<String>,
    pub tree: Value,
    #[serde(default)]
    pub data_model: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapRelations {
    #[serde(default)]
    pub tags: Vec<HeapRelationTag>,
    #[serde(default)]
    pub mentions: Vec<HeapRelationMention>,
    #[serde(default)]
    pub page_links: Vec<HeapPageLink>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapRelationTag {
    pub to_block_id: String,
    #[serde(default)]
    pub meta: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapRelationMention {
    pub to_block_id: String,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapPageLink {
    pub to_block_id: String,
    #[serde(default)]
    pub source_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapFileRef {
    pub hash: String,
    pub file_size: u64,
    pub name: String,
    #[serde(default)]
    pub mime_type: Option<String>,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub is_uploaded: Option<bool>,
    #[serde(default)]
    pub thumbnails: Vec<HeapThumbnail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapThumbnail {
    pub r#type: String,
    pub size: String,
    #[serde(default)]
    pub path: Option<String>,
    #[serde(default)]
    pub width: Option<u64>,
    #[serde(default)]
    pub height: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapAppConfig {
    pub id: String,
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub app_type: Option<String>,
    #[serde(default)]
    pub filter: Option<Value>,
    #[serde(default)]
    pub sort: Option<Value>,
    #[serde(default)]
    pub mapping: Option<Value>,
    #[serde(default)]
    pub settings: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapMeta {
    #[serde(default)]
    pub deleted_at: Option<String>,
    #[serde(default)]
    pub permanently_deleted_at: Option<String>,
    #[serde(default)]
    pub sidebar_order: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HeapFilesKeyFormat {
    #[serde(rename = "hash")]
    Hash,
    #[serde(rename = "hash:file_size")]
    HashFileSize,
}

impl Default for HeapFilesKeyFormat {
    fn default() -> Self {
        Self::HashFileSize
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum HeapRelationMapVersion {
    RelationsV0,
    RelationsV1,
}

impl Default for HeapRelationMapVersion {
    fn default() -> Self {
        Self::RelationsV1
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapProjectionHints {
    #[serde(default = "default_true")]
    pub mirror_mentions_to_relations: bool,
    #[serde(default)]
    pub files_key_format: HeapFilesKeyFormat,
    #[serde(default)]
    pub relation_map_version: HeapRelationMapVersion,
}

impl Default for HeapProjectionHints {
    fn default() -> Self {
        Self {
            mirror_mentions_to_relations: true,
            files_key_format: HeapFilesKeyFormat::HashFileSize,
            relation_map_version: HeapRelationMapVersion::RelationsV1,
        }
    }
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapCrdtProjection {
    #[serde(default)]
    pub artifact_id: Option<String>,
    #[serde(default)]
    pub base_version: Option<u64>,
    #[serde(default)]
    pub mutations: Vec<HeapCrdtMutationHint>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case", deny_unknown_fields)]
pub struct HeapCrdtMutationHint {
    pub op: String,
    pub path: String,
    #[serde(default)]
    pub value: Option<Value>,
    #[serde(default)]
    pub index: Option<usize>,
    #[serde(default)]
    pub delete_count: Option<usize>,
    #[serde(default)]
    pub text: Option<String>,
    #[serde(default)]
    pub clock: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalHeapFile {
    pub key: String,
    pub hash: String,
    pub file_size: u64,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mime_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_uploaded: Option<bool>,
    #[serde(default)]
    pub thumbnails: Vec<HeapThumbnail>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct CanonicalizedHeapBlock {
    pub mirror_mentions_to_relations: bool,
    pub relation_map_version: String,
    pub files_key_format: String,
    pub tags: Vec<String>,
    pub mentions_inline: Vec<String>,
    pub mentions_query: Vec<String>,
    pub page_links: Vec<String>,
    pub files: Vec<CanonicalHeapFile>,
    pub warnings: Vec<String>,
    pub surface_json: Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HeapBlockProjection {
    pub schema_version: String,
    pub artifact_id: String,
    pub workspace_id: String,
    pub block_id: String,
    pub block_type: String,
    pub title: String,
    pub surface_id: String,
    pub emitted_at: String,
    pub created_at: String,
    pub updated_at: String,
    pub actor_id: String,
    pub actor_role: String,
    pub mirror_mentions_to_relations: bool,
    pub relation_map_version: String,
    pub files_key_format: String,
    pub tags: Vec<String>,
    pub mentions_inline: Vec<String>,
    pub mentions_query: Vec<String>,
    pub page_links: Vec<String>,
    pub file_keys: Vec<String>,
    pub has_files: bool,
    #[serde(default)]
    pub attributes: Option<std::collections::BTreeMap<String, String>>,
}

pub fn parse_emit_heap_block(value: Value) -> Result<EmitHeapBlockRequest, HeapMapperError> {
    serde_json::from_value::<EmitHeapBlockRequest>(value).map_err(|err| {
        HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "Payload does not match EmitHeapBlock schema.",
            Some(json!({ "reason": err.to_string() })),
        )
    })
}

pub fn validate_emit_heap_block(request: &EmitHeapBlockRequest) -> Result<(), HeapMapperError> {
    if request.schema_version != HEAP_SCHEMA_VERSION {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_UNSUPPORTED_VERSION",
            "Unsupported heap schema version.",
            Some(json!({
                "expected": HEAP_SCHEMA_VERSION,
                "actual": request.schema_version
            })),
        ));
    }

    if request.mode != HEAP_MODE {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "mode must be 'heap'.",
            Some(json!({ "actual": request.mode })),
        ));
    }

    if !is_ulid(&request.workspace_id) {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "workspaceId must be a ULID.",
            Some(json!({ "workspaceId": request.workspace_id })),
        ));
    }

    if request.source.agent_id.trim().is_empty() {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "source.agentId is required.",
            None,
        ));
    }

    if DateTime::parse_from_rfc3339(&request.source.emitted_at).is_err() {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "source.emittedAt must be RFC3339.",
            Some(json!({ "emittedAt": request.source.emitted_at })),
        ));
    }

    if let Some(block_id) = request.block.id.as_deref() {
        if !is_ulid(block_id) {
            return Err(HeapMapperError::new(
                "HEAP_SCHEMA_INVALID",
                "block.id must be a ULID when provided.",
                Some(json!({ "blockId": block_id })),
            ));
        }
    }

    if request.block.r#type.trim().is_empty() || request.block.title.trim().is_empty() {
        return Err(HeapMapperError::new(
            "HEAP_SCHEMA_INVALID",
            "block.type and block.title are required.",
            None,
        ));
    }

    match request.content.payload_type {
        HeapPayloadType::A2ui => {
            if let Some(a2ui) = &request.content.a2ui {
                if a2ui.surface_id.trim().is_empty() || a2ui.protocol_version.trim().is_empty() {
                    return Err(HeapMapperError::new(
                        "HEAP_SCHEMA_INVALID",
                        "a2ui payload requires surface_id and protocol_version.",
                        None,
                    ));
                }
            } else {
                return Err(HeapMapperError::new(
                    "HEAP_SCHEMA_INVALID",
                    "a2ui payload requested but content.a2ui is missing.",
                    None,
                ));
            }
        }
        HeapPayloadType::RichText => {
            if request.content.rich_text.is_none() {
                return Err(HeapMapperError::new(
                    "HEAP_SCHEMA_INVALID",
                    "rich_text payload requested but content.rich_text is missing.",
                    None,
                ));
            }
        }
        HeapPayloadType::Media => {
            if request.content.media.is_none() {
                return Err(HeapMapperError::new(
                    "HEAP_SCHEMA_INVALID",
                    "media payload requested but content.media is missing.",
                    None,
                ));
            }
        }
        HeapPayloadType::StructuredData => {
            if request.content.structured_data.is_none() {
                return Err(HeapMapperError::new(
                    "HEAP_SCHEMA_INVALID",
                    "structured_data payload requested but content.structured_data is missing.",
                    None,
                ));
            }
        }
        HeapPayloadType::Pointer => {
            if request.content.pointer.is_none() {
                return Err(HeapMapperError::new(
                    "HEAP_SCHEMA_INVALID",
                    "pointer payload requested but content.pointer is missing.",
                    None,
                ));
            }
        }
    }

    for tag in &request.relations.tags {
        if !is_ulid(&tag.to_block_id) {
            return Err(HeapMapperError::new(
                "HEAP_SCHEMA_INVALID",
                "relations.tags[].toBlockId must be a ULID.",
                Some(json!({ "toBlockId": tag.to_block_id })),
            ));
        }
    }

    for mention in &request.relations.mentions {
        if !is_ulid(&mention.to_block_id) {
            return Err(HeapMapperError::new(
                "HEAP_SCHEMA_INVALID",
                "relations.mentions[].toBlockId must be a ULID.",
                Some(json!({ "toBlockId": mention.to_block_id })),
            ));
        }
    }

    for link in &request.relations.page_links {
        if !is_ulid(&link.to_block_id) {
            return Err(HeapMapperError::new(
                "HEAP_SCHEMA_INVALID",
                "relations.pageLinks[].toBlockId must be a ULID.",
                Some(json!({ "toBlockId": link.to_block_id })),
            ));
        }
    }

    for app in &request.apps {
        if !is_ulid(&app.id) {
            return Err(HeapMapperError::new(
                "HEAP_SCHEMA_INVALID",
                "apps[].id must be a ULID.",
                Some(json!({ "appId": app.id })),
            ));
        }
    }

    Ok(())
}

pub fn canonicalize_emit_heap_block(
    request: &EmitHeapBlockRequest,
) -> Result<CanonicalizedHeapBlock, HeapMapperError> {
    let mut warnings = Vec::new();

    let mut files = Vec::new();
    for file in &request.files {
        if file.hash.trim().len() < 8 || file.name.trim().is_empty() {
            return Err(HeapMapperError::new(
                "HEAP_CANONICALIZATION_ERROR",
                "files[] must include hash and name.",
                Some(json!({ "hash": file.hash, "name": file.name })),
            ));
        }

        let key = format!("{}:{}", file.hash.trim(), file.file_size);
        files.push(CanonicalHeapFile {
            key,
            hash: file.hash.trim().to_string(),
            file_size: file.file_size,
            name: file.name.trim().to_string(),
            mime_type: file.mime_type.clone(),
            path: file.path.clone(),
            is_uploaded: file.is_uploaded,
            thumbnails: file.thumbnails.clone(),
        });
    }

    let tags = request
        .relations
        .tags
        .iter()
        .map(|tag| tag.to_block_id.clone())
        .collect::<Vec<_>>();

    let mentions_inline = request
        .relations
        .mentions
        .iter()
        .map(|mention| {
            mention
                .label
                .as_ref()
                .map(|label| label.trim().to_string())
                .filter(|label| !label.is_empty())
                .unwrap_or_else(|| mention.to_block_id.clone())
        })
        .collect::<Vec<_>>();

    let mentions_query = if request.projection_hints.mirror_mentions_to_relations {
        mentions_inline.clone()
    } else {
        Vec::new()
    };

    if !request.projection_hints.mirror_mentions_to_relations && !mentions_inline.is_empty() {
        warnings.push(
            "mentions present but mirrorMentionsToRelations=false; mention filtering will not index them"
                .to_string(),
        );
    }

    if matches!(
        request.projection_hints.relation_map_version,
        HeapRelationMapVersion::RelationsV0
    ) {
        warnings.push(
            "relationMapVersion=relations_v0 accepted for compatibility and normalized to relations_v1"
                .to_string(),
        );
    }

    if matches!(
        request.projection_hints.files_key_format,
        HeapFilesKeyFormat::Hash
    ) {
        warnings.push(
            "filesKeyFormat=hash accepted for compatibility and normalized to hash:file_size"
                .to_string(),
        );
    }

    let page_links = request
        .relations
        .page_links
        .iter()
        .map(|link| link.to_block_id.clone())
        .collect::<Vec<_>>();

    let surface_json = derive_surface_json(request, &tags, &mentions_inline);

    Ok(CanonicalizedHeapBlock {
        mirror_mentions_to_relations: request.projection_hints.mirror_mentions_to_relations,
        relation_map_version: "relations_v1".to_string(),
        files_key_format: "hash:file_size".to_string(),
        tags,
        mentions_inline,
        mentions_query,
        page_links,
        files,
        warnings,
        surface_json,
    })
}

pub fn map_emit_heap_block_to_agui_mutations(
    request: &EmitHeapBlockRequest,
    canonical: &CanonicalizedHeapBlock,
) -> Result<Vec<AguiCrdtMutation>, HeapMapperError> {
    let mut entries = Vec::<(Option<String>, String, Value)>::new();

    entries.push((
        Some("/heap/source".to_string()),
        "source".to_string(),
        serde_json::to_value(&request.source).map_err(|err| {
            HeapMapperError::new(
                "HEAP_CANONICALIZATION_ERROR",
                "Failed to serialize source block.",
                Some(json!({ "reason": err.to_string() })),
            )
        })?,
    ));

    entries.push((
        Some("/heap/block".to_string()),
        "block".to_string(),
        serde_json::to_value(&request.block).map_err(|err| {
            HeapMapperError::new(
                "HEAP_CANONICALIZATION_ERROR",
                "Failed to serialize block metadata.",
                Some(json!({ "reason": err.to_string() })),
            )
        })?,
    ));

    if let Some(attributes) = &request.block.attributes {
        entries.push((
            Some("/heap/block".to_string()),
            "attributes".to_string(),
            serde_json::to_value(attributes).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize block attributes.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    entries.push((
        Some("/heap/content".to_string()),
        "payload_type".to_string(),
        serde_json::to_value(&request.content.payload_type).map_err(|err| {
            HeapMapperError::new(
                "HEAP_CANONICALIZATION_ERROR",
                "Failed to serialize payload type.",
                Some(json!({ "reason": err.to_string() })),
            )
        })?,
    ));

    if let Some(a2ui) = &request.content.a2ui {
        entries.push((
            Some("/heap/content".to_string()),
            "a2ui".to_string(),
            serde_json::to_value(a2ui).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize A2UI content.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    if let Some(rich_text) = &request.content.rich_text {
        entries.push((
            Some("/heap/content".to_string()),
            "rich_text".to_string(),
            serde_json::to_value(rich_text).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize rich_text content.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    if let Some(media) = &request.content.media {
        entries.push((
            Some("/heap/content".to_string()),
            "media".to_string(),
            serde_json::to_value(media).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize media content.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    if let Some(structured_data) = &request.content.structured_data {
        entries.push((
            Some("/heap/content".to_string()),
            "structured_data".to_string(),
            structured_data.clone(),
        ));
    }

    if let Some(pointer) = &request.content.pointer {
        entries.push((
            Some("/heap/content".to_string()),
            "pointer".to_string(),
            Value::String(pointer.clone()),
        ));
    }

    entries.push((
        Some("/heap/content".to_string()),
        "surface".to_string(),
        canonical.surface_json.clone(),
    ));

    entries.push((
        Some("/heap/relations".to_string()),
        "tags".to_string(),
        json!(canonical.tags),
    ));
    entries.push((
        Some("/heap/relations".to_string()),
        "mentions_inline".to_string(),
        json!(canonical.mentions_inline),
    ));
    entries.push((
        Some("/heap/relations".to_string()),
        "mentions_query".to_string(),
        json!(canonical.mentions_query),
    ));
    entries.push((
        Some("/heap/relations".to_string()),
        "page_links".to_string(),
        json!(canonical.page_links),
    ));

    entries.push((
        Some("/heap/projection".to_string()),
        "hints".to_string(),
        json!({
            "mirror_mentions_to_relations": canonical.mirror_mentions_to_relations,
            "files_key_format": canonical.files_key_format,
            "relation_map_version": canonical.relation_map_version,
        }),
    ));

    if let Some(meta) = &request.meta {
        entries.push((
            Some("/heap/meta".to_string()),
            "meta".to_string(),
            serde_json::to_value(meta).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize meta payload.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    for app in &request.apps {
        entries.push((
            Some("/heap/apps".to_string()),
            app.id.clone(),
            serde_json::to_value(app).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize app payload.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    for file in &canonical.files {
        entries.push((
            Some("/heap/files".to_string()),
            file.key.clone(),
            serde_json::to_value(file).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to serialize file payload.",
                    Some(json!({ "reason": err.to_string() })),
                )
            })?,
        ));
    }

    entries.sort_by(|left, right| {
        left.0
            .as_deref()
            .unwrap_or_default()
            .cmp(right.0.as_deref().unwrap_or_default())
            .then_with(|| left.1.cmp(&right.1))
    });

    entries
        .into_iter()
        .map(|(path, key, value)| {
            let value_json = serde_json::to_string(&value).map_err(|err| {
                HeapMapperError::new(
                    "HEAP_CANONICALIZATION_ERROR",
                    "Failed to encode mutation value.",
                    Some(json!({ "reason": err.to_string(), "key": key })),
                )
            })?;
            Ok(AguiCrdtMutation {
                path,
                key,
                value_json,
            })
        })
        .collect::<Result<Vec<_>, _>>()
}

pub fn project_heap_block(
    request: &EmitHeapBlockRequest,
    canonical: &CanonicalizedHeapBlock,
    artifact_id: &str,
    actor_id: &str,
    actor_role: &str,
    created_at: &str,
    updated_at: &str,
) -> HeapBlockProjection {
    let block_id = request
        .block
        .id
        .clone()
        .unwrap_or_else(|| artifact_id.to_string());

    HeapBlockProjection {
        schema_version: HEAP_SCHEMA_VERSION.to_string(),
        artifact_id: artifact_id.to_string(),
        workspace_id: request.workspace_id.clone(),
        block_id,
        block_type: request.block.r#type.clone(),
        title: request.block.title.clone(),
        surface_id: request
            .content
            .a2ui
            .as_ref()
            .map(|a| a.surface_id.clone())
            .unwrap_or_else(|| "non_a2ui_surface".to_string()),
        emitted_at: request.source.emitted_at.clone(),
        created_at: created_at.to_string(),
        updated_at: updated_at.to_string(),
        actor_id: actor_id.to_string(),
        actor_role: actor_role.to_string(),
        mirror_mentions_to_relations: canonical.mirror_mentions_to_relations,
        relation_map_version: canonical.relation_map_version.clone(),
        files_key_format: canonical.files_key_format.clone(),
        tags: canonical.tags.clone(),
        mentions_inline: canonical.mentions_inline.clone(),
        mentions_query: canonical.mentions_query.clone(),
        page_links: canonical.page_links.clone(),
        file_keys: canonical
            .files
            .iter()
            .map(|file| file.key.clone())
            .collect(),
        has_files: !canonical.files.is_empty(),
        attributes: request.block.attributes.clone(),
    }
}

pub fn derive_surface_json(
    request: &EmitHeapBlockRequest,
    tags: &[String],
    mentions: &[String],
) -> Value {
    if let Some(a2ui) = &request.content.a2ui {
        if a2ui
            .tree
            .as_object()
            .is_some_and(|obj| obj.contains_key("components"))
        {
            let mut tree = a2ui.tree.clone();
            if let Some(obj) = tree.as_object_mut() {
                obj.entry("surfaceId".to_string())
                    .or_insert_with(|| Value::String(a2ui.surface_id.clone()));
                obj.entry("title".to_string())
                    .or_insert_with(|| Value::String(request.block.title.clone()));
                let meta = obj.entry("meta".to_string()).or_insert_with(|| json!({}));
                if let Some(meta_obj) = meta.as_object_mut() {
                    meta_obj
                        .entry("tags".to_string())
                        .or_insert_with(|| json!(tags));
                    meta_obj
                        .entry("mentions".to_string())
                        .or_insert_with(|| json!(mentions));
                }
            }
            return tree;
        }
    }

    let description = match &request.content.payload_type {
        HeapPayloadType::RichText => request
            .content
            .rich_text
            .as_ref()
            .map(|rt| rt.plain_text.clone())
            .unwrap_or_else(|| "Empty text block".to_string()),
        HeapPayloadType::Media => "Media block".to_string(),
        HeapPayloadType::StructuredData => "Structured data block".to_string(),
        HeapPayloadType::Pointer => "Reference pointer block".to_string(),
        HeapPayloadType::A2ui => "A2UI fallback block".to_string(),
    };

    let surface_id = request
        .content
        .a2ui
        .as_ref()
        .map(|a| a.surface_id.clone())
        .unwrap_or_else(|| "fallback_surface".to_string());

    let meta = json!({
        "tags": tags,
        "mentions": mentions,
        "surface_type": "heap"
    });

    match &request.content.payload_type {
        HeapPayloadType::RichText => {
            let plain_text = request
                .content
                .rich_text
                .as_ref()
                .map(|rt| rt.plain_text.clone())
                .unwrap_or_default();
            return json!({
                "payload_type": "rich_text",
                "surfaceId": surface_id,
                "title": request.block.title,
                "text": plain_text,
                "plain_text": plain_text,
                "meta": meta,
            });
        }
        HeapPayloadType::Media => {
            return json!({
                "payload_type": "media",
                "surfaceId": surface_id,
                "title": request.block.title,
                "media": request.content.media,
                "meta": meta,
            });
        }
        HeapPayloadType::StructuredData => {
            return json!({
                "payload_type": "structured_data",
                "surfaceId": surface_id,
                "title": request.block.title,
                "structured_data": request.content.structured_data,
                "meta": meta,
            });
        }
        HeapPayloadType::Pointer => {
            return json!({
                "payload_type": "pointer",
                "surfaceId": surface_id,
                "title": request.block.title,
                "pointer": request.content.pointer,
                "meta": meta,
            });
        }
        HeapPayloadType::A2ui => {}
    }

    json!({
        "payload_type": "a2ui",
        "surfaceId": surface_id,
        "title": request.block.title,
        "root": "heap_root",
        "components": [
            {
                "id": "heap_root",
                "type": "Card",
                "props": {
                    "title": request.block.title,
                    "description": description
                },
                "children": []
            }
        ],
        "meta": meta
    })
}

fn is_ulid(value: &str) -> bool {
    if value.len() != 26 {
        return false;
    }
    value.chars().all(
        |ch| matches!(ch, '0'..='9' | 'A'..='H' | 'J'..='K' | 'M'..='N' | 'P'..='T' | 'V'..='Z'),
    )
}

pub fn parse_iso_timestamp(value: &str) -> Option<DateTime<Utc>> {
    DateTime::parse_from_rfc3339(value)
        .ok()
        .map(|ts| ts.with_timezone(&Utc))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture_payload() -> EmitHeapBlockRequest {
        EmitHeapBlockRequest {
            schema_version: "1.0.0".to_string(),
            mode: "heap".to_string(),
            workspace_id: "01ARZ3NDEKTSV4RRFFQ69G5FAV".to_string(),
            source: HeapSource {
                agent_id: "agent-zero".to_string(),
                session_id: Some("sess-1".to_string()),
                request_id: Some("req-1".to_string()),
                emitted_at: "2026-02-23T12:00:00Z".to_string(),
            },
            block: HeapBlock {
                id: Some("01ARZ3NDEKTSV4RRFFQ69G5FAW".to_string()),
                r#type: "widget".to_string(),
                title: "Revenue Snapshot".to_string(),
                icon: None,
                icon_type: None,
                color: None,
                main_tag: None,
                attributes: None,
                behaviors: None,
            },
            content: HeapContent {
                payload_type: HeapPayloadType::A2ui,
                a2ui: Some(HeapA2uiContent {
                    surface_id: "surface:rev-snap".to_string(),
                    protocol_version: "1.0.0".to_string(),
                    renderer: Some("react".to_string()),
                    view_type: None,
                    tree: json!({
                        "surfaceId": "surface:rev-snap",
                        "title": "Revenue Snapshot",
                        "root": "root",
                        "components": [{
                            "id": "root",
                            "type": "Card",
                            "props": {"title": "Revenue"},
                            "children": []
                        }]
                    }),
                    data_model: Some(json!({"quarter":"Q3"})),
                }),
                rich_text: Some(HeapRichTextContent {
                    plain_text: "Q3 summary".to_string(),
                    title_doc: None,
                    text_doc: None,
                }),
                media: None,
                structured_data: None,
                pointer: None,
            },
            relations: HeapRelations {
                tags: vec![HeapRelationTag {
                    to_block_id: "01ARZ3NDEKTSV4RRFFQ69G5FAX".to_string(),
                    meta: None,
                }],
                mentions: vec![HeapRelationMention {
                    to_block_id: "01ARZ3NDEKTSV4RRFFQ69G5FAY".to_string(),
                    label: Some("Project Alpha".to_string()),
                    source_path: None,
                }],
                page_links: vec![HeapPageLink {
                    to_block_id: "01ARZ3NDEKTSV4RRFFQ69G5FAZ".to_string(),
                    source_path: None,
                }],
            },
            files: vec![HeapFileRef {
                hash: "abc12345".to_string(),
                file_size: 42,
                name: "report.png".to_string(),
                mime_type: Some("image/png".to_string()),
                path: None,
                is_uploaded: Some(true),
                thumbnails: vec![],
            }],
            apps: vec![],
            meta: None,
            projection_hints: HeapProjectionHints::default(),
            crdt_projection: HeapCrdtProjection::default(),
        }
    }

    #[test]
    fn validate_rejects_bad_workspace_id() {
        let mut payload = fixture_payload();
        payload.workspace_id = "bad".to_string();
        let err = validate_emit_heap_block(&payload).expect_err("must reject invalid ulid");
        assert_eq!(err.code, "HEAP_SCHEMA_INVALID");
    }

    #[test]
    fn canonicalizes_files_and_mentions() {
        let payload = fixture_payload();
        let canonical = canonicalize_emit_heap_block(&payload).expect("canonical payload");
        assert_eq!(canonical.files[0].key, "abc12345:42");
        assert_eq!(canonical.mentions_inline, vec!["Project Alpha"]);
        assert_eq!(canonical.mentions_query, vec!["Project Alpha"]);
    }

    #[test]
    fn map_is_deterministic() {
        let payload = fixture_payload();
        let canonical = canonicalize_emit_heap_block(&payload).expect("canonical payload");
        let first = map_emit_heap_block_to_agui_mutations(&payload, &canonical).expect("first");
        let second = map_emit_heap_block_to_agui_mutations(&payload, &canonical).expect("second");
        assert_eq!(first, second);
    }
}
