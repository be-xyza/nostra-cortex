use anyhow::{Result, anyhow};
use async_trait::async_trait;
use nostra_extraction::{
    ExtractionMode, ExtractionRequestV1, ExtractionResultV1, ExtractionStatus, run_local_pipeline,
};
use std::sync::Arc;

pub const EXTRACTION_ASYNC_PAYLOAD_SCHEMA_V1: &str = r#"{
  "type": "object",
  "required": ["source_ref", "source_type", "extraction_mode", "fallback_policy", "timeout_seconds"],
  "properties": {
    "source_ref": {"type": "string"},
    "source_type": {"type": "string"},
    "schema_ref": {"type": ["string", "null"]},
    "content": {"type": "string"},
    "content_ref": {"type": ["string", "null"]},
    "artifact_path": {"type": ["string", "null"]},
    "mime_type": {"type": ["string", "null"]},
    "file_size": {"type": ["integer", "null"], "minimum": 0},
    "parser_profile": {"type": ["string", "null"]},
    "extraction_mode": {"enum": ["local", "cloud_fallback", "external_adapter"]},
    "fallback_policy": {
      "type": "object",
      "required": ["enabled", "min_confidence"],
      "properties": {
        "enabled": {"type": "boolean"},
        "min_confidence": {"type": "number"},
        "provider": {"type": ["string", "null"]},
        "allow_external_adapter": {"type": "boolean"}
      }
    },
    "timeout_seconds": {"type": "integer", "minimum": 1}
  },
  "oneOf": [
    {"required": ["content"], "not": {"required": ["content_ref"]}},
    {"required": ["content_ref"], "not": {"required": ["content"]}}
  ]
}"#;

#[async_trait]
pub trait ExtractionAdapter: Send + Sync {
    fn adapter_id(&self) -> &'static str;
    async fn extract(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1>;
}

pub struct LocalExtractionAdapter;

#[async_trait]
impl ExtractionAdapter for LocalExtractionAdapter {
    fn adapter_id(&self) -> &'static str {
        "local_pipeline"
    }

    async fn extract(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
        run_local_pipeline(request)
    }
}

pub struct CloudExtractionAdapterAzure;

#[async_trait]
impl ExtractionAdapter for CloudExtractionAdapterAzure {
    fn adapter_id(&self) -> &'static str {
        "azure_document_intelligence"
    }

    async fn extract(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
        if !env_bool("NOSTRA_AZURE_DOCINTEL_ENABLED", false) {
            return Err(anyhow!("azure document intelligence fallback is disabled"));
        }

        // Local-first default: simulate provider path unless explicit remote mode is requested.
        if env_bool("NOSTRA_AZURE_DOCINTEL_SIMULATE", true) {
            let mut result = run_local_pipeline(request)?;
            result.provenance.extraction_backend = self.adapter_id().to_string();
            result.provenance.model_id = "azure_document_intelligence:simulated".to_string();
            result
                .flags
                .push("cloud_fallback_simulated_local_parser".to_string());
            return Ok(result);
        }

        let endpoint = std::env::var("NOSTRA_AZURE_DOCINTEL_ENDPOINT")
            .map_err(|_| anyhow!("missing NOSTRA_AZURE_DOCINTEL_ENDPOINT"))?;
        let api_key = std::env::var("NOSTRA_AZURE_DOCINTEL_API_KEY")
            .map_err(|_| anyhow!("missing NOSTRA_AZURE_DOCINTEL_API_KEY"))?;
        if endpoint.trim().is_empty() || api_key.trim().is_empty() {
            return Err(anyhow!(
                "azure document intelligence endpoint or key is empty"
            ));
        }

        // Remote call intentionally deferred for local-first optimization.
        // Keep explicit error so operators can detect unsupported mode.
        Err(anyhow!(
            "azure remote call path is disabled; set NOSTRA_AZURE_DOCINTEL_SIMULATE=true for local-first mode"
        ))
    }
}

pub struct UnstractAdapter;

