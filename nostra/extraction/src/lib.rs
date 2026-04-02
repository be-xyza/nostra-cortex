pub mod adapters;
pub mod contribution_graph;
pub mod parser_bridge;
pub mod parser_contract;
pub mod stages;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionMode {
    Local,
    CloudFallback,
    ExternalAdapter,
}

impl Default for ExtractionMode {
    fn default() -> Self {
        Self::Local
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ExtractionFallbackPolicyV1 {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default = "default_min_confidence")]
    pub min_confidence: f32,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub allow_external_adapter: bool,
}

impl Default for ExtractionFallbackPolicyV1 {
    fn default() -> Self {
        Self {
            enabled: false,
            min_confidence: default_min_confidence(),
            provider: Some("azure_document_intelligence".to_string()),
            allow_external_adapter: false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionProvenanceV1 {
    pub source_version_id: String,
    pub model_id: String,
    pub produced_by_agent: String,
    pub perspective_scope: String,
    pub confidence: f32,
    pub purpose: String,
    pub extraction_backend: String,
    pub timestamp: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionRequestV1 {
    #[serde(default)]
    pub job_id: Option<String>,
    pub source_ref: String,
    pub source_type: String,
    #[serde(default)]
    pub schema_ref: Option<String>,
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub content: String,
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
    #[serde(default)]
    pub extraction_mode: ExtractionMode,
    #[serde(default)]
    pub fallback_policy: ExtractionFallbackPolicyV1,
    #[serde(default)]
    pub timeout_seconds: Option<u64>,
    #[serde(default)]
    pub index_to_knowledge: bool,
    #[serde(default)]
    pub idempotency_key: Option<String>,
    #[serde(default)]
    pub provenance_hint: Option<ExtractionProvenanceV1>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionEntityV1 {
    pub id: String,
    pub label: String,
    pub entity_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionRelationV1 {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation_type: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionBoundingBoxV1 {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionBlockV1 {
    pub block_type: String,
    pub text: String,
    #[serde(default)]
    pub bbox: Option<ExtractionBoundingBoxV1>,
    #[serde(default)]
    pub reading_order: Option<u32>,
    #[serde(default)]
    pub confidence: Option<f32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionPageV1 {
    pub page_number: u32,
    #[serde(default)]
    pub page_image_ref: Option<String>,
    #[serde(default)]
    pub blocks: Vec<ExtractionBlockV1>,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NormalizedDocumentV1 {
    #[serde(default)]
    pub parser_backend: String,
    #[serde(default)]
    pub parser_profile: Option<String>,
    #[serde(default)]
    pub pages: Vec<ExtractionPageV1>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtractionStatus {
    Submitted,
    Running,
    Completed,
    Failed,
    NeedsReview,
}

impl Default for ExtractionStatus {
    fn default() -> Self {
        Self::Submitted
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ExtractionResultV1 {
    pub job_id: String,
    pub source_ref: String,
    pub source_type: String,
    #[serde(default)]
    pub schema_ref: Option<String>,
    pub status: ExtractionStatus,
    #[serde(default)]
    pub flags: Vec<String>,
    pub confidence: f32,
    #[serde(default)]
    pub candidate_entities: Vec<ExtractionEntityV1>,
    #[serde(default)]
    pub candidate_relations: Vec<ExtractionRelationV1>,
    pub provenance: ExtractionProvenanceV1,
    #[serde(default)]
    pub attempted_backends: Vec<String>,
    #[serde(default)]
    pub fallback_reason: Option<String>,
    #[serde(default)]
    pub normalized_document: Option<NormalizedDocumentV1>,
    #[serde(default)]
    pub result_ref: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Document {
    pub request: ExtractionRequestV1,
    pub resolved_content: String,
    pub status: ExtractionStatus,
    pub confidence: f32,
    pub metadata: serde_json::Value,
    pub flags: Vec<String>,
    pub candidate_entities: Vec<ExtractionEntityV1>,
    pub candidate_relations: Vec<ExtractionRelationV1>,
    pub provenance: ExtractionProvenanceV1,
    pub attempted_backends: Vec<String>,
    pub fallback_reason: Option<String>,
    pub normalized_document: Option<NormalizedDocumentV1>,
    pub result_ref: Option<String>,
}

impl Document {
    pub fn from_request(request: &ExtractionRequestV1) -> Self {
        Self::from_request_with_content(request, None)
    }

    pub fn from_request_with_content(
        request: &ExtractionRequestV1,
        resolved_content: Option<String>,
    ) -> Self {
        let job_id = request
            .job_id
            .clone()
            .filter(|v| !v.trim().is_empty())
            .unwrap_or_else(|| format!("extract-{}", now_epoch_millis()));

        let mut provenance = request.provenance_hint.clone().unwrap_or_default();
        if provenance.source_version_id.trim().is_empty() {
            provenance.source_version_id = request
                .idempotency_key
                .clone()
                .unwrap_or_else(|| format!("v-{}", job_id));
        }
        if provenance.model_id.trim().is_empty() {
            provenance.model_id = "local/heuristic-v1".to_string();
        }
        if provenance.produced_by_agent.trim().is_empty() {
            provenance.produced_by_agent = "agent://nostra/extraction/local".to_string();
        }
        if provenance.perspective_scope.trim().is_empty() {
            provenance.perspective_scope = request
                .space_id
                .clone()
                .unwrap_or_else(|| "space://unknown".to_string());
        }
        if provenance.purpose.trim().is_empty() {
            provenance.purpose = "extraction".to_string();
        }
        if provenance.extraction_backend.trim().is_empty() {
            provenance.extraction_backend = "local_pipeline".to_string();
        }
        if provenance.timestamp.trim().is_empty() {
            provenance.timestamp = now_epoch_millis().to_string();
        }
        let resolved_content = if request.content.trim().is_empty() {
            resolved_content.unwrap_or_default()
        } else {
            request.content.clone()
        };

        Self {
            request: ExtractionRequestV1 {
                job_id: Some(job_id),
                ..request.clone()
            },
            resolved_content,
            status: ExtractionStatus::Running,
            confidence: 0.0,
            metadata: json!({}),
            flags: Vec::new(),
            candidate_entities: Vec::new(),
            candidate_relations: Vec::new(),
            provenance,
            attempted_backends: vec!["local_pipeline".to_string()],
            fallback_reason: None,
            normalized_document: None,
            result_ref: None,
        }
    }

    pub fn into_result(mut self) -> ExtractionResultV1 {
        if self.resolved_content.trim().is_empty() {
            self.status = ExtractionStatus::NeedsReview;
            self.flags.push("empty_input_content".to_string());
        }
        self.provenance.confidence = self.confidence;
        ExtractionResultV1 {
            job_id: self.request.job_id.clone().unwrap_or_default(),
            source_ref: self.request.source_ref,
            source_type: self.request.source_type,
            schema_ref: self.request.schema_ref,
            status: self.status,
            flags: self.flags,
            confidence: self.confidence,
            candidate_entities: self.candidate_entities,
            candidate_relations: self.candidate_relations,
            provenance: self.provenance,
            attempted_backends: self.attempted_backends,
            fallback_reason: self.fallback_reason,
            normalized_document: self.normalized_document,
            result_ref: self.result_ref,
        }
    }
}

pub trait Stage {
    fn name(&self) -> &'static str;
    fn process(&self, input: Document) -> Result<Document>;
}

pub trait PipelineAdapter: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    fn run(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1>;
}

pub struct ExtractionPipeline {
    stages: Vec<Box<dyn Stage>>,
}

impl ExtractionPipeline {
    pub fn new() -> Self {
        Self { stages: Vec::new() }
    }

    pub fn add_stage(&mut self, stage: Box<dyn Stage>) {
        self.stages.push(stage);
    }

    pub fn run(&self, initial_doc: Document) -> Result<Document> {
        let mut doc = initial_doc;
        for stage in &self.stages {
            doc = stage.process(doc)?;
        }
        Ok(doc)
    }
}

pub fn build_local_pipeline() -> ExtractionPipeline {
    let mut pipeline = ExtractionPipeline::new();
    pipeline.add_stage(Box::new(stages::ingest::IngestStage));
    pipeline.add_stage(Box::new(stages::extract::ExtractStage));
    pipeline.add_stage(Box::new(stages::reflect::ReflectStage));
    pipeline.add_stage(Box::new(stages::classify::ClassifyStage));
    pipeline.add_stage(Box::new(stages::verify::VerifyStage));
    pipeline
}

pub fn run_local_pipeline(request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
    let pipeline = build_local_pipeline();
    let doc = Document::from_request(request);
    let output = pipeline.run(doc)?;
    Ok(output.into_result())
}

pub fn run_local_pipeline_with_content(
    request: &ExtractionRequestV1,
    resolved_content: String,
) -> Result<ExtractionResultV1> {
    let pipeline = build_local_pipeline();
    let doc = Document::from_request_with_content(request, Some(resolved_content));
    let output = pipeline.run(doc)?;
    Ok(output.into_result())
}

pub fn validate_extraction_input_source(request: &ExtractionRequestV1) -> Result<(), String> {
    let has_inline_content = !request.content.trim().is_empty();
    let has_content_ref = request
        .content_ref
        .as_deref()
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false);
    if has_inline_content == has_content_ref {
        return Err("exactly one of content or content_ref must be populated".to_string());
    }
    Ok(())
}

fn default_min_confidence() -> f32 {
    0.85
}

fn now_epoch_millis() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn base_request() -> ExtractionRequestV1 {
        ExtractionRequestV1 {
            job_id: Some("job-det-1".to_string()),
            source_ref: "urn:test:doc".to_string(),
            source_type: "text".to_string(),
            schema_ref: Some("nostra.project".to_string()),
            space_id: Some("space:test".to_string()),
            content: "Nostra Cortex uses Rust and Motoko. Alice works at Zipstack Labs."
                .to_string(),
            content_ref: None,
            artifact_path: None,
            mime_type: Some("text/plain".to_string()),
            file_size: None,
            parser_profile: Some("local_heuristic".to_string()),
            extraction_mode: ExtractionMode::Local,
            fallback_policy: ExtractionFallbackPolicyV1::default(),
            timeout_seconds: Some(60),
            index_to_knowledge: false,
            idempotency_key: Some("source-v1".to_string()),
            provenance_hint: Some(ExtractionProvenanceV1 {
                source_version_id: "source-v1".to_string(),
                model_id: "local/heuristic-v1".to_string(),
                produced_by_agent: "agent://nostra/extraction/local".to_string(),
                perspective_scope: "space:test".to_string(),
                confidence: 0.0,
                purpose: "extraction".to_string(),
                extraction_backend: "local_pipeline".to_string(),
                timestamp: "2026-02-10T00:00:00Z".to_string(),
            }),
        }
    }

    #[test]
    fn local_pipeline_populates_required_provenance_fields() {
        let req = base_request();
        let result = run_local_pipeline(&req).expect("pipeline result");
        assert!(!result.provenance.source_version_id.trim().is_empty());
        assert!(!result.provenance.model_id.trim().is_empty());
        assert!(!result.provenance.produced_by_agent.trim().is_empty());
        assert!(!result.provenance.perspective_scope.trim().is_empty());
        assert!(!result.provenance.purpose.trim().is_empty());
    }

    #[test]
    fn verify_stage_sets_needs_review_when_confidence_floor_not_met() {
        let mut req = base_request();
        req.fallback_policy.min_confidence = 0.99;
        let result = run_local_pipeline(&req).expect("pipeline result");
        assert!(matches!(result.status, ExtractionStatus::NeedsReview));
        assert!(
            result
                .flags
                .iter()
                .any(|flag| flag.contains("verification_confidence_below_threshold"))
        );
    }

    #[test]
    fn local_pipeline_is_deterministic_for_fixed_input_and_provenance_hint() {
        let req = base_request();
        let first = run_local_pipeline(&req).expect("first");
        let second = run_local_pipeline(&req).expect("second");
        assert_eq!(
            format!("{:?}", first.status),
            format!("{:?}", second.status)
        );
        let first_entities = first
            .candidate_entities
            .iter()
            .map(|e| format!("{}|{}|{}", e.id, e.label, e.entity_type))
            .collect::<Vec<_>>();
        let second_entities = second
            .candidate_entities
            .iter()
            .map(|e| format!("{}|{}|{}", e.id, e.label, e.entity_type))
            .collect::<Vec<_>>();
        assert_eq!(first_entities, second_entities);
        let first_relations = first
            .candidate_relations
            .iter()
            .map(|r| {
                format!(
                    "{}|{}|{}|{}",
                    r.id, r.source_id, r.target_id, r.relation_type
                )
            })
            .collect::<Vec<_>>();
        let second_relations = second
            .candidate_relations
            .iter()
            .map(|r| {
                format!(
                    "{}|{}|{}|{}",
                    r.id, r.source_id, r.target_id, r.relation_type
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(first_relations, second_relations);
        assert!((first.confidence - second.confidence).abs() < f32::EPSILON);
    }
    #[test]
    fn content_ref_requests_can_run_with_host_resolved_content() {
        let mut req = base_request();
        req.content.clear();
        req.content_ref = Some("cortex://upload?id=upload-123".to_string());
        let result = run_local_pipeline_with_content(
            &req,
            "Docling extracts Nostra Cortex architecture notes.".to_string(),
        )
        .expect("pipeline result");
        assert!(result.normalized_document.is_some());
        assert!(validate_extraction_input_source(&req).is_ok());
    }

    #[test]
    fn extraction_input_source_requires_exactly_one_content_source() {
        let req = base_request();
        assert!(validate_extraction_input_source(&req).is_ok());

        let mut both = base_request();
        both.content_ref = Some("cortex://upload?id=upload-123".to_string());
        assert!(validate_extraction_input_source(&both).is_err());

        let mut neither = base_request();
        neither.content.clear();
        assert!(validate_extraction_input_source(&neither).is_err());
    }
}
