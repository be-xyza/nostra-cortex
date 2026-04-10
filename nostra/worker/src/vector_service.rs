// vector_service.rs
// High-level Vector Service combining EmbeddingProvider + VectorClient

use crate::config_service::ConfigService;
use crate::embedding_provider::{EmbeddingError, EmbeddingMetadata, EmbeddingProvider};
use crate::vector_client::VectorClient;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::RwLock;
use tokio::time::{Duration, timeout};

const COLLECTION_TEXT: &str = "text_vectors";
const COLLECTION_IMAGE: &str = "image_vectors";
const COLLECTION_AUDIO: &str = "audio_vectors";
const COLLECTION_VIDEO: &str = "video_segments";

/// Error types for VectorService operations
#[derive(Debug)]
pub enum VectorServiceError {
    EmbeddingError(EmbeddingError),
    StorageError(String),
    #[allow(dead_code)]
    ConfigError(String),
}

impl std::fmt::Display for VectorServiceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VectorServiceError::EmbeddingError(e) => write!(f, "Embedding error: {}", e),
            VectorServiceError::StorageError(e) => write!(f, "Storage error: {}", e),
            VectorServiceError::ConfigError(e) => write!(f, "Config error: {}", e),
        }
    }
}

impl std::error::Error for VectorServiceError {}

impl From<EmbeddingError> for VectorServiceError {
    fn from(e: EmbeddingError) -> Self {
        VectorServiceError::EmbeddingError(e)
    }
}