#[async_trait]
impl ExtractionAdapter for UnstractAdapter {
    fn adapter_id(&self) -> &'static str {
        "unstract_adapter"
    }

    async fn extract(&self, request: &ExtractionRequestV1) -> Result<ExtractionResultV1> {
        if !env_bool("NOSTRA_UNSTRACT_ENABLED", false) {
            return Err(anyhow!("unstract adapter is disabled"));
        }

        // Optional local simulation for adapter benchmarking in local-first mode.
        if env_bool("NOSTRA_UNSTRACT_SIMULATE", true) {
            let mut result = run_local_pipeline(request)?;
            result.provenance.extraction_backend = self.adapter_id().to_string();
            result.provenance.model_id = "unstract:simulated".to_string();
            result.flags.push("external_adapter_simulated".to_string());
            return Ok(result);
        }

        let endpoint = std::env::var("NOSTRA_UNSTRACT_ENDPOINT")
            .map_err(|_| anyhow!("missing NOSTRA_UNSTRACT_ENDPOINT"))?;
        if endpoint.trim().is_empty() {
            return Err(anyhow!("unstract endpoint is empty"));
        }

        Err(anyhow!(
            "unstract remote call path is disabled; set NOSTRA_UNSTRACT_SIMULATE=true for local-first mode"
        ))
    }
}

#[derive(Clone)]
pub struct ExtractionOrchestrator {
    local: Arc<dyn ExtractionAdapter>,
    azure: Arc<dyn ExtractionAdapter>,
    unstract: Arc<dyn ExtractionAdapter>,
}

impl Default for ExtractionOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

impl ExtractionOrchestrator {
    pub fn new() -> Self {
        Self {
            local: Arc::new(LocalExtractionAdapter),
            azure: Arc::new(CloudExtractionAdapterAzure),
            unstract: Arc::new(UnstractAdapter),
        }
    }

    pub async fn extract(&self, request: &ExtractionRequestV1) -> ExtractionResultV1 {
        let primary = match request.extraction_mode {
            ExtractionMode::ExternalAdapter => self.unstract.clone(),
            ExtractionMode::Local | ExtractionMode::CloudFallback => self.local.clone(),
        };

        let mut result = match primary.extract(request).await {
            Ok(r) => r,
            Err(err) => failed_result(
                request,
                primary.adapter_id(),
                format!("primary_adapter_error: {}", err),
            ),
        };

        if !should_fallback(request, &result) {
            if result.attempted_backends.is_empty() {
                result
                    .attempted_backends
                    .push(primary.adapter_id().to_string());
            }
            return result;
        }

        let fallback_provider = request
            .fallback_policy
            .provider
            .clone()
            .unwrap_or_else(|| "azure_document_intelligence".to_string())
            .to_lowercase();

        let fallback_adapter: Arc<dyn ExtractionAdapter> = if fallback_provider.contains("azure") {
            self.azure.clone()
        } else if fallback_provider.contains("unstract") {
            self.unstract.clone()
        } else {
            self.local.clone()
        };

        let fallback_reason = format!(
            "confidence_or_status_trigger:{:.2}:{:?}",
            result.confidence, result.status
        );
        match fallback_adapter.extract(request).await {
            Ok(mut fallback_result) => {
                if fallback_result.confidence >= result.confidence {
                    fallback_result.fallback_reason = Some(fallback_reason);
                    append_attempted_backend(
                        &mut fallback_result.attempted_backends,
                        primary.adapter_id(),
                    );
                    append_attempted_backend(
                        &mut fallback_result.attempted_backends,
                        fallback_adapter.adapter_id(),
                    );
                    fallback_result
                } else {
                    result
                        .flags
                        .push("fallback_result_lower_confidence".to_string());
                    append_attempted_backend(
                        &mut result.attempted_backends,
                        fallback_adapter.adapter_id(),
                    );
                    result.fallback_reason = Some(fallback_reason);
                    result
                }
            }
            Err(err) => {
                result.flags.push(format!(
                    "fallback_adapter_error:{}:{}",
                    fallback_adapter.adapter_id(),
                    err
                ));
                append_attempted_backend(
                    &mut result.attempted_backends,
                    fallback_adapter.adapter_id(),
                );
                result.fallback_reason = Some(fallback_reason);
                result
            }
        }
    }
}