impl From<anyhow::Error> for VectorServiceError {
    fn from(e: anyhow::Error) -> Self {
        VectorServiceError::StorageError(e.to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CeiMetadataV1 {
    pub contribution_id: String,
    pub source_uri: String,
    pub author: String,
    pub timestamp: String,
    #[serde(default)]
    pub lineage_refs: Vec<String>,
    #[serde(default)]
    pub source_version_id: Option<String>,
    #[serde(default)]
    pub model_id: Option<String>,
    #[serde(default)]
    pub perspective_scope: Option<String>,
    #[serde(default)]
    pub produced_by_agent: Option<String>,
    #[serde(default)]
    pub confidence: Option<f32>,
    #[serde(default)]
    pub purpose: Option<String>,
    #[serde(default)]
    pub modality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TimeRangeFilter {
    #[serde(default)]
    pub from_ms: Option<i64>,
    #[serde(default)]
    pub to_ms: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SearchFilters {
    #[serde(default)]
    pub space_id: Option<String>,
    #[serde(default)]
    pub source_type: Option<String>,
    #[serde(default)]
    pub time_range: Option<TimeRangeFilter>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub perspective_scope: Option<String>,
    #[serde(default)]
    pub produced_by_agent: Option<String>,
    #[serde(default)]
    pub source_version_id: Option<String>,
    #[serde(default)]
    pub modalities: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HybridConfig {
    pub vector_weight: f32,
    pub lexical_weight: f32,
    #[serde(default)]
    pub rerank_enabled: bool,
}

impl Default for HybridConfig {
    fn default() -> Self {
        Self {
            vector_weight: 0.65,
            lexical_weight: 0.35,
            rerank_enabled: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "lowercase")]
pub enum RetrievalMode {
    Vector,
    Lexical,
    #[default]
    Hybrid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    #[serde(default)]
    pub retrieval_mode: RetrievalMode,
    #[serde(default)]
    pub filters: SearchFilters,
    #[serde(default)]
    pub fusion_weights: HybridConfig,
    #[serde(default)]
    pub diagnostics: bool,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            retrieval_mode: RetrievalMode::Hybrid,
            filters: SearchFilters::default(),
            fusion_weights: HybridConfig::default(),
            diagnostics: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetrievalDiagnostic {
    pub vector_score: f32,
    pub lexical_score: f32,
    pub fused_score: f32,
    pub rank_reason: String,
    pub backend: String,
    pub embedding_model: String,
}

/// Result of a similarity search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub score: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub space_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub modality: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<CeiMetadataV1>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub diagnostic: Option<RetrievalDiagnostic>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexDocument {
    pub id: String,
    pub text: String,
    pub label: String,
    pub space_id: String,
    pub source_ref: String,
    pub source_type: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub timestamp_ms: Option<i64>,
    #[serde(default)]
    pub cei_metadata: Option<CeiMetadataV1>,
    #[serde(default)]
    pub modality: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexOutcome {
    pub indexed_count: usize,
    pub skipped_duplicate: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idempotency_key: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VectorServiceHealth {
    pub status: String,
    pub backend: String,
    pub vector_endpoint: String,
    pub vector_client_enabled: bool,
    pub embedding_provider_id: String,
    pub embedding_dimension: usize,
    pub embedding_probe_ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub embedding_probe_error: Option<String>,
    pub indexed_documents: usize,
    pub shadow_compare_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ShadowReport {
    pub enabled: bool,
    pub searches: u64,
    pub overlap_total: u64,
    pub union_total: u64,
    pub average_parity: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_parity: Option<f32>,
}

#[derive(Debug, Default)]
struct ShadowMetrics {
    searches: u64,
    overlap_total: u64,
    union_total: u64,
    last_parity: Option<f32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum VectorBackend {
    Elna,
    Mock,
}

impl VectorBackend {
    fn as_str(&self) -> &'static str {
        match self {
            VectorBackend::Elna => "elna",
            VectorBackend::Mock => "mock",
        }
    }
}

#[derive(Debug, Clone)]
struct IndexedDocument {
    id: String,
    text: String,
    space_id: String,
    source_ref: String,
    source_type: String,
    tags: Vec<String>,
    timestamp_ms: Option<i64>,
    provenance: Option<CeiMetadataV1>,
    modality: String,
    vector_collection: String,
}

/// High-level Vector Service
///
/// Combines embedding generation and vector storage into a unified API.
/// This is the main interface for semantic search in Nostra.
#[derive(Clone)]
pub struct VectorService {
    embedding_provider: Arc<dyn EmbeddingProvider>,
    vector_client: Option<VectorClient>,
    default_collection: String,
    backend: VectorBackend,
    vector_endpoint: String,
    vector_timeout_ms: u64,
    elna_fail_open: bool,
    shadow_compare: bool,
    shadow_metrics: Arc<Mutex<ShadowMetrics>>,
    documents: Arc<RwLock<HashMap<String, IndexedDocument>>>,
    idempotency_keys: Arc<RwLock<HashSet<String>>>,
}

impl VectorService {
    /// Create a new VectorService with Config-driven resolution
    pub fn new(
        embedding_provider: Arc<dyn EmbeddingProvider>,
        agent: Arc<ic_agent::Agent>,
        default_collection: String,
    ) -> Self {
        let default_collection = std::env::var("NOSTRA_VECTOR_COLLECTION")
            .ok()
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty())
            .unwrap_or(default_collection);

        let config = ConfigService::get();
        let (mut endpoint, _fallbacks) = config.get_vector_endpoints();
        if let Ok(env_endpoint) = std::env::var("NOSTRA_VECTOR_ENDPOINT") {
            if !env_endpoint.trim().is_empty() {
                endpoint = env_endpoint;
            }
        }

        let backend_override = std::env::var("VECTOR_BACKEND")
            .or_else(|_| std::env::var("NOSTRA_VECTOR_BACKEND"))
            .ok()
            .unwrap_or_default()
            .to_lowercase();

        let mut backend = match backend_override.as_str() {
            "elna" => VectorBackend::Elna,
            "mock" => VectorBackend::Mock,
            _ => {
                if ic_agent::export::Principal::from_text(&endpoint).is_ok() {
                    VectorBackend::Elna
                } else {
                    VectorBackend::Mock
                }
            }
        };

        let vector_client = if backend == VectorBackend::Elna {
            match ic_agent::export::Principal::from_text(&endpoint) {
                Ok(canister_id) => Some(VectorClient::new(agent, canister_id)),
                Err(_) => {
                    println!(
                        "VectorService: backend=elna but endpoint '{}' is not a Principal. Falling back to mock backend.",
                        endpoint
                    );
                    backend = VectorBackend::Mock;
                    None
                }
            }
        } else {
            None
        };

        let shadow_compare = std::env::var("NOSTRA_VECTOR_SHADOW_COMPARE")
            .map(|v| matches!(v.to_lowercase().as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        let vector_timeout_ms = std::env::var("NOSTRA_VECTOR_TIMEOUT_MS")
            .ok()
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(2500);
        let elna_fail_open = std::env::var("NOSTRA_ELNA_FAIL_OPEN")
            .map(|v| !matches!(v.to_lowercase().as_str(), "0" | "false" | "off" | "no"))
            .unwrap_or(true);

        Self {
            embedding_provider,
            vector_client,
            default_collection,
            backend,
            vector_endpoint: endpoint,
            vector_timeout_ms,
            elna_fail_open,
            shadow_compare,
            shadow_metrics: Arc::new(Mutex::new(ShadowMetrics::default())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            idempotency_keys: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    fn normalize_modality(raw: &str) -> Option<String> {
        match raw.trim().to_lowercase().as_str() {
            "text" => Some("text".to_string()),
            "image" | "images" => Some("image".to_string()),
            "audio" => Some("audio".to_string()),
            "video" | "video_segment" | "video_segments" => Some("video".to_string()),
            _ => None,
        }
    }

    fn infer_modality_from_source_type(source_type: &str) -> String {
        let normalized = source_type.trim().to_lowercase();
        if normalized.contains("image") || normalized.contains("screenshot") {
            "image".to_string()
        } else if normalized.contains("audio") || normalized.contains("voice") {
            "audio".to_string()
        } else if normalized.contains("video") {
            "video".to_string()
        } else {
            "text".to_string()
        }
    }

    fn resolve_doc_modality(&self, doc: &IndexDocument) -> String {
        if let Some(modality) = doc.modality.as_deref().and_then(Self::normalize_modality) {
            return modality;
        }
        if let Some(modality) = doc
            .cei_metadata
            .as_ref()
            .and_then(|meta| meta.modality.as_deref())
            .and_then(Self::normalize_modality)
        {
            return modality;
        }
        Self::infer_modality_from_source_type(&doc.source_type)
    }

    fn collection_for_modality(&self, modality: &str) -> String {
        match modality {
            "text" => COLLECTION_TEXT.to_string(),
            "image" => COLLECTION_IMAGE.to_string(),
            "audio" => COLLECTION_AUDIO.to_string(),
            "video" => COLLECTION_VIDEO.to_string(),
            _ => self.default_collection.clone(),
        }
    }

    /// Get metadata about the current embedding provider
    pub fn embedding_metadata(&self) -> EmbeddingMetadata {
        self.embedding_provider.metadata()
    }

    pub fn backend_name(&self) -> &'static str {
        self.backend.as_str()
    }

    pub async fn document_count(&self) -> usize {
        self.documents.read().await.len()
    }

    pub async fn health(&self) -> VectorServiceHealth {
        let probe = self.embedding_provider.embed("nostra-health-check").await;
        let (probe_ok, probe_error) = match probe {
            Ok(vec) => {
                if vec.len() == self.embedding_provider.dimension() {
                    (true, None)
                } else {
                    (
                        false,
                        Some(format!(
                            "dimension mismatch: expected {}, got {}",
                            self.embedding_provider.dimension(),
                            vec.len()
                        )),
                    )
                }
            }
            Err(e) => (false, Some(e.to_string())),
        };

        VectorServiceHealth {
            status: if probe_ok { "ok" } else { "degraded" }.to_string(),
            backend: self.backend.as_str().to_string(),
            vector_endpoint: self.vector_endpoint.clone(),
            vector_client_enabled: self.vector_client.is_some(),
            embedding_provider_id: self.embedding_provider.model_id().to_string(),
            embedding_dimension: self.embedding_provider.dimension(),
            embedding_probe_ok: probe_ok,
            embedding_probe_error: probe_error,
            indexed_documents: self.document_count().await,
            shadow_compare_enabled: self.shadow_compare,
        }
    }

    pub fn shadow_report(&self) -> ShadowReport {
        let metrics = self.shadow_metrics.lock().unwrap();
        let average_parity = if metrics.union_total == 0 {
            0.0
        } else {
            metrics.overlap_total as f32 / metrics.union_total as f32
        };

        ShadowReport {
            enabled: self.shadow_compare,
            searches: metrics.searches,
            overlap_total: metrics.overlap_total,
            union_total: metrics.union_total,
            average_parity,
            last_parity: metrics.last_parity,
        }
    }

    /// Initialize the vector collection (creates if not exists)
    pub async fn init_collection(&self) -> Result<(), VectorServiceError> {
        self.ensure_collection(&self.default_collection).await
    }

    async fn ensure_collection(&self, collection: &str) -> Result<(), VectorServiceError> {
        if let Some(client) = &self.vector_client {
            let dimension = self.embedding_provider.dimension() as u64;
            let created = timeout(
                Duration::from_millis(self.vector_timeout_ms),
                client.create_collection(collection, dimension),
            )
            .await;
            match created {
                Ok(Ok(())) => {}
                Ok(Err(e)) => {
                    let message = e.to_string();
                    if message.contains("UniqueViolation") {
                        // Collection already exists; treat as idempotent success.
                        return Ok(());
                    }
                    if self.elna_fail_open {
                        println!(
                            "VectorService: create_collection failed (fail-open enabled): {}",
                            message
                        );
                    } else {
                        return Err(VectorServiceError::StorageError(message));
                    }
                }
                Err(_) => {
                    if self.elna_fail_open {
                        println!(
                            "VectorService: create_collection timeout after {}ms (fail-open enabled)",
                            self.vector_timeout_ms
                        );
                    } else {
                        return Err(VectorServiceError::StorageError(format!(
                            "create_collection timeout after {}ms",
                            self.vector_timeout_ms
                        )));
                    }
                }
            }
        } else {
            println!(
                "VectorService [Mock]: Initialized collection '{}'",
                collection
            );
        }
        Ok(())
    }

    /// Register idempotency key; returns false if already seen.
    pub async fn register_idempotency_key(&self, key: &str) -> bool {
        if key.trim().is_empty() {
            return true;
        }

        let mut keys = self.idempotency_keys.write().await;
        if keys.contains(key) {
            false
        } else {
            keys.insert(key.to_string());
            true
        }
    }

    /// Index docs with CEI metadata and idempotency support.
    pub async fn index_documents(
        &self,
        documents: Vec<IndexDocument>,
        idempotency_key: Option<&str>,
    ) -> Result<IndexOutcome, VectorServiceError> {
        if documents.is_empty() {
            return Ok(IndexOutcome {
                indexed_count: 0,
                skipped_duplicate: false,
                idempotency_key: idempotency_key.map(|s| s.to_string()),
            });
        }

        if let Some(key) = idempotency_key {
            if !self.register_idempotency_key(key).await {
                return Ok(IndexOutcome {
                    indexed_count: 0,
                    skipped_duplicate: true,
                    idempotency_key: Some(key.to_string()),
                });
            }
        }

        // Ensure backing collections exist before attempting inserts.
        // Supports modality-specific namespaces while remaining compatible
        // with the configured default collection.
        let mut required_collections: HashSet<String> = HashSet::new();
        required_collections.insert(self.default_collection.clone());
        for doc in &documents {
            let modality = self.resolve_doc_modality(doc);
            required_collections.insert(self.collection_for_modality(&modality));
        }
        for collection in required_collections {
            self.ensure_collection(&collection).await?;
        }

        const MICRO_BATCH_SIZE: usize = 50;
        let mut indexed_count = 0usize;
        let mut touched_collections: HashSet<String> = HashSet::new();

        for (batch_idx, chunk) in documents.chunks(MICRO_BATCH_SIZE).enumerate() {
            for doc in chunk {
                if let Some(meta) = &doc.cei_metadata {
                    self.validate_cei_metadata(meta)?;
                }
            }

            let texts: Vec<&str> = chunk.iter().map(|doc| doc.text.as_str()).collect();
            let embeddings = self.embed_batch_checked(&texts).await?;

            if let Some(client) = &self.vector_client {
                // Vector canister currently supports one label per insert call.
                // Group by collection+label to preserve metadata semantics while
                // keeping modality-specific vector namespaces.
                type BatchMap = HashMap<(String, String), Vec<(String, Vec<f32>)>>;
                let mut by_collection_label: BatchMap = HashMap::new();
                for (doc, embedding) in chunk.iter().zip(embeddings.into_iter()) {
                    let modality = self.resolve_doc_modality(doc);
                    let collection = self.collection_for_modality(&modality);
                    by_collection_label
                        .entry((collection, doc.label.clone()))
                        .or_default()
                        .push((doc.id.clone(), embedding));
                }

                for ((collection, label), rows) in by_collection_label {
                    let ids: Vec<String> = rows.iter().map(|(id, _)| id.clone()).collect();
                    let vectors: Vec<Vec<f32>> = rows.into_iter().map(|(_, v)| v).collect();
                    let inserted = timeout(
                        Duration::from_millis(self.vector_timeout_ms),
                        client.insert(&collection, vectors, ids, &label),
                    )
                    .await;
                    match inserted {
                        Ok(Ok(())) => {
                            touched_collections.insert(collection);
                        }
                        Ok(Err(e)) => {
                            if self.elna_fail_open {
                                println!(
                                    "VectorService: insert failed on batch {} (fail-open enabled): {}",
                                    batch_idx, e
                                );
                            } else {
                                return Err(VectorServiceError::StorageError(format!(
                                    "Batch {}: {}",
                                    batch_idx, e
                                )));
                            }
                        }
                        Err(_) => {
                            if self.elna_fail_open {
                                println!(
                                    "VectorService: insert timeout on batch {} after {}ms (fail-open enabled)",
                                    batch_idx, self.vector_timeout_ms
                                );
                            } else {
                                return Err(VectorServiceError::StorageError(format!(
                                    "Batch {}: insert timeout after {}ms",
                                    batch_idx, self.vector_timeout_ms
                                )));
                            }
                        }
                    }
                }
            }

            {
                let mut store = self.documents.write().await;
                for doc in chunk {
                    let modality = self.resolve_doc_modality(doc);
                    let vector_collection = self.collection_for_modality(&modality);
                    let mut provenance = doc.cei_metadata.clone();
                    if let Some(meta) = provenance.as_mut() {
                        if meta.modality.is_none() {
                            meta.modality = Some(modality.clone());
                        }
                    }
                    store.insert(
                        doc.id.clone(),
                        IndexedDocument {
                            id: doc.id.clone(),
                            text: doc.text.clone(),
                            space_id: doc.space_id.clone(),
                            source_ref: doc.source_ref.clone(),
                            source_type: doc.source_type.clone(),
                            tags: doc.tags.clone(),
                            timestamp_ms: doc.timestamp_ms,
                            provenance,
                            modality,
                            vector_collection,
                        },
                    );
                    indexed_count += 1;
                }
            }
        }

        // ELNA only searches the built index; inserts append vectors but do not
        // automatically refresh the ANN graph. Rebuild once per indexing call.
        if !touched_collections.is_empty() {
            self.build_index_for_collections(&touched_collections)
                .await?;
        }

        Ok(IndexOutcome {
            indexed_count,
            skipped_duplicate: false,
            idempotency_key: idempotency_key.map(|s| s.to_string()),
        })
    }

    /// Index a single document.
    #[allow(dead_code)]
    pub async fn index_document(
        &self,
        id: &str,
        text: &str,
        label: &str,
    ) -> Result<(), VectorServiceError> {
        let doc = IndexDocument {
            id: id.to_string(),
            text: text.to_string(),
            label: label.to_string(),
            space_id: "default".to_string(),
            source_ref: label.to_string(),
            source_type: "document".to_string(),
            tags: vec![],
            timestamp_ms: None,
            cei_metadata: None,
            modality: None,
        };

        self.index_documents(vec![doc], None).await?;
        Ok(())
    }

    /// Legacy batch index adapter `(id, text, label)`.
    pub async fn index_batch(
        &self,
        documents: Vec<(&str, &str, &str)>,
    ) -> Result<(), VectorServiceError> {
        if documents.is_empty() {
            return Ok(());
        }

        let mapped: Vec<IndexDocument> = documents
            .into_iter()
            .map(|(id, text, label)| IndexDocument {
                id: id.to_string(),
                text: text.to_string(),
                label: label.to_string(),
                space_id: "default".to_string(),
                source_ref: label.to_string(),
                source_type: "legacy_batch".to_string(),
                tags: vec![],
                timestamp_ms: None,
                cei_metadata: None,
                modality: None,
            })
            .collect();

        self.index_documents(mapped, None).await?;
        Ok(())
    }

    /// Legacy search adapter.
    pub async fn search(
        &self,
        query: &str,
        top_k: i32,
    ) -> Result<Vec<SearchResult>, VectorServiceError> {
        self.search_with_options(query, top_k, SearchOptions::default())
            .await
    }

    /// Advanced search with retrieval mode, filters, and diagnostics.
    pub async fn search_with_options(
        &self,
        query: &str,
        top_k: i32,
        options: SearchOptions,
    ) -> Result<Vec<SearchResult>, VectorServiceError> {
        let limit = top_k.max(1) as usize;

        let filtered_docs = {
            let store = self.documents.read().await;
            store
                .values()
                .filter(|doc| self.matches_filters(doc, &options.filters))
                .cloned()
                .collect::<Vec<_>>()
        };

        if filtered_docs.is_empty() {
            return Ok(vec![]);
        }

        let lexical_scores = self.compute_lexical_scores(query, &filtered_docs);
        let candidate_pool_ids: Vec<String> =
            filtered_docs.iter().map(|doc| doc.id.clone()).collect();
        let mut target_collections: Vec<String> = if options.filters.modalities.is_empty() {
            filtered_docs
                .iter()
                .map(|doc| doc.vector_collection.clone())
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        } else {
            options
                .filters
                .modalities
                .iter()
                .filter_map(|raw| Self::normalize_modality(raw))
                .map(|modality| self.collection_for_modality(&modality))
                .collect::<HashSet<_>>()
                .into_iter()
                .collect()
        };
        if target_collections.is_empty() {
            target_collections.push(self.default_collection.clone());
        }
        target_collections.sort();

        let mut vector_rank_scores: HashMap<String, f32> = HashMap::new();
        let mut vector_ids_ranked: Vec<String> = vec![];

        if matches!(
            options.retrieval_mode,
            RetrievalMode::Vector | RetrievalMode::Hybrid
        ) {
            if let Some(client) = &self.vector_client {
                let query_embedding = self.embed_checked(query).await?;
                let mut vector_search_failed = false;

                for collection in &target_collections {
                    let search = timeout(
                        Duration::from_millis(self.vector_timeout_ms),
                        client.search(collection, query_embedding.clone(), top_k),
                    )
                    .await;

                    match search {
                        Ok(Ok(ids)) => {
                            for (idx, id) in ids.into_iter().enumerate() {
                                let rank_score = 1.0 / (60.0 + idx as f32 + 1.0);
                                let current = vector_rank_scores.entry(id.clone()).or_insert(0.0);
                                if rank_score > *current {
                                    *current = rank_score;
                                }
                                if !vector_ids_ranked.iter().any(|existing| existing == &id) {
                                    vector_ids_ranked.push(id);
                                }
                            }
                        }
                        Ok(Err(e)) => {
                            if self.elna_fail_open {
                                println!(
                                    "VectorService: search failed in '{}' (fail-open lexical fallback): {}",
                                    collection, e
                                );
                                vector_search_failed = true;
                                break;
                            } else {
                                return Err(VectorServiceError::StorageError(format!(
                                    "collection '{}': {}",
                                    collection, e
                                )));
                            }
                        }
                        Err(_) => {
                            if self.elna_fail_open {
                                println!(
                                    "VectorService: search timeout in '{}' after {}ms (fail-open lexical fallback)",
                                    collection, self.vector_timeout_ms
                                );
                                vector_search_failed = true;
                                break;
                            } else {
                                return Err(VectorServiceError::StorageError(format!(
                                    "collection '{}': search timeout after {}ms",
                                    collection, self.vector_timeout_ms
                                )));
                            }
                        }
                    }
                }

                if vector_search_failed || vector_rank_scores.is_empty() {
                    let mut lexical_ranked: Vec<(String, f32)> = lexical_scores
                        .iter()
                        .map(|(id, score)| (id.clone(), *score))
                        .collect();
                    lexical_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
                    for (idx, (id, _)) in lexical_ranked.into_iter().enumerate() {
                        let rank_score = 1.0 / (60.0 + idx as f32 + 1.0);
                        vector_rank_scores.insert(id.clone(), rank_score);
                        if !vector_ids_ranked.iter().any(|existing| existing == &id) {
                            vector_ids_ranked.push(id);
                        }
                    }
                }
            } else {
                // Mock fallback for local development without vector backend.
                // Use lexical ordering as a deterministic proxy to keep API usable.
                let mut lexical_ranked: Vec<(String, f32)> = lexical_scores
                    .iter()
                    .map(|(id, score)| (id.clone(), *score))
                    .collect();
                lexical_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));
                for (idx, (id, _)) in lexical_ranked.into_iter().enumerate() {
                    let rank_score = 1.0 / (60.0 + idx as f32 + 1.0);
                    vector_rank_scores.insert(id.clone(), rank_score);
                    vector_ids_ranked.push(id);
                }
            }

            if self.shadow_compare {
                self.log_shadow_parity(
                    &vector_ids_ranked,
                    &lexical_scores,
                    &candidate_pool_ids,
                    limit,
                );
            }
        }

        let mut candidate_ids: HashSet<String> = HashSet::new();
        match options.retrieval_mode {
            RetrievalMode::Vector => {
                candidate_ids.extend(vector_rank_scores.keys().cloned());
            }
            RetrievalMode::Lexical => {
                candidate_ids.extend(lexical_scores.keys().cloned());
            }
            RetrievalMode::Hybrid => {
                candidate_ids.extend(vector_rank_scores.keys().cloned());
                candidate_ids.extend(lexical_scores.keys().cloned());
            }
        }

        let doc_map: HashMap<String, IndexedDocument> = filtered_docs
            .into_iter()
            .map(|doc| (doc.id.clone(), doc))
            .collect();

        let mut merged_results: Vec<SearchResult> = candidate_ids
            .into_iter()
            .filter_map(|id| {
                let doc = doc_map.get(&id)?;
                let vector_score = *vector_rank_scores.get(&id).unwrap_or(&0.0);
                let lexical_score = *lexical_scores.get(&id).unwrap_or(&0.0);

                let mut fused_score = match options.retrieval_mode {
                    RetrievalMode::Vector => vector_score,
                    RetrievalMode::Lexical => lexical_score,
                    RetrievalMode::Hybrid => {
                        options.fusion_weights.vector_weight * vector_score
                            + options.fusion_weights.lexical_weight * lexical_score
                    }
                };

                if options.fusion_weights.rerank_enabled {
                    fused_score += 0.1 * self.rerank_bonus(query, &doc.text);
                }

                let rank_reason = self.rank_reason(
                    options.retrieval_mode.clone(),
                    vector_score,
                    lexical_score,
                    options.fusion_weights.rerank_enabled,
                );

                Some(SearchResult {
                    id: id.clone(),
                    score: fused_score,
                    content: Some(Self::content_snippet(&doc.text)),
                    source_ref: Some(doc.source_ref.clone()),
                    space_id: Some(doc.space_id.clone()),
                    source_type: Some(doc.source_type.clone()),
                    modality: Some(doc.modality.clone()),
                    tags: doc.tags.clone(),
                    provenance: doc.provenance.clone(),
                    diagnostic: if options.diagnostics {
                        Some(RetrievalDiagnostic {
                            vector_score,
                            lexical_score,
                            fused_score,
                            rank_reason,
                            backend: self.backend.as_str().to_string(),
                            embedding_model: self.embedding_provider.model_id(),
                        })
                    } else {
                        None
                    },
                })
            })
            .collect();

        merged_results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(Ordering::Equal)
                .then_with(|| a.id.cmp(&b.id))
        });
        merged_results.truncate(limit);

        Ok(merged_results)
    }

    /// Build the search index (call after batch indexing)
    pub async fn build_index(&self) -> Result<(), VectorServiceError> {
        let collections: HashSet<String> = {
            let store = self.documents.read().await;
            store
                .values()
                .map(|doc| doc.vector_collection.clone())
                .collect()
        };
        self.build_index_for_collections(&collections).await
    }

    async fn build_index_for_collections(
        &self,
        collections: &HashSet<String>,
    ) -> Result<(), VectorServiceError> {
        if let Some(client) = &self.vector_client {
            let mut targets: Vec<String> = if collections.is_empty() {
                vec![self.default_collection.clone()]
            } else {
                collections.iter().cloned().collect()
            };
            targets.sort();

            for collection in targets {
                let built = timeout(
                    Duration::from_millis(self.vector_timeout_ms),
                    client.build_index(&collection),
                )
                .await;
                match built {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        if self.elna_fail_open {
                            println!(
                                "VectorService: build_index failed for '{}' (fail-open enabled): {}",
                                collection, e
                            );
                        } else {
                            return Err(VectorServiceError::StorageError(format!(
                                "collection '{}': {}",
                                collection, e
                            )));
                        }
                    }
                    Err(_) => {
                        if self.elna_fail_open {
                            println!(
                                "VectorService: build_index timeout for '{}' after {}ms (fail-open enabled)",
                                collection, self.vector_timeout_ms
                            );
                        } else {
                            return Err(VectorServiceError::StorageError(format!(
                                "collection '{}': build_index timeout after {}ms",
                                collection, self.vector_timeout_ms
                            )));
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Generate embedding without storing (useful for query-time)
    #[allow(dead_code)]
    pub async fn embed_text(&self, text: &str) -> Result<Vec<f32>, VectorServiceError> {
        self.embed_checked(text).await
    }

    fn matches_filters(&self, doc: &IndexedDocument, filters: &SearchFilters) -> bool {
        if let Some(space_id) = &filters.space_id {
            if !doc.space_id.eq_ignore_ascii_case(space_id) {
                return false;
            }
        }

        if let Some(source_type) = &filters.source_type {
            if !doc.source_type.eq_ignore_ascii_case(source_type) {
                return false;
            }
        }

        if !filters.modalities.is_empty() {
            let normalized: Vec<String> = filters
                .modalities
                .iter()
                .filter_map(|raw| Self::normalize_modality(raw))
                .collect();
            if normalized.is_empty() {
                return false;
            }
            if !normalized
                .iter()
                .any(|modality| doc.modality.eq_ignore_ascii_case(modality))
            {
                return false;
            }
        }

        if !filters.tags.is_empty() {
            let doc_tags: HashSet<String> = doc.tags.iter().map(|t| t.to_lowercase()).collect();
            for required in &filters.tags {
                if !doc_tags.contains(&required.to_lowercase()) {
                    return false;
                }
            }
        }

        if let Some(range) = &filters.time_range {
            if let Some(ts) = doc.timestamp_ms {
                if let Some(from_ms) = range.from_ms {
                    if ts < from_ms {
                        return false;
                    }
                }
                if let Some(to_ms) = range.to_ms {
                    if ts > to_ms {
                        return false;
                    }
                }
            } else {
                // If filter asks for time constraints but doc has no timestamp, skip it.
                if range.from_ms.is_some() || range.to_ms.is_some() {
                    return false;
                }
            }
        }

        if let Some(required_scope) = &filters.perspective_scope {
            let Some(provenance) = &doc.provenance else {
                return false;
            };
            let Some(scope) = &provenance.perspective_scope else {
                return false;
            };
            if !scope.eq_ignore_ascii_case(required_scope) {
                return false;
            }
        }

        if let Some(required_agent) = &filters.produced_by_agent {
            let Some(provenance) = &doc.provenance else {
                return false;
            };
            let Some(agent) = &provenance.produced_by_agent else {
                return false;
            };
            if !agent.eq_ignore_ascii_case(required_agent) {
                return false;
            }
        }

        if let Some(required_source_version_id) = &filters.source_version_id {
            let Some(provenance) = &doc.provenance else {
                return false;
            };
            let Some(source_version_id) = &provenance.source_version_id else {
                return false;
            };
            if source_version_id != required_source_version_id {
                return false;
            }
        }

        true
    }

    fn compute_lexical_scores(
        &self,
        query: &str,
        docs: &[IndexedDocument],
    ) -> HashMap<String, f32> {
        let query_tokens = Self::tokenize(query);
        if query_tokens.is_empty() {
            return HashMap::new();
        }

        let query_set: HashSet<String> = query_tokens.iter().cloned().collect();
        let mut scores = HashMap::new();

        for doc in docs {
            let doc_tokens = Self::tokenize(&doc.text);
            if doc_tokens.is_empty() {
                continue;
            }
            let doc_set: HashSet<String> = doc_tokens.into_iter().collect();
            let overlap = query_set.intersection(&doc_set).count() as f32;
            if overlap > 0.0 {
                let norm = overlap / query_set.len() as f32;
                scores.insert(doc.id.clone(), norm);
            }
        }

        scores
    }

    fn rerank_bonus(&self, query: &str, text: &str) -> f32 {
        let q = query.to_lowercase();
        let body = text.to_lowercase();
        if body.contains(&q) { 1.0 } else { 0.0 }
    }

    fn rank_reason(
        &self,
        mode: RetrievalMode,
        vector_score: f32,
        lexical_score: f32,
        rerank_enabled: bool,
    ) -> String {
        let mut parts = Vec::new();
        match mode {
            RetrievalMode::Vector => parts.push("vector-only".to_string()),
            RetrievalMode::Lexical => parts.push("lexical-only".to_string()),
            RetrievalMode::Hybrid => {
                if vector_score > 0.0 {
                    parts.push("vector-match".to_string());
                }
                if lexical_score > 0.0 {
                    parts.push("lexical-match".to_string());
                }
                if vector_score == 0.0 && lexical_score == 0.0 {
                    parts.push("fallback".to_string());
                }
            }
        }

        if rerank_enabled {
            parts.push("rerank".to_string());
        }

        if parts.is_empty() {
            "unspecified".to_string()
        } else {
            parts.join("+")
        }
    }

    fn validate_cei_metadata(&self, metadata: &CeiMetadataV1) -> Result<(), VectorServiceError> {
        if metadata.contribution_id.trim().is_empty() {
            return Err(VectorServiceError::ConfigError(
                "CEI metadata invalid: contribution_id is required".to_string(),
            ));
        }
        if metadata.source_uri.trim().is_empty() {
            return Err(VectorServiceError::ConfigError(
                "CEI metadata invalid: source_uri is required".to_string(),
            ));
        }
        if metadata.timestamp.trim().is_empty() {
            return Err(VectorServiceError::ConfigError(
                "CEI metadata invalid: timestamp is required".to_string(),
            ));
        }
        if let Some(confidence) = metadata.confidence {
            if !(0.0..=1.0).contains(&confidence) {
                return Err(VectorServiceError::ConfigError(
                    "CEI metadata invalid: confidence must be within [0.0, 1.0]".to_string(),
                ));
            }
        }
        Ok(())
    }

    fn content_snippet(text: &str) -> String {
        const MAX_CHARS: usize = 280;
        let trimmed = text.trim();
        if trimmed.chars().count() <= MAX_CHARS {
            trimmed.to_string()
        } else {
            let mut out = trimmed.chars().take(MAX_CHARS).collect::<String>();
            out.push_str("...");
            out
        }
    }

    fn tokenize(text: &str) -> Vec<String> {
        text.split_whitespace()
            .map(|raw| {
                raw.chars()
                    .filter(|c| c.is_ascii_alphanumeric())
                    .collect::<String>()
                    .to_lowercase()
            })
            .filter(|t| !t.is_empty())
            .collect()
    }

    async fn embed_checked(&self, text: &str) -> Result<Vec<f32>, VectorServiceError> {
        let embedding = self.embedding_provider.embed(text).await?;
        self.ensure_dimension(&embedding)?;
        Ok(embedding)
    }

    async fn embed_batch_checked(
        &self,
        texts: &[&str],
    ) -> Result<Vec<Vec<f32>>, VectorServiceError> {
        let embeddings = self.embedding_provider.embed_batch(texts).await?;
        for embedding in &embeddings {
            self.ensure_dimension(embedding)?;
        }
        Ok(embeddings)
    }

    fn ensure_dimension(&self, embedding: &[f32]) -> Result<(), VectorServiceError> {
        let expected = self.embedding_provider.dimension();
        if embedding.len() != expected {
            return Err(VectorServiceError::ConfigError(format!(
                "Embedding dimension mismatch: expected {}, got {}",
                expected,
                embedding.len()
            )));
        }
        Ok(())
    }

    fn log_shadow_parity(
        &self,
        vector_ids: &[String],
        lexical_scores: &HashMap<String, f32>,
        candidate_pool_ids: &[String],
        top_k: usize,
    ) {
        let vector_top: HashSet<String> = vector_ids.iter().take(top_k).cloned().collect();

        let mut lexical_ranked: Vec<(String, f32)> = candidate_pool_ids
            .iter()
            .map(|id| (id.clone(), *lexical_scores.get(id).unwrap_or(&0.0)))
            .collect();
        lexical_ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(Ordering::Equal));

        let lexical_top: HashSet<String> = lexical_ranked
            .into_iter()
            .take(top_k)
            .map(|(id, _)| id)
            .collect();

        if vector_top.is_empty() && lexical_top.is_empty() {
            return;
        }

        let overlap = vector_top.intersection(&lexical_top).count();
        let denom = vector_top.union(&lexical_top).count().max(1);
        let parity = overlap as f32 / denom as f32;

        if let Ok(mut metrics) = self.shadow_metrics.lock() {
            metrics.searches += 1;
            metrics.overlap_total += overlap as u64;
            metrics.union_total += denom as u64;
            metrics.last_parity = Some(parity);
        }

        println!(
            "[VectorService][shadow] top{} overlap={} parity={:.2}",
            top_k, overlap, parity
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::embedding_provider::{EmbeddingError, EmbeddingProvider};
    use crate::mock_embedding::MockEmbeddingGenerator;
    use async_trait::async_trait;
    use ic_agent::export::Principal;
    use ic_agent::identity::AnonymousIdentity;
    use serde_json::json;
    use std::collections::{HashMap, HashSet};
    use tokio::sync::RwLock;

    fn make_service() -> VectorService {
        make_service_with_runtime(VectorBackend::Mock, "local_memory", true, false)
    }

    fn make_service_with_runtime(
        backend: VectorBackend,
        endpoint: &str,
        elna_fail_open: bool,
        shadow_compare: bool,
    ) -> VectorService {
        let provider = Arc::new(MockEmbeddingGenerator::new());
        let agent = Arc::new(
            ic_agent::Agent::builder()
                .with_url("http://127.0.0.1:4943")
                .with_identity(AnonymousIdentity)
                .build()
                .expect("agent"),
        );

        let vector_client = if backend == VectorBackend::Elna {
            Principal::from_text(endpoint)
                .ok()
                .map(|canister_id| VectorClient::new(agent.clone(), canister_id))
        } else {
            None
        };

        VectorService {
            embedding_provider: provider,
            vector_client,
            default_collection: "test_collection".to_string(),
            backend,
            vector_endpoint: endpoint.to_string(),
            vector_timeout_ms: 25,
            elna_fail_open,
            shadow_compare,
            shadow_metrics: Arc::new(Mutex::new(ShadowMetrics::default())),
            documents: Arc::new(RwLock::new(HashMap::new())),
            idempotency_keys: Arc::new(RwLock::new(HashSet::new())),
        }
    }

    #[test]
    fn test_embedding_metadata() {
        let mock_provider = Arc::new(MockEmbeddingGenerator::new());
        let meta = mock_provider.metadata();

        assert_eq!(meta.model_family, "mock");
        assert_eq!(meta.dimension, 384);
    }

    #[tokio::test]
    async fn test_idempotency_key() {
        let service = make_service();
        assert!(service.register_idempotency_key("abc").await);
        assert!(!service.register_idempotency_key("abc").await);
    }

    #[tokio::test]
    async fn test_hybrid_search_with_filters() {
        let service = make_service();

        let docs = vec![
            IndexDocument {
                id: "doc-1".to_string(),
                text: "Nostra governance and execution engine".to_string(),
                label: "book:a".to_string(),
                space_id: "space-alpha".to_string(),
                source_ref: "urn:source:a".to_string(),
                source_type: "book".to_string(),
                tags: vec!["governance".to_string(), "core".to_string()],
                timestamp_ms: Some(1000),
                cei_metadata: None,
                modality: None,
            },
            IndexDocument {
                id: "doc-2".to_string(),
                text: "Unrelated cooking notes".to_string(),
                label: "book:b".to_string(),
                space_id: "space-beta".to_string(),
                source_ref: "urn:source:b".to_string(),
                source_type: "note".to_string(),
                tags: vec!["food".to_string()],
                timestamp_ms: Some(2000),
                cei_metadata: None,
                modality: None,
            },
        ];

        let outcome = service
            .index_documents(docs, Some("test-key"))
            .await
            .expect("index ok");
        assert_eq!(outcome.indexed_count, 2);

        let opts = SearchOptions {
            retrieval_mode: RetrievalMode::Hybrid,
            filters: SearchFilters {
                space_id: Some("space-alpha".to_string()),
                source_type: Some("book".to_string()),
                time_range: Some(TimeRangeFilter {
                    from_ms: Some(500),
                    to_ms: Some(1500),
                }),
                tags: vec!["governance".to_string()],
                perspective_scope: None,
                produced_by_agent: None,
                source_version_id: None,
                modalities: vec![],
            },
            fusion_weights: HybridConfig::default(),
            diagnostics: true,
        };

        let results = service
            .search_with_options("governance engine", 10, opts)
            .await
            .expect("search ok");

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].id, "doc-1");
        assert!(results[0].diagnostic.is_some());
        let diag = results[0].diagnostic.clone().unwrap();
        assert!(!diag.rank_reason.trim().is_empty());
        assert!(!diag.backend.trim().is_empty());
        assert!(!diag.embedding_model.trim().is_empty());
    }

    #[tokio::test]
    async fn test_shadow_report_updates() {
        let service = make_service_with_runtime(VectorBackend::Mock, "local_memory", true, true);
        let docs = vec![IndexDocument {
            id: "doc-a".to_string(),
            text: "Hybrid retrieval parity sample".to_string(),
            label: "book:a".to_string(),
            space_id: "space-a".to_string(),
            source_ref: "urn:a".to_string(),
            source_type: "book".to_string(),
            tags: vec!["hybrid".to_string()],
            timestamp_ms: Some(1000),
            cei_metadata: None,
            modality: None,
        }];

        let _ = service
            .index_documents(docs, Some("shadow-report-key"))
            .await
            .unwrap();
        let _ = service
            .search_with_options(
                "hybrid retrieval",
                5,
                SearchOptions {
                    retrieval_mode: RetrievalMode::Hybrid,
                    ..SearchOptions::default()
                },
            )
            .await
            .unwrap();

        let report = service.shadow_report();
        assert!(report.enabled);
        assert!(report.searches >= 1);
    }

    #[tokio::test]
    async fn test_invalid_cei_metadata_rejected() {
        let service = make_service();
        let docs = vec![IndexDocument {
            id: "doc-bad".to_string(),
            text: "invalid cei metadata".to_string(),
            label: "book:bad".to_string(),
            space_id: "space-a".to_string(),
            source_ref: "urn:bad".to_string(),
            source_type: "book".to_string(),
            tags: vec![],
            timestamp_ms: Some(1000),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "".to_string(),
                source_uri: "".to_string(),
                author: "unknown".to_string(),
                timestamp: "".to_string(),
                lineage_refs: vec![],
                source_version_id: None,
                model_id: None,
                perspective_scope: None,
                produced_by_agent: None,
                confidence: None,
                purpose: None,
                modality: None,
            }),
            modality: None,
        }];

        let err = service
            .index_documents(docs, Some("invalid-cei-key"))
            .await
            .expect_err("expected CEI validation error");
        assert!(err.to_string().contains("CEI metadata invalid"));
    }

    #[tokio::test]
    async fn test_invalid_cei_confidence_rejected() {
        let service = make_service();
        let docs = vec![IndexDocument {
            id: "doc-bad-confidence".to_string(),
            text: "invalid cei confidence".to_string(),
            label: "book:bad-confidence".to_string(),
            space_id: "space-a".to_string(),
            source_ref: "urn:bad:confidence".to_string(),
            source_type: "book".to_string(),
            tags: vec![],
            timestamp_ms: Some(1000),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "c-bad-confidence".to_string(),
                source_uri: "urn:bad:confidence".to_string(),
                author: "did:example:alice".to_string(),
                timestamp: "2026-02-08T00:00:00Z".to_string(),
                lineage_refs: vec![],
                source_version_id: None,
                model_id: None,
                perspective_scope: None,
                produced_by_agent: None,
                confidence: Some(1.5),
                purpose: None,
                modality: None,
            }),
            modality: None,
        }];

        let err = service
            .index_documents(docs, Some("invalid-cei-confidence-key"))
            .await
            .expect_err("expected CEI confidence validation error");
        assert!(
            err.to_string()
                .contains("confidence must be within [0.0, 1.0]")
        );
    }

    #[test]
    fn test_cei_metadata_serialization_roundtrip() {
        let metadata = CeiMetadataV1 {
            contribution_id: "c-1".to_string(),
            source_uri: "urn:source:c-1".to_string(),
            author: "did:example:alice".to_string(),
            timestamp: "2026-02-07T00:00:00Z".to_string(),
            lineage_refs: vec!["v1".to_string(), "v2".to_string()],
            source_version_id: Some("sv-1".to_string()),
            model_id: Some("qwen3-embedding:0.6b".to_string()),
            perspective_scope: Some("space-alpha".to_string()),
            produced_by_agent: Some("agent://vector-worker".to_string()),
            confidence: Some(0.91),
            purpose: Some("retrieval".to_string()),
            modality: Some("text".to_string()),
        };

        let value = serde_json::to_value(&metadata).expect("serialize");
        assert_eq!(value["contribution_id"], json!("c-1"));
        assert_eq!(value["source_uri"], json!("urn:source:c-1"));
        assert_eq!(value["timestamp"], json!("2026-02-07T00:00:00Z"));
        assert_eq!(value["source_version_id"], json!("sv-1"));
        assert_eq!(value["produced_by_agent"], json!("agent://vector-worker"));

        let roundtrip: CeiMetadataV1 = serde_json::from_value(value).expect("deserialize");
        assert_eq!(roundtrip.contribution_id, "c-1");
        assert_eq!(roundtrip.lineage_refs.len(), 2);
        assert_eq!(roundtrip.source_version_id.as_deref(), Some("sv-1"));
        assert_eq!(roundtrip.confidence, Some(0.91));
    }

    #[tokio::test]
    async fn test_elna_invalid_endpoint_has_no_client() {
        let service = make_service_with_runtime(VectorBackend::Elna, "!!", true, false);
        let health = service.health().await;
        assert_eq!(health.backend, "elna");
        assert!(!health.vector_client_enabled);
    }

    struct FailingEmbedder;

    #[async_trait]
    impl EmbeddingProvider for FailingEmbedder {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
            Err(EmbeddingError::ApiError("provider unavailable".to_string()))
        }

        fn dimension(&self) -> usize {
            384
        }

        fn model_id(&self) -> String {
            "test/failing".to_string()
        }
    }

    #[tokio::test]
    async fn test_health_degraded_when_embedding_provider_unavailable() {
        let agent = Arc::new(
            ic_agent::Agent::builder()
                .with_url("http://127.0.0.1:4943")
                .with_identity(AnonymousIdentity)
                .build()
                .expect("agent"),
        );
        let service = VectorService::new(
            Arc::new(FailingEmbedder),
            agent,
            "test_collection".to_string(),
        );
        let health = service.health().await;
        assert_eq!(health.status, "degraded");
        assert!(!health.embedding_probe_ok);
        assert!(health.embedding_probe_error.is_some());
    }

    struct WrongDimensionEmbedder;

    #[async_trait]
    impl EmbeddingProvider for WrongDimensionEmbedder {
        async fn embed(&self, _text: &str) -> Result<Vec<f32>, EmbeddingError> {
            Ok(vec![0.1; 16])
        }

        fn dimension(&self) -> usize {
            384
        }

        fn model_id(&self) -> String {
            "test/wrong-dimension".to_string()
        }
    }

    #[tokio::test]
    async fn test_dimension_mismatch_rejected() {
        let agent = Arc::new(
            ic_agent::Agent::builder()
                .with_url("http://127.0.0.1:4943")
                .with_identity(AnonymousIdentity)
                .build()
                .expect("agent"),
        );
        let service = VectorService::new(
            Arc::new(WrongDimensionEmbedder),
            agent,
            "test_collection".to_string(),
        );
        let err = service
            .index_documents(
                vec![IndexDocument {
                    id: "doc-dim".to_string(),
                    text: "dimension mismatch".to_string(),
                    label: "test".to_string(),
                    space_id: "space".to_string(),
                    source_ref: "urn:dim".to_string(),
                    source_type: "note".to_string(),
                    tags: vec![],
                    timestamp_ms: None,
                    cei_metadata: Some(CeiMetadataV1 {
                        contribution_id: "dim".to_string(),
                        source_uri: "urn:dim".to_string(),
                        author: "did:example".to_string(),
                        timestamp: "2026-02-07T00:00:00Z".to_string(),
                        lineage_refs: vec![],
                        source_version_id: None,
                        model_id: None,
                        perspective_scope: None,
                        produced_by_agent: None,
                        confidence: None,
                        purpose: None,
                        modality: None,
                    }),
                    modality: None,
                }],
                Some("dim-key"),
            )
            .await
            .expect_err("expected dimension mismatch");
        assert!(err.to_string().contains("Embedding dimension mismatch"));
    }

    #[tokio::test]
    async fn test_elna_fail_open_falls_back_to_lexical() {
        let service = make_service_with_runtime(VectorBackend::Elna, "aaaaa-aa", true, false);
        let _ = service
            .index_documents(
                vec![IndexDocument {
                    id: "doc-elna-fallback".to_string(),
                    text: "ELNA fallback lexical behavior".to_string(),
                    label: "test".to_string(),
                    space_id: "space".to_string(),
                    source_ref: "urn:elna".to_string(),
                    source_type: "note".to_string(),
                    tags: vec!["elna".to_string()],
                    timestamp_ms: Some(1_700_000_123_000),
                    cei_metadata: Some(CeiMetadataV1 {
                        contribution_id: "elna".to_string(),
                        source_uri: "urn:elna".to_string(),
                        author: "did:example".to_string(),
                        timestamp: "2026-02-07T00:00:00Z".to_string(),
                        lineage_refs: vec![],
                        source_version_id: None,
                        model_id: None,
                        perspective_scope: None,
                        produced_by_agent: None,
                        confidence: None,
                        purpose: None,
                        modality: None,
                    }),
                    modality: None,
                }],
                Some("elna-fallback-key"),
            )
            .await
            .expect("index should fail-open");

        let results = service
            .search_with_options(
                "fallback lexical",
                5,
                SearchOptions {
                    retrieval_mode: RetrievalMode::Hybrid,
                    diagnostics: true,
                    ..SearchOptions::default()
                },
            )
            .await
            .expect("search should fail-open");

        assert!(!results.is_empty());
    }

    #[tokio::test]
    async fn test_elna_fail_closed_returns_error() {
        let service = make_service_with_runtime(VectorBackend::Elna, "aaaaa-aa", false, false);
        let result = service
            .index_documents(
                vec![IndexDocument {
                    id: "doc-elna-closed".to_string(),
                    text: "ELNA fail-closed behavior".to_string(),
                    label: "test".to_string(),
                    space_id: "space".to_string(),
                    source_ref: "urn:elna:closed".to_string(),
                    source_type: "note".to_string(),
                    tags: vec!["elna".to_string()],
                    timestamp_ms: Some(1_700_000_124_000),
                    cei_metadata: Some(CeiMetadataV1 {
                        contribution_id: "elna-closed".to_string(),
                        source_uri: "urn:elna:closed".to_string(),
                        author: "did:example".to_string(),
                        timestamp: "2026-02-07T00:00:00Z".to_string(),
                        lineage_refs: vec![],
                        source_version_id: None,
                        model_id: None,
                        perspective_scope: None,
                        produced_by_agent: None,
                        confidence: None,
                        purpose: None,
                        modality: None,
                    }),
                    modality: None,
                }],
                Some("elna-fail-closed-key"),
            )
            .await;

        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_metadata_filters_by_perspective_agent_and_source_version() {
        let service = make_service();

        let docs = vec![IndexDocument {
            id: "doc-meta-1".to_string(),
            text: "Scoped and agent-produced retrieval chunk".to_string(),
            label: "meta".to_string(),
            space_id: "space-alpha".to_string(),
            source_ref: "urn:meta:1".to_string(),
            source_type: "note".to_string(),
            tags: vec!["meta".to_string()],
            timestamp_ms: Some(1_700_000_222_000),
            cei_metadata: Some(CeiMetadataV1 {
                contribution_id: "meta-1".to_string(),
                source_uri: "urn:meta:1".to_string(),
                author: "did:example:alice".to_string(),
                timestamp: "2026-02-08T00:00:00Z".to_string(),
                lineage_refs: vec!["v3".to_string()],
                source_version_id: Some("sv-meta-1".to_string()),
                model_id: Some("qwen3-embedding:0.6b".to_string()),
                perspective_scope: Some("space-alpha".to_string()),
                produced_by_agent: Some("agent://knowledge-worker".to_string()),
                confidence: Some(0.87),
                purpose: Some("retrieval".to_string()),
                modality: Some("text".to_string()),
            }),
            modality: None,
        }];

        service
            .index_documents(docs, Some("meta-filter-key"))
            .await
            .expect("index ok");

        let filtered = service
            .search_with_options(
                "scoped retrieval chunk",
                5,
                SearchOptions {
                    retrieval_mode: RetrievalMode::Hybrid,
                    filters: SearchFilters {
                        perspective_scope: Some("space-alpha".to_string()),
                        produced_by_agent: Some("agent://knowledge-worker".to_string()),
                        source_version_id: Some("sv-meta-1".to_string()),
                        ..SearchFilters::default()
                    },
                    diagnostics: true,
                    ..SearchOptions::default()
                },
            )
            .await
            .expect("search filtered");

        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, "doc-meta-1");

        let no_match = service
            .search_with_options(
                "scoped retrieval chunk",
                5,
                SearchOptions {
                    retrieval_mode: RetrievalMode::Hybrid,
                    filters: SearchFilters {
                        produced_by_agent: Some("agent://different".to_string()),
                        ..SearchFilters::default()
                    },
                    ..SearchOptions::default()
                },
            )
            .await
            .expect("search mismatch");
        assert!(no_match.is_empty());
    }
}