fn should_fallback(request: &ExtractionRequestV1, result: &ExtractionResultV1) -> bool {
    if !request.fallback_policy.enabled {
        return false;
    }
    matches!(
        result.status,
        ExtractionStatus::Failed | ExtractionStatus::NeedsReview
    ) || result.confidence < request.fallback_policy.min_confidence
}

fn failed_result(
    request: &ExtractionRequestV1,
    backend: &str,
    reason: String,
) -> ExtractionResultV1 {
    let mut result = run_local_pipeline(request).unwrap_or_else(|_| ExtractionResultV1 {
        job_id: request.job_id.clone().unwrap_or_default(),
        source_ref: request.source_ref.clone(),
        source_type: request.source_type.clone(),
        schema_ref: request.schema_ref.clone(),
        status: ExtractionStatus::Failed,
        flags: vec![],
        confidence: 0.0,
        candidate_entities: vec![],
        candidate_relations: vec![],
        provenance: Default::default(),
        attempted_backends: vec![],
        fallback_reason: None,
        normalized_document: None,
        result_ref: None,
    });
    result.status = ExtractionStatus::Failed;
    result.flags.push(reason);
    result.provenance.extraction_backend = backend.to_string();
    append_attempted_backend(&mut result.attempted_backends, backend);
    result
}

fn append_attempted_backend(backends: &mut Vec<String>, backend: &str) {
    if !backends.iter().any(|v| v == backend) {
        backends.push(backend.to_string());
    }
}

fn env_bool(key: &str, default_value: bool) -> bool {
    std::env::var(key)
        .ok()
        .map(|v| {
            matches!(
                v.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes" | "on"
            )
        })
        .unwrap_or(default_value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nostra_extraction::{ExtractionFallbackPolicyV1, ExtractionMode, ExtractionRequestV1};

    fn make_request(mode: ExtractionMode, fallback_enabled: bool) -> ExtractionRequestV1 {
        ExtractionRequestV1 {
            job_id: Some("job-test".to_string()),
            source_ref: "urn:test".to_string(),
            source_type: "text".to_string(),
            schema_ref: Some("nostra.project".to_string()),
            space_id: Some("space:test".to_string()),
            content: "Nostra Cortex uses Rust and Motoko. Alice works at Zipstack Labs."
                .to_string(),
            content_ref: None,
            artifact_path: None,
            mime_type: Some("text/plain".to_string()),
            file_size: None,
            parser_profile: Some("docling".to_string()),
            extraction_mode: mode,
            fallback_policy: ExtractionFallbackPolicyV1 {
                enabled: fallback_enabled,
                min_confidence: 0.95,
                provider: Some("azure_document_intelligence".to_string()),
                allow_external_adapter: false,
            },
            timeout_seconds: Some(60),
            index_to_knowledge: false,
            idempotency_key: Some("idem-1".to_string()),
            provenance_hint: None,
        }
    }

    #[tokio::test]
    async fn local_mode_runs_without_fallback_by_default() {
        let orchestrator = ExtractionOrchestrator::new();
        let req = make_request(ExtractionMode::Local, false);
        let result = orchestrator.extract(&req).await;
        assert_eq!(result.job_id, "job-test");
        assert!(!result.attempted_backends.is_empty());
    }

    #[tokio::test]
    async fn cloud_fallback_attempts_azure_when_enabled() {
        let orchestrator = ExtractionOrchestrator::new();
        std::env::set_var("NOSTRA_AZURE_DOCINTEL_ENABLED", "true");
        std::env::set_var("NOSTRA_AZURE_DOCINTEL_SIMULATE", "true");
        let req = make_request(ExtractionMode::CloudFallback, true);
        let result = orchestrator.extract(&req).await;
        assert!(
            result
                .attempted_backends
                .iter()
                .any(|v| v == "azure_document_intelligence")
        );
    }
}
